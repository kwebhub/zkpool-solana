//! zkpool_backend — HTTP API + indexer.
//!
//! Stage 5.9.1: minimal server with `/api/health` only.
//! Read endpoints and `/api/withdraw` are added in the next sub-stages.

use std::sync::Arc;

use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
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

    // 6. Merkle tree (lazy — depends on Merkle service)
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

    // 8. Router (only /api/health for now)
    let app = Router::new()
        .route("/api/health", get(health))
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

/// `GET /api/health` — health check.
async fn health(State(state): State<AppState>) -> impl IntoResponse {
    // Simple DB check: try to fetch next_leaf_index.
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
