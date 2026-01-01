//! Dynamic Router
//!
//! Enterprise routing with pattern matching and dynamic route configuration.

use crate::config::RouteConfig;
use dashmap::DashMap;
use http::{Method, Uri};
use regex::Regex;
use std::sync::Arc;
use thiserror::Error;

/// Router errors
#[derive(Debug, Error)]
pub enum RouterError {
    /// No matching route found
    #[error("No route found for path: {0}")]
    NotFound(String),

    /// HTTP method not allowed for route
    #[error("Method not allowed: {0}")]
    MethodNotAllowed(String),

    /// Invalid route pattern syntax
    #[error("Invalid route pattern: {0}")]
    InvalidPattern(String),
}

/// Route match result
#[derive(Debug, Clone)]
pub struct RouteMatch {
    /// Matched route identifier
    pub route_id: String,
    /// Extracted path parameters
    pub path_params: Vec<(String, String)>,
    /// Route configuration
    pub config: RouteConfig,
}

/// Route pattern
#[derive(Debug, Clone)]
struct RoutePattern {
    regex: Option<Regex>,
    param_names: Vec<String>,
}

impl RoutePattern {
    /// Create a new route pattern
    fn new(pattern: String) -> Result<Self, RouterError> {
        let (regex, param_names) = Self::compile_pattern(&pattern)?;

        Ok(Self {
            regex: Some(regex),
            param_names,
        })
    }

    /// Compile pattern to regex
    fn compile_pattern(pattern: &str) -> Result<(Regex, Vec<String>), RouterError> {
        let mut regex_pattern = String::from("^");
        let mut param_names = Vec::new();
        let mut chars = pattern.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                ':' => {
                    // Named parameter
                    let mut param_name = String::new();
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_alphanumeric() || next_ch == '_' {
                            param_name.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    param_names.push(param_name);
                    regex_pattern.push_str("([^/]+)");
                }
                '*' => {
                    // Wildcard
                    regex_pattern.push_str(".*");
                }
                '/' | '.' | '-' | '_' => {
                    regex_pattern.push(ch);
                }
                _ if ch.is_alphanumeric() => {
                    regex_pattern.push(ch);
                }
                _ => {
                    regex_pattern.push('\\');
                    regex_pattern.push(ch);
                }
            }
        }

        regex_pattern.push('$');

        let regex = Regex::new(&regex_pattern)
            .map_err(|e| RouterError::InvalidPattern(e.to_string()))?;

        Ok((regex, param_names))
    }

    /// Match path against pattern
    fn matches(&self, path: &str) -> Option<Vec<(String, String)>> {
        if let Some(ref regex) = self.regex {
            if let Some(captures) = regex.captures(path) {
                let mut params = Vec::new();
                for (i, param_name) in self.param_names.iter().enumerate() {
                    if let Some(capture) = captures.get(i + 1) {
                        params.push((param_name.clone(), capture.as_str().to_string()));
                    }
                }
                return Some(params);
            }
        }
        None
    }
}

/// Route entry
#[derive(Clone)]
struct Route {
    id: String,
    pattern: RoutePattern,
    methods: Vec<Method>,
    config: RouteConfig,
    priority: i32,
}

/// Router
///
/// Dynamic routing with pattern matching, method filtering, and priority-based selection.
pub struct Router {
    routes: Arc<DashMap<String, Route>>,
    route_list: Arc<parking_lot::RwLock<Vec<Route>>>,
}

impl Router {
    /// Create a new router
    pub fn new() -> Self {
        Self {
            routes: Arc::new(DashMap::new()),
            route_list: Arc::new(parking_lot::RwLock::new(vec![])),
        }
    }

    /// Add a route
    pub fn add_route(&self, config: RouteConfig) -> Result<(), RouterError> {
        let pattern = RoutePattern::new(config.path.clone())?;

        let methods: Vec<Method> = config
            .methods
            .iter()
            .filter_map(|m| m.parse().ok())
            .collect();

        let route = Route {
            id: config.id.clone(),
            pattern,
            methods,
            config: config.clone(),
            priority: 0, // Can be configurable
        };

        self.routes.insert(config.id.clone(), route.clone());

        // Update sorted route list
        let mut route_list = self.route_list.write();
        route_list.push(route);
        route_list.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(())
    }

    /// Remove a route
    pub fn remove_route(&self, route_id: &str) {
        self.routes.remove(route_id);

        let mut route_list = self.route_list.write();
        route_list.retain(|r| r.id != route_id);
    }

    /// Update a route
    pub fn update_route(&self, config: RouteConfig) -> Result<(), RouterError> {
        self.remove_route(&config.id);
        self.add_route(config)
    }

    /// Find matching route
    pub fn find_route(&self, method: &Method, uri: &Uri) -> Result<RouteMatch, RouterError> {
        let path = uri.path();
        let route_list = self.route_list.read();

        for route in route_list.iter() {
            // Check if path matches pattern
            if let Some(path_params) = route.pattern.matches(path) {
                // Check if method is allowed
                if !route.methods.is_empty() && !route.methods.contains(method) {
                    continue;
                }

                return Ok(RouteMatch {
                    route_id: route.id.clone(),
                    path_params,
                    config: route.config.clone(),
                });
            }
        }

        Err(RouterError::NotFound(path.to_string()))
    }

    /// Get route by ID
    pub fn get_route(&self, route_id: &str) -> Option<RouteConfig> {
        self.routes.get(route_id).map(|r| r.config.clone())
    }

    /// Get all routes
    pub fn routes(&self) -> Vec<RouteConfig> {
        self.routes
            .iter()
            .map(|entry| entry.value().config.clone())
            .collect()
    }

    /// Check if route exists
    pub fn has_route(&self, route_id: &str) -> bool {
        self.routes.contains_key(route_id)
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Router {
    fn clone(&self) -> Self {
        Self {
            routes: Arc::clone(&self.routes),
            route_list: Arc::clone(&self.route_list),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LoadBalancerStrategy, UpstreamConfig};
    use std::time::Duration;

    fn create_test_route(id: &str, path: &str, methods: Vec<&str>) -> RouteConfig {
        RouteConfig {
            id: id.to_string(),
            path: path.to_string(),
            methods: methods.iter().map(|s| s.to_string()).collect(),
            upstreams: vec![UpstreamConfig {
                id: "test-upstream".to_string(),
                url: "http://localhost:8001".to_string(),
                weight: 1,
                health_check: None,
                connect_timeout: Duration::from_secs(5),
                max_retries: 3,
            }],
            load_balancer: LoadBalancerStrategy::RoundRobin,
            middleware: vec![],
            timeout: None,
            circuit_breaker_enabled: false,
            cache_enabled: false,
        }
    }

    #[test]
    fn test_exact_match() {
        let router = Router::new();
        router
            .add_route(create_test_route("test", "/api/users", vec!["GET"]))
            .unwrap();

        let uri: Uri = "/api/users".parse().unwrap();
        let route_match = router.find_route(&Method::GET, &uri).unwrap();

        assert_eq!(route_match.route_id, "test");
        assert!(route_match.path_params.is_empty());
    }

    #[test]
    fn test_wildcard_match() {
        let router = Router::new();
        router
            .add_route(create_test_route("test", "/api/*", vec!["GET"]))
            .unwrap();

        let uri: Uri = "/api/users/123".parse().unwrap();
        let route_match = router.find_route(&Method::GET, &uri).unwrap();

        assert_eq!(route_match.route_id, "test");
    }

    #[test]
    fn test_param_match() {
        let router = Router::new();
        router
            .add_route(create_test_route("test", "/api/users/:id", vec!["GET"]))
            .unwrap();

        let uri: Uri = "/api/users/123".parse().unwrap();
        let route_match = router.find_route(&Method::GET, &uri).unwrap();

        assert_eq!(route_match.route_id, "test");
        assert_eq!(route_match.path_params.len(), 1);
        assert_eq!(route_match.path_params[0].0, "id");
        assert_eq!(route_match.path_params[0].1, "123");
    }

    #[test]
    fn test_multiple_params() {
        let router = Router::new();
        router
            .add_route(create_test_route(
                "test",
                "/api/:resource/:id",
                vec!["GET"],
            ))
            .unwrap();

        let uri: Uri = "/api/users/123".parse().unwrap();
        let route_match = router.find_route(&Method::GET, &uri).unwrap();

        assert_eq!(route_match.path_params.len(), 2);
        assert_eq!(route_match.path_params[0].0, "resource");
        assert_eq!(route_match.path_params[0].1, "users");
        assert_eq!(route_match.path_params[1].0, "id");
        assert_eq!(route_match.path_params[1].1, "123");
    }

    #[test]
    fn test_method_not_allowed() {
        let router = Router::new();
        router
            .add_route(create_test_route("test", "/api/users", vec!["GET"]))
            .unwrap();

        let uri: Uri = "/api/users".parse().unwrap();
        let result = router.find_route(&Method::POST, &uri);

        assert!(result.is_err());
    }

    #[test]
    fn test_not_found() {
        let router = Router::new();
        router
            .add_route(create_test_route("test", "/api/users", vec!["GET"]))
            .unwrap();

        let uri: Uri = "/api/posts".parse().unwrap();
        let result = router.find_route(&Method::GET, &uri);

        assert!(result.is_err());
    }
}
