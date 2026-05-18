// src/lib.rs
// CPU substrate plugin entry point.

#![no_std]

use sokr::{
    SokrCapabilityFn, SokrCapabilityQuery, SokrCapabilityResponse, SokrCompletionFn,
    SokrCompletionQuery, SokrCompletionSignal, SokrDestroyFn, SokrDispatchFn, SokrDispatchRequest,
    SokrDispatchResponse, SokrResult, SokrSubstratePlugin, SokrVersion,
};

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
    _request: *const SokrDispatchRequest,
    _response: *mut SokrDispatchResponse,
) -> SokrResult {
    SokrResult::Ok
}

extern "C" fn completion(
    _query: *const SokrCompletionQuery,
    _signal: *mut SokrCompletionSignal,
) -> SokrResult {
    SokrResult::Ok
}

extern "C" fn destroy() {}

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
