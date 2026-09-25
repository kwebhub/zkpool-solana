//! Poseidon2 hashing via @noir-lang/noir_js.
//!
//! Why this file exists:
//!   Rust and JS Poseidon2 implementations produce DIFFERENT hashes than the
//!   Noir circuit's builtin. To guarantee byte-identical results, we load the
//!   SAME ACIR (hash2.json) that the withdrawal circuit uses, and execute it
//!   via noir_js. This is the only way to guarantee identity across layers.
//!
//! API:
//!   poseidon2Hash(left, right) -> Promise<string>  // 0x-prefixed 32-byte hex
//!
//! Both inputs and the output are 0x-prefixed hex strings representing BN254
//! field elements (32 bytes / 64 hex digits).

import { Noir } from "@noir-lang/noir_js";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const CIRCUIT_PATH = join(__dirname, "..", "circuits", "hash2.json");

// Load circuit once at module init.
const circuit = JSON.parse(readFileSync(CIRCUIT_PATH, "utf8"));
const noir = new Noir(circuit);

/**
 * Compute Poseidon2 hash of two field elements.
 *
 * @param {string} left  - 0x-prefixed hex string (field element)
 * @param {string} right - 0x-prefixed hex string (field element)
 * @returns {Promise<string>} 0x-prefixed hex string (field element)
 */
export async function poseidon2Hash(left, right) {
  const result = await noir.execute({ left, right });
  return result.returnValue;
}
