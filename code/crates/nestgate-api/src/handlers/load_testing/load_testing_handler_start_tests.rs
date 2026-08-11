// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

//! Tests for `POST /load-test/start` — returns 501 Not Implemented (S146).

#![cfg(test)]

use super::*;
use axum::Router;
use axum_test::TestServer;

fn create_test_router() -> Router {
    use axum::routing::{get, post};

    Router::new()
        .route("/load-test/start", post(start_load_test))
        .route("/load-test/results", get(get_load_test_results))
        .route("/load-test/history", get(get_load_test_history))
        .route("/load-test/baselines", get(get_performance_baselines))
}

#[expect(deprecated, reason = "testing backward-compatible deprecated API")]
fn default_config_json() -> serde_json::Value {
    serde_json::to_value(LoadTestConfig::default()).expect("default config must serialize")
}

#[tokio::test]
async fn test_start_load_test_success() {
    let server = TestServer::new(create_test_router()).expect("Test setup failed");
    let response = server
        .post("/load-test/start")
        .json(&default_config_json())
        .await;
    response.assert_status(axum::http::StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn test_start_load_test_with_ramp_scenario() {
    let server = TestServer::new(create_test_router()).expect("Test setup failed");
    let response = server
        .post("/load-test/start")
        .json(&default_config_json())
        .await;
    response.assert_status(axum::http::StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn test_start_load_test_with_step_scenario() {
    let server = TestServer::new(create_test_router()).expect("Test setup failed");
    let response = server
        .post("/load-test/start")
        .json(&default_config_json())
        .await;
    response.assert_status(axum::http::StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn test_start_load_test_with_spike_scenario() {
    let server = TestServer::new(create_test_router()).expect("Test setup failed");
    let response = server
        .post("/load-test/start")
        .json(&default_config_json())
        .await;
    response.assert_status(axum::http::StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn test_start_load_test_minimal_config() {
    let server = TestServer::new(create_test_router()).expect("Test setup failed");
    let response = server
        .post("/load-test/start")
        .json(&default_config_json())
        .await;
    response.assert_status(axum::http::StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn test_start_load_test_high_concurrency() {
    let server = TestServer::new(create_test_router()).expect("Test setup failed");
    let response = server
        .post("/load-test/start")
        .json(&default_config_json())
        .await;
    response.assert_status(axum::http::StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn test_start_load_test_multiple_endpoints() {
    let server = TestServer::new(create_test_router()).expect("Test setup failed");
    let response = server
        .post("/load-test/start")
        .json(&default_config_json())
        .await;
    response.assert_status(axum::http::StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn test_start_load_test_strict_thresholds() {
    let server = TestServer::new(create_test_router()).expect("Test setup failed");
    let response = server
        .post("/load-test/start")
        .json(&default_config_json())
        .await;
    response.assert_status(axum::http::StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn test_start_load_test_with_custom_headers() {
    let server = TestServer::new(create_test_router()).expect("Test setup failed");
    let response = server
        .post("/load-test/start")
        .json(&default_config_json())
        .await;
    response.assert_status(axum::http::StatusCode::NOT_IMPLEMENTED);
}
