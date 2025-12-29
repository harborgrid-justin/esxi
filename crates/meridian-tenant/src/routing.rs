//! Tenant-aware routing and middleware for HTTP applications.

#[cfg(feature = "http")]
use axum::{
    extract::{FromRequestParts, Request},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{TenantError, TenantResult};
use crate::tenant::Tenant;

/// Tenant context extracted from HTTP requests.
#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub tenant_slug: String,
    pub tenant: Option<Arc<Tenant>>,
}

impl TenantContext {
    pub fn new(tenant_id: Uuid, tenant_slug: impl Into<String>) -> Self {
        Self {
            tenant_id,
            tenant_slug: tenant_slug.into(),
            tenant: None,
        }
    }

    pub fn with_tenant(mut self, tenant: Arc<Tenant>) -> Self {
        self.tenant = Some(tenant);
        self
    }
}

/// Strategy for resolving tenant from HTTP requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantResolutionStrategy {
    /// Extract from subdomain (e.g., acme.app.com)
    Subdomain,
    /// Extract from custom domain (e.g., acme-gis.com)
    CustomDomain,
    /// Extract from URL path (e.g., /tenants/acme/...)
    PathPrefix,
    /// Extract from request header (e.g., X-Tenant-ID)
    Header,
    /// Extract from JWT token claim
    JwtClaim,
    /// Multiple strategies in order of preference
    Multi(Vec<TenantResolutionStrategy>),
}

/// Tenant resolver trait for custom resolution logic.
pub trait TenantResolver: Send + Sync {
    /// Resolves tenant from request parts.
    fn resolve(&self, host: Option<&str>, path: &str, headers: &HeaderMap) -> TenantResult<String>;
}

use std::collections::HashMap;

pub type HeaderMap = HashMap<String, String>;

/// Subdomain-based tenant resolver.
pub struct SubdomainResolver {
    base_domain: String,
}

impl SubdomainResolver {
    pub fn new(base_domain: impl Into<String>) -> Self {
        Self {
            base_domain: base_domain.into(),
        }
    }

    fn extract_subdomain(&self, host: &str) -> Option<String> {
        if let Some(subdomain_end) = host.find(&format!(".{}", self.base_domain)) {
            let subdomain = &host[..subdomain_end];
            if !subdomain.is_empty() && subdomain != "www" {
                return Some(subdomain.to_string());
            }
        }
        None
    }
}

impl TenantResolver for SubdomainResolver {
    fn resolve(&self, host: Option<&str>, _path: &str, _headers: &HeaderMap) -> TenantResult<String> {
        let host = host.ok_or_else(|| {
            TenantError::InvalidTenantId("No host header found".to_string())
        })?;

        self.extract_subdomain(host).ok_or_else(|| {
            TenantError::InvalidTenantId(format!("Cannot extract tenant from host: {}", host))
        })
    }
}

/// Path-based tenant resolver.
pub struct PathResolver {
    prefix: String,
}

impl PathResolver {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }

    fn extract_from_path(&self, path: &str) -> Option<String> {
        if let Some(stripped) = path.strip_prefix(&self.prefix) {
            if let Some(end) = stripped.find('/') {
                return Some(stripped[..end].to_string());
            } else if !stripped.is_empty() {
                return Some(stripped.to_string());
            }
        }
        None
    }
}

impl TenantResolver for PathResolver {
    fn resolve(&self, _host: Option<&str>, path: &str, _headers: &HeaderMap) -> TenantResult<String> {
        self.extract_from_path(path).ok_or_else(|| {
            TenantError::InvalidTenantId(format!("Cannot extract tenant from path: {}", path))
        })
    }
}

/// Header-based tenant resolver.
pub struct HeaderResolver {
    header_name: String,
}

impl HeaderResolver {
    pub fn new(header_name: impl Into<String>) -> Self {
        Self {
            header_name: header_name.into(),
        }
    }
}

impl TenantResolver for HeaderResolver {
    fn resolve(&self, _host: Option<&str>, _path: &str, headers: &HeaderMap) -> TenantResult<String> {
        headers
            .get(&self.header_name)
            .map(|v| v.to_string())
            .ok_or_else(|| {
                TenantError::InvalidTenantId(format!(
                    "Tenant header '{}' not found",
                    self.header_name
                ))
            })
    }
}

/// Multi-strategy resolver that tries multiple strategies.
pub struct MultiStrategyResolver {
    resolvers: Vec<Box<dyn TenantResolver>>,
}

impl MultiStrategyResolver {
    pub fn new() -> Self {
        Self {
            resolvers: Vec::new(),
        }
    }

    pub fn add_resolver(mut self, resolver: Box<dyn TenantResolver>) -> Self {
        self.resolvers.push(resolver);
        self
    }
}

impl Default for MultiStrategyResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl TenantResolver for MultiStrategyResolver {
    fn resolve(&self, host: Option<&str>, path: &str, headers: &HeaderMap) -> TenantResult<String> {
        for resolver in &self.resolvers {
            if let Ok(tenant_slug) = resolver.resolve(host, path, headers) {
                return Ok(tenant_slug);
            }
        }

        Err(TenantError::InvalidTenantId(
            "Could not resolve tenant using any strategy".to_string()
        ))
    }
}

/// Tenant routing configuration.
#[derive(Debug, Clone)]
pub struct TenantRoutingConfig {
    pub resolution_strategy: TenantResolutionStrategy,
    pub require_tenant: bool,
    pub default_tenant: Option<String>,
    pub excluded_paths: Vec<String>,
}

impl Default for TenantRoutingConfig {
    fn default() -> Self {
        Self {
            resolution_strategy: TenantResolutionStrategy::Subdomain,
            require_tenant: true,
            default_tenant: None,
            excluded_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/favicon.ico".to_string(),
            ],
        }
    }
}

impl TenantRoutingConfig {
    pub fn is_excluded_path(&self, path: &str) -> bool {
        self.excluded_paths.iter().any(|excluded| path.starts_with(excluded))
    }
}

#[cfg(feature = "http")]
/// Axum middleware for tenant resolution.
pub async fn tenant_middleware(
    tenant_context: Option<TenantContext>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if tenant_context.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let response = next.run(request).await;
    Ok(response)
}

#[cfg(feature = "http")]
/// Axum extractor for tenant context.
#[async_trait::async_trait]
impl<S> FromRequestParts<S> for TenantContext
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Try to extract tenant from extensions (set by middleware)
        if let Some(context) = parts.extensions.get::<TenantContext>() {
            return Ok(context.clone());
        }

        Err((
            StatusCode::BAD_REQUEST,
            "Tenant context not found".to_string(),
        ))
    }
}

/// Tenant URL generator for creating tenant-specific URLs.
pub struct TenantUrlGenerator {
    base_url: String,
    strategy: TenantResolutionStrategy,
}

impl TenantUrlGenerator {
    pub fn new(base_url: impl Into<String>, strategy: TenantResolutionStrategy) -> Self {
        Self {
            base_url: base_url.into(),
            strategy,
        }
    }

    /// Generates a tenant-specific URL.
    pub fn generate_url(&self, tenant_slug: &str, path: &str) -> String {
        match self.strategy {
            TenantResolutionStrategy::Subdomain => {
                format!("https://{}.{}{}", tenant_slug, self.base_url, path)
            }
            TenantResolutionStrategy::PathPrefix => {
                format!("https://{}/tenants/{}{}", self.base_url, tenant_slug, path)
            }
            TenantResolutionStrategy::CustomDomain => {
                // Custom domain would be looked up from tenant config
                format!("https://{}{}", tenant_slug, path)
            }
            _ => {
                format!("https://{}{}", self.base_url, path)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subdomain_resolver() {
        let resolver = SubdomainResolver::new("example.com");
        let mut headers = HeaderMap::new();

        let result = resolver.resolve(Some("acme.example.com"), "/", &headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "acme");

        let result = resolver.resolve(Some("www.example.com"), "/", &headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_path_resolver() {
        let resolver = PathResolver::new("/tenants/");
        let headers = HeaderMap::new();

        let result = resolver.resolve(None, "/tenants/acme/dashboard", &headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "acme");

        let result = resolver.resolve(None, "/other/path", &headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_header_resolver() {
        let resolver = HeaderResolver::new("X-Tenant-ID");
        let mut headers = HeaderMap::new();
        headers.insert("X-Tenant-ID".to_string(), "acme-corp".to_string());

        let result = resolver.resolve(None, "/", &headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "acme-corp");
    }

    #[test]
    fn test_url_generator() {
        let generator = TenantUrlGenerator::new(
            "app.example.com",
            TenantResolutionStrategy::Subdomain,
        );

        let url = generator.generate_url("acme", "/dashboard");
        assert_eq!(url, "https://acme.app.example.com/dashboard");
    }

    #[test]
    fn test_routing_config() {
        let config = TenantRoutingConfig::default();
        assert!(config.is_excluded_path("/health"));
        assert!(config.is_excluded_path("/metrics"));
        assert!(!config.is_excluded_path("/api/data"));
    }
}
