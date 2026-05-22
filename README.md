# sokr-plugins — Reference Plugin Implementations for SOKR

> Everything that is not the core.

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

---

## What is this repo?

This is the reference plugin workspace for
[SOKR](https://github.com/sokr-rs/sokr) — the Sovereign Open Kernel
Runtime.

SOKR's core (`sokr`) is a single immutable crate that exposes a
three-function C ABI contract. This repo contains the reference
implementations of substrate plugins, IR plugins, and dispatch
policies that prove the contract works and give users a working
starting point.

---

## What is a Plugin?

Any crate that implements `SokrSubstratePlugin` from the `sokr` crate
is a valid SOKR plugin. No registration, no permission, no ceremony.

```toml
[dependencies]
sokr = "0.3"
```

```rust
use sokr::{SokrSubstratePlugin, SokrResult, SokrVersion};

extern "C" fn my_capability(...) -> SokrResult { ... }
extern "C" fn my_dispatch(...) -> SokrResult { ... }
extern "C" fn my_completion(...) -> SokrResult { ... }
extern "C" fn my_destroy() { ... }

pub static MY_PLUGIN: SokrSubstratePlugin = SokrSubstratePlugin {
    version: SokrVersion::CURRENT,
    capability_fn: my_capability,
    dispatch_fn: my_dispatch,
    completion_fn: my_completion,
    destroy_fn: my_destroy,
    padding: [0; 16],
};
```

Third-party plugins can live anywhere — no need to be in this repo.

---

## Workspace Contents

### Available Plugins (Shipped & Tested)

#### Substrate

| Crate | Target | Integration Test |
|---|---|---|
| `sokr-cpu` | CPU — universal fallback, synchronous | [`crates/sokr-cpu/tests/plugin_contract.rs`](crates/sokr-cpu/tests/plugin_contract.rs) |

#### Dispatch Policy

| Crate | Strategy | Shipped Version |
|---|---|---|
| `sokr-dispatch-first` | First capable substrate wins | v0.1.2 |

---

### Planned Plugins (Future Phases)

#### Substrate Plugins — Phase 2+

| Crate | Target | Phase |
|---|---|---|
| `sokr-vulkan` | Vulkan-compatible GPUs (NVIDIA, AMD, Intel) | Phase 2 |
| `sokr-cuda` | NVIDIA CUDA | Phase 3 |
| `sokr-metal` | Apple Metal | Phase 3 |
| `sokr-webgpu` | Browser / Edge via wgpu | Phase 3 |
| `sokr-qpu` | Quantum processors (IBM Quantum via OpenQASM 3) | Future |
| `sokr-neuro` | Neuromorphic (Intel Loihi via LAVA) | Future |
| `sokr-photon` | Photonic compute (Lightmatter) | Future |

#### IR Plugins — Phase 2+

| Crate | IR Format | Phase |
|---|---|---|
| `sokr-spirv` | SPIR-V | Phase 2 |
| `sokr-ptx` | PTX (NVIDIA) | Phase 3 |
| `sokr-openqasm` | OpenQASM 3 | Future |
| `sokr-ir` | SOKR-native substrate-agnostic IR | Future |

#### Dispatch Policies — Phase 3+

| Crate | Strategy | Phase |
|---|---|---|
| `sokr-dispatch-perf` | Performance-profile-aware routing | Phase 3 |
| `sokr-dispatch-cost` | Cost-aware routing (cloud context) | Future |

#### Language Bindings — Phase 3

| Crate | Language | Phase |
|---|---|---|
| `sokr-python` | Python (PyO3) | Phase 3 |
| `sokr-wasm` | JavaScript / Browser (wasm-bindgen) | Phase 3 |

---

## Repository Structure

```
sokr-plugins/
├── crates/                                  (workspace members)
│   ├── sokr-cpu/                   ✓ shipped (v0.1.2)
│   ├── sokr-dispatch-first/        ✓ shipped (v0.1.2)
│   │
│   ├── sokr-spirv/                 Phase 2
│   ├── sokr-vulkan/                Phase 2
│   ├── sokr-cuda/                  Phase 3
│   ├── sokr-metal/                 Phase 3
│   ├── sokr-python/                Phase 3
│   ├── sokr-webgpu/                Phase 3
│   ├── sokr-dispatch-perf/         Phase 3
│   ├── sokr-qpu/                   Future
│   ├── sokr-neuro/                 Future
│   ├── sokr-photon/                Future
│   ├── sokr-ir/                    Future
│   ├── sokr-ptx/                   Future
│   └── sokr-openqasm/              Future
├── tests/                           (integration tests for sokr core)
├── examples/
├── benches/
└── Cargo.toml                       (workspace root, sokr = "0.3")
```

---

## Contributing a Plugin

1. Open a Plugin Proposal issue in
   [sokr-rs/sokr](https://github.com/sokr-rs/sokr/issues/new?template=plugin_proposal.md)
2. No RFC required — plugins are sovereign
3. Implement `SokrSubstratePlugin` correctly
4. Tests must pass: `cargo test -p <your-plugin>`
5. PR to this repo, or publish independently to crates.io

Third-party plugins published independently are equally valid.
The `sokr-` crate name prefix is conventional, not enforced.

---

## License

Licensed under either of

- MIT license ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your discretion.

---

*Copyright 2026 The SOKR Project — MIT OR Apache-2.0*
