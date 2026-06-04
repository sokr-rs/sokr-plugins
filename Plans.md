# Plans.md — sokr-plugins Phase 1: sokr-cpu Implementation

## Active Sprint — sokr-cpu Phase 1 STUB (does not meet DoD)

> Status correction (2026-06-04 review): the Phase 1 plugin is a stateless stub.
> The functions compile and the build is green, but they do not satisfy the
> Definition of Done. Markers below reflect actual implemented behavior, not the
> originally-claimed completion.

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 1.1 | Implement `dispatch_fn` + `completion_fn` + `destroy_fn` (consolidated) | (a) dispatch_fn generates unique tokens, stores state (b) completion_fn polls token status correctly (c) destroy_fn clears slots (d) No panics in FFI (e) cargo build --release passes | — | cc:stub [8b8150f] |
| 1.2 | Verify dispatch/completion round-trip with tests | test_1_2_dispatch_completion_roundtrip, test_1_2_multiple_concurrent_dispatches (5 dispatches) | 1.1 | cc:todo |
| 1.3 | Verify destroy_fn invalidates plugin state with tests | test_1_3_destroy_invalidates_tokens, test_1_3_destroy_clears_all_slots (10 dispatches) | 1.1 | cc:todo |
| 1.4 | Full integration test with sokr core | test_1_4_full_lifecycle (capability→dispatch→completion→destroy), test_1_4_multiple_computations (20 computations, interleaved polling) | 1.1, 1.2, 1.3 | cc:todo |

### Known gaps (from 2026-06-04 code review)

- **1.1 dispatch_fn**: returns a fixed token (`handle = 42`) on every call; no
  unique token generation and no result state is stored. DoD (a) not met.
- **1.1 completion_fn**: stateless constant check (`handle == 42`); never frees
  the slot, and double-poll keeps returning `Complete` instead of `Failed`.
  DoD (b) not met.
- **1.1 destroy_fn**: empty no-op; the plugin remains usable after teardown, so
  post-destroy queries do not return `Failed`. DoD (c) not met.
- **1.2 / 1.3 / 1.4 tests**: the named tests do not exist in the live tree.
  `tests/plugin_contract.rs` only constructs structs and asserts `size_of` /
  enum values — it never invokes capability/dispatch/completion/destroy. No
  behavioral round-trip, destroy, or full-lifecycle coverage exists.
- **Benchmark** (`benches/cpu_roundtrip.rs`): is an orphan target (not declared
  in any Cargo.toml) and `measure_roundtrip()` measures nothing. Compile errors
  were fixed on 2026-06-04, but the harness still does not exercise the plugin.

## Backlog

- Replace the fixed-token stub with real per-token state storage (slot map keyed
  by a unique `completion_token`), slot freeing on `Complete`, and `destroy_fn`
  cleanup that invalidates outstanding tokens.
- Write the behavioral tests claimed in 1.2 / 1.3 / 1.4 against the live plugin.
- Declare the benchmark as a real target and have it drive the plugin path.

See TODO.md for Phase 2+ (GPU, CUDA, Metal, etc.).

## Archive

(Completed tasks moved here as sprints finish.)

---

**Harness Integration**: Use `/harness-work all` to execute all tasks in order
(4 tasks → Breezing mode auto-selected).
