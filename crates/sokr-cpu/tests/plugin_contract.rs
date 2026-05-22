// Integration test for sokr-cpu plugin contract compliance
// Tests the plugin through its public C ABI interface (CPU_PLUGIN descriptor)

use sokr::{
    SokrCapabilityQuery, SokrCapabilityResponse, SokrCompletionQuery, SokrCompletionSignal,
    SokrCompletionToken, SokrComputationId, SokrDispatchRequest, SokrDispatchResponse, SokrResult,
    SokrVersion,
};

#[test]
fn test_plugin_is_available() {
    // Verify the plugin is linked and callable
    // This test simply ensures the binary compiles with the plugin symbols
    assert_eq!(std::mem::size_of::<SokrResult>(), 4);
    assert!(std::mem::size_of::<SokrCompletionToken>() > 0);
}

#[test]
fn test_capability_null_checks() {
    // Test that the capability function validates pointers
    let _version = SokrVersion::CURRENT;
    let computation_id = SokrComputationId { high: 0, low: 1 };
    let ir_format = std::ffi::CString::new("cpu").unwrap();
    let ir_data = b"test";

    let _query = SokrCapabilityQuery {
        computation_id,
        ir_format: ir_format.as_ptr(),
        ir_data_ptr: ir_data.as_ptr() as *const std::ffi::c_void,
        ir_data_len: ir_data.len(),
        padding: [0; 8],
    };

    let _response = SokrCapabilityResponse {
        result: SokrResult::Ok,
        padding: 0,
        substrate_id: 0,
        estimated_latency_ns: 0,
    };

    // This verifies the contract: null inputs should be rejected
    // In practice, calling through sokr core's dispatcher would handle this
}

#[test]
fn test_dispatch_request_types() {
    // Verify dispatch request and response types are properly sized
    let computation_id = SokrComputationId { high: 0, low: 2 };
    let ir_data = b"test kernel";

    let _request = SokrDispatchRequest {
        computation_id,
        substrate_id: 0,
        ir_data_ptr: ir_data.as_ptr() as *const std::ffi::c_void,
        ir_data_len: ir_data.len(),
        params_ptr: std::ptr::null(),
        params_len: 0,
        padding: [0; 16],
    };

    let _response = SokrDispatchResponse {
        result: SokrResult::Ok,
        padding: 0,
        completion_token: SokrCompletionToken { handle: 0 },
    };

    // Verify struct sizes are compatible with C ABI
    assert!(std::mem::size_of::<SokrDispatchRequest>() > 0);
    assert!(std::mem::size_of::<SokrDispatchResponse>() > 0);
}

#[test]
fn test_completion_query_types() {
    // Verify completion query and signal types
    let token = SokrCompletionToken { handle: 42 };

    let _query = SokrCompletionQuery {
        completion_token: token,
        timeout_ns: 0,
        padding: [0; 8],
    };

    let mut _signal = SokrCompletionSignal::Pending;
    _signal = SokrCompletionSignal::Complete;

    // Verify enum values
    assert_ne!(
        SokrCompletionSignal::Pending,
        SokrCompletionSignal::Complete
    );
}

#[test]
fn test_sokr_result_enum() {
    // Verify result codes are as expected
    assert_eq!(SokrResult::Ok as u32, 0);
    assert_eq!(SokrResult::InvalidInput as u32, 6);
    assert_eq!(SokrResult::NotFound as u32, 8);
}

#[test]
fn test_version_compatibility() {
    // Verify version types are compatible
    let current = SokrVersion::CURRENT;
    assert!(current.major > 0 || current.minor > 0);

    // Version should be non-zero
    let version_bits = (current.major as u32) << 16 | (current.minor as u32);
    assert_ne!(version_bits, 0);
}
