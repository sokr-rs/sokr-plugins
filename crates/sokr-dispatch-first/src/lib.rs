// SPDX-License-Identifier: MIT OR Apache-2.0
//! First-capable dispatch policy for SOKR.
//!
//! Maintains substrates in registration order. `first_capable()` returns the
//! first registered substrate, or `None` when none are registered.
//!
//! NOTE: capability filtering is not yet wired in — the registry does not query
//! each substrate's `capability_fn`, so the returned substrate is the first
//! *registered*, not a verified *capable* one. Real capability-based selection
//! (and a `NoCapableSubstrate` fallback) is deferred to a later phase.

use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchResult {
    Ok,
    Err,
}

pub struct SubstrateRegistry {
    substrates: Mutex<Vec<u32>>,
}

impl SubstrateRegistry {
    pub fn new() -> Self {
        SubstrateRegistry {
            substrates: Mutex::new(Vec::new()),
        }
    }

    pub fn register(&self, substrate_id: u32) -> DispatchResult {
        self.substrates.lock().unwrap().push(substrate_id);
        DispatchResult::Ok
    }

    pub fn substrate_count(&self) -> usize {
        self.substrates.lock().unwrap().len()
    }

    pub fn get_substrate(&self, index: usize) -> Option<u32> {
        self.substrates.lock().unwrap().get(index).copied()
    }

    pub fn clear(&self) {
        self.substrates.lock().unwrap().clear();
    }

    pub fn first_capable(&self) -> Option<u32> {
        self.substrates.lock().unwrap().first().copied()
    }
}

impl Default for SubstrateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = SubstrateRegistry::new();
        assert_eq!(registry.substrate_count(), 0);
    }

    #[test]
    fn test_register_single_substrate() {
        let registry = SubstrateRegistry::new();
        let result = registry.register(0);
        assert_eq!(result, DispatchResult::Ok);
        assert_eq!(registry.substrate_count(), 1);
    }

    #[test]
    fn test_register_multiple_substrates() {
        let registry = SubstrateRegistry::new();
        assert_eq!(registry.register(0), DispatchResult::Ok);
        assert_eq!(registry.register(1), DispatchResult::Ok);
        assert_eq!(registry.register(2), DispatchResult::Ok);
        assert_eq!(registry.substrate_count(), 3);
    }

    #[test]
    fn test_get_substrate_in_order() {
        let registry = SubstrateRegistry::new();
        registry.register(10);
        registry.register(20);
        registry.register(30);

        assert_eq!(registry.get_substrate(0), Some(10));
        assert_eq!(registry.get_substrate(1), Some(20));
        assert_eq!(registry.get_substrate(2), Some(30));
        assert_eq!(registry.get_substrate(3), None);
    }

    #[test]
    fn test_first_capable_single_substrate() {
        let registry = SubstrateRegistry::new();
        registry.register(5);
        assert_eq!(registry.first_capable(), Some(5));
    }

    #[test]
    fn test_first_capable_multiple_substrates() {
        let registry = SubstrateRegistry::new();
        registry.register(10);
        registry.register(20);
        registry.register(30);
        assert_eq!(registry.first_capable(), Some(10));
    }

    #[test]
    fn test_first_capable_none() {
        let registry = SubstrateRegistry::new();
        assert_eq!(registry.first_capable(), None);
    }

    #[test]
    fn test_clear_registry() {
        let registry = SubstrateRegistry::new();
        registry.register(0);
        registry.register(1);
        assert_eq!(registry.substrate_count(), 2);

        registry.clear();
        assert_eq!(registry.substrate_count(), 0);
    }

    #[test]
    fn test_empty_registry_zero_substrates() {
        let registry = SubstrateRegistry::new();
        assert_eq!(registry.substrate_count(), 0);
        assert_eq!(registry.first_capable(), None);
    }
}
