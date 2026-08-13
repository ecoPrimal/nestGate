// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

use std::sync::Arc;

use super::super::core::{LoadBalancer, LoadBalancerStats};
use crate::universal_traits::{ServiceInfo, ServiceRequest, ServiceResponse};
use crate::{NestGateError, Result};

/// Round-robin load balancer
pub struct RoundRobinLoadBalancer {
    counter: Arc<std::sync::atomic::AtomicUsize>,
    stats: Arc<parking_lot::RwLock<LoadBalancerStats>>,
}
impl RoundRobinLoadBalancer {
    /// Creates a new round-robin load balancer
    #[must_use]
    pub fn new() -> Self {
        Self {
            counter: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            stats: Arc::new(parking_lot::RwLock::new(LoadBalancerStats {
                algorithm: "round_robin".into(),
                ..LoadBalancerStats::default()
            })),
        }
    }
}

impl Default for RoundRobinLoadBalancer {
    /// Returns the default instance
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalancer for RoundRobinLoadBalancer {
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
                    algorithm: Some("round_robin".into()),
                },
            )));
        }

        let index = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % services.len();
        Ok(services[index].clone())
    }

    /// Record Response
    async fn record_response(
        &self,
        service: &ServiceInfo,
        _response: &ServiceResponse,
    ) -> Result<()> {
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
                message: Some("Round-robin does not support weights".into()),
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
        "round_robin"
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
    async fn round_robin_cycles_through_services_in_order() {
        let lb = RoundRobinLoadBalancer::new();
        let services = vec![svc("a"), svc("b"), svc("c")];
        let req = dummy_request();
        let s0 = lb
            .select_service(&services, &req)
            .await
            .expect("test: rr select 0");
        let s1 = lb
            .select_service(&services, &req)
            .await
            .expect("test: rr select 1");
        let s2 = lb
            .select_service(&services, &req)
            .await
            .expect("test: rr select 2");
        let s3 = lb
            .select_service(&services, &req)
            .await
            .expect("test: rr select 3");
        assert_eq!(s0.name, "a");
        assert_eq!(s1.name, "b");
        assert_eq!(s2.name, "c");
        assert_eq!(s3.name, "a");
        assert_eq!(lb.algorithm(), "round_robin");
    }

    #[tokio::test]
    async fn round_robin_empty_services_errors() {
        let lb = RoundRobinLoadBalancer::new();
        let err = lb
            .select_service(&[], &dummy_request())
            .await
            .expect_err("test: rr empty");
        assert!(err.to_string().contains("No services") || err.to_string().contains("available"));
    }

    #[tokio::test]
    async fn round_robin_record_response_updates_stats() {
        let lb = RoundRobinLoadBalancer::new();
        let a = svc("a");
        lb.record_response(&a, &ok_response())
            .await
            .expect("test: rr record");
        let stats = lb.get_stats().await.expect("test: rr stats");
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.service_stats.get("a").map(|s| s.requests), Some(1));
    }

    #[tokio::test]
    async fn round_robin_update_weights_not_supported() {
        let lb = RoundRobinLoadBalancer::new();
        let err = lb.update_weights(&[]).await.expect_err("test: rr weights");
        assert!(err.to_string().contains("NotImplemented") || err.to_string().contains("weights"));
    }
}
