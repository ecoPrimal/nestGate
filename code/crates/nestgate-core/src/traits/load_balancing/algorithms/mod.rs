// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective

//! # Load Balancing Algorithm Implementations
//! Algorithms functionality and utilities.
// Basic load balancing algorithms (Round Robin, Random, Least Connections)

mod least_connections;
mod random;
mod resource_based;
mod round_robin;

pub use least_connections::LeastConnectionsLoadBalancer;
pub use random::RandomLoadBalancer;
pub use resource_based::ResourceBasedLoadBalancer;
pub use round_robin::RoundRobinLoadBalancer;
