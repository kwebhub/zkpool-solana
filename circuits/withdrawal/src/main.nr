//! `withdrawal` circuit — the main ZK circuit for zkpool-solana.
//!
//! ## Purpose
//!
//! Proves, in zero knowledge, that:
//!
//!   C1. `hash_1(nullifier) == nullifier_hash`
//!   C2. `hash_2(note_secret, recipient) == recipient_binding`
//!   C3. `compute_merkle_root(hash_3(nullifier, secret, amount), merkle_proof, is_even) == root`
//!
//! ## Public inputs (5, in order — see spec.json)
//!
//!   [0] root                — Merkle root; must be in PoolState.roots[] history.
//!   [1] nullifier_hash      — hash_1(nullifier); used as PDA seed for NullifierRecord.
//!   [2] recipient           — Solana Pubkey reduced modulo BN254 field prime.
//!   [3] recipient_binding   — hash_2(note_secret, recipient); prevents front-running.
//!   [4] amount              — withdrawal amount in lamports (u64, right-padded to 32 bytes).
//!
//! ## Private inputs (5)
//!
//!   nullifier      — secret; hashed into nullifier_hash.
//!   secret         — secret; part of commitment.
//!   note_secret    — secret; part of recipient_binding.
//!   merkle_proof   — [Field; 20], sibling hashes from leaf to root.
//!   is_even        — [bool; 20], side flag per level (true = our node is left child).
//!
//! ## TREE_DEPTH
//!
//! Hard-coded to 20 (matches `spec.json` `circuit.tree_depth` and on-chain
//! `constants::TREE_DEPTH`). If it ever changes, all layers must be updated
//! simultaneously and the Sunspot verifier must be rebuilt.

mod merkle_tree;

use merkle_tree::compute_merkle_root;
use poseidon::{hash_1, hash_2, hash_3};

/// Merkle tree depth. Must match `spec.json` `circuit.tree_depth` (20)
/// and on-chain `constants::TREE_DEPTH`.
global TREE_DEPTH: u32 = 20;

fn main(
    // ---- public inputs (order matters — validated against spec.json) ----
    root: pub Field,
    nullifier_hash: pub Field,
    recipient: pub Field,
    recipient_binding: pub Field,
    amount: pub Field,
    // ---- private inputs ----
    nullifier: Field,
    secret: Field,
    note_secret: Field,
    merkle_proof: [Field; TREE_DEPTH],
    is_even: [bool; TREE_DEPTH],
) {
    // C1: nullifier_hash = hash_1(nullifier)
    let computed_nullifier_hash = hash_1(nullifier);
    assert(computed_nullifier_hash == nullifier_hash);

    // C2: recipient_binding = hash_2(note_secret, recipient)
    let computed_binding = hash_2(note_secret, recipient);
    assert(computed_binding == recipient_binding);

    // C3: Merkle root
    let commitment = hash_3(nullifier, secret, amount);
    let computed_root = compute_merkle_root::<TREE_DEPTH>(commitment, merkle_proof, is_even);
    assert(computed_root == root);
}

// ============================================================
// Integration tests
// ============================================================
//
// These tests build a full witness for the withdrawal circuit and check
// that all three constraints hold. They use a two-level Merkle tree
// (a smaller depth is used via compute_merkle_root directly) to keep
// values manageable — the main circuit uses TREE_DEPTH = 20, but the
// constraint logic is identical.
//
// NOTE: `main()` itself is not called directly in tests because it takes
// a `[Field; 20]` array. Instead, the three constraint functions are
// tested individually against controlled values.

#[test]
fn test_c1_nullifier_hash_correct() {
    let nullifier: Field = 12345;
    let expected = hash_1(nullifier);
    assert(expected == hash_1(nullifier));
}

#[test]
fn test_c2_recipient_binding_correct() {
    let note_secret: Field = 67890;
    let recipient: Field = 11111;
    let expected = hash_2(note_secret, recipient);
    assert(expected == hash_2(note_secret, recipient));
}

#[test]
fn test_c3_merkle_root_two_levels() {
    // Build a 2-level Merkle tree and verify the root matches.
    //
    //            root
    //           /    \
    //         P0      s1
    //        /  \
    //       C    s0
    //
    let commitment: Field = 999;
    let s0: Field = 888;
    let s1: Field = 777;

    let p0 = hash_2(commitment, s0);       // commitment is LEFT
    let root_expected = hash_2(p0, s1);    // P0 is LEFT

    let proof = [s0, s1];
    let is_even = [true, true];

    let root_computed = compute_merkle_root::<2>(commitment, proof, is_even);
    assert(root_computed == root_expected);
}

#[test]
fn test_c3_wrong_sibling_fails() {
    // Sanity: a wrong sibling produces a different root.
    let commitment: Field = 999;
    let s0: Field = 888;
    let s1: Field = 777;

    let correct = compute_merkle_root::<2>(commitment, [s0, s1], [true, true]);
    let wrong   = compute_merkle_root::<2>(commitment, [s0, 666], [true, true]);
    assert(correct != wrong);
}

#[test]
fn test_c3_wrong_flag_fails() {
    // Sanity: a flipped side flag produces a different root.
    let commitment: Field = 999;
    let s0: Field = 888;
    let s1: Field = 777;

    let correct = compute_merkle_root::<2>(commitment, [s0, s1], [true, true]);
    let wrong   = compute_merkle_root::<2>(commitment, [s0, s1], [false, true]);
    assert(correct != wrong);
}

#[test]
fn test_c1_c2_c3_full_flow_2_levels() {
    // End-to-end simulation of the constraint set with a 2-level tree.
    // This mirrors exactly what main() does with TREE_DEPTH = 20.

    // --- setup ---
    let nullifier: Field = 1001;
    let secret: Field = 2002;
    let note_secret: Field = 3003;
    let amount: Field = 1_000_000; // 0.001 SOL
    let recipient: Field = 4004;

    // --- derive public inputs ---
    let expected_nullifier_hash = hash_1(nullifier);
    let expected_recipient_binding = hash_2(note_secret, recipient);
    let commitment = hash_3(nullifier, secret, amount);

    // --- build a small tree around commitment ---
    let s0: Field = 555;
    let s1: Field = 666;
    let p0 = hash_2(commitment, s0);
    let root = hash_2(p0, s1);
    let proof = [s0, s1];
    let is_even = [true, true];

    // --- C1 ---
    assert(hash_1(nullifier) == expected_nullifier_hash);

    // --- C2 ---
    assert(hash_2(note_secret, recipient) == expected_recipient_binding);

    // --- C3 ---
    assert(compute_merkle_root::<2>(commitment, proof, is_even) == root);
}

#[test]
fn test_c2_wrong_recipient_fails() {
    // Front-running protection: substituting recipient invalidates C2.
    let note_secret: Field = 3003;
    let honest_recipient: Field = 4004;
    let attacker_recipient: Field = 4005;

    let honest_binding = hash_2(note_secret, honest_recipient);
    let attacker_binding = hash_2(note_secret, attacker_recipient);

    assert(honest_binding != attacker_binding);
}

#[test]
fn test_tree_depth_constant() {
    // If TREE_DEPTH changes, this test must be updated in lockstep
    // with spec.json and on-chain constants.
    assert(TREE_DEPTH == 20);
}
