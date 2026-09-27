//! Shared API types between the lib and the bin.
//!
//! These types live in the lib (not in `main.rs`) because the bin uses
//! them with `reqwest::Response::json::<T>()`, and `T` must come from the
//! same crate instance as the `serde` used by `reqwest`. Putting them in
//! the lib guarantees this.

use serde::{Deserialize, Serialize};

/// Merkle tree depth — must match the circuit, spec, and on-chain constant.
pub const TREE_DEPTH: usize = 20;

/// Request body for `POST /api/withdraw`.
///
/// Field names and encoding (bare hex, no `0x`) match the prover service's
/// `WitnessInputs` — see `services/prover/src/witness.rs`.
///
/// Hex strings:
///   - `root`, `nullifier_hash`, `recipient_binding`, `nullifier`, `secret`,
///     `note_secret`: 64-char bare hex (32 bytes)
///   - `recipient`, `amount`: shorter hex is allowed (nargo pads)
///   - `merkle_proof[i]`: any hex length; `"00"` for empty siblings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WithdrawRequest {
    // Public
    pub root: String,
    pub nullifier_hash: String,
    pub recipient: String,
    pub recipient_binding: String,
    pub amount: String,
    // Private
    pub nullifier: String,
    pub secret: String,
    pub note_secret: String,
    pub merkle_proof: Vec<String>,
    pub is_even: Vec<bool>,
}

/// Response body from `/api/withdraw`.
///
/// Base64-encoded proof + public witness — the wire format the frontend
/// expects. The prover returns bare hex; the backend converts.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WithdrawResponse {
    /// Base64-encoded Groth16 proof (324 bytes).
    pub proof: String,
    /// Base64-encoded public witness (172 bytes).
    pub public_witness: String,
}

/// Internal representation of the prover's response (bare hex).
///
/// Used only inside `post_withdraw` — not exposed on the API.
#[derive(Debug, Clone, Deserialize)]
pub struct ProverResponse {
    pub proof: String,
    pub public_witness: String,
}

impl WithdrawRequest {
    /// Validate shape before forwarding to the prover.
    pub fn validate(&self) -> Result<(), String> {
        if self.merkle_proof.len() != TREE_DEPTH {
            return Err(format!(
                "merkle_proof must have {} elements, got {}",
                TREE_DEPTH,
                self.merkle_proof.len()
            ));
        }
        if self.is_even.len() != TREE_DEPTH {
            return Err(format!(
                "is_even must have {} elements, got {}",
                TREE_DEPTH,
                self.is_even.len()
            ));
        }
        Ok(())
    }
}
