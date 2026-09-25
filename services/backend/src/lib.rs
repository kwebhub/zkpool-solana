//! zkpool_backend library.
//!
//! Modules:
//!   - `config`     — environment configuration.
//!   - `db`         — Postgres database wrapper.
//!   - `cache`      — Redis cache wrapper.
//!   - `tree`       — incremental Merkle tree (Redis + Merkle service).
//!   - `logging`    — tracing initialization.
//!   - `metrics`    — custom Prometheus metrics.
//!   - `rate_limit` — Redis-based rate limiting middleware.
//!   - `indexer`    — background worker: Solana RPC → DB + tree.
//!
//! Additional modules (HTTP handlers) are added in the next sub-stage.

pub mod cache;
pub mod config;
pub mod db;
pub mod indexer;
pub mod logging;
pub mod metrics;
pub mod rate_limit;
pub mod tree;
