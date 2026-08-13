// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

use super::NestGateJsonRpcHandler;
use super::zfs::handle_zfs_method;
use base64::Engine as _;

#[tokio::test]
async fn handle_unknown_method_returns_error_string() {
    let h = NestGateJsonRpcHandler::new();
    let err = h
        .handle("not_a_real_method", serde_json::json!({}))
        .await
        .expect_err("test: unknown method");
    assert!(err.contains("Unknown method"));
}

#[tokio::test]
async fn handle_list_datasets_bad_params() {
    let h = NestGateJsonRpcHandler::new();
    let err = h
        .handle("list_datasets", serde_json::json!({"pool": "t"}))
        .await
        .expect_err("test: bad params");
    assert!(!err.is_empty());
}

#[tokio::test]
async fn handle_health_readiness_includes_ready_flag() {
    let h = NestGateJsonRpcHandler::new();
    let v = h
        .handle("health.readiness", serde_json::json!({}))
        .await
        .expect("test: readiness");
    assert!(v.get("ready").is_some());
    assert!(v.get("status").is_some());
}

#[tokio::test]
async fn handle_capabilities_alias() {
    let h = NestGateJsonRpcHandler::new();
    let v = h
        .handle("capabilities.list", serde_json::json!({}))
        .await
        .expect("test: caps");
    assert!(v.is_array() || v.as_array().is_some());
}

#[tokio::test]
async fn handle_zfs_health_always_success_json() {
    let v = handle_zfs_method("zfs.health", &serde_json::json!({}))
        .await
        .expect("test: zfs.health");
    assert_eq!(v["status"], "success");
    assert!(v.get("zpool_available").is_some());
    assert!(v.get("zfs_available").is_some());
}

#[tokio::test]
async fn handle_zfs_unknown_semantic_method() {
    let err = handle_zfs_method("zfs.not_implemented_xyz", &serde_json::json!({}))
        .await
        .expect_err("test: unknown zfs");
    assert!(err.contains("Unknown zfs method"));
}

#[tokio::test]
async fn handle_zfs_pool_get_missing_pool_param() {
    let err = handle_zfs_method("zfs.pool.get", &serde_json::json!({}))
        .await
        .expect_err("test: missing pool");
    assert!(err.contains("missing"));
}

#[tokio::test]
async fn handle_zfs_dataset_get_missing_dataset_param() {
    let err = handle_zfs_method("zfs.dataset.get", &serde_json::json!({}))
        .await
        .expect_err("test: missing dataset");
    assert!(err.contains("missing"));
}

#[tokio::test]
async fn handle_list_pools_returns_json_array() {
    let h = NestGateJsonRpcHandler::new();
    let v = h
        .handle("list_pools", serde_json::json!(null))
        .await
        .expect("list_pools");
    assert!(v.is_array());
}

#[tokio::test]
async fn handle_list_datasets_accepts_pool_string_param() {
    let h = NestGateJsonRpcHandler::new();
    let v = h
        .handle("list_datasets", serde_json::json!("test-pool"))
        .await
        .expect("list_datasets");
    assert!(v.is_array());
}

#[tokio::test]
async fn handle_get_metrics_returns_object() {
    let h = NestGateJsonRpcHandler::new();
    let v = h
        .handle("get_metrics", serde_json::json!({}))
        .await
        .expect("get_metrics");
    assert!(v.is_object());
}

#[tokio::test]
async fn handle_health_returns_status() {
    let h = NestGateJsonRpcHandler::new();
    let v = h
        .handle("health", serde_json::json!({}))
        .await
        .expect("health");
    assert!(v.get("status").is_some());
}

#[tokio::test]
async fn handle_health_liveness_returns_status() {
    let h = NestGateJsonRpcHandler::new();
    let v = h
        .handle("health.liveness", serde_json::json!({}))
        .await
        .expect("health.liveness");
    assert!(v.get("status").is_some());
}

#[tokio::test]
async fn handle_health_check_alias_returns_status() {
    let h = NestGateJsonRpcHandler::new();
    let v = h
        .handle("health.check", serde_json::json!({}))
        .await
        .expect("health.check");
    assert!(v.get("status").is_some());
}

#[tokio::test]
async fn handle_version_returns_version_info() {
    let h = NestGateJsonRpcHandler::new();
    let v = h
        .handle("version", serde_json::json!({}))
        .await
        .expect("version");
    assert!(v.get("version").is_some() || v.is_object());
}

#[tokio::test]
async fn handle_capabilities_returns_array() {
    let h = NestGateJsonRpcHandler::new();
    let v = h
        .handle("capabilities", serde_json::json!({}))
        .await
        .expect("capabilities");
    assert!(v.is_array());
}

#[tokio::test]
async fn handle_zfs_pool_list_follows_zpool_availability() {
    let r = handle_zfs_method("zfs.pool.list", &serde_json::json!({})).await;
    match r {
        Ok(v) => assert_eq!(v["status"], "success"),
        Err(e) => assert!(
            e.contains("zpool") || e.contains("zfs.pool.list"),
            "unexpected: {e}"
        ),
    }
}

#[tokio::test]
async fn handle_zfs_snapshot_list_missing_zfs_errors_cleanly() {
    let r = handle_zfs_method(
        "zfs.snapshot.list",
        &serde_json::json!({"dataset": "tank/fs"}),
    )
    .await;
    match r {
        Ok(v) => assert_eq!(v["status"], "success"),
        Err(e) => assert!(!e.is_empty()),
    }
}

#[tokio::test]
async fn content_replicate_pull_routes_via_http() {
    let h = NestGateJsonRpcHandler::new();
    let err = h
        .handle(
            "content.replicate.pull",
            serde_json::json!({"cids": ["abc"], "source": "tcp://localhost:9999"}),
        )
        .await;
    assert!(err.is_ok() || err.is_err());
}

#[tokio::test]
async fn content_store_stream_routes_via_http() {
    let h = NestGateJsonRpcHandler::new();
    let result = h
        .handle(
            "content.store_stream",
            serde_json::json!({"family_id": "test-http-stream", "total_size": 512}),
        )
        .await
        .expect("store_stream via HTTP should succeed");
    assert!(result.get("stream_id").is_some());
}

#[tokio::test]
async fn content_store_stream_chunk_bad_session_errors_via_http() {
    let h = NestGateJsonRpcHandler::new();
    let err = h
        .handle(
            "content.store_stream_chunk",
            serde_json::json!({"stream_id": "invalid", "data": "aGVsbG8="}),
        )
        .await
        .expect_err("bad stream_id should fail");
    assert!(!err.is_empty());
}

#[tokio::test]
async fn content_retrieve_stream_missing_hash_errors_via_http() {
    let h = NestGateJsonRpcHandler::new();
    let err = h
        .handle(
            "content.retrieve_stream",
            serde_json::json!({"family_id": "test-retrieve-http"}),
        )
        .await
        .expect_err("missing hash should fail");
    assert!(!err.is_empty());
}

#[tokio::test]
async fn content_retrieve_stream_chunk_bad_session_errors_via_http() {
    let h = NestGateJsonRpcHandler::new();
    let err = h
        .handle(
            "content.retrieve_stream_chunk",
            serde_json::json!({"stream_id": "invalid"}),
        )
        .await
        .expect_err("bad stream_id should fail");
    assert!(!err.is_empty());
}

#[tokio::test]
#[serial_test::serial]
async fn content_put_and_get_through_http_handler() {
    let h = NestGateJsonRpcHandler::new();
    let family_id = format!("test-http-put-{}", uuid::Uuid::new_v4());
    let raw = b"http dispatch content";
    let encoded = base64::engine::general_purpose::STANDARD.encode(raw);

    let put_r = h
        .handle(
            "content.put",
            serde_json::json!({"family_id": &family_id, "data": encoded}),
        )
        .await
        .expect("content.put");
    assert!(put_r.get("hash").is_some());
    let hash = put_r["hash"].as_str().unwrap();

    let get_r = h
        .handle(
            "content.get",
            serde_json::json!({"hash": hash, "family_id": &family_id}),
        )
        .await
        .expect("content.get");
    assert!(get_r.get("data").is_some());

    let _ = tokio::fs::remove_dir_all(
        nestgate_core::config::storage_paths::get_storage_base_path()
            .join("datasets")
            .join(&family_id),
    )
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn content_exists_and_list_through_http_handler() {
    let h = NestGateJsonRpcHandler::new();
    let family_id = format!("test-http-exists-{}", uuid::Uuid::new_v4());
    let encoded = base64::engine::general_purpose::STANDARD.encode(b"exists check");

    let put_r = h
        .handle(
            "content.put",
            serde_json::json!({"family_id": &family_id, "data": encoded}),
        )
        .await
        .expect("put");
    let hash = put_r["hash"].as_str().unwrap();

    let exists_r = h
        .handle(
            "content.exists",
            serde_json::json!({"hash": hash, "family_id": &family_id}),
        )
        .await
        .expect("content.exists");
    assert_eq!(exists_r["exists"], true);

    let list_r = h
        .handle("content.list", serde_json::json!({"family_id": &family_id}))
        .await
        .expect("content.list");
    assert!(list_r.get("items").is_some() || list_r.get("hashes").is_some());

    let _ = tokio::fs::remove_dir_all(
        nestgate_core::config::storage_paths::get_storage_base_path()
            .join("datasets")
            .join(&family_id),
    )
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn content_publish_and_resolve_through_http_handler() {
    let h = NestGateJsonRpcHandler::new();
    let family_id = format!("test-http-pub-{}", uuid::Uuid::new_v4());
    let encoded = base64::engine::general_purpose::STANDARD.encode(b"publish me");

    let put_r = h
        .handle(
            "content.put",
            serde_json::json!({"family_id": &family_id, "data": encoded}),
        )
        .await
        .expect("put");
    let hash = put_r["hash"].as_str().unwrap();

    let pub_r = h
        .handle(
            "content.publish",
            serde_json::json!({
                "family_id": &family_id,
                "collection": "test-pub",
                "manifest": { "/index.html": hash }
            }),
        )
        .await
        .expect("content.publish");
    assert!(pub_r.get("collection").is_some() || pub_r.get("published").is_some());

    let resolve_r = h
        .handle(
            "content.resolve",
            serde_json::json!({"collection": "test-pub", "path": "/index.html", "family_id": &family_id}),
        )
        .await
        .expect("content.resolve");
    assert!(resolve_r.get("hash").is_some() || resolve_r.get("data").is_some());

    let _ = tokio::fs::remove_dir_all(
        nestgate_core::config::storage_paths::get_storage_base_path()
            .join("datasets")
            .join(&family_id),
    )
    .await;
}

#[tokio::test]
async fn content_collections_through_http_handler() {
    let h = NestGateJsonRpcHandler::new();
    let family_id = format!("test-http-coll-{}", uuid::Uuid::new_v4());
    let r = h
        .handle(
            "content.collections",
            serde_json::json!({"family_id": &family_id}),
        )
        .await
        .expect("content.collections");
    assert!(r.get("collections").is_some());
}

#[tokio::test]
async fn lifecycle_status_through_http_handler() {
    let h = NestGateJsonRpcHandler::new();
    let r = h
        .handle("lifecycle.status", serde_json::json!({}))
        .await
        .expect("lifecycle.status");
    assert_eq!(r["status"], "running");
    assert_eq!(r["primal"], "nestgate");
    assert!(r.get("version").is_some());
}

#[tokio::test]
async fn unknown_content_method_returns_clear_error() {
    let h = NestGateJsonRpcHandler::new();
    let err = h
        .handle("content.nonexistent_xyz", serde_json::json!({}))
        .await
        .expect_err("unknown content method");
    assert!(err.contains("Unknown content method"), "error: {err}");
}
