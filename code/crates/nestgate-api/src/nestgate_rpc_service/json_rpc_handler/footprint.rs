// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

/// footPrint `footprint.*` methods — CAS-backed project persistence.
///
/// # Errors
///
/// Returns `Err` for unknown footprint methods or handler failures.
pub(super) async fn handle_footprint_method(
    method: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    use nestgate_core::rpc::footprint_ops;

    let result = match method {
        "footprint.save" => footprint_ops::save(params).await,
        "footprint.get" => footprint_ops::get(params).await,
        "footprint.list" => footprint_ops::list(params).await,
        "footprint.delete" => footprint_ops::delete(params).await,
        "footprint.history" => footprint_ops::history(params).await,
        _ => return Err(format!("Unknown footprint method: {method}")),
    };
    result.map_err(|e| format!("{method} failed: {e}"))
}
