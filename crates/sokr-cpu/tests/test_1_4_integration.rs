/// Task 1.4: Full integration test - end-to-end with sokr core
/// Registers CPU plugin, runs capability→dispatch→completion→destroy cycle.
use sokr::{
    SokrCapabilityQuery, SokrCapabilityResponse, SokrCompletionQuery, SokrCompletionSignal,
    SokrCompletionToken, SokrComputationId, SokrDispatchRequest, SokrDispatchResponse, SokrResult,
    SokrVersion,
};

extern "C" {
    fn capability(
        version: *const SokrVersion,
        query: *const SokrCapabilityQuery,
        response: *mut SokrCapabilityResponse,
    ) -> SokrResult;
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
fn test_1_4_full_lifecycle() {
    // Step 1: Capability query (sanity check)
    let version = SokrVersion::CURRENT;
    let computation_id = SokrComputationId { high: 1, low: 2 };
    let ir_format = std::ffi::CString::new("sokr-cpu-ir").unwrap();
    let ir_data = b"some ir";

    let query = SokrCapabilityQuery {
        computation_id,
        ir_format: ir_format.as_ptr(),
        ir_data_ptr: ir_data.as_ptr() as *const std::ffi::c_void,
        ir_data_len: ir_data.len(),
        padding: [0; 8],
    };

    let mut cap_resp = SokrCapabilityResponse {
        result: SokrResult::Ok,
        padding: 0,
        substrate_id: 0,
        estimated_latency_ns: 0,
    };

    let cap_result = unsafe {
        capability(
            &version as *const _,
            &query as *const _,
            &mut cap_resp as *mut _,
        )
    };

    assert_eq!(cap_result, SokrResult::Ok, "capability query failed");
    assert_eq!(
        cap_resp.result,
        SokrResult::Ok,
        "capability response not Ok"
    );
    assert_eq!(cap_resp.substrate_id, 0, "unexpected substrate_id");

    // Step 2: Dispatch computation
    let req = SokrDispatchRequest {
        computation_id,
        substrate_id: 0,
        ir_data_ptr: ir_data.as_ptr() as *const std::ffi::c_void,
        ir_data_len: ir_data.len(),
        params_ptr: std::ptr::null(),
        params_len: 0,
        padding: [0; 16],
    };

    let mut dispatch_resp = SokrDispatchResponse {
        result: SokrResult::Ok,
        padding: 0,
        completion_token: SokrCompletionToken { handle: 0 },
    };

    let dispatch_result = unsafe { dispatch(&req as *const _, &mut dispatch_resp as *mut _) };

    assert_eq!(dispatch_result, SokrResult::Ok, "dispatch failed");
    assert_eq!(
        dispatch_resp.result,
        SokrResult::Ok,
        "dispatch response not Ok"
    );
    assert_ne!(
        dispatch_resp.completion_token.handle, 0,
        "dispatch returned zero token"
    );

    let token = dispatch_resp.completion_token;

    // Step 3: Poll completion
    let comp_query = SokrCompletionQuery {
        completion_token: token,
        timeout_ns: 0,
        padding: [0; 8],
    };

    let mut sig = SokrCompletionSignal::Pending;
    let comp_result = unsafe { completion(&comp_query as *const _, &mut sig as *mut _) };

    assert_eq!(comp_result, SokrResult::Ok, "completion query failed");
    assert_eq!(
        sig,
        SokrCompletionSignal::Complete,
        "completion signal not Complete"
    );

    // Step 4: Deregister (destroy)
    unsafe {
        destroy();
    }

    // Step 5: Verify plugin is unusable after destroy
    let mut sig_after = SokrCompletionSignal::Pending;
    unsafe {
        completion(&comp_query as *const _, &mut sig_after as *mut _);
    }
    assert_eq!(
        sig_after,
        SokrCompletionSignal::Failed,
        "completion should fail after destroy"
    );
}

#[test]
fn test_1_4_multiple_computations() {
    const NUM_COMPUTATIONS: u64 = 20;
    let ir_data = b"computation";

    let mut dispatch_tokens = Vec::new();

    // Dispatch multiple computations
    for i in 0..NUM_COMPUTATIONS {
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

        unsafe {
            dispatch(&req as *const _, &mut resp as *mut _);
        }
        assert_eq!(resp.result, SokrResult::Ok);
        dispatch_tokens.push(resp.completion_token);
    }

    // Poll all completions in different order (interleaved)
    for i in (0..dispatch_tokens.len()).rev() {
        let token = dispatch_tokens[i];
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
    }

    // Poll same tokens again (should still work - sync model)
    for (idx, token) in dispatch_tokens.iter().enumerate() {
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
            SokrCompletionSignal::Complete,
            "second poll failed for token {}",
            idx
        );
    }
}
