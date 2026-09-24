//! LiteSVM integration tests for zk_pool.
//!
//! Tests run entirely in-process against a local Solana VM. They do not
//! require a validator, devnet, or SOL. Two programs are loaded:
//!   - `zk_pool` (the pool itself),
//!   - `sunspot_verifier` (the Groth16 verifier, compiled in stage 3.3).
//!
//! The `withdraw` test uses a real proof produced by `sunspot prove` in
//! stage 3.5. See `proofs/` for the fixtures.

#[cfg(test)]
mod helpers;

#[cfg(test)]
mod test_pool;

#[cfg(test)]
mod test_deposit;

#[cfg(test)]
mod test_withdraw;

#[cfg(test)]
mod test_double_spend;
