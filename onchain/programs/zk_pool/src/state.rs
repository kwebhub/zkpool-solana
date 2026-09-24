//! State accounts.
//!
//! Two account types:
//!   - `PoolState` — the pool metadata: authority, Merkle tree state, deposit counter.
//!   - `NullifierRecord` — a per-nullifier marker that prevents double-spend.

use anchor_lang::prelude::*;

use crate::constants::{MAX_LEAVES, ROOT_HISTORY_SIZE};

/// Pool state account.
///
/// PDA seeds: `[POOL_SEED]`.
///
/// Contains:
///   - authority:           who created the pool (for future admin actions).
///   - next_leaf_index:     the next free index in the Merkle tree.
///   - total_deposits:      monotonic counter of successful deposits.
///   - current_root_index:  index into `roots` of the most recent root.
///   - roots:               ring buffer of the last ROOT_HISTORY_SIZE roots.
///
/// Size: 8 + 32 + 8 + 8 + 8 + (10 × 32) = 384 bytes.
#[account]
#[derive(InitSpace)]
pub struct PoolState {
    /// Pool creator. Will be used for future admin actions (pause, upgrade).
    pub authority: Pubkey,

    /// Next free leaf index in the Merkle tree.
    /// Invariant: `next_leaf_index < MAX_LEAVES`.
    pub next_leaf_index: u64,

    /// Total number of successful deposits.
    pub total_deposits: u64,

    /// Index in `roots` of the most recent root.
    pub current_root_index: u64,

    /// Ring buffer of the last `ROOT_HISTORY_SIZE` roots.
    /// Empty slots are all-zero and are treated as "not present".
    pub roots: [[u8; 32]; ROOT_HISTORY_SIZE],
}

impl PoolState {
    /// Returns `true` if `root` is in the recent roots history.
    ///
    /// Iterates over all `ROOT_HISTORY_SIZE` slots. All-zero roots are
    /// skipped (they represent empty slots before any deposit).
    pub fn is_known_root(&self, root: &[u8; 32]) -> bool {
        if *root == [0u8; 32] {
            return false;
        }
        self.roots.iter().any(|r| r == root)
    }

    /// Adds `new_root` to the ring buffer, moving the cursor forward.
    pub fn add_root(&mut self, new_root: [u8; 32]) {
        let next_index = ((self.current_root_index + 1) as usize) % ROOT_HISTORY_SIZE;
        self.roots[next_index] = new_root;
        self.current_root_index = next_index as u64;
    }

    /// Returns the current (most recent) root.
    pub fn current_root(&self) -> [u8; 32] {
        self.roots[self.current_root_index as usize]
    }

    /// Returns `true` if the tree has room for another leaf.
    pub fn has_room(&self) -> bool {
        self.next_leaf_index < MAX_LEAVES
    }
}

/// Per-nullifier marker that prevents double-spend.
///
/// PDA seeds: `[NULLIFIER_RECORD_SEED, pool, nullifier_hash]`.
///
/// If this account exists, the nullifier has been used. The `init`
/// constraint in the `withdraw` instruction will fail if the account
/// already exists — that is how double-spend is prevented.
///
/// Size: 8 + 32 + 32 + 8 + 8 = 88 bytes.
#[account]
#[derive(InitSpace)]
pub struct NullifierRecord {
    /// Pool this nullifier belongs to.
    pub pool: Pubkey,

    /// The nullifier hash that was spent.
    pub nullifier_hash: [u8; 32],

    /// Recipient that received the SOL.
    pub recipient: Pubkey,

    /// Amount transferred, in lamports.
    pub amount: u64,

    /// Unix timestamp of the withdrawal.
    pub timestamp: i64,
}
