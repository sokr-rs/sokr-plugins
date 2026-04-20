# sokr-plugins Architecture

> Reference Plugin Implementations for SOKR

---

## Relationship to sokr Core

This repo contains plugins. The contract they implement lives in
[sokr-rs/sokr](https://github.com/sokr-rs/sokr).

```
sokr-rs/sokr           ← the contract (immutable)
sokr-rs/sokr-plugins   ← reference implementations of the contract
```

The core has zero knowledge of this repo. Plugins depend on `sokr`.
`sokr` does not depend on any plugin. This is the correct direction.

---

## Plugin Contract

Every plugin in this repo implements `SokrSubstratePlugin`:

```rust
pub struct SokrSubstratePlugin {
    pub version:        SokrVersion,
    pub capability_fn:  SokrCapabilityFn,
    pub dispatch_fn:    SokrDispatchFn,
    pub completion_fn:  SokrCompletionFn,
    pub destroy_fn:     SokrDestroyFn,
    pub padding:        [u8; 16],
}
```

The three functions map to state machine transitions:

```
capability_fn  →  Can I accept this computation?
dispatch_fn    →  Accept it, begin execution, return completion token
completion_fn  →  Poll token status: Pending / Complete / Failed / TimedOut
```

`destroy_fn` is called exactly once by the core on deregistration.

---

## Plugin Categories

### Substrate Plugins

Accept computations and execute them on hardware.
Each substrate plugin is hardware-specific — it knows about one class
of hardware and nothing else.

**`sokr-cpu`** — The universal fallback. Every machine has a CPU.
Always capable. Synchronous execution. No external dependencies.
This is the first plugin implemented and the one all integration
tests run against.

**`sokr-vulkan`** — GPU compute via Vulkan. Accepts SPIR-V binary
(via `sokr-spirv` IR plugin). Works on NVIDIA, AMD, Intel Arc, and
any Vulkan 1.2+ compatible device. Uses `ash` for raw Vulkan access
and `gpu-allocator` for memory management.

**`sokr-cuda`** — NVIDIA CUDA. Accepts PTX binary (via `sokr-ptx`
IR plugin). Uses `cust` crate. NVIDIA hardware only.

**`sokr-metal`** — Apple Metal compute. macOS and iOS only.
Apple Silicon unified memory path avoids staging buffer overhead.

**`sokr-webgpu`** — wgpu-based substrate. Runs in browser via
WebAssembly or natively via Vulkan/Metal/DX12 backends.

**`sokr-qpu`** — IBM Quantum via Qiskit Runtime REST API.
Accepts OpenQASM 3 IR. Completion model: measurement collapse.
Capability metadata includes qubit count, T1/T2, gate error rates.

**`sokr-neuro`** — Intel Loihi via LAVA framework bridge.
Accepts spike graph IR. Completion model: convergence signal.

**`sokr-photon`** — Lightmatter photonic compute.
Pending public SDK. Stub capability returns `CapabilityDenied` until
SDK ships.

---

### IR Plugins

IR plugins do not execute computations — they translate between
representation formats. They declare which IR formats they accept
at Capability query time.

**`sokr-spirv`** — Validates and reflects SPIR-V binaries.
Magic number: `0x53505256`. Validates via `spirv-val`. Extracts
workgroup size, descriptor bindings, entry point names.

**`sokr-ptx`** — Validates PTX assembly.
Magic string: `.version`. Passes to `sokr-cuda` substrate.

**`sokr-openqasm`** — Validates OpenQASM 3 programs.
Parses header, version-checks, rejects OpenQASM 2.

**`sokr-ir`** — SOKR-native substrate-agnostic computation graph.
Compiles down to SPIR-V, PTX, OpenQASM 3, or spike graph.
The write-once target for full portability. Defined separately
from core — IR spec is versioned independently.

---

### Dispatch Policy Plugins

Dispatch policies decide which substrate handles which computation.
They implement the same vtable as substrate plugins — the core does
not distinguish between them structurally.

**`sokr-dispatch-first`** — Iterates registered substrates in
registration order. Dispatches to the first one returning `Ok` on
Capability. Falls back to `NoCapableSubstrate` if none capable.
Never silently fails.

**`sokr-dispatch-perf`** — Records historical dispatch latency per
substrate per IR format. Routes to the substrate with lowest
observed latency. Falls back to first-capable if no profile exists.
Profiles persist across sessions in a flat binary file.

**`sokr-dispatch-cost`** — Cost-aware routing for cloud contexts.
Routes based on per-substrate pricing metadata. Future.

---

### Language Bindings

Language bindings expose SOKR to non-Rust consumers. They wrap the
C ABI surface of `sokr` in ergonomic language-native APIs.

**`sokr-python`** — PyO3 bindings. Publishes `sokr` on PyPI.
`ComputeContext`, `Kernel`, `CompletionHandle` Python classes.

**`sokr-wasm`** — wasm-bindgen bindings. Publishes `@sokr/webgpu`
on npm. `sokr.init()`, `sokr.dispatch()` async JS API.

---

## Workspace Structure

All crates share workspace-level configuration:

```toml
[workspace.dependencies]
sokr = "0.2"           # pinned to compatible core version

[workspace.lints.rust]
unsafe_code = "deny"   # each plugin controls its own unsafe boundary

[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

Each plugin crate has its own `Cargo.toml` with:
- `version` inherited from workspace
- `[lib] crate-type = ["cdylib", "rlib"]` for loadable + testable
- Hardware-specific dependencies not shared with other plugins

---

## Version Compatibility

All plugins in this workspace track `sokr` core compatibility:

```
sokr-plugins v0.x  →  sokr v0.2.x (Phase 1 core)
sokr-plugins v1.x  →  sokr v1.x   (stable core ABI)
```

A plugin's published version reflects its own maturity, not the
core's version. `sokr-cpu` may be at `v0.3.0` while `sokr-qpu`
is at `v0.1.0` — both compatible with the same core.

---

## Integration Testing

Integration tests in `examples/` and `tests/` load plugins
as external dependencies and exercise full round-trips:

```
Register sokr-cpu → Capability query → Dispatch → Completion
```

These tests run in CI without GPU hardware — CPU substrate is
the universal integration test target.

GPU substrate tests (`sokr-vulkan`, `sokr-cuda`) require hardware
and run only in tagged release CI with appropriate runners.

---

## Non-Goals

- This repo does not define the plugin contract — that is `sokr`
- Plugins here are reference implementations, not authoritative
- Third-party plugins published independently are equally valid
- This repo does not gate what counts as a SOKR plugin

---

*Copyright 2026 The SOKR Project — MIT OR Apache-2.0*
