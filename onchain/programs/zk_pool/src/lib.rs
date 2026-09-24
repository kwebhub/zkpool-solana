//! zk_pool — private SOL transfers on Solana using ZK proofs.
//!
//! Stage 4.1.6: `pool` and `deposit` instructions.
//! `withdraw` is added in 4.1.7.

use anchor_lang::prelude::*;

declare_id!("8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm");

pub mod constants;
pub mod encoding;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

#[program]
pub mod zk_pool {
    use super::*;

    /// Initialize the pool.
    ///
    /// Creates two PDAs:
    ///   - `PoolState` (seeds `[POOL_SEED]`),
    ///   - `vault`     (seeds `[VAULT_SEED, pool]`).
    ///
    /// Can be called only once.
    pub fn pool(ctx: Context<Pool>) -> Result<()> {
        instructions::pool::handler_pool(ctx)
    }

    /// Deposit SOL into the pool.
    ///
    /// The depositor provides a commitment and the new Merkle root after
    /// inserting it. The instruction transfers SOL to the vault and updates
    /// the tree metadata.
    ///
    /// **Trust model:** does NOT verify that `new_root` is the correct
    /// result of inserting `commitment`. See `docs/DEMO-NOTICE.md`.
    pub fn deposit(
        ctx: Context<Deposit>,
        commitment: [u8; 32],
        new_root: [u8; 32],
        amount: u64,
    ) -> Result<()> {
        instructions::deposit::handler_deposit(ctx, commitment, new_root, amount)
    }
}
