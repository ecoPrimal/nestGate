// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

//! CAS gossip event emission layer.
//!
//! Every successful CAS write or replication event emits a [`CasEvent`] through
//! the global [`GossipEmitter`]. The emitter is non-blocking: events are sent to
//! a bounded channel and logged at `debug` level. A future gossip bridge
//! (discovered at runtime via `gossip.inject` capability) can drain the channel
//! and forward events to the mesh.
//!
//! This design follows primal sovereignty: nestGate has self-knowledge only and
//! does not import swarmVine or any gossip primal directly. The bridge is
//! capability-discovered at runtime.

use serde::Serialize;
use std::sync::Mutex;
use tokio::sync::mpsc;
use tracing::debug;

const CHANNEL_CAPACITY: usize = 256;

/// A CAS lifecycle event suitable for gossip injection.
#[derive(Debug, Clone, Serialize)]
pub struct CasEvent {
    /// Event type discriminator.
    pub kind: CasEventKind,
    /// BLAKE3 content hash (hex).
    pub hash: String,
    /// Byte size of the stored object (0 if unknown).
    pub size: u64,
    /// Originating pipeline (e.g. `content.put`, `content.ingest`, `coord.ingest`).
    pub pipeline: &'static str,
    /// Whether the write was deduplicated (object already existed).
    pub deduplicated: bool,
    /// Family ID scope (empty string if unscoped).
    pub family_id: String,
}

/// Discriminates the type of CAS event.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CasEventKind {
    /// A new object was stored via content-addressed write.
    ObjectStored,
    /// An object was replicated from a remote gate.
    ObjectReplicated,
    /// A manifest (multi-object reference) was published.
    ManifestPublished,
    /// A footprint revision was stored.
    FootprintStored,
    /// A coordination artifact was ingested.
    CoordIngested,
    /// An external object was cached locally.
    ExternalCached,
}

static GOSSIP_TX: Mutex<Option<mpsc::Sender<CasEvent>>> = Mutex::new(None);

/// Emit a CAS event to the gossip channel (non-blocking, best-effort).
///
/// If no subscriber is attached or the channel is full the event is dropped
/// silently — gossip is eventually-consistent by design and must never block
/// CAS writes.
pub fn emit(event: CasEvent) {
    debug!(
        kind = ?event.kind,
        hash = %event.hash,
        size = event.size,
        pipeline = event.pipeline,
        dedup = event.deduplicated,
        "gossip: CAS event"
    );
    if let Ok(guard) = GOSSIP_TX.lock()
        && let Some(tx) = guard.as_ref()
    {
        let _ = tx.try_send(event);
    }
}

/// Subscribe to gossip events (for bridge / test consumption).
///
/// Replaces any previous subscriber. Returns a receiver that drains incoming
/// events. Only one consumer should be active at a time.
#[must_use]
pub fn subscribe() -> mpsc::Receiver<CasEvent> {
    let (tx, rx) = mpsc::channel(CHANNEL_CAPACITY);
    if let Ok(mut guard) = GOSSIP_TX.lock() {
        *guard = Some(tx);
    }
    rx
}

/// Convenience: emit an `ObjectStored` event.
pub fn emit_object_stored(
    hash: impl Into<String>,
    size: u64,
    pipeline: &'static str,
    deduplicated: bool,
    family_id: impl Into<String>,
) {
    emit(CasEvent {
        kind: CasEventKind::ObjectStored,
        hash: hash.into(),
        size,
        pipeline,
        deduplicated,
        family_id: family_id.into(),
    });
}

/// Convenience: emit an `ObjectReplicated` event.
pub fn emit_object_replicated(
    hash: impl Into<String>,
    size: u64,
    pipeline: &'static str,
    family_id: impl Into<String>,
) {
    emit(CasEvent {
        kind: CasEventKind::ObjectReplicated,
        hash: hash.into(),
        size,
        pipeline,
        deduplicated: false,
        family_id: family_id.into(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn emit_and_receive_object_stored() {
        let mut rx = subscribe();
        emit_object_stored("abc123", 1024, "content.put", false, "test-family");

        let event = rx.try_recv().expect("should receive event");
        assert_eq!(event.hash, "abc123");
        assert_eq!(event.size, 1024);
        assert_eq!(event.pipeline, "content.put");
        assert!(!event.deduplicated);
        assert_eq!(event.family_id, "test-family");
        assert!(matches!(event.kind, CasEventKind::ObjectStored));
    }

    #[tokio::test]
    async fn emit_replicated_event() {
        let mut rx = subscribe();
        emit_object_replicated("def456", 2048, "content.replicate.pull", "remote-family");

        let event = rx.try_recv().expect("should receive event");
        assert_eq!(event.hash, "def456");
        assert!(matches!(event.kind, CasEventKind::ObjectReplicated));
    }

    #[test]
    fn emit_without_receiver_does_not_panic() {
        emit_object_stored("no-receiver", 0, "test", false, "");
    }
}
