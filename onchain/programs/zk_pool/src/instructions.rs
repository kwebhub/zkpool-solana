//! Instruction modules.
//!
//! Stage 4.1.5: `pool` — initialize the pool.
//! Remaining instructions (`deposit`, `withdraw`) are added in 4.1.6 – 4.1.7.

pub mod pool;
pub use pool::*;
