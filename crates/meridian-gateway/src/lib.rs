//! Meridian Gateway
//!
//! Enterprise API Gateway for the $983M Meridian Platform v0.5
//!
//! ## Features
//!
//! - **Dynamic Routing**: Pattern-based routing with parameters and wildcards
//! - **Load Balancing**: Round-robin, least-connections, weighted, IP hash, random
//! - **Reverse Proxy**: High-performance proxying with connection pooling
//! - **Circuit Breaker**: Automatic fault detection and recovery
//! - **Rate Limiting**: Token bucket algorithm with per-user/route limits
//! - **Authentication**: JWT, API Key, OAuth support
//! - **Response Caching**: TTL-based caching with LRU eviction
//! - **CORS**: Configurable CORS policies
//! - **Metrics**: Prometheus integration
//! - **Health Checking**: Periodic health checks for upstreams
//! - **Request/Response Transformation**: Header and path manipulation
//!
//! ## Example
//!
//! ```rust,no_run
//! use meridian_gateway::{Gateway, config::GatewayConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = GatewayConfig::default();
//!     let gateway = Gateway::new(config).await.unwrap();
//!     gateway.start().await.unwrap();
//! }
//! ```

#![warn(missing_docs)]

pub mod cache;
pub mod circuit;
pub mod config;
pub mod gateway;
pub mod metrics;
pub mod middleware;

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::any,
    Router as AxumRouter,
};
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::{info, error};

pub use cache::{CacheKey, ResponseCache};
pub use circuit::{CircuitBreaker, CircuitBreakerState, HealthChecker, HealthStatus};
pub use config::GatewayConfig;
pub use gateway::{LoadBalancer, LoadBalancerStrategy, ProxyClient, Router};
pub use metrics::MetricsCollector;
pub use middleware::{
    AuthMiddleware, CorsMiddleware, LoggingMiddleware, RateLimitMiddleware, TransformMiddleware,
};

/// Gateway error types
#[derive(Debug, Error)]
pub enum GatewayError {
    /// Configuration validation error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Gateway initialization error
    #[error("Initialization error: {0}")]
    Init(String),

    /// Server runtime error
    #[error("Server error: {0}")]
    Server(String),

    /// Routing error
    #[error("Route error: {0}")]
    Route(String),
}

/// Gateway State
#[derive(Clone)]
pub struct GatewayState {
    config: Arc<GatewayConfig>,
    router: Arc<Router>,
    proxy: Arc<ProxyClient>,
    cache: Arc<ResponseCache>,
    metrics: Arc<dyn MetricsCollector>,
    load_balancers: Arc<dashmap::DashMap<String, LoadBalancer>>,
    circuit_breakers: Arc<dashmap::DashMap<String, CircuitBreaker>>,
}

/// Main Gateway
pub struct Gateway {
    state: GatewayState,
    app: AxumRouter,
}

impl Gateway {
    /// Create a new gateway instance
    pub async fn new(config: GatewayConfig) -> Result<Self, GatewayError> {
        // Validate configuration
        config.validate().map_err(|e| GatewayError::Config(e.to_string()))?;

        // Initialize metrics collector
        let metrics: Arc<dyn MetricsCollector> = if config.metrics.enabled {
            Arc::new(
                metrics::prometheus::PrometheusCollector::new(config.metrics.latency_buckets.clone())
                    .map_err(|e| GatewayError::Init(e.to_string()))?,
            )
        } else {
            Arc::new(metrics::NoOpCollector)
        };

        // Initialize router
        let router = Arc::new(Router::new());
        for route_config in &config.routes {
            router
                .add_route(route_config.clone())
                .map_err(|e| GatewayError::Route(e.to_string()))?;
        }

        // Initialize proxy client
        let proxy = Arc::new(ProxyClient::new(gateway::proxy::ProxyConfig {
            connect_timeout: config.server.request_timeout,
            request_timeout: config.server.request_timeout,
            ..Default::default()
        }));

        // Initialize cache
        let cache = Arc::new(ResponseCache::new(
            config.cache.max_size,
            config.cache.default_ttl,
        ));

        // Initialize load balancers for each route
        let load_balancers = Arc::new(dashmap::DashMap::new());
        for route in &config.routes {
            let lb = LoadBalancer::from_config(
                route.upstreams.clone(),
                route.load_balancer.clone(),
            )
            .map_err(|e| GatewayError::Init(e.to_string()))?;

            load_balancers.insert(route.id.clone(), lb);
        }

        // Initialize circuit breakers
        let circuit_breakers = Arc::new(dashmap::DashMap::new());
        for route in &config.routes {
            if route.circuit_breaker_enabled {
                let breaker = CircuitBreaker::new(config.circuit_breaker.clone());
                circuit_breakers.insert(route.id.clone(), breaker);
            }
        }

        let state = GatewayState {
            config: Arc::new(config.clone()),
            router,
            proxy,
            cache,
            metrics,
            load_balancers,
            circuit_breakers,
        };

        // Build application
        let app = Self::build_app(state.clone());

        Ok(Self { state, app })
    }

    /// Build the Axum application
    fn build_app(state: GatewayState) -> AxumRouter {
        let mut app = AxumRouter::new()
            .route("/*path", any(handle_request));

        // Add metrics endpoint
        if state.config.metrics.enabled {
            app = app.route(
                &state.config.metrics.endpoint,
                axum::routing::get(handle_metrics),
            );
        }

        app.with_state(state)
    }

    /// Start the gateway server
    pub async fn start(self) -> Result<(), GatewayError> {
        let addr = self.state.config.server.bind;

        info!("Starting Meridian Gateway v0.5.0 on {}", addr);
        info!("Routes configured: {}", self.state.config.routes.len());
        info!("Metrics enabled: {}", self.state.config.metrics.enabled);
        info!("Cache enabled: {}", self.state.config.cache.enabled);
        info!("Rate limiting enabled: {}", self.state.config.rate_limit.enabled);

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| GatewayError::Server(e.to_string()))?;

        info!("Gateway listening on {}", addr);

        axum::serve(listener, self.app)
            .await
            .map_err(|e| GatewayError::Server(e.to_string()))?;

        Ok(())
    }

    /// Get gateway state
    pub fn state(&self) -> &GatewayState {
        &self.state
    }
}

/// Main request handler
async fn handle_request(
    State(state): State<GatewayState>,
    request: Request,
) -> Response {
    // Save request details before moving request
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();

    // Find matching route
    let route_match = match state.router.find_route(&method, &uri) {
        Ok(m) => m,
        Err(e) => {
            error!("Route not found: {}", e);
            return (StatusCode::NOT_FOUND, "Not Found").into_response();
        }
    };

    // Get load balancer for route
    let lb = match state.load_balancers.get(&route_match.route_id) {
        Some(lb) => lb.clone(),
        None => {
            error!("No load balancer for route: {}", route_match.route_id);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response();
        }
    };

    // Select upstream
    let upstream = match lb.select(None) {
        Ok(u) => u,
        Err(e) => {
            error!("Failed to select upstream: {}", e);
            return (StatusCode::SERVICE_UNAVAILABLE, "Service Unavailable").into_response();
        }
    };

    // Check circuit breaker
    if let Some(breaker) = state.circuit_breakers.get(&route_match.route_id) {
        if breaker.allow_request().is_err() {
            error!("Circuit breaker open for route: {}", route_match.route_id);
            return (StatusCode::SERVICE_UNAVAILABLE, "Service Temporarily Unavailable").into_response();
        }
    }

    // Check cache
    if route_match.config.cache_enabled && state.config.cache.enabled {
        let cache_key = CacheKey::from_path(path.clone());
        if let Some(cached) = state.cache.get(&cache_key) {
            state.metrics.record_cache_hit(&path);

            let mut response = Response::builder()
                .status(cached.status)
                .body(Body::from(cached.body))
                .unwrap();

            for (name, value) in cached.headers {
                if let Ok(n) = name.parse::<http::HeaderName>() {
                    if let Ok(v) = value.parse::<http::HeaderValue>() {
                        response.headers_mut().insert(n, v);
                    }
                }
            }

            return response;
        } else {
            state.metrics.record_cache_miss(&path);
        }
    }

    // Track connection
    upstream.acquire();
    state.metrics.increment_connections();

    // Forward request
    let start = std::time::Instant::now();
    let response = match state.proxy.forward(request, &upstream.url).await {
        Ok(r) => r,
        Err(e) => {
            error!("Proxy error: {}", e);

            // Record failure in circuit breaker
            if let Some(breaker) = state.circuit_breakers.get(&route_match.route_id) {
                breaker.record_failure();
            }

            upstream.release();
            state.metrics.decrement_connections();

            return e.into();
        }
    };

    let duration = start.elapsed().as_secs_f64();

    // Record success in circuit breaker
    if let Some(breaker) = state.circuit_breakers.get(&route_match.route_id) {
        breaker.record_success();
    }

    // Record metrics
    state.metrics.record_request(
        &route_match.route_id,
        method.as_str(),
        response.status().as_u16(),
        duration,
    );

    upstream.release();
    state.metrics.decrement_connections();

    response
}

/// Metrics endpoint handler
async fn handle_metrics(State(state): State<GatewayState>) -> impl IntoResponse {
    // Simple snapshot for now - in production, export full Prometheus metrics
    let snapshot = state.metrics.snapshot();
    let metrics_text = format!(
        "# Meridian Gateway Metrics\n\
         meridian_gateway_requests_total {}\n\
         meridian_gateway_active_connections {}\n\
         meridian_gateway_cache_hits_total {}\n\
         meridian_gateway_cache_misses_total {}\n\
         meridian_gateway_rate_limited_total {}\n",
        snapshot.total_requests,
        snapshot.active_connections,
        snapshot.cache_hits,
        snapshot.cache_misses,
        snapshot.rate_limited
    );

    (StatusCode::OK, metrics_text).into_response()
}
