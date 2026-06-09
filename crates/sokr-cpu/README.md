# sokr-cpu

**Synchronous CPU substrate plugin for SOKR** — the reference implementation
that proves the core `capability → dispatch → completion` contract closes end to
end on a general-purpose processor. No GPU or accelerator required.

## Features

- ✅ **Disclaim-correct capability** — claims only the `sokr-noop` IR and
  disclaims everything else with `CapabilityDenied`. It does not lie about
  what it can do.
- ✅ **Synchronous execution** — `dispatch_fn` runs the (no-op) computation
  inline; the result is ready before it returns.
- ✅ **Real completion table** — issues a unique non-zero token per dispatch,
  consumes it on the first terminal poll, and clears all tokens on destroy.
- ✅ **FFI-safe** — implements the `sokr` 0.3 `SokrSubstratePlugin` C ABI
  exactly (`repr(C)`, `padding: [u8; 8]`, `SokrVersion::CURRENT`).
- ✅ **`no_std`** — the plugin uses only `core`-level types; build the bare
  no_std library with `cargo build -p sokr-cpu --no-default-features`. The
  default `std` feature is for the tests, example, and benchmark.

## What it computes

This is a proof-of-contract substrate, not a compute kernel. It accepts exactly
one IR format — `ir_format == "sokr-noop"` — whose "execution" is a no-op that
produces a completed token. Every other IR is disclaimed so the core's
capability scan can fall through to a real substrate.

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
sokr-cpu = "0.2"
```

Register the substrate with the core and route a computation through it. The
core assigns the `substrate_id` you must use for dispatch (the plugin cannot
know its own id — see [ARCHITECTURE.md](ARCHITECTURE.md)):

```rust
use sokr::ffi::sokr_register_substrate;
use sokr_cpu::CPU_PLUGIN;

let mut substrate_id = 0u64;
// SAFETY: both pointers are valid and non-null.
let result = unsafe { sokr_register_substrate(&CPU_PLUGIN, &mut substrate_id) };
assert!(result.is_ok());
// Dispatch with request.substrate_id = substrate_id ...
```

See [`examples/cpu_end_to_end.rs`](examples/cpu_end_to_end.rs) for the complete
`register → capability → dispatch → completion → deregister` cycle:

```sh
cargo run --example cpu_end_to_end
```

Benchmark the synchronous `dispatch → completion` roundtrip:

```sh
cargo bench -p sokr-cpu
```

## Completion model

The ABI is asynchronous (dispatch issues a token, completion polls it), but CPU
work here is synchronous: when `dispatch_fn` returns, the computation has
already run and its token is `Complete`. `completion_fn` reports that status and
**consumes** the token, so a token is valid for exactly one terminal poll;
re-polling a consumed token is disclaimed (`NotFound`). `destroy_fn` invalidates
every outstanding token.

## ⚠️ Single-threaded

The completion table lives in a module `static` because the SOKR vtable carries
no per-plugin context. This is sound only under the core's pre-1.0
single-threaded invariant. Do not call the plugin from multiple threads
concurrently. See [ARCHITECTURE.md](ARCHITECTURE.md) for the upstream
`context: *mut c_void` proposal that would remove the static.

## Version

- **sokr-cpu**: v0.2.0
- **Depends on**: `sokr` 0.3 (advertises `SokrVersion::CURRENT`)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## See Also

- [ARCHITECTURE.md](ARCHITECTURE.md) — execution model, disclaim semantics, ABI seams
- [sokr-rs/sokr](https://github.com/sokr-rs/sokr) — SOKR compute abstraction core
- [sokr-dispatch-first](../sokr-dispatch-first) — First-capable dispatch policy
