// src/lib.rs
// CPU substrate plugin entry point.

#![no_std]

use sokr::{
    SokrSubstratePlugin,
    SokrVersion,
    SokrResult,
    SokrCapabilityFn,
    SokrDispatchFn,
    SokrCompletionFn,
    SokrDestroyFn,
    SokrCapabilityQuery,
    SokrCapabilityResponse,
    SokrDispatchRequest,
    SokrDispatchResponse,
    SokrCompletionQuery,
    SokrCompletionSignal,
};

extern "C" fn capability(
    _version: *const SokrVersion,
    _query: *const SokrCapabilityQuery,
    response: *mut SokrCapabilityResponse,
) -> SokrResult {
    if !response.is_null() {
        unsafe { (*response).result = SokrResult::Ok };
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
    padding: [0; 16],
};
