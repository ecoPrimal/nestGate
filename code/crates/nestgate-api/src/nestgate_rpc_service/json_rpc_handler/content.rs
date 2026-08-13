// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

/// Ecosystem-standard `content.*` semantic methods — delegates to the canonical
/// content-addressed storage handlers in `nestgate-rpc`.
///
/// # Errors
///
/// Returns `Err` for unknown content methods or handler failures.
pub(super) async fn handle_content_method(
    method: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    use nestgate_core::rpc::content_ops;

    let result = match method {
        "content.put" => content_ops::put(params).await,
        "content.get" => content_ops::get(params).await,
        "content.exists" => content_ops::exists(params).await,
        "content.list" => content_ops::list(params).await,
        "content.publish" => content_ops::publish(params).await,
        "content.resolve" => content_ops::resolve(params).await,
        "content.promote" => content_ops::promote(params).await,
        "content.collections" => content_ops::collections(params).await,
        "content.fetch_heads" => content_ops::fetch_heads(params).await,
        "content.push" => content_ops::push(params).await,
        "content.replicate" => content_ops::replicate(params).await,
        "content.replicate.pull" => content_ops::replicate_pull(params).await,
        "content.sync" => content_ops::sync(params).await,
        "content.store_stream" => content_ops::store_stream_begin(params).await,
        "content.store_stream_chunk" => content_ops::store_stream_chunk(params).await,
        "content.retrieve_stream" => content_ops::retrieve_stream_begin(params).await,
        "content.retrieve_stream_chunk" => content_ops::retrieve_stream_chunk(params).await,
        _ => return Err(format!("Unknown content method: {method}")),
    };
    result.map_err(|e| format!("{method} failed: {e}"))
}
