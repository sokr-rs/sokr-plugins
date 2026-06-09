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
- [x] **`no_std` support.** `#![cfg_attr(not(feature = "std"), no_std)]` with a
      default-on `std` feature (`std = ["sokr/std"]`); `sokr` is pulled with
      `default-features = false`. The plugin builds with
      `cargo build -p sokr-cpu --no-default-features`, enforced by a CI job.
      The std-based tests/example/benchmark are separate targets that keep the
      default `std` feature.
- [x] **Real benchmark.** `benches/cpu_roundtrip.rs` (declared `[[bench]]`,
      `harness = false`) drives the actual `dispatch → completion` roundtrip
      through `sokr` core. Replaces the repo-root orphan that measured nothing.

## Follow-ups

- [ ] 🟡 **Propose `context: *mut c_void` to sokr core.** Would remove the
      static completion table and let the plugin learn its own `substrate_id`.
      The proposal is written up in `ARCHITECTURE.md` §5; **filing it is an
      upstream task on `sokr-rs/sokr`**, not a change this crate can make. Left
      open as an external tracking item.

## Resolved (won't-do, with rationale)

- **Configurable table capacity** — `COMPLETION_TABLE_CAPACITY` stays a
  compile-time constant (256). This is a no-op reference substrate; a runtime
  knob or cargo feature would be unjustified complexity (YAGNI). Revisit only if
  a concrete workload needs more than 256 dispatched-but-unpolled tokens at once.
