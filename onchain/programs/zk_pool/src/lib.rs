//! zk_pool — private SOL transfers on Solana using ZK proofs.
//!
//! Stage 4.1.5: `pool` instruction — initialize the pool.
//! Remaining instructions (`deposit`, `withdraw`) are added in 4.1.6 – 4.1.7.

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
    /// Can be called only once. Subsequent calls fail with
    /// `AccountAlreadyInUse` (from the `init` constraint).
    pub fn pool(ctx: Context<Pool>) -> Result<()> {
        instructions::pool::handler(ctx)
    }
}
