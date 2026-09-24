//! Test-only module: generates a valid witness for the withdrawal circuit
//! and prints it in `Prover.toml` format.
//!
//! Usage:
//!   cd circuits/withdrawal
//!   nargo test test_generate_valid_inputs --show-output
//!
//! Copy the output between `=== Prover.toml ===` and `=== End ===` into
//! `Prover.toml`. That file is gitignored and is consumed by `nargo execute`
//! to produce a witness for `sunspot prove`.
//!
//! IMPORTANT: this module is compiled into the circuit. Any change here
//! changes the ACIR, and therefore CCS/PK/VK/.so and the verifier Program ID.
//! Keep the logic minimal and stable.
//!
//! NOTE: Field values must be < 2^254 (BN254 prime). All literals below are
//! chosen small enough to fit.

use crate::compute_merkle_root;
use poseidon::{hash_1, hash_2, hash_3};

/// Builds a valid witness for a 20-level tree and prints it in Prover.toml
/// format. The tree is synthetic — we don't need it to be canonical, we only
/// need the root to be internally consistent so the constraint passes.
#[test]
fn test_generate_valid_inputs() {
    // --- private inputs ---
    let nullifier: Field = 111111111111111111;
    let secret: Field = 222222222222222222;
    let note_secret: Field = 333333333333333333;
    let amount: Field = 1_000_000; // 0.001 SOL in lamports

    // --- derive commitment, nullifier_hash, recipient_binding ---
    let commitment = hash_3(nullifier, secret, amount);
    let nullifier_hash = hash_1(nullifier);

    let recipient: Field = 444444444444444444;
    let recipient_binding = hash_2(note_secret, recipient);

    // --- build synthetic 20-level Merkle tree ---
    let s0: Field = 555555555555555555;

    let mut merkle_proof: [Field; 20] = [0; 20];
    let mut is_even: [bool; 20] = [false; 20];

    merkle_proof[0] = s0;
    is_even[0] = true;
    for i in 1..20 {
        merkle_proof[i] = 0;
        is_even[i] = true;
    }

    let root = compute_merkle_root::<20>(commitment, merkle_proof, is_even);

    // --- print Prover.toml (markers only; content parsed by a host script) ---
    println("=== Prover.toml ===");
    println(root);
    println(nullifier_hash);
    println(recipient);
    println(recipient_binding);
    println(amount);
    println(nullifier);
    println(secret);
    println(note_secret);

    // merkle_proof: 20 field elements, one per line
    for i in 0..20 {
        println(merkle_proof[i]);
    }

    // is_even: 20 booleans, printed as 1/0
    for i in 0..20 {
        if is_even[i] {
            println(1);
        } else {
            println(0);
        }
    }
    println("=== End ===");
}
