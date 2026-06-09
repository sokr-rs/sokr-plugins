//! # sokr-cpu — synchronous CPU substrate plugin
//!
//! Reference implementation of the [`SokrSubstratePlugin`] contract for
//! general-purpose CPUs. It is the simplest substrate that proves the SOKR
//! dispatch loop closes end to end: `capability → dispatch → completion`.
//!
//! ## IR accepted
//!
//! This plugin **disclaims** every computation except the no-op IR
//! `ir_format == "sokr-noop"`. "Executing" a `sokr-noop` computation does
//! nothing observable except produce a completed token — it exists to prove
//! the contract, not to compute. Accepting only one known format keeps the
//! plugin honest: a CPU that claimed *every* IR while doing no real work would
//! be a "capability lie" (see `ARCHITECTURE.md`).
//!
//! ## Completion model
//!
//! The ABI is asynchronous (`dispatch` issues a token, `completion` polls it),
//! but CPU work here is synchronous: [`dispatch`] runs the (no-op) computation
//! inline and records a `Complete` entry in a static completion table keyed by
//! a unique non-zero token. [`completion`] reports that status and **consumes**
//! the token, so a second poll of the same token disclaims (`NotFound`).
//! [`destroy`] clears the whole table.
//!
//! ## ⚠️ Single-threaded constraint
//!
//! The SOKR vtable has **no per-plugin context argument** — the four function
//! pointers are bare `extern "C" fn` with nowhere to stash per-dispatch state.
//! The completion table therefore lives in a module `static` guarded only by a
//! single-threaded-access invariant, mirroring the sokr core's own pre-1.0
//! `SokrRegistry` design. This is sound **only** because the core is
//! single-threaded until v1.0.0. Do not call these functions from multiple
//! threads concurrently. (`cargo test` runs tests on parallel threads, so the
//! integration tests serialize access with a mutex.) See `ARCHITECTURE.md` for
//! the upstream `context: *mut c_void` proposal that would remove the static.

use core::cell::UnsafeCell;
use core::ffi::{c_char, CStr};

use sokr::{
    SokrCapabilityFn, SokrCapabilityQuery, SokrCapabilityResponse, SokrCompletionFn,
    SokrCompletionQuery, SokrCompletionSignal, SokrCompletionToken, SokrDestroyFn, SokrDispatchFn,
    SokrDispatchRequest, SokrDispatchResponse, SokrResult, SokrSubstratePlugin, SokrVersion,
};

/// The only IR format this substrate claims.
const ACCEPTED_IR_FORMAT: &[u8] = b"sokr-noop";

/// Maximum number of outstanding (un-consumed, un-cleared) completion tokens.
///
/// Synchronous tokens are consumed on their first terminal poll, so this only
/// bounds how many dispatched-but-unpolled computations may coexist. Kept small
/// to stay constrained-machine friendly.
const COMPLETION_TABLE_CAPACITY: usize = 256;

/// One completion-table entry. `handle == 0` marks a free slot.
#[derive(Clone, Copy)]
struct Slot {
    handle: u64,
    signal: SokrCompletionSignal,
}

/// Fixed-capacity table of outstanding completion tokens plus the monotonic
/// handle allocator. Accessed only under the single-threaded invariant.
struct CompletionTable {
    slots: [Slot; COMPLETION_TABLE_CAPACITY],
    next_handle: u64,
}

impl CompletionTable {
    const fn new() -> Self {
        Self {
            slots: [Slot {
                handle: 0,
                signal: SokrCompletionSignal::Pending,
            }; COMPLETION_TABLE_CAPACITY],
            // Handle 0 is the reserved invalid/unset sentinel; start at 1.
            next_handle: 1,
        }
    }

    /// Allocate the next non-zero handle that is not already live.
    fn allocate_handle(&mut self) -> u64 {
        loop {
            let candidate = self.next_handle;
            self.next_handle = self.next_handle.wrapping_add(1);
            if self.next_handle == 0 {
                self.next_handle = 1;
            }
            if candidate != 0 && !self.slots.iter().any(|slot| slot.handle == candidate) {
                return candidate;
            }
        }
    }

    /// Store a new entry, returning its non-zero handle, or `None` if full.
    fn insert(&mut self, signal: SokrCompletionSignal) -> Option<u64> {
        // Find a free slot *before* burning a handle so a full table is a no-op.
        let index = self.slots.iter().position(|slot| slot.handle == 0)?;
        let handle = self.allocate_handle();
        self.slots[index] = Slot { handle, signal };
        Some(handle)
    }

    /// Consume the entry for `handle`, returning its signal if present.
    ///
    /// Consuming makes the token single-use: re-polling a consumed token finds
    /// nothing and the plugin disclaims it.
    fn take(&mut self, handle: u64) -> Option<SokrCompletionSignal> {
        if handle == 0 {
            return None;
        }
        let index = self.slots.iter().position(|slot| slot.handle == handle)?;
        let signal = self.slots[index].signal;
        self.slots[index] = Slot {
            handle: 0,
            signal: SokrCompletionSignal::Pending,
        };
        Some(signal)
    }

    /// Invalidate every outstanding token and reset the allocator.
    fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = Slot {
                handle: 0,
                signal: SokrCompletionSignal::Pending,
            };
        }
        self.next_handle = 1;
    }
}

/// `Sync` wrapper around the completion table.
///
/// # Safety
/// `Sync` is sound here **only** under the core's single-threaded invariant
/// (Phase < 1.0). All access goes through the FFI entry points below, each of
/// which documents that contract.
struct SyncTable(UnsafeCell<CompletionTable>);

// SAFETY: single-threaded access invariant; see `SyncTable` docs.
unsafe impl Sync for SyncTable {}

impl SyncTable {
    const fn new() -> Self {
        Self(UnsafeCell::new(CompletionTable::new()))
    }

    const fn get(&self) -> *mut CompletionTable {
        self.0.get()
    }
}

/// Process-global completion table. See the single-threaded constraint above.
static TABLE: SyncTable = SyncTable::new();

/// Returns true if `ir_format` names the IR this substrate claims.
///
/// # Safety
/// `ptr` must be null, or a valid null-terminated C string for the duration of
/// the call (guaranteed by the `SokrCapabilityFn` pointer contract).
unsafe fn ir_format_is_accepted(ptr: *const c_char) -> bool {
    if ptr.is_null() {
        return false;
    }
    // SAFETY: caller guarantees a valid null-terminated C string when non-null.
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_bytes() == ACCEPTED_IR_FORMAT
}

/// Capability query: claim `sokr-noop`, disclaim everything else.
///
/// Returns `Ok` (claim) for `ir_format == "sokr-noop"`, otherwise
/// `CapabilityDenied` (disclaim). It never returns `InvalidIR`/`InvalidInput`
/// as a "no": the core treats any non-`Ok`-non-`CapabilityDenied` result as
/// "this plugin owns the request but failed" and aborts the whole capability
/// scan, which would starve other plugins.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn capability(
    _version: *const SokrVersion,
    query: *const SokrCapabilityQuery,
    response: *mut SokrCapabilityResponse,
) -> SokrResult {
    // The core guarantees non-null pointers, but a direct caller might not.
    // Disclaim (never `InvalidInput`) so we can never abort a capability scan.
    if query.is_null() || response.is_null() {
        return SokrResult::CapabilityDenied;
    }

    // SAFETY: pointers are non-null (checked) and valid per the fn-pointer
    // contract for the duration of the call.
    let accepted = unsafe { ir_format_is_accepted((*query).ir_format) };

    // SAFETY: `response` is non-null (checked) and valid for writes.
    unsafe {
        if accepted {
            (*response).result = SokrResult::Ok;
        } else {
            (*response).result = SokrResult::CapabilityDenied;
        }
        (*response).padding = 0;
        // We cannot know our core-assigned `substrate_id`: the ABI passes no
        // per-plugin context. Callers MUST route dispatch with the id returned
        // by `sokr_register_substrate`. See ARCHITECTURE: ABI-context seam.
        (*response).substrate_id = 0;
        // Synchronous no-op: latency is negligible, reported as 0 ("unknown").
        (*response).estimated_latency_ns = 0;
    }

    if accepted {
        SokrResult::Ok
    } else {
        SokrResult::CapabilityDenied
    }
}

/// Dispatch: run the (no-op) computation synchronously and issue a token.
///
/// The core has already null/zero-length-checked the request's IR and params
/// and routed us by `substrate_id`, so this only allocates a completion token.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn dispatch(
    _request: *const SokrDispatchRequest,
    response: *mut SokrDispatchResponse,
) -> SokrResult {
    if response.is_null() {
        return SokrResult::InvalidInput;
    }

    // SAFETY: single-threaded access invariant (Phase < 1.0); see module docs.
    let table = unsafe { &mut *TABLE.get() };

    // The synchronous "computation" is a no-op; record it as already complete.
    match table.insert(SokrCompletionSignal::Complete) {
        Some(handle) => {
            // SAFETY: `response` is non-null (checked) and valid for writes.
            unsafe {
                (*response).result = SokrResult::Ok;
                (*response).padding = 0;
                (*response).completion_token = SokrCompletionToken { handle };
            }
            SokrResult::Ok
        }
        None => {
            // We own this substrate_id but cannot take more outstanding tokens.
            // A genuine failure (not a disclaim); the core propagates it.
            // SAFETY: `response` is non-null (checked) and valid for writes.
            unsafe {
                (*response).result = SokrResult::DispatchFailed;
                (*response).padding = 0;
                (*response).completion_token = SokrCompletionToken { handle: 0 };
            }
            SokrResult::DispatchFailed
        }
    }
}

/// Completion poll: report and consume our token, or disclaim a foreign one.
///
/// Returns `Ok` (with `signal` written) for tokens we issued, or `NotFound` to
/// disclaim tokens we do not own. On the disclaim path we leave `signal`
/// untouched: the core sets it to `Failed` itself once every plugin disclaims.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn completion(
    query: *const SokrCompletionQuery,
    signal: *mut SokrCompletionSignal,
) -> SokrResult {
    if query.is_null() || signal.is_null() {
        return SokrResult::InvalidInput;
    }

    // SAFETY: `query` is non-null (checked) and valid for reads.
    let handle = unsafe { (*query).completion_token.handle };

    // SAFETY: single-threaded access invariant (Phase < 1.0); see module docs.
    let table = unsafe { &mut *TABLE.get() };

    match table.take(handle) {
        Some(status) => {
            // SAFETY: `signal` is non-null (checked) and valid for writes.
            unsafe {
                *signal = status;
            }
            SokrResult::Ok
        }
        // Disclaim: not our token. Leave `signal` untouched (the core writes
        // `Failed` after the disclaim scan) and return the disclaim code.
        None => SokrResult::NotFound,
    }
}

/// Teardown: invalidate every outstanding completion token. Never panics.
pub extern "C" fn destroy() {
    // SAFETY: single-threaded access invariant (Phase < 1.0); see module docs.
    let table = unsafe { &mut *TABLE.get() };
    table.clear();
}

/// Static plugin descriptor. This is the crate's single exported C symbol: a
/// C host registers the substrate by passing `&CPU_PLUGIN` to
/// `sokr_register_substrate`. The entry-point functions are reached only
/// through this vtable, so they are intentionally not `#[no_mangle]`.
#[no_mangle]
pub static CPU_PLUGIN: SokrSubstratePlugin = SokrSubstratePlugin {
    version: SokrVersion::CURRENT,
    capability_fn: capability as SokrCapabilityFn,
    dispatch_fn: dispatch as SokrDispatchFn,
    completion_fn: completion as SokrCompletionFn,
    destroy_fn: destroy as SokrDestroyFn,
    // Assigned by the core on registration; left 0 in the static descriptor.
    substrate_id: 0,
    padding: [0; 8],
};
