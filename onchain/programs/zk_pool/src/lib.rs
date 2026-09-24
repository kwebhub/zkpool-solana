//! zk_pool — private SOL transfers on Solana using ZK proofs.
//!
//! Stage 4.1.7: `pool`, `deposit`, `withdraw` instructions.

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

    /// Withdraw SOL from the pool using a Groth16 proof.
    ///
    /// Verifies the proof via CPI to the Sunspot verifier, prevents
    /// double-spend via `NullifierRecord` PDA, and transfers SOL from
    /// the vault to the recipient.
    #[allow(clippy::too_many_arguments)]
    pub fn withdraw(
        ctx: Context<Withdraw>,
        proof: Vec<u8>,
        nullifier_hash: [u8; 32],
        root: [u8; 32],
        recipient: Pubkey,
        amount: u64,
        recipient_binding: [u8; 32],
    ) -> Result<()> {
        instructions::withdraw::handler_withdraw(
            ctx,
            proof,
            nullifier_hash,
            root,
            recipient,
            amount,
            recipient_binding,
        )
    }
}
