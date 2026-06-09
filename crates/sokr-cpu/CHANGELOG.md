# Changelog — sokr-cpu

All notable changes to sokr-cpu will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-06-09

This release replaces the placeholder dispatch path with a real, contract-faithful
implementation and proves the SOKR loop end to end.

### Changed
- **Capability now uses disclaim semantics.** `capability_fn` claims only
  `ir_format == "sokr-noop"` (returns `Ok`) and returns `CapabilityDenied` for
  every other IR. It no longer falsely claims all computations, so it is no
  longer a "capability lie". This is a behavioral change from 0.1.x, hence the
  minor version bump.
- Plugin entry-point functions (`capability`/`dispatch`/`completion`/`destroy`)
  are no longer `#[no_mangle]`; they are reached exclusively through the
  `CPU_PLUGIN` vtable. `CPU_PLUGIN` remains the crate's single exported C
  symbol. This removes the generic-symbol collision risk that previously kept
  the crate's tests out of CI.

### Added
- Real synchronous completion table (single-threaded, `UnsafeCell`-backed):
  `dispatch_fn` issues a unique non-zero completion token per call and records
  a `Complete` entry; `completion_fn` reports the status and **consumes** the
  token, so a second poll disclaims with `NotFound`; `destroy_fn` clears every
  outstanding slot.
- `examples/cpu_end_to_end.rs` — the full
  `register → capability → dispatch → completion → deregister` cycle through the
  real `sokr` core. This is the previously-missing proof artifact.
- Behavioral integration tests in `tests/plugin_contract.rs` that drive the
  plugin through `sokr` core (lifecycle, multi-dispatch, destroy invalidation,
  capability disclaim), replacing the previous `size_of`-only assertions.
- `no_std` support: `#![cfg_attr(not(feature = "std"), no_std)]` with a
  default-on `std` feature. The bare plugin builds with
  `cargo build -p sokr-cpu --no-default-features` (enforced by a CI job); the
  std-based tests, example, and benchmark keep the default feature.
- `benches/cpu_roundtrip.rs` (declared `[[bench]]`, `harness = false`) — a real
  benchmark of the `dispatch → completion` roundtrip through `sokr` core,
  replacing a repo-root orphan that measured nothing.
- `ARCHITECTURE.md` and `TODO.md` for the crate.

### Fixed
- `destroy_fn` is a real teardown (was an empty no-op).
- README and CHANGELOG no longer describe behavior the code never had.

### Removed
- `examples/test_capability.rs`, which redefined private copies of the ABI
  types and tested a local function rather than the plugin. Superseded by
  `examples/cpu_end_to_end.rs`.
- The repo-root `benches/` orphan (`cpu_roundtrip.rs` + stale `RESULTS.md`),
  which was declared by no crate and measured nothing. Replaced by the crate's
  declared benchmark.

## [0.1.2] - 2026-05-22

### Changed
- Aligned with sokr core `0.3.0`: the vtable gained a `substrate_id` field and
  `padding` is `[u8; 8]`. The descriptor advertises `SokrVersion::CURRENT`.

### Added
- Initial `tests/plugin_contract.rs` (struct/enum ABI assertions). These
  asserted type layout only and did not yet exercise the plugin functions.

## [0.1.1] - 2026-05-22

### Fixed
- Integrated sokr workspace dependency (path-based for development)
- Simplified implementation using sokr types directly
- Fixed capability response substrate_id field
- Fixed dispatch/completion token handling
- Added complete package metadata (description, license, authors)

## [0.1.0] - 2026-05-21

### Added
- Initial synchronous CPU substrate plugin implementation
- `capability_fn`: Always returns Ok (CPU accepts any computation)
- `dispatch_fn`: Generates unique completion tokens, stores results in fixed-size array
- `completion_fn`: Returns Complete immediately (synchronous execution model)
- `destroy_fn`: Clears all pending result slots
- Full test coverage: dispatch roundtrip, destroy invalidation, multi-computation interleaved polling
- Static `CPU_PLUGIN` descriptor for SOKR core registration

---

**Repository**: [sokr-rs/sokr-plugins](https://github.com/sokr-rs/sokr-plugins)  
**License**: MIT OR Apache-2.0
