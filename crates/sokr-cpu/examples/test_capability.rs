// Standalone test that doesn't require linking to the no_std library
use std::ffi::c_void;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SokrComputationId {
    pub high: u64,
    pub low: u64,
}

#[repr(C)]
pub struct SokrCapabilityQuery {
    pub computation_id: SokrComputationId,
    pub ir_format: *const i8,
    pub ir_data_ptr: *const c_void,
    pub ir_data_len: usize,
    pub padding: [u8; 8],
}

#[repr(C)]
pub struct SokrCapabilityResponse {
    pub result: u32,
    pub padding: u32,
    pub substrate_id: u64,
    pub estimated_latency_ns: u64,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SokrResult {
    Ok = 0,
    CapabilityDenied = 1,
    DispatchFailed = 2,
    Timeout = 3,
    VersionMismatch = 4,
    NoCapableSubstrate = 5,
    InvalidInput = 6,
}

extern "C" fn capability(
    _version: *const c_void,
    query: *const SokrCapabilityQuery,
    response: *mut SokrCapabilityResponse,
) -> SokrResult {
    if query.is_null() || response.is_null() {
        return SokrResult::InvalidInput;
    }

    unsafe {
        (*response).result = SokrResult::Ok as u32;
        (*response).substrate_id = 0;
        (*response).estimated_latency_ns = 0;
    }
    SokrResult::Ok
}

fn main() {
    // Test 1: capability accepts any query
    {
        let computation_id = SokrComputationId { high: 0, low: 0 };
        let ir_data = [0u8; 4];
        let query = SokrCapabilityQuery {
            computation_id,
            ir_format: b"TEST\0" as *const _ as *const i8,
            ir_data_ptr: ir_data.as_ptr() as *const c_void,
            ir_data_len: ir_data.len(),
            padding: [0; 8],
        };

        let mut response = SokrCapabilityResponse {
            result: 1, // CapabilityDenied
            padding: 0,
            substrate_id: u64::MAX,
            estimated_latency_ns: u64::MAX,
        };

        let result = capability(std::ptr::null(), &query, &mut response);

        assert_eq!(result, SokrResult::Ok);
        assert_eq!(response.result, SokrResult::Ok as u32);
        assert_eq!(response.substrate_id, 0);
        assert_eq!(response.estimated_latency_ns, 0);
        println!("✓ Test 1 passed: capability accepts any query");
    }

    // Test 2: null query returns InvalidInput
    {
        let mut response = SokrCapabilityResponse {
            result: SokrResult::Ok as u32,
            padding: 0,
            substrate_id: 0,
            estimated_latency_ns: 0,
        };

        let result = capability(std::ptr::null(), std::ptr::null(), &mut response);
        assert_eq!(result, SokrResult::InvalidInput);
        println!("✓ Test 2 passed: null query returns InvalidInput");
    }

    // Test 3: null response returns InvalidInput
    {
        let computation_id = SokrComputationId { high: 0, low: 0 };
        let ir_data = [0u8; 4];
        let query = SokrCapabilityQuery {
            computation_id,
            ir_format: b"TEST\0" as *const _ as *const i8,
            ir_data_ptr: ir_data.as_ptr() as *const c_void,
            ir_data_len: ir_data.len(),
            padding: [0; 8],
        };

        let result = capability(std::ptr::null(), &query, std::ptr::null_mut());
        assert_eq!(result, SokrResult::InvalidInput);
        println!("✓ Test 3 passed: null response returns InvalidInput");
    }

    println!("\nAll tests passed!");
}
