/// Task 1.2: Verify dispatch→completion round-trip
/// Dispatch a computation, then poll completion and verify Complete status.
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
}

#[test]
fn test_1_2_dispatch_completion_roundtrip() {
    let ir_data = b"simple computation";
    let req = SokrDispatchRequest {
        computation_id: SokrComputationId { high: 1, low: 2 },
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

    // Step 1: Dispatch
    let dispatch_result = unsafe { dispatch(&req as *const _, &mut resp as *mut _) };
    assert_eq!(dispatch_result, SokrResult::Ok, "dispatch failed");
    assert_eq!(
        resp.result,
        SokrResult::Ok,
        "dispatch response result not Ok"
    );
    assert_ne!(
        resp.completion_token.handle, 0,
        "dispatch returned invalid token (0)"
    );

    let token = resp.completion_token;

    // Step 2: Poll completion
    let query = SokrCompletionQuery {
        completion_token: token,
        timeout_ns: 0,
        padding: [0; 8],
    };
    let mut sig = SokrCompletionSignal::Pending;

    let completion_result = unsafe { completion(&query as *const _, &mut sig as *mut _) };
    assert_eq!(completion_result, SokrResult::Ok, "completion failed");
    assert_eq!(
        sig,
        SokrCompletionSignal::Complete,
        "completion status not Complete"
    );
}

#[test]
fn test_1_2_multiple_concurrent_dispatches() {
    let ir_data = b"test";

    let mut tokens = Vec::new();

    for i in 0..5 {
        let req = SokrDispatchRequest {
            computation_id: SokrComputationId {
                high: i,
                low: i + 1,
            },
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
        tokens.push(resp.completion_token);
    }

    // Verify all can be polled and complete
    for (idx, token) in tokens.iter().enumerate() {
        let query = SokrCompletionQuery {
            completion_token: *token,
            timeout_ns: 0,
            padding: [0; 8],
        };
        let mut sig = SokrCompletionSignal::Pending;

        let result = unsafe { completion(&query as *const _, &mut sig as *mut _) };
        assert_eq!(
            result,
            SokrResult::Ok,
            "completion failed for dispatch {}",
            idx
        );
        assert_eq!(
            sig,
            SokrCompletionSignal::Complete,
            "completion not Complete for dispatch {}",
            idx
        );
    }
}
