//! Parse a saved deposit note (JSON) into a structured object.
//!
//! The note format matches `DepositForm.vue::copyNote()`:
//! {
//!   "nullifier":     "<bare hex, 64 chars>",
//!   "secret":        "<bare hex, 64 chars>",
//!   "note_secret":   "<bare hex, 64 chars>",
//!   "amount":        "<bare hex, 64 chars>",
//!   "commitment":    "<bare hex, 64 chars>",
//!   "nullifier_hash":"<bare hex, 64 chars>",
//!   "tx_signature":  "<base58>",
//!   "pool_pda":      "<base58>"
//! }

export interface ParsedNote {
  nullifier: string;
  secret: string;
  noteSecret: string;
  amount: string;
  commitment: string;
  nullifierHash: string;
  txSignature: string;
  poolPda: string;
}

const HEX64_RE = /^[0-9a-f]{64}$/;

function requireHex64(v: unknown, name: string): string {
  if (typeof v !== "string" || !HEX64_RE.test(v)) {
    throw new Error(`note.${name} must be a 64-char bare hex string`);
  }
  return v;
}

function requireString(v: unknown, name: string): string {
  if (typeof v !== "string" || v.length === 0) {
    throw new Error(`note.${name} must be a non-empty string`);
  }
  return v;
}

/**
 * Parse JSON text (or a pre-parsed object) into a validated `ParsedNote`.
 */
export function parseNote(input: string | Record<string, unknown>): ParsedNote {
  let raw: Record<string, unknown>;
  if (typeof input === "string") {
    try {
      raw = JSON.parse(input);
    } catch (e) {
      throw new Error(`note is not valid JSON: ${e instanceof Error ? e.message : String(e)}`);
    }
  } else {
    raw = input;
  }

  return {
    nullifier: requireHex64(raw.nullifier, "nullifier"),
    secret: requireHex64(raw.secret, "secret"),
    noteSecret: requireHex64(raw.note_secret, "note_secret"),
    amount: requireHex64(raw.amount, "amount"),
    commitment: requireHex64(raw.commitment, "commitment"),
    nullifierHash: requireHex64(raw.nullifier_hash, "nullifier_hash"),
    txSignature: requireString(raw.tx_signature, "tx_signature"),
    poolPda: requireString(raw.pool_pda, "pool_pda"),
  };
}
