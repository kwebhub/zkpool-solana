//! Instruction modules.
//!
//! Stage 4.1.6: `pool` (initialize), `deposit` (deposit SOL).
//! Remaining instruction (`withdraw`) is added in 4.1.7.
//!
//! We use glob re-exports because Anchor's `#[program]` macro needs to see
//! the generated `__client_accounts_*` structures for CPI. The handler
//! functions have distinct names (`handler_pool`, `handler_deposit`) to
//! avoid ambiguity.

pub mod deposit;
pub mod pool;

pub use deposit::*;
pub use pool::*;
