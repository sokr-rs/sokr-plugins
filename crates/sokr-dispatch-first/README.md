# sokr-dispatch-first

**First-capable dispatch policy for SOKR** — reference implementation for substrate selection strategy.

## Features

- ✅ First-capable strategy — iterate substrates, dispatch to first accepting `capability` query
- ✅ Registration order — substrates queried in insertion order
- ✅ Zero dependencies — pure Rust
- ✅ Thread-safe — uses `std::sync::Mutex`
- ✅ No allocation — fixed-size registry

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
sokr-dispatch-first = "0.1"
```

Use in your dispatch selection logic:

```rust
use sokr_dispatch_first::SubstrateRegistry;

let registry = SubstrateRegistry::new();
registry.register(substrate_id_1);
registry.register(substrate_id_2);

// Query for first capable substrate
if let Some(substrate_id) = registry.first_capable() {
    // Dispatch to this substrate
}
```

## Architecture

The registry maintains an ordered list of substrate IDs and provides simple query operations:

- **`new()`**: Create empty registry
- **`register(substrate_id)`**: Add substrate to end of list
- **`first_capable()`**: Return first registered substrate (foundation for "first-capable" strategy)
- **`substrate_count()`**: Count registered substrates
- **`get_substrate(index)`**: Retrieve substrate at index
- **`clear()`**: Remove all substrates

## Dispatch Strategy

The "first-capable" strategy:

1. User registers substrates with dispatcher in preferred order
2. For each computation, iterate registered substrates in order
3. Query `capability_fn` on each
4. Dispatch to first substrate returning `Ok`
5. Return `NoCapableSubstrate` if none accept the computation

## Version

- **sokr-dispatch-first**: v0.1.1
- **No external dependencies**

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## See Also

- [sokr-rs/sokr](https://github.com/sokr-rs/sokr) — SOKR compute abstraction core
- [sokr-cpu](../sokr-cpu) — Synchronous CPU substrate plugin
