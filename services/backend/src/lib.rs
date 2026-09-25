//! zkpool_backend library.
//!
//! Modules:
//!   - `config`  — environment configuration.
//!   - `db`      — Postgres database wrapper.
//!   - `cache`   — Redis cache wrapper.
//!   - `tree`    — incremental Merkle tree (Redis + Merkle service).
//!   - `logging` — tracing initialization.
//!   - `metrics` — custom Prometheus metrics.
//!
//! Additional modules (indexer, http handlers, etc.) are added in subsequent
//! sub-stages of Stage 5.

pub mod cache;
pub mod config;
pub mod db;
pub mod logging;
pub mod metrics;
pub mod tree;
