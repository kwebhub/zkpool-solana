//! HTTP integration tests for the prover service.
//!
//! Uses `tower::ServiceExt::oneshot` for in-process requests — no port binding.
//! The `/prove` test is `#[ignore]` (requires nargo + sunspot + artifacts).

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use zkpool_prover::config::Config;
use zkpool_prover::prover::Prover;
use zkpool_prover::server::{build_router, AppState};
use zkpool_prover::witness::WitnessInputs;

fn test_config() -> Config {
    Config {
        port: 4002,
        circuit_dir: "/home/ubuntu/circuits/withdrawal".to_string(),
        nargo_bin: "nargo".to_string(),
        sunspot_bin: "sunspot".to_string(),
        nargo_timeout_secs: 60,
        sunspot_timeout_secs: 60,
    }
}

fn make_app() -> axum::Router {
    let prover = Prover::new(test_config());
    let state = Arc::new(AppState { prover });
    build_router(state)
}

fn sample_inputs() -> WitnessInputs {
    WitnessInputs {
        root: "1e8508c3c11def8bfecd33c4bf97ce7fd065fea15eafe55c47e35bd056f1c6b2".to_string(),
        nullifier_hash: "1412cc9d862599e6869a1881c8562062b98537456d9035819288219b5cd3e6e4"
            .to_string(),
        recipient: "062afbde1181c71c".to_string(),
        recipient_binding: "200cfdb247b0436ba0483327abcf8f83ed65f75a0db8d9ebbb611561d7d3b2e1"
            .to_string(),
        amount: "0f4240".to_string(),
        nullifier: "018abef7846071c7".to_string(),
        secret: "03157def08c0e38e".to_string(),
        note_secret: "04a03ce68d215555".to_string(),
        merkle_proof: std::iter::once("07b5bad595e238e3".to_string())
            .chain(std::iter::repeat("00".to_string()).take(19))
            .collect(),
        is_even: vec![true; 20],
    }
}

#[tokio::test]
async fn test_health() {
    let app = make_app();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let s = String::from_utf8(body.to_vec()).unwrap();
    assert!(s.contains("\"status\":\"ok\""));
}

#[tokio::test]
async fn test_prove_bad_payload_returns_422() {
    let app = make_app();
    // Missing required fields -> axum's Json extractor returns 422.
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/prove")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_prove_wrong_merkle_len_returns_500() {
    let app = make_app();
    let mut inputs = sample_inputs();
    inputs.merkle_proof.pop(); // 19 instead of 20
    let payload = serde_json::to_vec(&inputs).unwrap();
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/prove")
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let s = String::from_utf8(body.to_vec()).unwrap();
    assert!(s.contains("merkle_proof must have 20 elements"));
}

#[tokio::test]
#[ignore = "requires nargo + sunspot + circuit artifacts"]
async fn test_prove_real() {
    let app = make_app();
    let payload = serde_json::to_vec(&sample_inputs()).unwrap();
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/prove")
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let proof = v["proof"].as_str().unwrap();
    let pw = v["public_witness"].as_str().unwrap();
    assert_eq!(proof.len(), 648, "proof hex length");
    assert_eq!(pw.len(), 344, "public witness hex length");
}
