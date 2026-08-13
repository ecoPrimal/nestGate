// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

use nestgate_zfs::command::ZfsOperations;
use nestgate_zfs::native::{is_zfs_available, is_zpool_available};
use serde_json::json;

/// Ecosystem-standard `zfs.*` semantic methods (GAP-MATRIX-04 alignment).
///
/// Uses [`ZfsOperations`] directly rather than tarpc so the response schema
/// matches the raw ZFS output exposed on the UDS surface.
///
/// # Errors
///
/// Returns `Err` for unknown methods, unavailable tools, or ZFS failures.
pub(super) async fn handle_zfs_method(
    method: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    match method {
        "zfs.pool.list" => {
            require_zpool().await?;
            let ops = ZfsOperations::new();
            ops.list_pools()
                .await
                .map(|pools| json!({"status": "success", "pools": pools}))
                .map_err(|e| format!("zfs.pool.list failed: {e}"))
        }
        "zfs.pool.get" => {
            let pool = str_param(params, "pool")?;
            require_zpool().await?;
            let ops = ZfsOperations::new();
            ops.pool_status(pool)
                .await
                .map(|s| {
                    json!({
                        "status": "success", "pool": pool,
                        "state": s.state, "scan": s.scan, "errors": s.errors,
                    })
                })
                .map_err(|e| format!("zfs.pool.get failed: {e}"))
        }
        "zfs.pool.health" => {
            require_zpool().await?;
            let ops = ZfsOperations::new();
            ops.list_pools()
                .await
                .map(|pools| {
                    let bad: Vec<&str> = pools
                        .iter()
                        .filter(|p| {
                            let h = p.health.to_ascii_lowercase();
                            !(h.contains("online") || h == "ok" || h == "healthy")
                        })
                        .map(|p| p.name.as_str())
                        .collect();
                    json!({"status":"success","pool_count":pools.len(),"pools_unhealthy":bad})
                })
                .map_err(|e| format!("zfs.pool.health failed: {e}"))
        }
        "zfs.dataset.list" => {
            require_zfs().await?;
            let pool = params.get("pool").and_then(serde_json::Value::as_str);
            let ops = ZfsOperations::new();
            ops.list_datasets(pool)
                .await
                .map(|ds| json!({"status": "success", "datasets": ds}))
                .map_err(|e| format!("zfs.dataset.list failed: {e}"))
        }
        "zfs.dataset.get" => {
            let name = str_param(params, "dataset")?;
            require_zfs().await?;
            let ops = ZfsOperations::new();
            ops.list_datasets(Some(name))
                .await
                .map_err(|e| format!("zfs.dataset.get failed: {e}"))
                .and_then(|ds| {
                    ds.into_iter()
                        .find(|d| d.name == name)
                        .map(|d| json!({"status":"success","dataset":d}))
                        .ok_or_else(|| format!("dataset {name:?} not found"))
                })
        }
        "zfs.snapshot.list" => {
            require_zfs().await?;
            let ds = params.get("dataset").and_then(serde_json::Value::as_str);
            let ops = ZfsOperations::new();
            ops.list_snapshots(ds)
                .await
                .map(|snaps| json!({"status": "success", "snapshots": snaps}))
                .map_err(|e| format!("zfs.snapshot.list failed: {e}"))
        }
        "zfs.health" => {
            let zpool_ok = is_zpool_available().await;
            let zfs_ok = is_zfs_available().await;
            Ok(json!({"status":"success","zpool_available":zpool_ok,"zfs_available":zfs_ok}))
        }
        _ => Err(format!("Unknown zfs method: {method}")),
    }
}

fn str_param<'a>(params: &'a serde_json::Value, key: &str) -> Result<&'a str, String> {
    params
        .get(key)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("missing '{key}' parameter"))
}

async fn require_zpool() -> Result<(), String> {
    if is_zpool_available().await {
        Ok(())
    } else {
        Err("zpool is not available on this system".into())
    }
}

async fn require_zfs() -> Result<(), String> {
    if is_zfs_available().await {
        Ok(())
    } else {
        Err("zfs is not available on this system".into())
    }
}
