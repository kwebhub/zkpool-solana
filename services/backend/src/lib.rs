//! zkpool_backend library.
//!
//! Modules:
//!   - `config` — environment configuration.
//!   - `db`     — Postgres database wrapper.
//!
//! Additional modules (cache, tree, indexer, metrics, etc.) are added in
//! subsequent sub-stages of Stage 5.

pub mod config;
pub mod db;
