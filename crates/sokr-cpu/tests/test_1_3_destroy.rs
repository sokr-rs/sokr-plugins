/// Task 1.3: Verify destroy_fn clears all state
/// Dispatch computation, destroy plugin, verify subsequent queries return Failed.
use sokr::{
    SokrCompletionQuery, SokrCompletionSignal, SokrCompletionToken, SokrComputationId,
    SokrDispatchRequest, SokrDispatchResponse, SokrResult,
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
    fn destroy();
}

#[test]
fn test_1_3_destroy_invalidates_tokens() {
    let ir_data = b"test";
    let req = SokrDispatchRequest {
        computation_id: SokrComputationId { high: 1, low: 1 },
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

    // Dispatch
    unsafe {
        dispatch(&req as *const _, &mut resp as *mut _);
    }
    let token = resp.completion_token;
    assert_ne!(token.handle, 0);

    // Verify completion works before destroy
    let query = SokrCompletionQuery {
        completion_token: token,
        timeout_ns: 0,
        padding: [0; 8],
    };
    let mut sig = SokrCompletionSignal::Pending;
    unsafe {
        completion(&query as *const _, &mut sig as *mut _);
    }
    assert_eq!(sig, SokrCompletionSignal::Complete);

    // Destroy plugin
    unsafe {
        destroy();
    }

    // After destroy, query should return Failed
    let mut sig_after = SokrCompletionSignal::Pending;
    unsafe {
        completion(&query as *const _, &mut sig_after as *mut _);
    }
    assert_eq!(
        sig_after,
        SokrCompletionSignal::Failed,
        "completion should return Failed after destroy"
    );
}

#[test]
fn test_1_3_destroy_clears_all_slots() {
    let ir_data = b"test";

    let mut tokens = Vec::new();

    // Dispatch multiple times
    for i in 0..10 {
        let req = SokrDispatchRequest {
            computation_id: SokrComputationId { high: i, low: i },
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

        unsafe {
            dispatch(&req as *const _, &mut resp as *mut _);
        }
        tokens.push(resp.completion_token);
    }

    // Destroy
    unsafe {
        destroy();
    }

    // All tokens should now return Failed
    for (idx, token) in tokens.iter().enumerate() {
        let query = SokrCompletionQuery {
            completion_token: *token,
            timeout_ns: 0,
            padding: [0; 8],
        };
        let mut sig = SokrCompletionSignal::Pending;

        unsafe {
            completion(&query as *const _, &mut sig as *mut _);
        }
        assert_eq!(
            sig,
            SokrCompletionSignal::Failed,
            "token {} should be invalid after destroy",
            idx
        );
    }
}
