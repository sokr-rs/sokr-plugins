// src/lib.rs
// CPU substrate plugin entry point.

#![no_std]

use core::sync::atomic::{AtomicU64, Ordering};
use sokr::{
    SokrCapabilityFn, SokrCapabilityQuery, SokrCapabilityResponse, SokrCompletionFn,
    SokrCompletionQuery, SokrCompletionSignal, SokrCompletionToken, SokrDestroyFn, SokrDispatchFn,
    SokrDispatchRequest, SokrDispatchResponse, SokrResult, SokrSubstratePlugin, SokrVersion,
};

const MAX_SLOTS: usize = 1024;

static TOKEN_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Copy, Clone)]
struct ResultSlot {
    token: u64,
    executed: bool,
}

static mut RESULT_SLOTS: [Option<ResultSlot>; MAX_SLOTS] = [None; MAX_SLOTS];
static mut NEXT_SLOT: usize = 0;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn capability(
    _version: *const SokrVersion,
    query: *const SokrCapabilityQuery,
    response: *mut SokrCapabilityResponse,
) -> SokrResult {
    if query.is_null() || response.is_null() {
        return SokrResult::InvalidInput;
    }

    unsafe {
        (*response).result = SokrResult::Ok;
        (*response).substrate_id = 0;
        (*response).estimated_latency_ns = 0;
    }
    SokrResult::Ok
}

extern "C" fn dispatch(
    request: *const SokrDispatchRequest,
    response: *mut SokrDispatchResponse,
) -> SokrResult {
    if request.is_null() || response.is_null() {
        return SokrResult::InvalidInput;
    }

    unsafe {
        let req = &*request;

        if req.ir_data_ptr.is_null() || req.ir_data_len == 0 {
            return SokrResult::InvalidInput;
        }

        let mut token_handle = TOKEN_COUNTER.fetch_add(1, Ordering::SeqCst);
        if token_handle == 0 {
            token_handle = TOKEN_COUNTER.fetch_add(1, Ordering::SeqCst);
        }

        let slot_idx = NEXT_SLOT % MAX_SLOTS;
        RESULT_SLOTS[slot_idx] = Some(ResultSlot {
            token: token_handle,
            executed: true,
        });
        NEXT_SLOT += 1;

        let resp = &mut *response;
        resp.result = SokrResult::Ok;
        resp.completion_token = SokrCompletionToken {
            handle: token_handle,
        };
    }

    SokrResult::Ok
}

#[allow(static_mut_refs)]
extern "C" fn completion(
    query: *const SokrCompletionQuery,
    signal: *mut SokrCompletionSignal,
) -> SokrResult {
    if query.is_null() || signal.is_null() {
        return SokrResult::InvalidInput;
    }

    unsafe {
        let q = &*query;
        let sig = &mut *signal;

        if q.completion_token.handle == 0 {
            *sig = SokrCompletionSignal::Failed;
            return SokrResult::Ok;
        }

        let mut found = false;
        for s in RESULT_SLOTS.iter().flatten() {
            if s.token == q.completion_token.handle {
                if s.executed {
                    *sig = SokrCompletionSignal::Complete;
                } else {
                    *sig = SokrCompletionSignal::Pending;
                }
                found = true;
                break;
            }
        }

        if !found {
            *sig = SokrCompletionSignal::Failed;
        }
    }

    SokrResult::Ok
}

#[allow(static_mut_refs)]
extern "C" fn destroy() {
    unsafe {
        RESULT_SLOTS.fill(None);
        NEXT_SLOT = 0;
    }
}

// Static plugin descriptor required by the SOKR core.
#[no_mangle]
pub static CPU_PLUGIN: SokrSubstratePlugin = SokrSubstratePlugin {
    version: SokrVersion::CURRENT,
    capability_fn: capability as SokrCapabilityFn,
    dispatch_fn: dispatch as SokrDispatchFn,
    completion_fn: completion as SokrCompletionFn,
    destroy_fn: destroy as SokrDestroyFn,
    substrate_id: 0,
    padding: [0; 8],
};
