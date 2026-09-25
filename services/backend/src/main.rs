//! zkpool_backend — HTTP API + indexer.
//!
//! Stage 5.9.3: full endpoint set.

use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;
use tracing::{error, info};

use zkpool_backend::api_types::{WithdrawRequest, WithdrawResponse};
use zkpool_backend::cache::Cache;
use zkpool_backend::config::Config;
use zkpool_backend::db::Db;
use zkpool_backend::indexer::Indexer;
use zkpool_backend::logging;
use zkpool_backend::metrics;
use zkpool_backend::rate_limit::{self, RateLimiter};
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
    let config = Arc::new(Config::from_env()?);
    logging::init_logging()?;
    info!("logging initialized");
    metrics::init_metrics()?;
    info!("metrics initialized");

    let db = Db::connect(&config.database_url).await?;
    info!("connected to Postgres");

    let cache = Cache::connect(&config.redis_url).await?;
    info!("connected to Redis");
    let cache = Arc::new(Mutex::new(cache));

    let merkle_client = MerkleClient::new(config.merkle_url.clone());
    let tree = MerkleTree::new(
        config.pool_address.clone(),
        config.merkle_tree_depth,
        (*cache.lock().await).clone(),
        merkle_client,
    )?;
    let tree = Arc::new(Mutex::new(tree));
    info!("Merkle tree handle created");

    let state = AppState {
        config: config.clone(),
        db: db.clone(),
        cache: cache.clone(),
        tree: tree.clone(),
    };

    let rate_limiter = RateLimiter::new(
        (*cache.lock().await).clone(),
        config.rate_limit_withdraw_per_min,
        config.rate_limit_read_per_min,
    );

    // Spawn indexer.
    {
        let indexer_config = config.clone();
        let indexer_db = db.clone();
        let indexer_cache = cache.clone();
        let indexer_tree = tree.clone();
        tokio::spawn(async move {
            let indexer = Indexer::new(indexer_config, indexer_db, indexer_cache, indexer_tree);
            if let Err(e) = indexer.run().await {
                error!("indexer terminated: {:#}", e);
            }
        });
        info!("indexer spawned");
    }

    let read_routes = Router::new()
        .route("/api/commitments", get(get_commitments))
        .route("/api/root", get(get_root))
        .route("/api/proof", get(get_proof))
        .layer(middleware::from_fn_with_state(
            rate_limiter.clone(),
            rate_limit::read_middleware,
        ));

    let withdraw_routes = Router::new()
        .route("/api/withdraw", post(post_withdraw))
        .layer(middleware::from_fn_with_state(
            rate_limiter.clone(),
            rate_limit::withdraw_middleware,
        ));

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/metrics", get(metrics_endpoint))
        .merge(read_routes)
        .merge(withdraw_routes)
        .with_state(state);

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

async fn get_commitments(
    State(state): State<AppState>,
    Query(q): Query<PoolQuery>,
) -> impl IntoResponse {
    let pool = q
        .pool_address
        .unwrap_or_else(|| state.config.pool_address.clone());
    let cache_key = format!("cache:commitments:{}", pool);

    {
        let mut cache = state.cache.lock().await;
        if let Ok(Some(cached)) = cache.get(&cache_key).await {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&cached) {
                return (StatusCode::OK, Json(parsed)).into_response();
            }
        }
    }

    let rows = match state.db.list_commitments(&pool).await {
        Ok(rows) => rows,
        Err(e) => {
            error!("list_commitments failed: {:#}", e);
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

    {
        let mut cache = state.cache.lock().await;
        if let Ok(s) = serde_json::to_string(&response) {
            let _ = cache.set_ex(&cache_key, &s, 30).await;
        }
    }

    (StatusCode::OK, Json(response)).into_response()
}

async fn get_root(State(state): State<AppState>, Query(q): Query<PoolQuery>) -> impl IntoResponse {
    let pool = q
        .pool_address
        .unwrap_or_else(|| state.config.pool_address.clone());
    let cache_key = format!("cache:root:{}", pool);

    {
        let mut cache = state.cache.lock().await;
        if let Ok(Some(cached)) = cache.get(&cache_key).await {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&cached) {
                return (StatusCode::OK, Json(parsed)).into_response();
            }
        }
    }

    let root = match state.db.latest_root(&pool).await {
        Ok(r) => r,
        Err(e) => {
            error!("latest_root failed: {:#}", e);
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

async fn metrics_endpoint() -> impl IntoResponse {
    let body = metrics::render_metrics();
    (
        StatusCode::OK,
        [("Content-Type", "text/plain; version=0.0.4")],
        body,
    )
}

/// `POST /api/withdraw` — proxy the witness to the prover service.
async fn post_withdraw(
    State(state): State<AppState>,
    Json(req): Json<WithdrawRequest>,
) -> impl IntoResponse {
    let url = format!("{}/prove", state.config.prover_url);

    let resp = reqwest::Client::new().post(&url).json(&req).send().await;

    let resp = match resp {
        Ok(r) => r,
        Err(e) => {
            error!("prover request failed: {:#}", e);
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": "prover unreachable"})),
            )
                .into_response();
        }
    };

    let status = resp.status();
    if status.is_success() {
        let parsed: Result<WithdrawResponse, _> = resp.json().await;
        match parsed {
            Ok(out) => (StatusCode::OK, Json(out)).into_response(),
            Err(e) => {
                error!("prover response parse error: {:#}", e);
                (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({"error": "prover returned invalid response"})),
                )
                    .into_response()
            }
        }
    } else if status.is_client_error() {
        let body: String = resp.text().await.unwrap_or_default();
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid witness",
                "prover_status": status.as_u16(),
                "prover_body": body,
            })),
        )
            .into_response()
    } else {
        let body: String = resp.text().await.unwrap_or_default();
        error!("prover 5xx: status={}, body={}", status, body);
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({"error": "prover internal error"})),
        )
            .into_response()
    }
}
