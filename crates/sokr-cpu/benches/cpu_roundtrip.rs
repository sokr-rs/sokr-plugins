//! Benchmark: synchronous CPU substrate dispatch → completion roundtrip.
//!
//! Measures the steady-state per-computation cost of the plugin's hot path
//! through the real `sokr` core: `dispatch` (issue token) followed by
//! `completion` (report + consume token). Registration is one-time setup and is
//! kept outside the measured loop.
//!
//! Run with: `cargo bench -p sokr-cpu`
//!
//! Declared with `harness = false`, so this is a plain `main` rather than a
//! libtest bench harness. It is single-threaded, matching the core's pre-1.0
//! single-threaded invariant.

use std::time::Instant;

use sokr::ffi::{
    sokr_completion, sokr_deregister_substrate, sokr_dispatch, sokr_register_substrate,
};
use sokr::{
    SokrCompletionQuery, SokrCompletionSignal, SokrCompletionToken, SokrComputationId,
    SokrDispatchRequest, SokrDispatchResponse, SokrResult,
};
use sokr_cpu::CPU_PLUGIN;

const WARMUP: usize = 1_000;
const ITERATIONS: usize = 100_000;

fn main() {
    // One-time setup: register the substrate and capture its core-assigned id.
    let mut substrate_id: u64 = 0;
    // SAFETY: both pointers are valid and non-null.
    let result = unsafe { sokr_register_substrate(&CPU_PLUGIN, &mut substrate_id) };
    assert_eq!(result, SokrResult::Ok, "registration failed: {result:?}");

    let ir_data = b"bench";

    for _ in 0..WARMUP {
        roundtrip(substrate_id, ir_data);
    }

    let mut samples_ns: Vec<u128> = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        roundtrip(substrate_id, ir_data);
        samples_ns.push(start.elapsed().as_nanos());
    }

    // Teardown.
    let result = sokr_deregister_substrate(substrate_id);
    assert_eq!(result, SokrResult::Ok, "deregister failed: {result:?}");

    report(&mut samples_ns);
}

/// One measured unit of work: dispatch a no-op and poll it to completion.
#[inline]
fn roundtrip(substrate_id: u64, ir_data: &[u8]) {
    let request = SokrDispatchRequest {
        computation_id: SokrComputationId { high: 0, low: 0 },
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
    debug_assert_eq!(result, SokrResult::Ok);

    let query = SokrCompletionQuery {
        completion_token: dispatch.completion_token,
        timeout_ns: 0,
        padding: [0; 8],
    };
    let mut signal = SokrCompletionSignal::Pending;
    // SAFETY: `query`/`signal` outlive the call and are valid, non-null.
    let result = unsafe { sokr_completion(&query, &mut signal) };
    debug_assert_eq!(result, SokrResult::Ok);
    debug_assert_eq!(signal, SokrCompletionSignal::Complete);
}

/// Print min / median / mean / p99 / max over the collected samples.
fn report(samples_ns: &mut [u128]) {
    samples_ns.sort_unstable();
    let n = samples_ns.len();
    let min = samples_ns[0];
    let max = samples_ns[n - 1];
    let median = samples_ns[n / 2];
    let p99 = samples_ns[(n * 99) / 100];
    let mean = samples_ns.iter().sum::<u128>() / n as u128;

    println!("=== sokr-cpu dispatch → completion roundtrip ===");
    println!("samples: {n} (after {WARMUP} warmup)\n");
    println!("  min:    {min} ns");
    println!("  median: {median} ns");
    println!("  mean:   {mean} ns");
    println!("  p99:    {p99} ns");
    println!("  max:    {max} ns");
}
