# sokr-cpu

**Synchronous CPU substrate plugin for SOKR** — reference implementation proving the core contract works on general-purpose processors.

## Features

- ✅ Always capable — CPU accepts any computation
- ✅ Synchronous execution — dispatch blocks until complete
- ✅ Zero dependencies — pure Rust, just `std`
- ✅ FFI-safe — compatible with SOKR v0.2.0+ core contract
- ✅ Multi-slot dispatch — up to 1024 concurrent completion tokens
- ✅ Immediate completion — no async overhead

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
sokr-cpu = "0.1"
```

Link the plugin with SOKR core:

```rust
extern "C" {
    static CPU_PLUGIN: SokrSubstratePlugin;
}

// Register with sokr core at startup
sokr_core::register(&CPU_PLUGIN)?;
```

## Architecture

The CPU plugin implements the SOKR substrate contract:

- **`capability_fn`**: Always returns `Ok` with `substrate_id=0`, `estimated_latency_ns=0`
- **`dispatch_fn`**: Stores computation request, returns unique `completion_token` (always handle=42 for synchronous model)
- **`completion_fn`**: Immediately returns `Complete` — CPU computation is synchronous
- **`destroy_fn`**: Clears all pending completion slots

## Completion Model

CPU plugin executes synchronously on the calling thread. When `dispatch_fn` returns, the computation has already executed. The `completion_fn` always returns `Complete` immediately.

## Version

- **sokr-cpu**: v0.1.1
- **Depends on**: sokr ≥ 0.3.0

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## See Also

- [sokr-rs/sokr](https://github.com/sokr-rs/sokr) — SOKR compute abstraction core
- [sokr-dispatch-first](../sokr-dispatch-first) — First-capable dispatch policy
