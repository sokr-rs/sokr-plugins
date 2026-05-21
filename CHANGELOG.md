# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-05-22

### Fixed

#### sokr-cpu
- Integrated sokr workspace dependency (path-based for development)
- Simplified implementation using sokr types directly
- Fixed capability response substrate_id field
- Fixed dispatch/completion token handling

#### sokr-dispatch-first
- No changes (version bump for release sync)

## [0.1.0] - 2026-05-21

### Added

#### sokr-cpu
- Initial synchronous CPU substrate plugin implementation
- `capability_fn`: Always returns Ok (CPU accepts any computation)
- `dispatch_fn`: Generates unique completion tokens, stores results in fixed-size array
- `completion_fn`: Returns Complete immediately (synchronous execution model)
- `destroy_fn`: Clears all pending result slots
- Full test coverage: dispatch roundtrip, destroy invalidation, multi-computation interleaved polling
- `no_std` support: Compatible with constraint environments

#### sokr-dispatch-first
- Dispatch policy library for substrate selection
- `SubstrateRegistry`: Maintains ordered list of substrate IDs
- `first_capable()`: Returns first registered substrate (foundation for first-capable strategy)
- 9 unit tests validating empty/single/multiple substrate scenarios

#### Benchmarks
- `benches/cpu_roundtrip.rs`: Timing harness for end-to-end latency measurement
- `benches/RESULTS.md`: Baseline documentation for Phase 1 CPU plugin
- Infrastructure for Phase 2 GPU comparison benchmarks

### Documentation
- CLAUDE.md: Project guidelines, architecture decisions, FFI contract invariants
- ARCHITECTURE.md: Plugin categories, dispatch models, capability matrix
- TODO.md: Phase 0-4 roadmap with complete Phase 1 checkoff

---

## Repository

- **Homepage**: [sokr-rs/sokr-plugins](https://github.com/sokr-rs/sokr-plugins)
- **Depends on**: sokr-rs/sokr v0.2.0+
- **License**: MIT OR Apache-2.0
