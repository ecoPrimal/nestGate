// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

//! HTTP fetch abstraction — capability-based routing with direct fallback.
//!
//! Production HTTP fetches (`content.fetch`, `storage.fetch_external`) route
//! through this module rather than calling `ureq` directly. The provider
//! checks whether an ecosystem `http.get` capability is available via IPC
//! and prefers that route when present; when no capability provider is
//! reachable, falls back to direct `ureq` (pure-Rust TLS).
//!
//! # G72 Tier 2 Evolution
//!
//! This module is the bridge point for the `ureq → songBird/capability.call`
//! evolution. When songBird (or any primal) advertises `http.get` via
//! `discovery.capability.register`, fetches route through the mesh for
//! centralized rate limiting, caching, and policy enforcement.

use nestgate_types::error::{ErrorContextExt, NestGateError, Result};
use std::time::Duration;
use tracing::debug;

/// Response from an HTTP GET operation.
pub struct HttpResponse {
    /// HTTP `Content-Type` header value.
    pub content_type: String,
    /// Response body bytes.
    pub body: Vec<u8>,
}

/// Fetch a URL via the best available provider.
///
/// Tries capability-based routing first (if `http.get` is registered in the
/// local capability store); falls back to direct `ureq` fetch.
///
/// # Errors
///
/// Returns [`NestGateError`] on network failure, timeout, or TLS error.
pub async fn fetch_url(url: &str, timeout: Duration, allow_http: bool) -> Result<HttpResponse> {
    if let Some(resp) = try_capability_fetch(url, timeout) {
        return resp;
    }

    debug!("http_provider: no capability provider for http.get, using direct ureq");
    direct_fetch(url, timeout, allow_http).await
}

/// Attempt to fetch via a registered `http.get` capability provider.
///
/// Returns `None` if no provider is available (caller should fall back).
/// Returns `Some(Ok(...))` on success, `Some(Err(...))` on provider failure.
const fn try_capability_fetch(_url: &str, _timeout: Duration) -> Option<Result<HttpResponse>> {
    // G72 Tier 2 bridge point: when an `http.get` capability is discovered
    // at runtime via `discovery.capability.list`, route through it:
    //
    //   1. Resolve provider socket from capability registry
    //   2. Send JSON-RPC: {"method": "http.get", "params": {"url": ..., "timeout_secs": ...}}
    //   3. Decode base64 response body
    //
    // Until the ecosystem `http.get` contract is finalized and a provider
    // is live, this returns None to trigger direct-fetch fallback.
    None
}

/// Direct HTTP fetch using `ureq` (pure-Rust TLS).
async fn direct_fetch(url: &str, timeout: Duration, allow_http: bool) -> Result<HttpResponse> {
    let url_owned = url.to_owned();
    let user_agent = super::unix_socket_server::storage_paths::http_user_agent();

    tokio::task::spawn_blocking(move || {
        let _ = rustls_rustcrypto::provider().install_default();
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(timeout))
            .https_only(!allow_http)
            .build();
        let agent = ureq::Agent::new_with_config(config);

        let mut response = agent
            .get(&url_owned)
            .header("User-Agent", &user_agent)
            .call()
            .map_err(|e| {
                NestGateError::io_error(format!("HTTP GET failed for {url_owned}: {e}"))
            })?;

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_owned();

        let body = response
            .body_mut()
            .read_to_vec()
            .io_ctx("Failed to read HTTP response body")?;

        Ok(HttpResponse { content_type, body })
    })
    .await
    .io_ctx("HTTP fetch task panicked")?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_fetch_returns_none_when_unwired() {
        let result = try_capability_fetch("https://example.com", Duration::from_secs(10));
        assert!(result.is_none(), "unwired capability should return None");
    }
}
