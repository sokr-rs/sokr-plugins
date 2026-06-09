# sokr-cpu — TODO

Atomic task breakdown for `sokr-cpu`. The "first green end-to-end" milestone is
**done** as of v0.2.0; remaining items are follow-ups.

## Milestone: first green end-to-end (v0.2.0) — DONE

- [x] `capability_fn` claims `sokr-noop`, disclaims all else with
      `CapabilityDenied` (never `InvalidIR`/`InvalidInput`).
- [x] `dispatch_fn` issues a unique non-zero completion token per call and
      stores a `Complete` entry in a static table.
- [x] `completion_fn` reports the stored status and consumes the token; a second
      poll disclaims with `NotFound` (no `*signal` write on the disclaim path).
- [x] `destroy_fn` clears all outstanding slots; never panics.
- [x] Static `CPU_PLUGIN` descriptor advertises `SokrVersion::CURRENT`,
      `substrate_id = 0`, `padding: [u8; 8]`.
- [x] Behavioral integration tests through the real `sokr` core:
  - [x] `test_1_2_dispatch_completion_roundtrip`
  - [x] `test_1_2_multiple_concurrent_dispatches` (5 outstanding tokens)
  - [x] `test_1_3_destroy_invalidates_tokens`
  - [x] `test_1_3_destroy_clears_all_slots` (10 dispatches)
  - [x] `test_1_4_full_lifecycle` (capability → dispatch → completion → destroy)
  - [x] `test_1_4_multiple_computations` (20 computations, interleaved polling)
  - [x] `test_capability_accepts_noop_disclaims_others`
- [x] `examples/cpu_end_to_end.rs` proves the loop via `sokr` core.
- [x] CHANGELOG `0.2.0` entry; README and CHANGELOG match the code.
- [x] CI runs the crate's tests (no longer excluded) and lints all targets.

## Follow-ups

- [ ] 🟡 **Propose `context: *mut c_void` to sokr core.** Removes the static
      completion table and lets the plugin learn its own `substrate_id`. Tracked
      in `ARCHITECTURE.md` §5. Upstream change — not a plugin task.
- [ ] 🟢 **`no_std` support.** CLAUDE.md targets `--no-default-features` for
      `sokr-cpu`. The implementation already uses only `core`-level types; the
      blocker is the std-based integration tests. Split: `#![no_std]` lib +
      std-gated tests, or move the std E2E into a separate harness crate.
- [ ] 🟢 **Wire up the benchmark.** `benches/cpu_roundtrip.rs` (repo root) is an
      orphan target that measures nothing; declare it and have it drive the real
      dispatch path, or delete it.
- [ ] 🟢 **Configurable table capacity.** `COMPLETION_TABLE_CAPACITY` is a
      compile-time constant; revisit if a real workload needs more headroom.
