# Plans.md — sokr-plugins Phase 1: sokr-cpu Implementation

## Active Sprint — sokr-cpu Phase 1 (DONE, v0.2.0)

> Completion (2026-06-09): the stub identified in the 2026-06-04 review has been
> replaced with a real, contract-faithful implementation. All Phase 1 DoD items
> are met and the SOKR loop is proven end to end (`cargo run --example
> cpu_end_to_end`). Released as `sokr-cpu` v0.2.0 (minor bump: capability moved
> from claim-all to `sokr-noop` disclaim semantics).

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 1.1 | Implement `dispatch_fn` + `completion_fn` + `destroy_fn` (consolidated) | (a) dispatch_fn generates unique tokens, stores state (b) completion_fn polls token status correctly (c) destroy_fn clears slots (d) No panics in FFI (e) cargo build --release passes | — | cc:done |
| 1.2 | Verify dispatch/completion round-trip with tests | test_1_2_dispatch_completion_roundtrip, test_1_2_multiple_concurrent_dispatches (5 dispatches) | 1.1 | cc:done |
| 1.3 | Verify destroy_fn invalidates plugin state with tests | test_1_3_destroy_invalidates_tokens, test_1_3_destroy_clears_all_slots (10 dispatches) | 1.1 | cc:done |
| 1.4 | Full integration test with sokr core | test_1_4_full_lifecycle (capability→dispatch→completion→destroy), test_1_4_multiple_computations (20 computations, interleaved polling) | 1.1, 1.2, 1.3 | cc:done |

### Resolution (2026-06-09)

- **1.1 dispatch_fn**: now issues a unique non-zero token per call from a
  monotonic allocator and stores a `Complete` entry in a fixed-capacity static
  table. DoD (a) met.
- **1.1 completion_fn**: looks the token up, reports its status, and **consumes**
  the slot; a second poll disclaims with `NotFound` (the core surfaces
  `Failed`). DoD (b) met.
- **1.1 destroy_fn**: clears every outstanding slot and resets the allocator, so
  post-destroy queries are no longer recognized. DoD (c) met.
- **1.2 / 1.3 / 1.4 tests**: the named tests now exist in
  `crates/sokr-cpu/tests/plugin_contract.rs` and drive the plugin through the
  real `sokr` core (serialized to honor the single-threaded invariant).
- **Capability**: claims only `ir_format == "sokr-noop"`, disclaiming everything
  else with `CapabilityDenied` (no capability lie).
- **CI**: `sokr-cpu` tests re-enabled (no longer `--exclude`d) and clippy now
  lints `--all-targets`.

## Backlog

- **Benchmark** (`benches/cpu_roundtrip.rs`): still an orphan target (not
  declared in any Cargo.toml) and `measure_roundtrip()` measures nothing.
  Declare it as a real target driving the live dispatch path, or delete it.
- Propose an additive `context: *mut c_void` vtable field to sokr core so the
  plugin can drop the static completion table and learn its own `substrate_id`
  (see `crates/sokr-cpu/ARCHITECTURE.md` §5). Upstream change.
- `no_std` support for `sokr-cpu` (`--no-default-features`); blocker is the
  std-based integration tests (see `crates/sokr-cpu/TODO.md`).

See TODO.md for Phase 2+ (GPU, CUDA, Metal, etc.).

## Archive

(Completed tasks moved here as sprints finish.)

---

**Harness Integration**: Use `/harness-work all` to execute all tasks in order
(4 tasks → Breezing mode auto-selected).
