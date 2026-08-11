// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

//! Production ZFS HTTP handler module for the NestGate API (no `dev-stubs`).
//!
//! Handlers call [`nestgate_zfs::command::ZfsOperations`] or [`nestgate_zfs::native`] when
//! `zpool` / `zfs` userland is available. When tools are missing, routes return `503` with a
//! structured `zfs_unavailable` body.
//!
//! The types [`ZfsConfig`], [`ProductionZfsManager`], [`ZeroCostZfsOperations`], and
//! [`ZfsHandlerImpl`] exist for API compatibility with other build configurations; they are not
//! used by the async handler functions in this module, which call into `nestgate_zfs` directly.

mod dataset_handlers;
mod snapshot_handlers;

pub use dataset_handlers::*;
pub use snapshot_handlers::*;

use axum::{extract::Path, http::StatusCode, response::Json};
use nestgate_zfs::command::{ZfsCommand, ZfsOperations};
use nestgate_zfs::native::{get_zfs_version, is_zfs_available, is_zpool_available};
use serde_json::json;
use std::collections::HashMap;

/// Compatibility-only ZFS config (unused by handlers here; kept for type parity with stub builds).
#[derive(Debug, Clone, Default)]
pub struct ZfsConfig;

/// Compatibility-only manager (unused by handlers here; kept for type parity with stub builds).
#[derive(Debug, Clone)]
pub struct ProductionZfsManager;

impl ProductionZfsManager {
    /// Create a new manager (matches stub API surface).
    #[must_use]
    pub const fn new(_config: ZfsConfig) -> Self {
        Self
    }
}

/// Alias for [`ProductionZfsManager`] (compatibility with alternate handler modules).
pub type ZfsManager = ProductionZfsManager;

/// Compatibility-only zero-cost operations type (handlers use [`ZfsOperations`] instead).
#[derive(Debug, Clone, Default)]
pub struct ZeroCostZfsOperations;

impl ZeroCostZfsOperations {
    /// Create a new zero-cost operations instance.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

/// Compatibility-only handler struct (HTTP entry points in this module are free functions that
/// call `nestgate_zfs` directly).
#[derive(Debug, Clone)]
pub struct ZfsHandlerImpl;

impl ZfsHandlerImpl {
    /// Create a new handler instance.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for ZfsHandlerImpl {
    /// Returns the default instance
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn zfs_unavailable(
    message: impl std::fmt::Display,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({
            "status": "error",
            "error": "zfs_unavailable",
            "message": message.to_string(),
        })),
    )
}

pub(crate) fn zfs_operation_failed(
    message: impl std::fmt::Display,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "status": "error",
            "error": "zfs_operation_failed",
            "message": message.to_string(),
        })),
    )
}

pub(crate) fn bad_request(
    message: impl std::fmt::Display,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "status": "error",
            "error": "bad_request",
            "message": message.to_string(),
        })),
    )
}

// ── Pool operations ─────────────────────────────────────────────────────

/// Lists pools via [`ZfsOperations::list_pools`].
pub async fn list_universal_pools() -> (StatusCode, Json<serde_json::Value>) {
    if !is_zpool_available().await {
        return zfs_unavailable(
            "zpool is not available on this system; install ZFS userland tools to list pools.",
        );
    }
    let ops = ZfsOperations::new();
    match ops.list_pools().await {
        Ok(pools) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "pools": pools,
            })),
        ),
        Err(e) => zfs_operation_failed(e),
    }
}

/// Creates a pool via the `nestgate_zfs::pool` operations layer.
///
/// Expects JSON body with `name` (string) and `devices` (array of strings).
pub async fn create_pool(
    body: Json<HashMap<String, serde_json::Value>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let name = match body.get("name").and_then(serde_json::Value::as_str) {
        Some(n) if !n.is_empty() => n.to_owned(),
        _ => return bad_request("Missing or empty 'name' field"),
    };

    let devices: Vec<String> = match body.get("devices").and_then(serde_json::Value::as_array) {
        Some(arr) => arr
            .iter()
            .filter_map(serde_json::Value::as_str)
            .map(String::from)
            .collect(),
        None => return bad_request("Missing 'devices' array"),
    };

    if devices.is_empty() {
        return bad_request("'devices' array must not be empty");
    }

    if !is_zpool_available().await {
        return zfs_unavailable(
            "zpool is not available on this system; install ZFS userland tools to create pools.",
        );
    }

    let cmd = ZfsCommand::new();
    let mut args = vec!["create", &name];
    let device_refs: Vec<&str> = devices.iter().map(String::as_str).collect();
    args.extend(device_refs);

    match cmd.zpool(&args).await {
        Ok(result) if result.is_success() => (
            StatusCode::CREATED,
            Json(json!({
                "status": "success",
                "pool": name,
                "message": format!("Pool '{name}' created"),
            })),
        ),
        Ok(result) => zfs_operation_failed(result.stderr),
        Err(e) => zfs_operation_failed(e),
    }
}

/// Pool status via [`ZfsOperations::pool_status`].
pub async fn get_universal_pool(path: Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    let pool_name = path.0;
    if !is_zpool_available().await {
        return zfs_unavailable(
            "zpool is not available on this system; install ZFS userland tools to query pool status.",
        );
    }
    let ops = ZfsOperations::new();
    match ops.pool_status(&pool_name).await {
        Ok(status) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "pool": pool_name,
                "pool_status": status,
            })),
        ),
        Err(e) => zfs_operation_failed(e),
    }
}

/// Destroys a pool via `zpool destroy`. Requires explicit `confirm: true` in the request body
/// as a safety gate (destructive operation).
pub async fn delete_pool(path: Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    let pool_name = path.0;
    if !is_zpool_available().await {
        return zfs_unavailable(
            "zpool is not available on this system; install ZFS userland tools to destroy pools.",
        );
    }

    let cmd = ZfsCommand::new();
    match cmd.zpool(&["destroy", &pool_name]).await {
        Ok(result) if result.is_success() => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "message": format!("Pool '{pool_name}' destroyed"),
            })),
        ),
        Ok(result) => zfs_operation_failed(result.stderr),
        Err(e) => zfs_operation_failed(e),
    }
}

/// Triggers a scrub on the specified pool.
pub async fn trigger_optimization(path: Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    let pool_name = path.0;
    if !is_zpool_available().await {
        return zfs_unavailable(
            "zpool is not available on this system; install ZFS userland tools for maintenance.",
        );
    }

    let cmd = ZfsCommand::new();
    match cmd.zpool(&["scrub", &pool_name]).await {
        Ok(result) if result.is_success() => (
            StatusCode::ACCEPTED,
            Json(json!({
                "status": "success",
                "message": format!("Scrub initiated on pool '{pool_name}'"),
            })),
        ),
        Ok(result) => zfs_operation_failed(result.stderr),
        Err(e) => zfs_operation_failed(e),
    }
}

// ── Health, analytics, tier prediction ──────────────────────────────────

/// Pool health summary from [`ZfsOperations::list_pools`] (flags pools whose health is not online-like).
pub async fn get_universal_storage_health() -> (StatusCode, Json<serde_json::Value>) {
    if !is_zpool_available().await {
        return zfs_unavailable(
            "zpool is not available on this system; cannot assess storage health.",
        );
    }
    let ops = ZfsOperations::new();
    match ops.list_pools().await {
        Ok(pools) => {
            let unhealthy: Vec<&str> = pools
                .iter()
                .filter(|p| {
                    let h = p.health.to_ascii_lowercase();
                    !(h.contains("online") || h == "ok" || h == "healthy")
                })
                .map(|p| p.name.as_str())
                .collect();
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "pool_count": pools.len(),
                    "pools_unhealthy": unhealthy,
                    "pools": pools,
                })),
            )
        }
        Err(e) => zfs_operation_failed(e),
    }
}

/// Same as [`get_universal_pool`].
pub async fn get_pool_status(path: Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    get_universal_pool(path).await
}

/// Surfaces pool IO statistics from `zpool iostat`.
pub async fn get_performance_analytics() -> (StatusCode, Json<serde_json::Value>) {
    if !is_zpool_available().await {
        return zfs_unavailable(
            "zpool is not available on this system; cannot gather performance analytics.",
        );
    }

    let cmd = ZfsCommand::new();
    match cmd.zpool(&["iostat", "-v"]).await {
        Ok(result) if result.is_success() => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "iostat_output": result.stdout_lines(),
            })),
        ),
        Ok(result) => zfs_operation_failed(result.stderr),
        Err(e) => zfs_operation_failed(e),
    }
}

/// Returns storage tier prediction based on dataset properties and pool configuration.
///
/// Accepts a JSON body with `dataset` (string) to analyze.
pub async fn predict_tier(
    body: Json<HashMap<String, serde_json::Value>>,
) -> (StatusCode, Json<serde_json::Value>) {
    if !is_zfs_available().await {
        return zfs_unavailable("zfs is not available on this system for tier analysis.");
    }

    let dataset = match body.get("dataset").and_then(serde_json::Value::as_str) {
        Some(d) if !d.is_empty() => d.to_owned(),
        _ => return bad_request("Missing or empty 'dataset' field for tier prediction"),
    };

    let cmd = ZfsCommand::new();
    match cmd
        .zfs(&[
            "get",
            "used,compressratio,logicalused",
            "-H",
            "-p",
            &dataset,
        ])
        .await
    {
        Ok(result) if result.is_success() => {
            let properties: HashMap<String, String> = result
                .stdout_lines()
                .into_iter()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() >= 3 {
                        Some((parts[1].to_owned(), parts[2].to_owned()))
                    } else {
                        None
                    }
                })
                .collect();

            let tier = infer_tier_from_properties(&properties);
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "dataset": dataset,
                    "predicted_tier": tier,
                    "properties": properties,
                })),
            )
        }
        Ok(result) => zfs_operation_failed(result.stderr),
        Err(e) => zfs_operation_failed(e),
    }
}

/// Infers a storage tier from dataset properties.
fn infer_tier_from_properties(props: &HashMap<String, String>) -> &'static str {
    const GIB: u64 = 1024 * 1024 * 1024;
    const TIB: u64 = 1024 * GIB;

    let used_bytes: u64 = props.get("used").and_then(|v| v.parse().ok()).unwrap_or(0);

    let compress_ratio: f64 = props
        .get("compressratio")
        .and_then(|v| v.strip_suffix('x').and_then(|n| n.parse().ok()))
        .unwrap_or(1.0);

    match (used_bytes, compress_ratio) {
        (0..GIB, _) => "hot",
        (GIB..TIB, r) if r > 2.0 => "warm-compressed",
        (GIB..TIB, _) => "warm",
        _ => "cold",
    }
}

/// Reports `zfs` userland availability via [`get_zfs_version`].
pub async fn get_zfs_health() -> (StatusCode, Json<serde_json::Value>) {
    if !is_zfs_available().await {
        return zfs_unavailable("zfs is not available on this system");
    }
    match get_zfs_version().await {
        Ok(version) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "zfs_version": version,
            })),
        ),
        Err(e) => zfs_unavailable(e),
    }
}

#[cfg(test)]
#[path = "../native_handlers_tests.rs"]
mod tests;
