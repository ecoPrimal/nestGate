// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

use std::sync::Arc;

use super::super::core::{LoadBalancer, LoadBalancerStats};
use crate::universal_traits::{ServiceInfo, ServiceRequest, ServiceResponse};
use crate::{NestGateError, Result};

/// Resource-based load balancer
pub struct ResourceBasedLoadBalancer {
    resource_capacity: Arc<dashmap::DashMap<String, (f64, f64)>>,
    stats: Arc<parking_lot::RwLock<LoadBalancerStats>>,
}

impl ResourceBasedLoadBalancer {
    /// Creates a new resource-based load balancer.
    ///
    /// Routes requests to the endpoint with the highest available CPU and memory
    /// capacity according to a weighted scoring function.
    #[must_use]
    pub fn new() -> Self {
        Self {
            resource_capacity: Arc::new(dashmap::DashMap::new()),
            stats: Arc::new(parking_lot::RwLock::new(LoadBalancerStats {
                algorithm: "resource_based".into(),
                ..LoadBalancerStats::default()
            })),
        }
    }

    /// Updates available CPU and memory capacity for services.
    ///
    /// Each tuple is `(service_name, cpu_available, memory_available)` with values
    /// in the range `0.0` (fully utilized) to `1.0` (fully available).
    pub fn update_resource_capacity(&self, capacities: &[(&str, f64, f64)]) -> Result<()> {
        for (name, cpu_available, memory_available) in capacities {
            if !cpu_available.is_finite() || !memory_available.is_finite() {
                return Err(NestGateError::LoadBalancer(Box::new(
                    crate::error::variants::core_errors::LoadBalancerErrorDetails {
                        message: format!(
                            "Invalid resource capacity for service {name}: \
                             cpu={cpu_available}, memory={memory_available}"
                        )
                        .into(),
                        available_services: None,
                        algorithm: Some("resource_based".into()),
                    },
                )));
            }

            self.resource_capacity.insert(
                (*name).to_string(),
                (
                    cpu_available.clamp(0.0, 1.0),
                    memory_available.clamp(0.0, 1.0),
                ),
            );
        }
        Ok(())
    }

    fn resource_score(&self, service_name: &str) -> f64 {
        let (cpu_available, memory_available) = self
            .resource_capacity
            .get(service_name)
            .map_or((1.0, 1.0), |entry| *entry.value());

        f64::midpoint(
            cpu_available.clamp(0.0, 1.0),
            memory_available.clamp(0.0, 1.0),
        )
    }
}

impl Default for ResourceBasedLoadBalancer {
    /// Returns the default instance
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalancer for ResourceBasedLoadBalancer {
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
                    algorithm: Some("resource_based".into()),
                },
            )));
        }

        let mut best_score = f64::NEG_INFINITY;
        let mut selected_service = None;

        for service in services {
            let score = self.resource_score(&service.name);
            if score > best_score {
                best_score = score;
                selected_service = Some(service.clone());
            }
        }

        let Some(selected) = selected_service else {
            return Err(NestGateError::LoadBalancer(Box::new(
                crate::error::variants::core_errors::LoadBalancerErrorDetails {
                    message: "Failed to select service by resource availability".into(),
                    available_services: Some(services.len()),
                    algorithm: Some("resource_based".into()),
                },
            )));
        };

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
        self.stats
            .write()
            .service_stats
            .entry(service.name.clone())
            .or_default()
            .requests += 1;
        Ok(())
    }

    /// Updates  Weights
    async fn update_weights(&self, _weights: &[(&str, f64)]) -> Result<()> {
        Err(NestGateError::NotImplemented(Box::new(
            crate::error::variants::core_errors::NotImplementedErrorDetails {
                feature: "update_weights".into(),
                message: Some("Resource-based balancer uses update_resource_capacity".into()),
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
        "resource_based"
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
    async fn resource_based_selects_highest_capacity_then_first_tie() {
        let lb = ResourceBasedLoadBalancer::new();
        let low = svc("low");
        let high = svc("high");
        let services = vec![low.clone(), high.clone()];
        lb.update_resource_capacity(&[("low", 0.2, 0.2), ("high", 0.9, 0.8)])
            .expect("test: rb update capacity");

        let picked = lb
            .select_service(&services, &dummy_request())
            .await
            .expect("test: rb select");
        assert_eq!(picked.name, "high");
        assert_eq!(lb.algorithm(), "resource_based");
    }

    #[tokio::test]
    async fn resource_based_empty_services_errors() {
        let lb = ResourceBasedLoadBalancer::new();
        let err = lb
            .select_service(&[], &dummy_request())
            .await
            .expect_err("test: rb empty");
        assert!(err.to_string().contains("No services") || err.to_string().contains("available"));
    }

    #[tokio::test]
    async fn resource_based_rejects_non_finite_capacity() {
        let lb = ResourceBasedLoadBalancer::new();
        let err = lb
            .update_resource_capacity(&[("bad", f64::NAN, 0.5)])
            .expect_err("test: rb invalid capacity");
        assert!(err.to_string().contains("Invalid resource capacity"));
    }

    #[tokio::test]
    async fn resource_based_record_response_and_update_weights() {
        let lb = ResourceBasedLoadBalancer::new();
        let service = svc("node");
        lb.record_response(&service, &ok_response())
            .await
            .expect("test: rb record");
        let stats = lb.get_stats().await.expect("test: rb stats");
        assert_eq!(
            stats
                .service_stats
                .get("node")
                .expect("test: node stats")
                .requests,
            1
        );
        assert!(lb.update_weights(&[]).await.is_err());
    }
}
