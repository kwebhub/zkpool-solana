//! Shared API types between the lib and the bin.
//!
//! These types live in the lib (not in `main.rs`) because the bin uses
//! them with `reqwest::Response::json::<T>()`, and `T` must come from the
//! same crate instance as the `serde` used by `reqwest`. Putting them in
//! the lib guarantees this.

use serde::{Deserialize, Serialize};

/// Request body for `POST /api/withdraw`.
///
/// The witness is passed through to the prover service as-is. We don't
/// inspect it — the prover validates it against the circuit.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WithdrawRequest {
    /// Base64-encoded witness blob (the same format `sunspot prove`
    /// consumes from `withdrawal.gz`).
    pub witness: String,
}

/// Response body from `/api/withdraw`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WithdrawResponse {
    /// Base64-encoded Groth16 proof (324 bytes).
    pub proof: String,
    /// Base64-encoded public witness (172 bytes).
    pub public_witness: String,
}
