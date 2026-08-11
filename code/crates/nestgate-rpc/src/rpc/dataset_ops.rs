// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

//! Public stateless dataset operations API.
//!
//! Wraps `unix_socket_server::dataset_handlers` behind a `StorageState`-free
//! interface so that **any crate** (including `nestgate-api`) can call dataset
//! operations without constructing internal RPC state.
//!
//! # Errors
//!
//! All functions return [`nestgate_types::error::NestGateError`] on invalid
//! parameters or I/O failure.

use crate::rpc::shared_state::shared_state;
use crate::rpc::unix_socket_server::dataset_handlers;
use nestgate_types::error::Result;
use serde_json::Value;

/// `dataset.convergence` — CAS provenance state per filesystem path.
///
/// Scans a directory, computes BLAKE3 hashes, and compares against CAS to
/// report convergence percentage and per-file status.
///
/// # Errors
///
/// Returns error on invalid parameters (missing/relative path) or I/O failure.
pub async fn convergence(params: &Value) -> Result<Value> {
    dataset_handlers::dataset_convergence(Some(params), shared_state()).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn convergence_missing_path_errors() {
        let params = json!({"family_id": "test-convergence"});
        let err = convergence(&params).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn convergence_relative_path_errors() {
        let params = json!({"path": "relative/path", "family_id": "test"});
        let err = convergence(&params).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn convergence_nonexistent_dir_returns_missing() {
        let params = json!({
            "path": "/tmp/nestgate_dataset_ops_nonexistent_test_dir",
            "family_id": "test-ops"
        });
        let result = convergence(&params).await.unwrap();
        assert_eq!(result["state"], "MISSING");
    }
}
