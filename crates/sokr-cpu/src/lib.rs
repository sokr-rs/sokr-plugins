// src/lib.rs
// CPU substrate plugin entry point.
//
// Note: sokr-cpu is not no_std to enable integration testing with std-based test harnesses.
// Phase 2.1 may re-evaluate embedded constraints if needed.

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
        // Return substrate_id from the plugin descriptor (will be assigned by core)
        // For capability query, any non-zero ID indicates acceptance
        (*response).substrate_id = 1;
        (*response).estimated_latency_ns = 0;
    }
    SokrResult::Ok
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn dispatch(
    _request: *const SokrDispatchRequest,
    response: *mut SokrDispatchResponse,
) -> SokrResult {
    if response.is_null() {
        return SokrResult::InvalidInput;
    }

    unsafe {
        // Return a valid completion token (non-zero handle)
        (*response).result = SokrResult::Ok;
        (*response).completion_token.handle = 42; // Fixed token for synchronous execution
    }
    SokrResult::Ok
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn completion(
    query: *const SokrCompletionQuery,
    signal: *mut SokrCompletionSignal,
) -> SokrResult {
    if query.is_null() || signal.is_null() {
        return SokrResult::InvalidInput;
    }

    unsafe {
        // CPU plugin executes synchronously, so all known tokens are complete
        let q = &*query;
        if q.completion_token.handle == 42 {
            *signal = SokrCompletionSignal::Complete;
            return SokrResult::Ok;
        }
        // Token not recognized by this plugin
        SokrResult::NotFound
    }
}

#[no_mangle]
pub extern "C" fn destroy() {}

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
