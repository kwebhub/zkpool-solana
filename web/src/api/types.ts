//! TypeScript types for the backend HTTP API.
//!
//! All types mirror `services/backend/src/main.rs` responses.

/** Bare-hex string (no `0x` prefix). */
export type Hex = string;

/** Base64-encoded bytes. */
export type Base64 = string;

// ---- /api/health ----

export interface HealthResponse {
  status: "ok" | "degraded";
  db: boolean;
  version: string;
}

// ---- /api/commitments ----

export interface CommitmentRecord {
  id: number;
  leaf_index: number;
  /** Bare hex, 64 chars. */
  commitment: Hex;
  pool_address: string;
  /** Transaction signature; may be null for synthetic rows. */
  tx_signature: string | null;
  /** ISO-8601 timestamp. */
  created_at: string;
}

export interface CommitmentsResponse {
  pool_address: string;
  count: number;
  commitments: CommitmentRecord[];
}

// ---- /api/root ----

export interface RootResponse {
  pool_address: string;
  /** Bare hex, 64 chars; `null` if no root yet. */
  root: Hex | null;
}

// ---- /api/proof ----

export interface ProofResponse {
  pool_address: string;
  leaf_index: number;
  /** 20 sibling hashes, bare hex. */
  proof: Hex[];
  /** 20 side flags. true = our node is the left child. */
  is_even: boolean[];
}

// ---- /api/withdraw ----

export interface WithdrawRequest {
  // Public
  root: Hex;
  nullifier_hash: Hex;
  recipient: Hex;
  recipient_binding: Hex;
  amount: Hex;
  // Private
  nullifier: Hex;
  secret: Hex;
  note_secret: Hex;
  merkle_proof: Hex[];
  is_even: boolean[];
}

export interface WithdrawResponse {
  /** Base64-encoded Groth16 proof (324 bytes). */
  proof: Base64;
  /** Base64-encoded public witness (172 bytes). */
  public_witness: Base64;
}

// ---- Errors ----

export interface ApiError {
  error: string;
  [key: string]: unknown;
}
