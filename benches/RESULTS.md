# sokr-plugins Benchmark Results

## Phase 1 Baseline — CPU Substrate Roundtrip

**Benchmark**: `benches/cpu_roundtrip.rs`
**Metric**: End-to-end latency of synchronous CPU substrate (dispatch → completion cycle)
**Date**: 2026-05-21
**Environment**: Development build (panic=abort), x86_64-unknown-linux-gnu

### CPU Plugin (Synchronous)

The CPU plugin executes synchronously on the calling thread:
- `dispatch_fn` generates unique token, stores result slot immediately
- `completion_fn` returns `Complete` on first poll (no async wait)
- No I/O, no context switches, no allocation per dispatch

**Expected behavior**: Completion returns instantly after dispatch.

### Measurement Infrastructure

Full integration benchmarks require sokr core to be testable. Current Phase 1 baseline captures:
- Token generation (AtomicU64::fetch_add)
- Result slot allocation (fixed-size array)
- Status polling (linear scan for token match)

### Future Baselines

When Phase 2 (GPU substrates) ships, we will measure:
- CPU vs. Vulkan latency on identical workload
- SOKR overhead vs. raw `ash` dispatch
- Memory allocation patterns (staging → device-local → readback)

---

*See TODO.md Phase 1 for completion tracking.*
