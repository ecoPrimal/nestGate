// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

//! JSON-RPC 2.0 handler that wraps the tarpc service for HTTP access.
//!
//! Orchestration and other capability peers can use this for initial
//! discovery before escalating to tarpc.  Supports both legacy method
//! names (`list_pools`) and ecosystem-standard semantic names
//! (`zfs.pool.list`) per `SEMANTIC_METHOD_NAMING_STANDARD.md`.

mod content;
mod coord;
mod footprint;
mod zfs;

#[cfg(test)]
mod tests;

use nestgate_core::constants::system::DEFAULT_SERVICE_NAME;
use serde_json::json;
use tarpc::context::Context;

use super::NestGateRpc;
use super::tarpc_server::NestGateRpcServer;

use content::handle_content_method;
use coord::handle_coord_method;
use footprint::handle_footprint_method;
use zfs::handle_zfs_method;

/// JSON-RPC handler that delegates to the tarpc server implementation
pub struct NestGateJsonRpcHandler {
    server: NestGateRpcServer,
}

impl NestGateJsonRpcHandler {
    /// Create new JSON-RPC handler
    #[must_use]
    pub fn new() -> Self {
        Self {
            server: NestGateRpcServer::default(),
        }
    }

    /// Route a JSON-RPC method call to the appropriate tarpc handler.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the method is unknown or serialisation fails.
    pub async fn handle(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match method {
            "list_pools" => {
                let ctx = Context::current();
                let result = self.server.clone().list_pools(ctx).await;
                serde_json::to_value(result)
                    .map_err(|e| format!("Failed to serialize list_pools result: {e}"))
            }
            "list_datasets" => {
                let pool: String = serde_json::from_value(params).map_err(|e| e.to_string())?;
                let ctx = Context::current();
                let result = self.server.clone().list_datasets(ctx, pool).await;
                serde_json::to_value(result)
                    .map_err(|e| format!("Failed to serialize list_datasets result: {e}"))
            }
            "get_metrics" => {
                let ctx = Context::current();
                let result = self.server.clone().get_metrics(ctx).await;
                serde_json::to_value(result)
                    .map_err(|e| format!("Failed to serialize get_metrics result: {e}"))
            }
            "health.liveness" => serde_json::to_value(serde_json::json!({
                "status": "alive",
                "primal": DEFAULT_SERVICE_NAME,
            }))
            .map_err(|e| format!("Failed to serialize health.liveness result: {e}")),
            "health" | "health.check" => {
                let ctx = Context::current();
                let result = self.server.clone().health(ctx).await;
                serde_json::to_value(result)
                    .map_err(|e| format!("Failed to serialize health result: {e}"))
            }
            "health.readiness" => {
                let ctx = Context::current();
                let h = self.server.clone().health(ctx).await;
                let ready = h.status != "unhealthy";
                serde_json::to_value(serde_json::json!({
                    "ready": ready,
                    "status": h.status,
                    "version": h.version,
                    "uptime_seconds": h.uptime_seconds,
                    "pools_healthy": h.pools_healthy,
                    "pools_total": h.pools_total,
                }))
                .map_err(|e| format!("Failed to serialize health.readiness result: {e}"))
            }
            "version" => {
                let ctx = Context::current();
                let result = self.server.clone().version(ctx).await;
                serde_json::to_value(result)
                    .map_err(|e| format!("Failed to serialize version result: {e}"))
            }
            "capabilities" | "capabilities.list" => {
                let ctx = Context::current();
                let result = self.server.clone().capabilities(ctx).await;
                serde_json::to_value(result)
                    .map_err(|e| format!("Failed to serialize capabilities result: {e}"))
            }
            // Content-addressed storage (delegates to canonical handlers)
            m if m.starts_with("content.") => handle_content_method(m, &params).await,
            // Coordination domain — ecosystem state served from CAS
            m if m.starts_with("coord.") => handle_coord_method(m, &params).await,
            // footPrint domain — CAS-backed project persistence
            m if m.starts_with("footprint.") => handle_footprint_method(m, &params).await,
            "lifecycle.status" => Ok(json!({
                "status": "running",
                "primal": DEFAULT_SERVICE_NAME,
                "version": env!("CARGO_PKG_VERSION"),
            })),
            m if m.starts_with("zfs.") => handle_zfs_method(m, &params).await,
            _ => Err(format!("Unknown method: {method}")),
        }
    }
}

impl Default for NestGateJsonRpcHandler {
    fn default() -> Self {
        Self::new()
    }
}
