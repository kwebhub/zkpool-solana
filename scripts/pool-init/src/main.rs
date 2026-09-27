//! pool-init — initialize the zkpool on-chain pool on Solana devnet.
//!
//! Builds the `pool` instruction by hand (8-byte discriminator + 3 accounts
//! + system program) and sends it via raw JSON-RPC. No `solana-client`
//! dependency — avoids version conflicts with the modular `solana-*` crates.

fn main() -> anyhow::Result<()> {
    println!("pool-init placeholder");
    Ok(())
}
