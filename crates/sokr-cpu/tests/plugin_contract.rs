//! Behavioral contract tests for `sokr-cpu`, driven through the real `sokr`
//! core FFI (`register → capability → dispatch → completion → deregister`).
//!
//! ## Serialization
//!
//! sokr core's substrate registry is a process-global `static` guarded by a
//! single-threaded-access invariant (Phase < 1.0), and `sokr-cpu`'s completion
//! table is likewise global. `cargo test` runs `#[test]`s on parallel threads,
//! so every test here holds [`SERIAL`] for its whole body to keep access
//! single-threaded. Each test also deregisters what it registers so the shared
//! state is clean for the next test.

use std::ffi::CString;
use std::sync::{Mutex, MutexGuard, PoisonError};

use sokr::ffi::{
    sokr_capability, sokr_completion, sokr_deregister_substrate, sokr_dispatch,
    sokr_register_substrate,
};
use sokr::{
    SokrCapabilityQuery, SokrCapabilityResponse, SokrCompletionQuery, SokrCompletionSignal,
    SokrCompletionToken, SokrComputationId, SokrDispatchRequest, SokrDispatchResponse, SokrResult,
};
use sokr_cpu::CPU_PLUGIN;

/// Process-wide lock enforcing the single-threaded contract during tests.
static SERIAL: Mutex<()> = Mutex::new(());

/// Acquire the serialization lock, ignoring poisoning from a failed test.
fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Register `CPU_PLUGIN` and return its core-assigned non-zero substrate id.
fn register() -> u64 {
    let mut id: u64 = 0;
    // SAFETY: `&CPU_PLUGIN` and `&mut id` are valid, aligned, non-null.
    let result = unsafe { sokr_register_substrate(&CPU_PLUGIN, &mut id) };
    assert_eq!(result, SokrResult::Ok, "registration must succeed");
    assert_ne!(id, 0, "core must assign a non-zero substrate id");
    id
}

/// A capability response pre-seeded with sentinel garbage, to prove the plugin
/// overwrites every field.
fn dirty_capability_response() -> SokrCapabilityResponse {
    SokrCapabilityResponse {
        result: SokrResult::DispatchFailed,
        padding: u32::MAX,
        substrate_id: u64::MAX,
        estimated_latency_ns: u64::MAX,
    }
}

/// Run a capability query through the core for the given IR format.
fn query_capability(ir_format: &str, ir_data: &[u8]) -> (SokrResult, SokrCapabilityResponse) {
    let format = CString::new(ir_format).unwrap();
    let query = SokrCapabilityQuery {
        computation_id: SokrComputationId { high: 0, low: 1 },
        ir_format: format.as_ptr(),
        ir_data_ptr: ir_data.as_ptr().cast(),
        ir_data_len: ir_data.len(),
        padding: [0; 8],
    };
    let mut response = dirty_capability_response();
    // SAFETY: `query`/`response` outlive the call and are valid, non-null.
    let result = unsafe { sokr_capability(&query, &mut response) };
    (result, response)
}

/// Dispatch one `sokr-noop` computation to `substrate_id`, returning the token.
fn dispatch_noop(substrate_id: u64, low: u64) -> SokrCompletionToken {
    let ir_data = b"noop";
    let request = SokrDispatchRequest {
        computation_id: SokrComputationId { high: 0, low },
        substrate_id,
        ir_data_ptr: ir_data.as_ptr().cast(),
        ir_data_len: ir_data.len(),
        params_ptr: std::ptr::null(),
        params_len: 0,
        padding: [0; 16],
    };
    let mut response = SokrDispatchResponse {
        result: SokrResult::DispatchFailed,
        padding: u32::MAX,
        completion_token: SokrCompletionToken { handle: 0 },
    };
    // SAFETY: `request`/`response` outlive the call and are valid, non-null.
    let result = unsafe { sokr_dispatch(&request, &mut response) };
    assert_eq!(result, SokrResult::Ok, "dispatch must succeed");
    assert_ne!(
        response.completion_token.handle, 0,
        "dispatch must issue a non-zero completion token"
    );
    response.completion_token
}

/// Poll a completion token through the core, returning `(result, signal)`.
fn poll(token: SokrCompletionToken) -> (SokrResult, SokrCompletionSignal) {
    let query = SokrCompletionQuery {
        completion_token: token,
        timeout_ns: 0,
        padding: [0; 8],
    };
    let mut signal = SokrCompletionSignal::Pending;
    // SAFETY: `query`/`signal` outlive the call and are valid, non-null.
    let result = unsafe { sokr_completion(&query, &mut signal) };
    (result, signal)
}

#[test]
fn test_capability_accepts_noop_disclaims_others() {
    let _guard = serial();
    let id = register();

    let (result, response) = query_capability("sokr-noop", b"noop");
    assert_eq!(result, SokrResult::Ok, "sokr-noop must be claimed");
    assert_eq!(response.result, SokrResult::Ok);

    // Anything that is not sokr-noop must be disclaimed (CapabilityDenied) so
    // the core's scan can fall through to other plugins — never InvalidIR.
    let (result, response) = query_capability("sokr-matmul", b"data");
    assert_eq!(
        result,
        SokrResult::CapabilityDenied,
        "unknown IR must be disclaimed, not claimed or errored"
    );
    assert_eq!(response.result, SokrResult::CapabilityDenied);

    assert_eq!(sokr_deregister_substrate(id), SokrResult::Ok);
}

#[test]
fn test_1_2_dispatch_completion_roundtrip() {
    let _guard = serial();
    let id = register();

    let (cap, _) = query_capability("sokr-noop", b"noop");
    assert_eq!(cap, SokrResult::Ok);

    let token = dispatch_noop(id, 1);
    let (result, signal) = poll(token);
    assert_eq!(result, SokrResult::Ok);
    assert_eq!(signal, SokrCompletionSignal::Complete);

    assert_eq!(sokr_deregister_substrate(id), SokrResult::Ok);
}

#[test]
fn test_1_2_multiple_concurrent_dispatches() {
    let _guard = serial();
    let id = register();

    // Five outstanding tokens before any are polled; all must be distinct.
    let mut tokens = Vec::new();
    for low in 0..5 {
        let token = dispatch_noop(id, low);
        assert!(
            !tokens.contains(&token.handle),
            "each dispatch must yield a unique token"
        );
        tokens.push(token.handle);
    }

    for &handle in &tokens {
        let (result, signal) = poll(SokrCompletionToken { handle });
        assert_eq!(result, SokrResult::Ok);
        assert_eq!(signal, SokrCompletionSignal::Complete);
    }

    assert_eq!(sokr_deregister_substrate(id), SokrResult::Ok);
}

#[test]
fn test_1_3_destroy_invalidates_tokens() {
    let _guard = serial();
    let id = register();

    let token = dispatch_noop(id, 1);

    // Deregistration calls the plugin's destroy_fn, clearing its table.
    assert_eq!(sokr_deregister_substrate(id), SokrResult::Ok);

    // No substrate is registered now, so the token is unrecognized: the core
    // reports NotFound and forces the signal to Failed.
    let (result, signal) = poll(token);
    assert_eq!(result, SokrResult::NotFound);
    assert_eq!(signal, SokrCompletionSignal::Failed);
}

#[test]
fn test_1_3_destroy_clears_all_slots() {
    let _guard = serial();
    let id = register();

    // Ten outstanding tokens, none polled.
    let stale: Vec<SokrCompletionToken> = (0..10).map(|low| dispatch_noop(id, low)).collect();

    // destroy_fn (via deregister) must clear every slot.
    assert_eq!(sokr_deregister_substrate(id), SokrResult::Ok);

    // Re-register: the global table persists across registrations, so any slot
    // destroy failed to clear would still report Complete here.
    let id2 = register();
    for token in &stale {
        let (result, _signal) = poll(*token);
        assert_eq!(
            result,
            SokrResult::NotFound,
            "destroy must have cleared every outstanding slot"
        );
    }

    // Slots are free again: a fresh dispatch still works.
    let fresh = dispatch_noop(id2, 99);
    let (result, signal) = poll(fresh);
    assert_eq!(result, SokrResult::Ok);
    assert_eq!(signal, SokrCompletionSignal::Complete);

    assert_eq!(sokr_deregister_substrate(id2), SokrResult::Ok);
}

#[test]
fn test_1_4_full_lifecycle() {
    let _guard = serial();
    let id = register();

    // capability
    let (cap, response) = query_capability("sokr-noop", b"payload");
    assert_eq!(cap, SokrResult::Ok);
    assert_eq!(response.result, SokrResult::Ok);

    // dispatch
    let token = dispatch_noop(id, 7);

    // completion: Complete on first poll
    let (result, signal) = poll(token);
    assert_eq!(result, SokrResult::Ok);
    assert_eq!(signal, SokrCompletionSignal::Complete);

    // the token is single-use: a second poll is disclaimed and forced to Failed
    let (result, signal) = poll(token);
    assert_eq!(result, SokrResult::NotFound);
    assert_eq!(signal, SokrCompletionSignal::Failed);

    // destroy
    assert_eq!(sokr_deregister_substrate(id), SokrResult::Ok);
}

#[test]
fn test_1_4_multiple_computations() {
    let _guard = serial();
    let id = register();

    // Dispatch 20 computations, interleaving nothing yet; collect unique tokens.
    let mut tokens = Vec::new();
    for low in 0..20 {
        let token = dispatch_noop(id, low);
        assert!(
            !tokens.contains(&token.handle),
            "tokens must be unique across computations"
        );
        tokens.push(token.handle);
    }

    // Interleaved polling: each token completes exactly once.
    for &handle in &tokens {
        let (result, signal) = poll(SokrCompletionToken { handle });
        assert_eq!(result, SokrResult::Ok);
        assert_eq!(signal, SokrCompletionSignal::Complete);
    }

    // Every token is now consumed.
    for &handle in &tokens {
        let (result, _signal) = poll(SokrCompletionToken { handle });
        assert_eq!(result, SokrResult::NotFound);
    }

    assert_eq!(sokr_deregister_substrate(id), SokrResult::Ok);
}

#[test]
fn test_plugin_descriptor_advertises_current_version() {
    // The vtable must advertise the core's current ABI version so that
    // sokr_register_substrate's compatibility check accepts it.
    assert_eq!(CPU_PLUGIN.version, sokr::SokrVersion::CURRENT);
    assert_eq!(
        CPU_PLUGIN.substrate_id, 0,
        "static descriptor id is unassigned"
    );
    assert_eq!(CPU_PLUGIN.padding, [0u8; 8]);
}
