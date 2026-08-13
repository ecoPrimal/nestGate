// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

/// Coordination domain `coord.*` methods — delegates to `coord_ops` which
/// wraps `coord_handlers` from `nestgate-rpc`.
///
/// # Errors
///
/// Returns `Err` for unknown coordination methods or handler failures.
pub(super) async fn handle_coord_method(
    method: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    use nestgate_core::rpc::coord_ops;

    let result = match method {
        "coord.blurbs.current" => coord_ops::blurbs_current(params).await,
        "coord.blurbs.list" => coord_ops::blurbs_list(params).await,
        "coord.blurbs.get" => coord_ops::blurbs_get(params).await,
        "coord.fragos.list" => coord_ops::fragos_list(params).await,
        "coord.fragos.get" => coord_ops::fragos_get(params).await,
        "coord.waves.current" => coord_ops::waves_current(params).await,
        "coord.waves.history" => coord_ops::waves_history(params).await,
        "coord.heads.get" => coord_ops::heads_get(params).await,
        "coord.heads.all" => coord_ops::heads_all(params).await,
        "coord.topology" => coord_ops::topology(params).await,
        "coord.depot.status" => coord_ops::depot_status(params).await,
        "coord.provenance" => coord_ops::provenance(params).await,
        "coord.ingest" => coord_ops::ingest(params).await,
        _ => return Err(format!("Unknown coord method: {method}")),
    };
    result.map_err(|e| format!("{method} failed: {e}"))
}
