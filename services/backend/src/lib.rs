//! zkpool_backend library.
//!
//! Modules:
//!   - `config` — environment configuration.
//!   - `db`     — Postgres database wrapper.
//!   - `cache`  — Redis cache wrapper.
//!   - `tree`   — incremental Merkle tree (Redis + Merkle service).
//!
//! Additional modules (indexer, metrics, etc.) are added in subsequent
//! sub-stages of Stage 5.

pub mod cache;
pub mod config;
pub mod db;
pub mod tree;
