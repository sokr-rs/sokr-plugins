// SPDX-License-Identifier: MIT OR Apache-2.0
// CPU substrate plugin for SOKR — synchronous execution on calling thread.
//
// Implements the SOKR substrate plugin contract with minimal dependencies.
// Matches sokr v0.2.0 FFI types for compatibility.

use std::sync::atomic::{AtomicU64, Ordering};

// FFI type definitions matching SOKR substrate plugin contract.
// Kept here for standalone publication without sokr dependency.

#[repr(C)]
pub struct SokrVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SokrVersion {
    pub const CURRENT: SokrVersion = SokrVersion {
        major: 0,
        minor: 2,
        patch: 0,
    };
}

#[repr(C)]
pub struct SokrComputationId {
    pub high: u64,
    pub low: u64,
}

#[repr(C)]
pub struct SokrCapabilityQuery {
    pub computation_id: SokrComputationId,
    pub ir_format: *const core::ffi::c_char,
    pub ir_data_ptr: *const core::ffi::c_void,
    pub ir_data_len: usize,
    pub padding: [u8; 8],
}

#[repr(C)]
pub struct SokrCapabilityResponse {
    pub result: SokrResult,
    pub padding: u32,
    pub substrate_id: u32,
    pub estimated_latency_ns: u64,
}

#[repr(C)]
pub struct SokrDispatchRequest {
    pub computation_id: SokrComputationId,
    pub substrate_id: u32,
    pub ir_data_ptr: *const core::ffi::c_void,
    pub ir_data_len: usize,
    pub params_ptr: *const core::ffi::c_void,
    pub params_len: usize,
    pub padding: [u8; 16],
}

#[repr(C)]
pub struct SokrDispatchResponse {
    pub result: SokrResult,
    pub padding: u32,
    pub completion_token: SokrCompletionToken,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SokrCompletionToken {
    pub handle: u64,
}

#[repr(C)]
pub struct SokrCompletionQuery {
    pub completion_token: SokrCompletionToken,
    pub timeout_ns: u64,
    pub padding: [u8; 8],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SokrCompletionSignal {
    Pending = 0,
    Complete = 1,
    Failed = 2,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SokrResult {
    Ok = 0,
    InvalidInput = 1,
    NoCapableSubstrate = 2,
}

pub type SokrCapabilityFn = unsafe extern "C" fn(
    version: *const SokrVersion,
    query: *const SokrCapabilityQuery,
    response: *mut SokrCapabilityResponse,
) -> SokrResult;

pub type SokrDispatchFn = unsafe extern "C" fn(
    request: *const SokrDispatchRequest,
    response: *mut SokrDispatchResponse,
) -> SokrResult;

pub type SokrCompletionFn = unsafe extern "C" fn(
    query: *const SokrCompletionQuery,
    signal: *mut SokrCompletionSignal,
) -> SokrResult;

pub type SokrDestroyFn = unsafe extern "C" fn();

#[repr(C)]
pub struct SokrSubstratePlugin {
    pub version: SokrVersion,
    pub capability_fn: SokrCapabilityFn,
    pub dispatch_fn: SokrDispatchFn,
    pub completion_fn: SokrCompletionFn,
    pub destroy_fn: SokrDestroyFn,
    pub padding: [u8; 16],
}

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
    padding: [0; 16],
};
