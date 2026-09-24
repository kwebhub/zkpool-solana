//! zk_pool — private SOL transfers on Solana using ZK proofs.
//!
//! Stage 4.1.1: minimal skeleton that compiles.
//! Real instructions are added in stages 4.1.5 – 4.1.7.

use anchor_lang::prelude::*;

declare_id!("8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm");

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

#[program]
pub mod zk_pool {
    // No instructions yet — added in stages 4.1.5 – 4.1.7.
}
