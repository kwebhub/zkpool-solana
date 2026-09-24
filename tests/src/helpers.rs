//! Helpers for LiteSVM-based tests.
//!
//! This module provides:
//!   - `setup_svm()` — creates a LiteSVM instance with both programs loaded.
//!   - Loading of `zk_pool.so` and `withdrawal.so` from the build artifacts.
//!
//! Additional helpers (creating accounts, calling instructions, reading
//! state) are added in subsequent sub-stages.

use litesvm::LiteSVM;
use solana_address::Address;

/// Path to the compiled `zk_pool` program.
const ZK_POOL_SO: &str = "/home/ubuntu/onchain/target/deploy/zk_pool.so";

/// Path to the compiled Sunspot verifier program.
const VERIFIER_SO: &str = "/home/ubuntu/circuits/withdrawal/target/withdrawal.so";

/// Program IDs.
pub const ZK_POOL_ID: &str = "8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm";
pub const VERIFIER_ID: &str = "5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ";

/// Creates a LiteSVM instance with both programs loaded:
///   - `zk_pool`
///   - the Sunspot verifier (needed for `withdraw` CPI).
pub fn setup_svm() -> LiteSVM {
    let mut svm = LiteSVM::new();

    let zk_pool_id: Address = ZK_POOL_ID.parse().expect("valid zk_pool ID");
    let verifier_id: Address = VERIFIER_ID.parse().expect("valid verifier ID");

    svm.add_program_from_file(zk_pool_id, ZK_POOL_SO)
        .expect("failed to load zk_pool.so");

    svm.add_program_from_file(verifier_id, VERIFIER_SO)
        .expect("failed to load withdrawal.so");

    svm
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_svm_loads_both_programs() {
        let svm = setup_svm();
        let zk_pool_id: Address = ZK_POOL_ID.parse().unwrap();
        let verifier_id: Address = VERIFIER_ID.parse().unwrap();

        assert!(svm.get_account(&zk_pool_id).is_some(), "zk_pool not loaded");
        assert!(
            svm.get_account(&verifier_id).is_some(),
            "verifier not loaded"
        );
    }
}
