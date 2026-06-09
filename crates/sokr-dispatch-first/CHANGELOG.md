# Changelog — sokr-dispatch-first

All notable changes to sokr-dispatch-first will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2] - 2026-05-22

### Changed
- Version bump for the coordinated Phase 1 release. No functional changes to
  this crate: `sokr-dispatch-first` is standalone and does not depend on `sokr`,
  so the sokr 0.3.0 compatibility work in that release cycle did not affect it.

## [0.1.1] - 2026-05-22

### Changed
- Version bump for coordinated Phase 1 release with sokr-cpu

## [0.1.0] - 2026-05-21

### Added
- Initial dispatch policy library implementation
- `SubstrateRegistry`: Maintains ordered list of substrate IDs
- `first_capable()`: Returns first registered substrate
- `register()`: Add substrate to registry
- `substrate_count()`: Query registry size
- `get_substrate(index)`: Retrieve substrate at index
- `clear()`: Remove all substrates
- 9 unit tests validating empty/single/multiple substrate scenarios
- Full package metadata and documentation

---

**Repository**: [sokr-rs/sokr-plugins](https://github.com/sokr-rs/sokr-plugins)  
**License**: MIT OR Apache-2.0
