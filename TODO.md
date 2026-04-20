# sokr-plugins — Development TODO

> Reference Plugin Implementations for SOKR
> Last Updated: 2026-04-20
> Legend: 🔴 Critical path · 🟡 Important · 🟢 Nice-to-have

---

## Vision

Working reference implementations for every SOKR plugin category —
substrate, IR, dispatch policy, language binding — proving the core
contract works across GPU, CPU, QPU, neuromorphic, and photonic hardware.

Depends on: [sokr-rs/sokr](https://github.com/sokr-rs/sokr) `v0.2.0`

---

## Phase 0 — Workspace Setup
> Scaffold the repo. Nothing runs yet.

- [ ] 🔴 Create `sokr-rs/sokr-plugins` GitHub repo
- [ ] 🔴 Initialise Cargo workspace
  - [ ] Root `Cargo.toml` with `[workspace]` members
  - [ ] `resolver = "2"`
  - [ ] `[workspace.dependencies]` — pin `sokr = "0.2"` when available
  - [ ] `[workspace.lints]` — shared lint config
  - [ ] `[profile.dev] panic = "abort"`
  - [ ] `[profile.release] panic = "abort"`
- [ ] 🔴 Move from `sokr-rs/sokr`:
  - [ ] `crates/sokr-cpu/` → this repo
  - [ ] `crates/sokr-dispatch-first/` → this repo
- [ ] 🔴 LICENSE-MIT, LICENSE-APACHE, README, ARCHITECTURE, TODO
- [ ] 🔴 `.github/workflows/ci.yml`
  - [ ] `cargo check --workspace`
  - [ ] `cargo test --workspace`
  - [ ] `cargo clippy -- -D warnings`
  - [ ] `cargo fmt --check`
- [ ] 🔴 `.pre-commit-config.yaml` — fmt, check, clippy
- [ ] 🟡 Dependabot — weekly cargo and github-actions updates

---

## Phase 1 — CPU Substrate + First Dispatch Policy `v0.1.0`
> Proves the core contract end-to-end. No GPU required.

### sokr-cpu
- [ ] 🔴 Scaffold `crates/sokr-cpu/`
  - [ ] `Cargo.toml` — `sokr` dependency, `crate-type = ["cdylib", "rlib"]`
  - [ ] `src/lib.rs` — `SokrSubstratePlugin` static instance
  - [ ] `src/capability.rs` — always returns `Ok`
  - [ ] `src/dispatch.rs` — synchronous CPU execution
  - [ ] `src/completion.rs` — immediate `Complete`
- [ ] 🔴 Implement `capability_fn`
  - [ ] Always return `SokrResult::Ok` — CPU accepts any computation
  - [ ] Set `estimated_latency_ns = 0`
  - [ ] Unit test: any query returns capable
  - [ ] Unit test: null query pointer returns `InvalidInput`
- [ ] 🔴 Implement `dispatch_fn`
  - [ ] Accept raw byte payload as computation unit
  - [ ] Execute synchronously on calling thread
  - [ ] Store result keyed by unique `completion_token`
  - [ ] Unit test: dispatch stores result retrievable via completion
  - [ ] Unit test: two dispatches get distinct tokens
- [ ] 🔴 Implement `completion_fn`
  - [ ] Return `Complete` immediately — synchronous dispatch
  - [ ] Free result slot after `Complete`
  - [ ] Unit test: returns `Complete` after dispatch
  - [ ] Unit test: returns `Failed` for unknown token
  - [ ] Unit test: double-poll after `Complete` returns `Failed`
- [ ] 🔴 Implement `destroy_fn`
  - [ ] Clean up all pending completion slots
  - [ ] Unit test: destroy called, then plugin unusable
- [ ] 🔴 Integration test — full round-trip
  - [ ] Register with `sokr` core
  - [ ] Capability → assert `Ok`
  - [ ] Dispatch → assert `Ok` with valid token
  - [ ] Completion → assert `Complete`
  - [ ] Deregister → assert `Ok`

### sokr-dispatch-first
- [ ] 🔴 Scaffold `crates/sokr-dispatch-first/`
  - [ ] `Cargo.toml` — `sokr` dependency
  - [ ] `src/lib.rs` — dispatch policy implementation
- [ ] 🔴 Implement first-capable strategy
  - [ ] Iterate registered substrates in registration order
  - [ ] Dispatch to first returning `Ok` on capability
  - [ ] Unit test: single substrate — dispatches to it
  - [ ] Unit test: multiple — dispatches to first capable
  - [ ] Unit test: none capable — returns `NoCapableSubstrate`
  - [ ] Unit test: zero substrates — returns `NoCapableSubstrate`

### Benchmarks (Phase 1)
- [ ] 🟡 `benches/cpu_roundtrip.rs`
  - [ ] Measure: register → capability → dispatch → completion
  - [ ] Record baseline in `benches/RESULTS.md`

---

## Phase 2 — First Real GPU Substrate `v0.2.0`
> SOKR runs real GPU workloads. The plugin model is proven on hardware.

### sokr-spirv (IR Plugin)
- [ ] 🔴 Scaffold `crates/sokr-spirv/`
  - [ ] `Cargo.toml` — `sokr`, `spirv-tools`
  - [ ] `src/lib.rs`, `src/validate.rs`, `src/reflect.rs`
- [ ] 🔴 Accept SPIR-V binary
  - [ ] IR format identifier: `SOKR_IR_SPIRV = 0x53505256`
  - [ ] Validate magic number `0x07230203`
  - [ ] Return `InvalidIR` if absent or zero-length
  - [ ] Unit test: valid SPIR-V accepted
  - [ ] Unit test: invalid magic rejected
- [ ] 🔴 Validate via `spirv-val`
  - [ ] Unit test: valid shader passes
  - [ ] Unit test: invalid shader returns `InvalidIR`
- [ ] 🟡 SPIR-V reflection
  - [ ] Extract `LocalSize`, descriptor bindings, entry point names
  - [ ] Unit test: reflection matches known shader metadata

### sokr-vulkan (Substrate Plugin)
- [ ] 🔴 Scaffold `crates/sokr-vulkan/`
  - [ ] `Cargo.toml` — `sokr`, `ash`, `gpu-allocator`
  - [ ] `src/lib.rs`, `src/device.rs`, `src/pipeline.rs`
  - [ ] `src/memory.rs`, `src/dispatch.rs`, `src/completion.rs`
- [ ] 🔴 Implement `capability_fn`
  - [ ] Enumerate via `vkEnumeratePhysicalDevices`
  - [ ] Check `VK_QUEUE_COMPUTE_BIT`
  - [ ] Unit test: mock device with compute queue → capable
  - [ ] Unit test: mock device without → denied
- [ ] 🔴 Implement `dispatch_fn`
  - [ ] `VkShaderModule` from SPIR-V binary
  - [ ] `VkComputePipeline` creation
  - [ ] `vkCmdDispatch` + fence submission
  - [ ] Unit test: valid SPIR-V dispatches successfully
- [ ] 🔴 Implement `completion_fn`
  - [ ] `vkGetFenceStatus` poll
  - [ ] `vkWaitForFences` with `timeout_ns`
  - [ ] Unit test: completion returns `Complete` after dispatch
  - [ ] Unit test: zero timeout returns `Pending` or `Complete`
- [ ] 🔴 Multi-device support
  - [ ] Register each physical device as separate substrate
  - [ ] Unit test: two devices register as two substrate IDs
- [ ] 🟡 Memory management
  - [ ] Staging → device-local → readback path
  - [ ] `gpu-allocator` for sub-allocation
  - [ ] Unit test: data round-trips upload → compute → readback
- [ ] 🟡 Pipeline caching
  - [ ] `VkPipelineCache` reuse on same SPIR-V hash
  - [ ] Benchmark: cache hit vs miss latency

### Benchmarks (Phase 2)
- [ ] 🔴 CPU vs Vulkan baseline
  - [ ] Array addition — 1M elements
  - [ ] Matrix multiply — 512×512
  - [ ] Record in `benches/RESULTS.md`
- [ ] 🟡 SOKR overhead vs raw `ash` dispatch
  - [ ] Target: < 5% overhead

---

## Phase 3 — Ecosystem `v0.3.0`
> CUDA, Metal, Python, WebGPU, performance dispatch.

### sokr-cuda
- [ ] 🟡 `sokr-ptx` IR plugin — validate PTX magic string `.version`
- [ ] 🟡 CUDA substrate via `cust`
  - [ ] `cuDeviceGetCount` enumeration
  - [ ] `cuModuleLoadData` + `cuLaunchKernel` dispatch
  - [ ] CUDA stream → completion token mapping
  - [ ] Unit test: PTX vector addition on CUDA device

### sokr-metal
- [ ] 🟡 macOS/iOS only (`cfg(target_os = "macos")`)
  - [ ] `MTLComputeCommandEncoder` dispatch
  - [ ] Apple Silicon unified memory path
  - [ ] Unit test: Metal compute on Apple Silicon

### sokr-python
- [ ] 🟡 PyO3 bindings
  - [ ] `ComputeContext`, `Kernel`, `CompletionHandle` Python classes
  - [ ] `maturin` build + PyPI publish pipeline

### sokr-webgpu
- [ ] 🟡 `wgpu` substrate — Vulkan/Metal/DX12/WebGPU backend
  - [ ] WASM compilation target
  - [ ] `wasm-bindgen` JavaScript API
  - [ ] Publish as `@sokr/webgpu` on npm

### sokr-dispatch-perf
- [ ] 🟡 Per-substrate latency profiling
  - [ ] Fixed-size ring buffer — no heap allocation
  - [ ] Profile persistence to flat binary file
  - [ ] Unit test: routes to faster substrate after profiling

---

## Phase 4 — Future Substrates
> QPU, Neuromorphic, Photonic. The horizon.

### sokr-qpu
- [ ] 🟢 `sokr-openqasm` IR plugin — OpenQASM 3 validation
- [ ] 🟢 IBM Quantum backend via Qiskit Runtime REST API
  - [ ] Capability: qubit count, gate set, T1/T2
  - [ ] Dispatch: submit job, return job ID as token
  - [ ] Completion: poll job status endpoint

### sokr-neuro
- [ ] 🟢 Spike graph IR definition and validator
- [ ] 🟢 Intel Loihi via LAVA framework bridge
  - [ ] Completion: convergence signal
  - [ ] Streaming partial results before convergence

### sokr-photon
- [ ] 🟢 Optical circuit IR definition and validator
- [ ] 🟢 Lightmatter backend (pending public SDK)
  - [ ] Stub capability returns `CapabilityDenied` until SDK ships

### sokr-ir (Sovereign IR)
- [ ] 🟢 Substrate-agnostic computation graph specification
- [ ] 🟢 Compilers: SOKR-IR → SPIR-V, PTX, OpenQASM 3, spike graph

---

## Contribution Policy

- All contributions require DCO sign-off (`Signed-off-by:` in commit)
- No RFC required for plugins — plugins are sovereign
- Open a Plugin Proposal in `sokr-rs/sokr` before large new plugins
- Copyright assigned to **The SOKR Project**
- License: **MIT OR Apache-2.0** — no exceptions

---

*Copyright 2026 The SOKR Project — MIT OR Apache-2.0*
