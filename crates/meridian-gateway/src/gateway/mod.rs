//! Gateway Module
//!
//! Core gateway functionality including routing, proxying, and load balancing.

pub mod load_balancer;
pub mod proxy;
pub mod router;

pub use load_balancer::{LoadBalancer, LoadBalancerStrategy};
pub use proxy::ProxyClient;
pub use router::Router;
