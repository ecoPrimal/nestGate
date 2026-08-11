// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

//! ZFS dataset HTTP handlers — list, create, get, delete, get/set properties.

use axum::{extract::Path, http::StatusCode, response::Json};
use nestgate_zfs::command::{ZfsCommand, ZfsOperations};
use nestgate_zfs::native::is_zfs_available;
use serde_json::json;
use std::collections::HashMap;

use super::{bad_request, zfs_operation_failed, zfs_unavailable};

/// Lists datasets via [`ZfsOperations::list_datasets`].
pub async fn list_datasets() -> (StatusCode, Json<serde_json::Value>) {
    if !is_zfs_available().await {
        return zfs_unavailable(
            "zfs is not available on this system; install ZFS userland tools to list datasets.",
        );
    }
    let ops = ZfsOperations::new();
    match ops.list_datasets(None).await {
        Ok(datasets) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "datasets": datasets,
            })),
        ),
        Err(e) => zfs_operation_failed(e),
    }
}

/// Creates a dataset via [`ZfsOperations::create_dataset`].
///
/// Expects JSON body with `name` (string) and optional `properties` (object of key-value pairs).
pub async fn create_dataset(
    body: Json<HashMap<String, serde_json::Value>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let name = match body.get("name").and_then(serde_json::Value::as_str) {
        Some(n) if !n.is_empty() => n.to_owned(),
        _ => return bad_request("Missing or empty 'name' field"),
    };

    if !is_zfs_available().await {
        return zfs_unavailable(
            "zfs is not available on this system; install ZFS userland tools to create datasets.",
        );
    }

    let properties: Option<HashMap<String, String>> = body
        .get("properties")
        .and_then(serde_json::Value::as_object)
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_owned())))
                .collect()
        });

    let ops = ZfsOperations::new();
    match ops.create_dataset(&name, properties.as_ref()).await {
        Ok(()) => (
            StatusCode::CREATED,
            Json(json!({
                "status": "success",
                "dataset": name,
                "message": format!("Dataset '{name}' created"),
            })),
        ),
        Err(e) => zfs_operation_failed(e),
    }
}

/// Resolves one dataset by scanning [`ZfsOperations::list_datasets`] (returns `404` if missing).
pub async fn get_dataset(path: Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    let name = path.0;
    if !is_zfs_available().await {
        return zfs_unavailable(
            "zfs is not available on this system; install ZFS userland tools to query datasets.",
        );
    }
    let ops = ZfsOperations::new();
    match ops.list_datasets(None).await {
        Ok(datasets) => {
            if let Some(ds) = datasets.into_iter().find(|d| d.name == name) {
                (
                    StatusCode::OK,
                    Json(json!({
                        "status": "success",
                        "dataset": ds,
                    })),
                )
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "status": "error",
                        "error": "dataset_not_found",
                        "message": format!("No dataset named {name:?}"),
                    })),
                )
            }
        }
        Err(e) => zfs_operation_failed(e),
    }
}

/// Destroys a dataset via [`ZfsOperations::destroy_dataset`].
pub async fn delete_dataset(path: Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    let name = path.0;
    if !is_zfs_available().await {
        return zfs_unavailable(
            "zfs is not available on this system; install ZFS userland tools to destroy datasets.",
        );
    }

    let ops = ZfsOperations::new();
    match ops.destroy_dataset(&name).await {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "message": format!("Dataset '{name}' destroyed"),
            })),
        ),
        Err(e) => zfs_operation_failed(e),
    }
}

/// Retrieves dataset properties via `zfs get all`.
pub async fn get_dataset_properties(path: Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    let name = path.0;
    if !is_zfs_available().await {
        return zfs_unavailable(
            "zfs is not available on this system; install ZFS userland tools to query properties.",
        );
    }

    let cmd = ZfsCommand::new();
    match cmd
        .zfs(&["get", "all", "-H", "-o", "property,value,source", &name])
        .await
    {
        Ok(result) if result.is_success() => {
            let properties: HashMap<String, serde_json::Value> = result
                .stdout_lines()
                .into_iter()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() >= 3 {
                        Some((
                            parts[0].to_owned(),
                            json!({"value": parts[1], "source": parts[2]}),
                        ))
                    } else {
                        None
                    }
                })
                .collect();
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "dataset": name,
                    "properties": properties,
                })),
            )
        }
        Ok(result) => zfs_operation_failed(result.stderr),
        Err(e) => zfs_operation_failed(e),
    }
}

/// Sets dataset properties via `zfs set`.
///
/// Expects JSON body with property key-value pairs.
pub async fn set_dataset_properties(
    path: Path<String>,
    body: Json<HashMap<String, serde_json::Value>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let name = path.0;
    if !is_zfs_available().await {
        return zfs_unavailable(
            "zfs is not available on this system; install ZFS userland tools to set properties.",
        );
    }

    if body.is_empty() {
        return bad_request("Request body must contain at least one property to set");
    }

    let cmd = ZfsCommand::new();
    let mut errors = Vec::new();
    let mut successes = Vec::new();

    for (key, value) in body.iter() {
        let val_str = match value.as_str() {
            Some(s) => s.to_owned(),
            None => value.to_string(),
        };
        let prop_arg = format!("{key}={val_str}");
        match cmd.zfs(&["set", &prop_arg, &name]).await {
            Ok(result) if result.is_success() => successes.push(key.clone()),
            Ok(result) => errors.push(format!("{key}: {}", result.stderr)),
            Err(e) => errors.push(format!("{key}: {e}")),
        }
    }

    if errors.is_empty() {
        (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "dataset": name,
                "properties_set": successes,
            })),
        )
    } else {
        (
            StatusCode::MULTI_STATUS,
            Json(json!({
                "status": "partial",
                "properties_set": successes,
                "errors": errors,
            })),
        )
    }
}
