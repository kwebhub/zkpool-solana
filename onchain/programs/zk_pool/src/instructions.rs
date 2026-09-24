//! Instruction modules.
//!
//! Stage 4.1.7: `pool`, `deposit`, `withdraw`.
//!
//! We use glob re-exports because Anchor's `#[program]` macro needs to see
//! the generated `__client_accounts_*` structures for CPI. The handler
//! functions have distinct names (`handler_pool`, `handler_deposit`,
//! `handler_withdraw`) to avoid ambiguity.

pub mod deposit;
pub mod pool;
pub mod withdraw;

pub use deposit::*;
pub use pool::*;
pub use withdraw::*;
