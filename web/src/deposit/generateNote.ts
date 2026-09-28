//! Generate a deposit note: four secrets + derived commitment.
//!
//! The note is what the user must save to withdraw later. Loss of the note
//! means loss of funds (no on-chain recovery).

import { computeHashes } from "../noir/hashes";

export interface DepositNote {
  /** Random field element, hashed into nullifier_hash. */
  nullifier: string;
  /** Random field element, part of the commitment. */
  secret: string;
  /** Random field element, part of recipient_binding. */
  noteSecret: string;
  /** Deposit amount (lamports, as hex, 64 chars). */
  amount: string;
  /** Derived commitment = hash_3(nullifier, secret, amount). Bare hex. */
  commitment: string;
  /** Derived nullifier_hash = hash_1(nullifier). Bare hex. */
  nullifierHash: string;
}

/**
 * Generate a random field element as bare hex (64 chars).
 *
 * Uses Web Crypto API. Top byte is masked to keep the value comfortably
 * below the BN254 field prime (2^254), matching the Noir circuit's
 * expectation.
 */
function randomFieldHex(): string {
  const bytes = new Uint8Array(32);
  crypto.getRandomValues(bytes);
  // Clear the top 3 bits — the BN254 prime begins with 0x30, so keeping
  // the top byte ≤ 0x1f guarantees the value is below the modulus.
  bytes[0] &= 0x1f;
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

/**
 * Generate a fresh deposit note for the given amount (lamports).
 */
export async function generateNote(amountLamports: bigint): Promise<DepositNote> {
  const nullifier = randomFieldHex();
  const secret = randomFieldHex();
  const noteSecret = randomFieldHex();
  const amount = amountLamports.toString(16).padStart(64, "0");

  const { commitment, nullifierHash } = await computeHashes(nullifier, secret, amount);

  return {
    nullifier,
    secret,
    noteSecret,
    amount,
    commitment,
    nullifierHash,
  };
}
