# sokr-cpu — Architecture

`sokr-cpu` is the simplest possible SOKR substrate: a synchronous CPU backend
whose only job is to prove the core contract (`capability → dispatch →
completion`) closes end to end. This document explains the design decisions that
are not obvious from the source.

## 1. Execution model: synchronous work behind an async ABI

The SOKR ABI is asynchronous by shape:

```text
dispatch(request) -> completion_token        // start work, hand back a handle
completion(token) -> Pending | Complete | …  // poll the handle until terminal
```

CPU work, however, is synchronous — there is nothing to wait for. We reconcile
the two by running the (no-op) computation **inline inside `dispatch_fn`** and
recording an already-`Complete` entry in a completion table keyed by a unique
non-zero token. `completion_fn` then simply reports that stored status.

```text
dispatch_fn:   run no-op  →  table.insert(Complete)  →  return token (handle != 0)
completion_fn: table.take(handle)  →  Ok + write Complete   (token consumed)
                              └─ miss →  NotFound            (disclaim, leave signal)
destroy_fn:    table.clear()                                 (invalidate all tokens)
```

This is the canonical shape every future async substrate (GPU, QPU) will reuse —
the only difference is that they will insert `Pending` and flip it to `Complete`
later, instead of inserting `Complete` immediately.

### Token lifecycle

A token is **single-use**: `completion_fn` consumes the slot on the first
terminal poll, so re-polling a delivered token finds nothing and the plugin
disclaims it (`NotFound`), which the core surfaces as `Failed`. Tokens that are
never polled are reclaimed wholesale by `destroy_fn` (called by the core on
deregistration). `handle == 0` is the reserved invalid/unset sentinel and is
never issued.

The table has a fixed capacity (`COMPLETION_TABLE_CAPACITY`, currently 256) to
stay constrained-machine friendly. Because synchronous tokens are consumed on
their first poll, this only bounds how many *dispatched-but-unpolled*
computations may coexist. A dispatch into a full table returns
`DispatchFailed` — a genuine failure (not a disclaim), which the core propagates
verbatim.

## 2. Disclaim semantics (why the exact result codes matter)

The core does **not** route `capability` and `completion` to a single plugin —
it **scans every registered plugin in turn**. The result code is how a plugin
says "mine" vs "not mine", and using the wrong code corrupts the scan for
*other* plugins:

| Entry point  | "claim" (mine) | "disclaim" (not mine) | any other non-`Ok`           |
|--------------|----------------|-----------------------|------------------------------|
| `capability` | `Ok`           | `CapabilityDenied`    | "I own it but failed" → scan **aborts** |
| `completion` | `Ok`           | `NotFound`            | "I own it but failed" → scan **aborts** |

Consequences honored by this crate:

- `capability_fn` returns **exactly `CapabilityDenied`** for any IR that is not
  `sokr-noop`. It must never return `InvalidIR`/`InvalidInput` as a "no", or it
  would abort the whole capability scan and starve the substrates registered
  after it.
- `completion_fn` returns **`NotFound`** for tokens it did not issue, and — on
  that disclaim path — leaves `*signal` untouched. The core itself writes
  `Failed` once every plugin has disclaimed, so the plugin must not pre-empt it.

### Why claim only `sokr-noop`?

A CPU can in principle attempt anything, so it is tempting to make `capability`
return `Ok` for every IR. That is a **capability lie**: a substrate that claims
a computation it cannot actually perform (here, anything but a no-op) will
"win" the scan and then fail or silently no-op at dispatch, starving a substrate
that could really do the work. Capability must be conservative, so this plugin
claims exactly the one IR it can honor.

## 3. Dispatch is routed, not scanned

Unlike capability/completion, `sokr_dispatch` looks up
`find_by_substrate_id(request.substrate_id)` and calls that one plugin directly.
The core has already null-checked `ir_data_ptr`/`params_ptr` and rejected
zero-length IR before `dispatch_fn` runs, and `dispatch_fn` is only reached for
a `substrate_id` that resolved to this plugin. So `dispatch_fn` re-validates
nothing about the request — it just allocates a token. (Note the
`SokrDispatchRequest` carries no `ir_format`, so dispatch *cannot* re-check the
format; it trusts that capability already gated it.)

## 4. The static completion table & the single-threaded constraint

The four vtable entries are bare `extern "C" fn` — **no `self`, no user-data, no
context argument**. A plugin therefore has nowhere to store per-dispatch state
except a module `static`. We use one (`TABLE`), wrapped in `UnsafeCell` behind a
hand-written `unsafe impl Sync`, mirroring the sokr core's own `SyncRegistry`.

This is sound **only** because the core is single-threaded until v1.0.0 (see the
core's `ffi.rs` thread-safety note). We deliberately do **not** spawn threads or
add locking inside the plugin: that would both fight the core's own
single-threaded contract and be premature. The constraint is documented loudly
in the module docs, the README, and here.

Practical consequence for testing: `cargo test` runs `#[test]`s on parallel
threads, and both the core registry and this table are process-global. The
integration tests therefore serialize every registry-touching test behind a
single `Mutex` to preserve the single-threaded invariant.

## 5. ABI-context-limitation seam (flag upstream, do not implement here)

Two awkward spots both stem from the missing context argument:

1. **The plugin cannot know its own `substrate_id`.** The core assigns the id at
   registration and stores it in *its* copy of the descriptor; the plugin's
   `static CPU_PLUGIN.substrate_id` stays `0`, and no entry point is handed the
   assigned id. So `capability_fn` writes `substrate_id = 0` into its response,
   and **callers must route dispatch using the id returned by
   `sokr_register_substrate`** (the example does exactly this).
2. **Per-dispatch state must be global** (§4), with all the single-threaded
   caveats that implies.

Both would be resolved by a single additive ABI change in the core: a
`context: *mut c_void` field in `SokrSubstratePlugin`, passed back to each entry
point. The plugin could then stash a heap-allocated, per-registration state
object (its assigned id, its own completion table) and drop the `static`
entirely — enabling multiple independent instances and lock-scoped concurrency.

This is an **upstream** change to the immutable core contract and is out of
scope for a plugin. It is recorded here and in `TODO.md` to propose to the sokr
core maintainers; this crate does not implement or depend on it.

## Capability metadata

- **Hardware**: any CPU; no SIMD/feature detection (the work is a no-op).
- **IR accepted**: `ir_format == "sokr-noop"` only.
- **Completion model**: synchronous (immediate `Complete`), consume-on-poll.
- **Alignment/padding**: `#[repr(C)]` throughout; vtable `padding: [u8; 8]`.
- **Threads**: single-threaded only (pre-1.0 core invariant).
