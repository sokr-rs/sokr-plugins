//! End-to-end proof that the SOKR contract closes on a CPU substrate.
//!
//! Runs the full loop through the real `sokr` core:
//!
//! ```text
//! register → capability → dispatch → completion → deregister
//! ```
//!
//! Run with: `cargo run --example cpu_end_to_end`
//!
//! This is single-threaded `main`, matching the core's pre-1.0 single-threaded
//! invariant; do not adapt it to drive the core from multiple threads.

use std::ffi::CString;

use sokr::ffi::{
    sokr_capability, sokr_completion, sokr_deregister_substrate, sokr_dispatch,
    sokr_register_substrate,
};
use sokr::{
    SokrCapabilityQuery, SokrCapabilityResponse, SokrCompletionQuery, SokrCompletionSignal,
    SokrCompletionToken, SokrComputationId, SokrDispatchRequest, SokrDispatchResponse, SokrResult,
};
use sokr_cpu::CPU_PLUGIN;

fn main() {
    // 1. Register the CPU substrate with the core. The core assigns the id we
    //    must use to route dispatch (the plugin cannot know it itself).
    let mut substrate_id: u64 = 0;
    // SAFETY: `&CPU_PLUGIN` and `&mut substrate_id` are valid and non-null.
    let result = unsafe { sokr_register_substrate(&CPU_PLUGIN, &mut substrate_id) };
    assert_eq!(result, SokrResult::Ok, "registration failed: {result:?}");
    assert_ne!(substrate_id, 0, "core must assign a non-zero substrate id");
    println!("registered sokr-cpu  → substrate_id = {substrate_id}");

    // 2. Ask whether the substrate can fulfill a `sokr-noop` computation.
    let ir_format = CString::new("sokr-noop").unwrap();
    let ir_data = b"hello, sokr";
    let computation_id = SokrComputationId {
        high: 0,
        low: 0xC0FFEE,
    };

    let query = SokrCapabilityQuery {
        computation_id,
        ir_format: ir_format.as_ptr(),
        ir_data_ptr: ir_data.as_ptr().cast(),
        ir_data_len: ir_data.len(),
        padding: [0; 8],
    };
    let mut capability = SokrCapabilityResponse {
        result: SokrResult::CapabilityDenied,
        padding: 0,
        substrate_id: 0,
        estimated_latency_ns: 0,
    };
    // SAFETY: `query`/`capability` outlive the call and are valid, non-null.
    let result = unsafe { sokr_capability(&query, &mut capability) };
    assert_eq!(result, SokrResult::Ok, "capability denied: {result:?}");
    println!("capability(sokr-noop) → {:?}", capability.result);

    // 3. Dispatch the computation, routing by the core-assigned id.
    let request = SokrDispatchRequest {
        computation_id,
        substrate_id,
        ir_data_ptr: ir_data.as_ptr().cast(),
        ir_data_len: ir_data.len(),
        params_ptr: std::ptr::null(),
        params_len: 0,
        padding: [0; 16],
    };
    let mut dispatch = SokrDispatchResponse {
        result: SokrResult::DispatchFailed,
        padding: 0,
        completion_token: SokrCompletionToken { handle: 0 },
    };
    // SAFETY: `request`/`dispatch` outlive the call and are valid, non-null.
    let result = unsafe { sokr_dispatch(&request, &mut dispatch) };
    assert_eq!(result, SokrResult::Ok, "dispatch failed: {result:?}");
    let token = dispatch.completion_token;
    assert_ne!(token.handle, 0, "dispatch must issue a non-zero token");
    println!(
        "dispatch              → completion_token.handle = {}",
        token.handle
    );

    // 4. Poll completion until the computation finishes (immediate for CPU).
    let completion_query = SokrCompletionQuery {
        completion_token: token,
        timeout_ns: 0,
        padding: [0; 8],
    };
    let mut signal = SokrCompletionSignal::Pending;
    // SAFETY: `completion_query`/`signal` outlive the call and are valid, non-null.
    let result = unsafe { sokr_completion(&completion_query, &mut signal) };
    assert_eq!(
        result,
        SokrResult::Ok,
        "completion query failed: {result:?}"
    );
    assert_eq!(
        signal,
        SokrCompletionSignal::Complete,
        "expected Complete, got {signal:?}"
    );
    println!("completion            → {signal:?}");

    // 5. Tear down: deregister invokes the plugin's destroy_fn.
    let result = sokr_deregister_substrate(substrate_id);
    assert_eq!(result, SokrResult::Ok, "deregister failed: {result:?}");
    println!("deregistered sokr-cpu → {result:?}");

    println!("\n✓ end-to-end dispatch → completion cycle succeeded");
}
