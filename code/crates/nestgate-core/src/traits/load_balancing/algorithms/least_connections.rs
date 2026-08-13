// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use super::super::core::{LoadBalancer, LoadBalancerStats};
use crate::universal_traits::{ServiceInfo, ServiceRequest, ServiceResponse};
use crate::{NestGateError, Result};

/// Least connections load balancer
pub struct LeastConnectionsLoadBalancer {
    connection_counts: Arc<dashmap::DashMap<String, Arc<AtomicU64>>>,
    stats: Arc<parking_lot::RwLock<LoadBalancerStats>>,
}
impl LeastConnectionsLoadBalancer {
    /// Creates a new least connections load balancer.
    ///
    /// The balancer tracks active connections per endpoint and routes
    /// new requests to the endpoint with the fewest active connections.
    #[must_use]
    pub fn new() -> Self {
        Self {
            connection_counts: Arc::new(dashmap::DashMap::new()),
            stats: Arc::new(parking_lot::RwLock::new(LoadBalancerStats {
                algorithm: "least_connections".into(),
                ..LoadBalancerStats::default()
            })),
        }
    }
}

impl Default for LeastConnectionsLoadBalancer {
    /// Returns the default instance
    fn default() -> Self {
        Self::new()
    }
}

impl LeastConnectionsLoadBalancer {
    fn active_connections(&self, service_name: &str) -> u64 {
        self.connection_counts
            .get(service_name)
            .map_or(0, |counter| counter.load(Ordering::Relaxed))
    }

    fn increment_active_connections(&self, service_name: &str) {
        self.connection_counts
            .entry(service_name.to_string())
            .or_insert_with(|| Arc::new(AtomicU64::new(0)))
            .fetch_add(1, Ordering::Relaxed);
    }

    fn decrement_active_connections(&self, service_name: &str) {
        if let Some(counter) = self.connection_counts.get(service_name) {
            counter
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                    current.checked_sub(1)
                })
                .ok();
        }
    }
}

impl LoadBalancer for LeastConnectionsLoadBalancer {
    /// Select Service
    async fn select_service(
        &self,
        services: &[ServiceInfo],
        _request: &ServiceRequest,
    ) -> Result<ServiceInfo> {
        if services.is_empty() {
            return Err(NestGateError::LoadBalancer(Box::new(
                crate::error::variants::core_errors::LoadBalancerErrorDetails {
                    message: "No services available".into(),
                    available_services: Some(0),
                    algorithm: Some("least_connections".into()),
                },
            )));
        }

        let mut min_connections = u64::MAX;
        let mut selected_service = None;

        for service in services {
            let connections = self.active_connections(&service.name);
            if connections < min_connections {
                min_connections = connections;
                selected_service = Some(service.clone());
            }
        }

        let Some(selected) = selected_service else {
            return Err(NestGateError::LoadBalancer(Box::new(
                crate::error::variants::core_errors::LoadBalancerErrorDetails {
                    message: "Failed to select service with least connections".into(),
                    available_services: Some(services.len()),
                    algorithm: Some("least_connections".into()),
                },
            )));
        };

        self.increment_active_connections(&selected.name);

        {
            let mut stats = self.stats.write();
            stats.total_requests += 1;
            stats
                .service_stats
                .entry(selected.name.clone())
                .or_default()
                .requests += 1;
        }

        Ok(selected)
    }

    /// Record Response
    async fn record_response(
        &self,
        service: &ServiceInfo,
        _response: &ServiceResponse,
    ) -> Result<()> {
        self.decrement_active_connections(&service.name);

        {
            let mut stats = self.stats.write();
            stats.total_requests += 1;
            stats
                .service_stats
                .entry(service.name.clone())
                .or_default()
                .requests += 1;
        }
        Ok(())
    }

    /// Updates  Weights
    async fn update_weights(&self, _weights: &[(&str, f64)]) -> Result<()> {
        Err(NestGateError::NotImplemented(Box::new(
            crate::error::variants::core_errors::NotImplementedErrorDetails {
                feature: "update_weights".into(),
                message: Some("Least connections does not support weights".into()),
                planned_version: None,
            },
        )))
    }

    /// Gets Stats
    async fn get_stats(&self) -> Result<LoadBalancerStats> {
        Ok(self.stats.read().clone())
    }

    /// Algorithm
    fn algorithm(&self) -> &'static str {
        "least_connections"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::load_balancing::core::LoadBalancer;
    use crate::universal_traits::orchestration::ServiceStatus;
    use crate::universal_traits::{ServiceInfo, ServiceRequest, ServiceResponse};
    use std::collections::HashMap;
    use std::time::SystemTime;

    fn svc(name: &str) -> ServiceInfo {
        ServiceInfo {
            id: format!("id-{name}"),
            name: name.to_string(),
            version: "1.0.0".into(),
            capabilities: vec![],
            status: ServiceStatus::Healthy,
            last_seen: SystemTime::UNIX_EPOCH,
        }
    }

    fn dummy_request() -> ServiceRequest {
        ServiceRequest {
            service_id: "svc".into(),
            action: "ping".into(),
            parameters: HashMap::default(),
            timeout_seconds: Some(5),
        }
    }

    fn ok_response() -> ServiceResponse {
        ServiceResponse {
            success: true,
            data: None,
            error_message: None,
            execution_time_ms: 1,
        }
    }

    #[tokio::test]
    async fn least_connections_prefers_lower_load_then_first_tie() {
        let lb = LeastConnectionsLoadBalancer::new();
        let a = svc("heavy");
        let b = svc("light");
        let services = vec![a.clone(), b.clone()];
        let req = dummy_request();
        let first = lb
            .select_service(&services, &req)
            .await
            .expect("test: lc first");
        assert_eq!(first.name, "heavy");
        let second = lb
            .select_service(&services, &req)
            .await
            .expect("test: lc second");
        assert_eq!(second.name, "light");
        lb.record_response(&first, &ok_response())
            .await
            .expect("test: lc record heavy");
        lb.record_response(&second, &ok_response())
            .await
            .expect("test: lc record light");
        assert_eq!(lb.algorithm(), "least_connections");
    }

    #[tokio::test]
    async fn least_connections_empty_services_errors() {
        let lb = LeastConnectionsLoadBalancer::new();
        let err = lb
            .select_service(&[], &dummy_request())
            .await
            .expect_err("test: lc empty");
        assert!(err.to_string().contains("No services") || err.to_string().contains("available"));
    }

    #[tokio::test]
    async fn least_connections_update_weights_not_supported() {
        let lb = LeastConnectionsLoadBalancer::new();
        assert!(lb.update_weights(&[]).await.is_err());
    }
}
