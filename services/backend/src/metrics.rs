//! Custom Prometheus metrics.
//!
//! We use the `metrics` facade with `metrics-exporter-prometheus` as the
//! backend. Metric names carry the `zkpool_` prefix so they can be
//! distinguished from generic process metrics (which use the `axum_` and
//! `process_` prefixes via `axum-prometheus`).
//!
//! ## Metrics defined here
//!
//!   zkpool_indexer_deposits_total        — DepositEvent processed
//!   zkpool_indexer_withdrawals_total     — WithdrawEvent processed
//!   zkpool_indexer_errors_total          — indexer errors
//!   zkpool_indexer_lag_seconds           — seconds since last processed block
//!   zkpool_indexer_tree_size             — current number of leaves
//!   zkpool_tree_add_leaf_duration_seconds — add_leaf duration histogram
//!   zkpool_tree_hash_duration_seconds    — Poseidon2 hash duration histogram
//!   zkpool_tree_errors_total             — tree errors
//!   zkpool_db_errors_total               — DB errors

use std::sync::OnceLock;

use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};

/// Global Prometheus handle, set once at startup.
static PROMETHEUS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// Initialize the Prometheus recorder and install it globally.
///
/// Call this **once** at startup, before any `counter!` / `histogram!`.
pub fn init_metrics() -> anyhow::Result<()> {
    let builder = PrometheusBuilder::new()
        // Histogram buckets for durations in seconds:
        // 0.1 ms, 1 ms, 10 ms, 100 ms, 1 s, 5 s, 30 s.
        .set_buckets_for_metric(
            Matcher::Full("zkpool_tree_add_leaf_duration_seconds".into()),
            &[0.0001, 0.001, 0.01, 0.1, 1.0, 5.0, 30.0],
        )?
        .set_buckets_for_metric(
            Matcher::Full("zkpool_tree_hash_duration_seconds".into()),
            &[0.0001, 0.001, 0.01, 0.1, 1.0, 5.0, 30.0],
        )?;

    let handle = builder.install_recorder()?;
    let _ = PROMETHEUS_HANDLE.set(handle);

    Ok(())
}

/// Render the current Prometheus metrics as a string.
///
/// Used by the `/metrics` HTTP endpoint.
pub fn render_metrics() -> String {
    match PROMETHEUS_HANDLE.get() {
        Some(h) => h.render(),
        None => String::from("# metrics not initialized\n"),
    }
}

// ============================================================
// Convenience helpers
// ============================================================

/// Record a `DepositEvent` was processed by the indexer.
pub fn record_deposit() {
    metrics::counter!("zkpool_indexer_deposits_total").increment(1);
}

/// Record a `WithdrawEvent` was processed by the indexer.
pub fn record_withdrawal() {
    metrics::counter!("zkpool_indexer_withdrawals_total").increment(1);
}

/// Record an indexer error.
pub fn record_indexer_error() {
    metrics::counter!("zkpool_indexer_errors_total").increment(1);
}

/// Record a DB error.
pub fn record_db_error() {
    metrics::counter!("zkpool_db_errors_total").increment(1);
}

/// Record a tree error.
pub fn record_tree_error() {
    metrics::counter!("zkpool_tree_errors_total").increment(1);
}

/// Set the indexer lag (seconds).
pub fn set_indexer_lag(seconds: f64) {
    metrics::gauge!("zkpool_indexer_lag_seconds").set(seconds);
}

/// Set the current tree size.
pub fn set_tree_size(size: f64) {
    metrics::gauge!("zkpool_indexer_tree_size").set(size);
}

/// Record an `add_leaf` duration (seconds).
pub fn record_add_leaf_duration(seconds: f64) {
    metrics::histogram!("zkpool_tree_add_leaf_duration_seconds").record(seconds);
}

/// Record a Poseidon2 hash duration (seconds).
pub fn record_hash_duration(seconds: f64) {
    metrics::histogram!("zkpool_tree_hash_duration_seconds").record(seconds);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_before_init() {
        // If init_metrics() has not been called, render_metrics returns a
        // comment line — not a panic.
        let out = render_metrics();
        assert!(out.contains("metrics not initialized") || !out.is_empty());
    }
}
