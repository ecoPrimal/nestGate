// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

//! **HEALTH TYPES** — System health monitoring

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Health status.
///
/// ⚠ DIVERGES from `cellmembrane_types::HealthStatus` — this is an enum
/// (Healthy/Degraded/Unhealthy/Unknown) while canonical is a struct with `healthy: bool`,
/// `message`, `uptime_secs`, `checks`, and extensible `details` map.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// System is fully operational
    Healthy,
    /// System is operational but with reduced performance
    Degraded,
    /// System is not operational
    Unhealthy,
    /// Health status cannot be determined
    Unknown,
}

/// Health check result.
///
/// ⚠ DIVERGES from `cellmembrane_types::HealthCheck` — uses `component` instead of `name`,
/// `status: HealthStatus` (enum) instead of `passed: bool`, and has `timestamp: SystemTime`
/// and `response_time_ms: u64` not in canonical.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Component being checked
    pub component: String,
    /// Health status of the component
    pub status: HealthStatus,
    /// Optional message with additional details
    pub message: Option<String>,
    /// Timestamp of the health check
    pub timestamp: SystemTime,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Additional metadata about the health check
    pub metadata: HashMap<String, serde_json::Value>,
}

/// System health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Systemhealth
pub struct SystemHealth {
    /// Overall system health status
    pub overall_status: HealthStatus,
    /// Individual component health checks
    pub components: Vec<HealthCheck>,
    /// Timestamp of last health check update
    pub last_updated: SystemTime,
    /// System uptime in seconds
    pub uptime_seconds: u64,
}
