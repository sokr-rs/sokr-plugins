use sokr::{
    SokrCompletionQuery, SokrCompletionSignal, SokrCompletionToken, SokrDispatchRequest,
    SokrDispatchResponse, SokrResult,
};

extern "C" {
    fn dispatch(
        request: *const SokrDispatchRequest,
        response: *mut SokrDispatchResponse,
    ) -> SokrResult;
    fn completion(
        query: *const SokrCompletionQuery,
        signal: *mut SokrCompletionSignal,
    ) -> SokrResult;
}

#[test]
fn test_dispatch_stores_result_retrievable_by_completion() {
    let ir_data = b"test ir data";
    let req = SokrDispatchRequest {
        computation_id: Default::default(),
        substrate_id: 0,
        ir_data_ptr: ir_data.as_ptr() as *const std::ffi::c_void,
        ir_data_len: ir_data.len(),
        params_ptr: std::ptr::null(),
        params_len: 0,
        padding: [0; 16],
    };

    let mut resp = SokrDispatchResponse {
        result: SokrResult::Ok,
        padding: 0,
        completion_token: SokrCompletionToken { handle: 0 },
    };

    let result = unsafe { dispatch(&req as *const _, &mut resp as *mut _) };
    assert_eq!(result, SokrResult::Ok);
    assert_ne!(resp.completion_token.handle, 0);

    let token = resp.completion_token;

    let query = SokrCompletionQuery {
        completion_token: token,
        timeout_ns: 0,
        padding: [0; 8],
    };
    let mut sig = SokrCompletionSignal::Pending;

    let result = unsafe { completion(&query as *const _, &mut sig as *mut _) };
    assert_eq!(result, SokrResult::Ok);
    assert_eq!(sig, SokrCompletionSignal::Complete);
}

#[test]
fn test_dispatch_returns_distinct_tokens() {
    let ir_data = b"test";
    let req = SokrDispatchRequest {
        computation_id: Default::default(),
        substrate_id: 0,
        ir_data_ptr: ir_data.as_ptr() as *const std::ffi::c_void,
        ir_data_len: ir_data.len(),
        params_ptr: std::ptr::null(),
        params_len: 0,
        padding: [0; 16],
    };

    let mut resp1 = SokrDispatchResponse {
        result: SokrResult::Ok,
        padding: 0,
        completion_token: SokrCompletionToken { handle: 0 },
    };
    let mut resp2 = SokrDispatchResponse {
        result: SokrResult::Ok,
        padding: 0,
        completion_token: SokrCompletionToken { handle: 0 },
    };

    unsafe {
        dispatch(&req as *const _, &mut resp1 as *mut _);
        dispatch(&req as *const _, &mut resp2 as *mut _);
    }

    assert_ne!(resp1.completion_token.handle, resp2.completion_token.handle);
}

#[test]
fn test_dispatch_null_request_returns_error() {
    let mut resp = SokrDispatchResponse {
        result: SokrResult::Ok,
        padding: 0,
        completion_token: SokrCompletionToken { handle: 0 },
    };
    let result = unsafe { dispatch(std::ptr::null(), &mut resp as *mut _) };
    assert_eq!(result, SokrResult::InvalidInput);
}

#[test]
fn test_dispatch_null_response_returns_error() {
    let ir_data = b"test";
    let req = SokrDispatchRequest {
        computation_id: Default::default(),
        substrate_id: 0,
        ir_data_ptr: ir_data.as_ptr() as *const std::ffi::c_void,
        ir_data_len: ir_data.len(),
        params_ptr: std::ptr::null(),
        params_len: 0,
        padding: [0; 16],
    };
    let result = unsafe { dispatch(&req as *const _, std::ptr::null_mut()) };
    assert_eq!(result, SokrResult::InvalidInput);
}

#[test]
fn test_completion_unknown_token_returns_failed() {
    let query = SokrCompletionQuery {
        completion_token: SokrCompletionToken { handle: 999999 },
        timeout_ns: 0,
        padding: [0; 8],
    };
    let mut sig = SokrCompletionSignal::Pending;

    let result = unsafe { completion(&query as *const _, &mut sig as *mut _) };
    assert_eq!(result, SokrResult::Ok);
    assert_eq!(sig, SokrCompletionSignal::Failed);
}
