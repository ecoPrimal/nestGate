// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::Arc;

use super::super::core::{LoadBalancer, LoadBalancerStats};
use crate::universal_traits::{ServiceInfo, ServiceRequest, ServiceResponse};
use crate::{NestGateError, Result};

/// Random load balancer
pub struct RandomLoadBalancer {
    rng: Arc<parking_lot::Mutex<StdRng>>,
    stats: Arc<parking_lot::RwLock<LoadBalancerStats>>,
}
impl RandomLoadBalancer {
    /// Creates a new random load balancer.
    ///
    /// The balancer uses cryptographically secure randomness to distribute
    /// requests evenly across available endpoints.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rng: Arc::new(parking_lot::Mutex::new(StdRng::from_os_rng())),
            stats: Arc::new(parking_lot::RwLock::new(LoadBalancerStats {
                algorithm: "random".into(),
                ..LoadBalancerStats::default()
            })),
        }
    }
}

impl Default for RandomLoadBalancer {
    /// Returns the default instance
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalancer for RandomLoadBalancer {
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
                    algorithm: Some("random".into()),
                },
            )));
        }

        let index = {
            let mut rng = self.rng.lock();
            rng.random_range(0..services.len())
        };
        let selected = services[index].clone();

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
                message: Some("Random load balancer does not support weights".into()),
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
        "random"
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
    async fn random_balancer_selects_from_non_empty_list() {
        let lb = RandomLoadBalancer::new();
        let services = vec![svc("x"), svc("y")];
        let picked = lb
            .select_service(&services, &dummy_request())
            .await
            .expect("test: random pick");
        assert!(picked.name == "x" || picked.name == "y");
        let stats = lb.get_stats().await.expect("test: random stats");
        assert!(stats.total_requests >= 1);
        assert_eq!(lb.algorithm(), "random");
    }

    #[tokio::test]
    async fn random_balancer_empty_services_errors() {
        let lb = RandomLoadBalancer::new();
        let err = lb
            .select_service(&[], &dummy_request())
            .await
            .expect_err("test: random empty");
        assert!(err.to_string().contains("No services") || err.to_string().contains("available"));
    }

    #[tokio::test]
    async fn random_balancer_record_response_and_update_weights() {
        let lb = RandomLoadBalancer::new();
        let s = svc("z");
        lb.record_response(&s, &ok_response())
            .await
            .expect("test: random record");
        let stats = lb.get_stats().await.expect("test: random stats");
        assert!(stats.service_stats.get("z").map_or(0, |x| x.requests) >= 1);
        assert!(lb.update_weights(&[]).await.is_err());
    }
}
