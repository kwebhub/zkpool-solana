//! Program constants.
//!
//! All values here must be consistent with:
//!   - `circuits/withdrawal/spec.json` (circuit-level constants),
//!   - `circuits/withdrawal/src/main.nr` (on-circuit globals),
//!   - the verifier program deployed at `VERIFIER_PROGRAM_ID`.

use anchor_lang::prelude::*;

// ============================================================
// PDA seeds
// ============================================================

/// Seed for the `PoolState` PDA.
pub const POOL_SEED: &[u8] = b"pool3";

/// Seed for the vault PDA (holds SOL).
pub const VAULT_SEED: &[u8] = b"vault3";

/// Seed for the per-nullifier record PDA (prevents double-spend).
pub const NULLIFIER_RECORD_SEED: &[u8] = b"nullifier_record";

// ============================================================
// Merkle tree
// ============================================================

/// Merkle tree depth.
///
/// Must equal:
///   - `spec.json` → `circuit.tree_depth` (20),
///   - `circuits/withdrawal/src/main.nr` → `global TREE_DEPTH` (20).
pub const TREE_DEPTH: usize = 20;

/// Maximum number of leaves: 2^TREE_DEPTH = 1 048 576.
pub const MAX_LEAVES: u64 = 1 << TREE_DEPTH;

/// Number of historical roots to keep.
///
/// Roots change on every deposit. We keep the last `ROOT_HISTORY_SIZE`
/// roots, so that a user can submit a proof built against a recent (but
/// not necessarily the latest) root.
pub const ROOT_HISTORY_SIZE: usize = 10;

/// Empty root — all-zero root, used before any deposit.
pub const EMPTY_ROOT: [u8; 32] = [0u8; 32];

// ============================================================
// ZK proof
// ============================================================

/// Number of public inputs for the withdrawal circuit.
///
/// Must equal `spec.json` → `circuit.nr_public_inputs` (5).
/// Public inputs, in order:
///   [0] root
///   [1] nullifier_hash
///   [2] recipient
///   [3] recipient_binding
///   [4] amount
pub const NR_PUBLIC_INPUTS: u32 = 5;

/// Total size of the encoded public inputs (bytes).
///
/// Layout: 12-byte header + 5 × 32-byte fields = 172 bytes.
/// Must equal `spec.json` → `witness_layout.total_bytes` (172).
pub const PUBLIC_INPUTS_BYTES: usize = 172;

/// Length of the Groth16 proof (bytes).
///
/// Groth16 proofs are constant-size: 2 G1 points (32 bytes each) +
/// 1 G2 point (64 bytes) + metadata.
pub const PROOF_LEN: usize = 324;

/// Verifier program ID (deployed in stage 3.4).
///
/// The `withdraw` instruction calls this program via CPI to verify the
/// Groth16 proof. If the circuit is ever changed, the verifier program
/// must be rebuilt and this constant must be updated.
pub const VERIFIER_PROGRAM_ID: Pubkey = pubkey!("5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ");

// ============================================================
// Economics
// ============================================================

/// Minimum deposit amount, in lamports (0.001 SOL).
///
/// Prevents spam deposits that would fill the Merkle tree with dust.
pub const MIN_DEPOSIT_AMOUNT: u64 = 1_000_000;
