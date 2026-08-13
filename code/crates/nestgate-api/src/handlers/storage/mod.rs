// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

//! Storage API: pools, datasets, snapshots, and probe helpers.

mod http_handlers;
mod probes;
mod types;

#[cfg(test)]
mod tests;

pub use super::handler_types::StorageHandler;
pub use http_handlers::{
    get_storage_datasets, get_storage_metrics, get_storage_pools, get_storage_snapshots,
};
pub use types::{
    StorageDataset, StorageDatasetInfo, StorageMetrics, StoragePool, StoragePoolInfo,
    StorageSnapshot, StorageSnapshotInfo,
};
