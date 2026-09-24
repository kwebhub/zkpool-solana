//! Program events.
//!
//! Events are emitted by the program and consumed by the off-chain indexer
//! (backend). They are parsed from transaction logs by their 8-byte Anchor
//! discriminator.
//!
//! IMPORTANT: do not change the field order or types after the program is
//! deployed — the indexer relies on a stable binary layout. If a change is
//! unavoidable, update `services/backend/src/indexer.rs` in lockstep.

use anchor_lang::prelude::*;

/// Emitted on every successful deposit.
///
/// Binary layout (after 8-byte discriminator):
///   commitment      [u8; 32]
///   leaf_index      u64
///   new_root        [u8; 32]
///   timestamp       i64
/// Total payload: 88 bytes.
#[event]
pub struct DepositEvent {
    /// Commitment inserted into the Merkle tree.
    pub commitment: [u8; 32],

    /// Index of the leaf in the tree (0-based).
    pub leaf_index: u64,

    /// New Merkle root after inserting the commitment.
    pub new_root: [u8; 32],

    /// Unix timestamp of the block.
    pub timestamp: i64,
}

/// Emitted on every successful withdrawal.
///
/// Binary layout (after 8-byte discriminator):
///   nullifier_hash  [u8; 32]
///   recipient       Pubkey (32 bytes)
///   amount          u64
///   timestamp       i64
/// Total payload: 80 bytes.
#[event]
pub struct WithdrawEvent {
    /// Nullifier hash of the spent deposit.
    pub nullifier_hash: [u8; 32],

    /// Recipient that received the SOL.
    pub recipient: Pubkey,

    /// Amount transferred, in lamports.
    pub amount: u64,

    /// Unix timestamp of the block.
    pub timestamp: i64,
}
