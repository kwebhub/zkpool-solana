//! zkpool_backend — HTTP API + indexer.
//!
//! Stage 5.9.2: add read endpoints:
//!   - GET /api/health
//!   - GET /api/commitments?pool_address=X
//!   - GET /api/root?pool_address=X
//!   - GET /api/proof?pool_address=X&leaf_index=N  (stub)
//!   - GET /metrics
//!
//! POST /api/withdraw + rate limiting added in 5.9.3.

use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;
use tracing::info;

use zkpool_backend::cache::Cache;
use zkpool_backend::config::Config;
use zkpool_backend::db::Db;
use zkpool_backend::logging;
use zkpool_backend::metrics;
use zkpool_backend::tree::{MerkleClient, MerkleTree};

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Db,
    pub cache: Arc<Mutex<Cache>>,
    pub tree: Arc<Mutex<MerkleTree>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Config
    let config = Arc::new(Config::from_env()?);

    // 2. Logging
    logging::init_logging()?;
    info!("logging initialized");

    // 3. Metrics
    metrics::init_metrics()?;
    info!("metrics initialized");

    // 4. Postgres
    let db = Db::connect(&config.database_url).await?;
    info!("connected to Postgres");

    // 5. Redis
    let cache = Cache::connect(&config.redis_url).await?;
    info!("connected to Redis");
    let cache = Arc::new(Mutex::new(cache));

    // 6. Merkle tree
    let merkle_client = MerkleClient::new(config.merkle_url.clone());
    let tree = MerkleTree::new(
        config.pool_address.clone(),
        config.merkle_tree_depth,
        (*cache.lock().await).clone(),
        merkle_client,
    )?;
    let tree = Arc::new(Mutex::new(tree));
    info!("Merkle tree handle created (not initialized yet)");

    // 7. App state
    let state = AppState {
        config: config.clone(),
        db,
        cache,
        tree,
    };

    // 8. Router
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/commitments", get(get_commitments))
        .route("/api/root", get(get_root))
        .route("/api/proof", get(get_proof))
        .route("/metrics", get(metrics_endpoint))
        .with_state(state);

    // 9. Bind and serve
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("server listening on {}", addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("server error: {}", e))?;

    Ok(())
}

// ============================================================
// Handlers
// ============================================================

/// `GET /api/health` — health check.
async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let db_ok = state
        .db
        .next_leaf_index(&state.config.pool_address)
        .await
        .is_ok();

    let status = if db_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(json!({
            "status": if db_ok { "ok" } else { "degraded" },
            "db": db_ok,
            "version": env!("CARGO_PKG_VERSION"),
        })),
    )
}

#[derive(Debug, Deserialize)]
struct PoolQuery {
    #[serde(default)]
    pool_address: Option<String>,
}

/// `GET /api/commitments?pool_address=X` — list all commitments.
///
/// If `pool_address` is not provided, uses the configured pool address.
/// Cached in Redis for 30 seconds.
async fn get_commitments(
    State(state): State<AppState>,
    Query(q): Query<PoolQuery>,
) -> impl IntoResponse {
    let pool = q
        .pool_address
        .unwrap_or_else(|| state.config.pool_address.clone());
    let cache_key = format!("cache:commitments:{}", pool);

    // Try cache first.
    {
        let mut cache = state.cache.lock().await;
        if let Ok(Some(cached)) = cache.get(&cache_key).await {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&cached) {
                return (StatusCode::OK, Json(parsed)).into_response();
            }
        }
    }

    // Query DB.
    let rows = match state.db.list_commitments(&pool).await {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("list_commitments failed: {:#}", e);
            metrics::record_db_error();
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db query failed"})),
            )
                .into_response();
        }
    };

    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|c| {
            json!({
                "id": c.id,
                "leaf_index": c.leaf_index,
                "commitment": hex::encode(&c.commitment),
                "pool_address": c.pool_address,
                "tx_signature": c.tx_signature,
                "created_at": c.created_at.to_rfc3339(),
            })
        })
        .collect();

    let response = json!({
        "pool_address": pool,
        "count": items.len(),
        "commitments": items,
    });

    // Cache for 30 seconds.
    {
        let mut cache = state.cache.lock().await;
        if let Ok(s) = serde_json::to_string(&response) {
            let _ = cache.set_ex(&cache_key, &s, 30).await;
        }
    }

    (StatusCode::OK, Json(response)).into_response()
}

/// `GET /api/root?pool_address=X` — current Merkle root.
async fn get_root(State(state): State<AppState>, Query(q): Query<PoolQuery>) -> impl IntoResponse {
    let pool = q
        .pool_address
        .unwrap_or_else(|| state.config.pool_address.clone());
    let cache_key = format!("cache:root:{}", pool);

    // Try cache first.
    {
        let mut cache = state.cache.lock().await;
        if let Ok(Some(cached)) = cache.get(&cache_key).await {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&cached) {
                return (StatusCode::OK, Json(parsed)).into_response();
            }
        }
    }

    // Query DB.
    let root = match state.db.latest_root(&pool).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("latest_root failed: {:#}", e);
            metrics::record_db_error();
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db query failed"})),
            )
                .into_response();
        }
    };

    let response = match root {
        Some(root_bytes) => json!({
            "pool_address": pool,
            "root": hex::encode(&root_bytes),
        }),
        None => json!({
            "pool_address": pool,
            "root": null,
        }),
    };

    // Cache for 30 seconds.
    {
        let mut cache = state.cache.lock().await;
        if let Ok(s) = serde_json::to_string(&response) {
            let _ = cache.set_ex(&cache_key, &s, 30).await;
        }
    }

    (StatusCode::OK, Json(response)).into_response()
}

#[derive(Debug, Deserialize)]
struct ProofQuery {
    #[serde(default)]
    pool_address: Option<String>,
    leaf_index: u64,
}

/// `GET /api/proof?pool_address=X&leaf_index=N` — Merkle proof.
///
/// **STUB:** not implemented yet. Requires the full Merkle tree, which
/// lives in the Merkle service. Implementation is deferred until we have
/// the Merkle service running (Stage 6).
async fn get_proof(
    State(_state): State<AppState>,
    Query(q): Query<ProofQuery>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not implemented",
            "reason": "requires the Merkle service (Stage 6)",
            "pool_address": q.pool_address,
            "leaf_index": q.leaf_index,
        })),
    )
}

/// `GET /metrics` — Prometheus exposition.
async fn metrics_endpoint() -> impl IntoResponse {
    let body = metrics::render_metrics();
    (
        StatusCode::OK,
        [("Content-Type", "text/plain; version=0.0.4")],
        body,
    )
}
