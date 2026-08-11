// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

//! ZFS snapshot HTTP handlers — list, create, delete.

use axum::{extract::Path, http::StatusCode, response::Json};
use nestgate_zfs::command::{ZfsCommand, ZfsOperations};
use nestgate_zfs::native::is_zfs_available;
use serde_json::json;
use std::collections::HashMap;

use super::{bad_request, zfs_operation_failed, zfs_unavailable};

/// Lists snapshots via [`ZfsOperations::list_snapshots`].
pub async fn list_snapshots() -> (StatusCode, Json<serde_json::Value>) {
    if !is_zfs_available().await {
        return zfs_unavailable(
            "zfs is not available on this system; install ZFS userland tools to list snapshots.",
        );
    }
    let ops = ZfsOperations::new();
    match ops.list_snapshots(None).await {
        Ok(snapshots) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "snapshots": snapshots,
            })),
        ),
        Err(e) => zfs_operation_failed(e),
    }
}

/// Creates a snapshot via [`ZfsOperations::create_snapshot`].
///
/// Expects JSON body with `dataset` (string) and `name` (string).
pub async fn create_snapshot(
    body: Json<HashMap<String, serde_json::Value>>,
) -> (StatusCode, Json<serde_json::Value>) {
    if !is_zfs_available().await {
        return zfs_unavailable(
            "zfs is not available on this system; install ZFS userland tools to create snapshots.",
        );
    }

    let dataset = match body.get("dataset").and_then(serde_json::Value::as_str) {
        Some(d) if !d.is_empty() => d.to_owned(),
        _ => return bad_request("Missing or empty 'dataset' field"),
    };

    let snap_name = match body.get("name").and_then(serde_json::Value::as_str) {
        Some(n) if !n.is_empty() => n.to_owned(),
        _ => return bad_request("Missing or empty 'name' field"),
    };

    let ops = ZfsOperations::new();
    match ops.create_snapshot(&dataset, &snap_name).await {
        Ok(()) => (
            StatusCode::CREATED,
            Json(json!({
                "status": "success",
                "snapshot": format!("{dataset}@{snap_name}"),
                "message": format!("Snapshot '{dataset}@{snap_name}' created"),
            })),
        ),
        Err(e) => zfs_operation_failed(e),
    }
}

/// Destroys a snapshot via `zfs destroy`.
pub async fn delete_snapshot(path: Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    let snapshot_name = path.0;
    if !is_zfs_available().await {
        return zfs_unavailable(
            "zfs is not available on this system; install ZFS userland tools to destroy snapshots.",
        );
    }

    let cmd = ZfsCommand::new();
    match cmd.zfs(&["destroy", &snapshot_name]).await {
        Ok(result) if result.is_success() => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "message": format!("Snapshot '{snapshot_name}' destroyed"),
            })),
        ),
        Ok(result) => zfs_operation_failed(result.stderr),
        Err(e) => zfs_operation_failed(e),
    }
}
