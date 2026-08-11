// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

//! Global shared [`StorageState`] singleton for stateless RPC facades.
//!
//! Content, dataset, coordination, and footprint operations all need a
//! `StorageState` reference but have no instance-level state of their own.
//! Rather than creating 7+ duplicate `OnceLock<StorageState>` statics, this
//! module owns a single global instance that is lazily initialized on first
//! access.
//!
//! `StorageState::new()` is infallible (all constituent backends —
//! `TemplateStorage`, `AuditStorage` — construct without I/O errors), so
//! initialization cannot fail and no `expect` is required.

use crate::rpc::unix_socket_server::StorageState;
use std::sync::LazyLock;

static SHARED: LazyLock<StorageState> = LazyLock::new(StorageState::new_infallible);

/// Return a reference to the process-global [`StorageState`].
///
/// Lazily initialized on first call. Thread-safe, zero-cost after init.
pub fn shared_state() -> &'static StorageState {
    &SHARED
}
