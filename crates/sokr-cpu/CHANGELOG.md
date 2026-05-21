# Changelog — sokr-cpu

All notable changes to sokr-cpu will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
