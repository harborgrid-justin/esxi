//! Security context for zero-trust evaluation
//!
//! Captures all relevant information about a request for policy evaluation:
//! - User identity and attributes
//! - Device information
//! - Network context
//! - Time and location
//! - Risk indicators
//!
//! ## Zero Trust Principles
//! Every request is evaluated based on comprehensive context,
//! not just authentication. Context includes:
//! - WHO: User identity, roles, clearance
//! - WHAT: Resource being accessed
//! - WHEN: Time of day, business hours
//! - WHERE: Network location, geography
//! - HOW: Device posture, security state

use std::net::IpAddr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{SecurityError, SecurityResult};

/// Request context for zero-trust evaluation
///
/// Contains all information needed to make access control decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// User ID making the request
    pub user_id: String,

    /// User's roles
    pub roles: Vec<String>,

    /// User's permissions
    pub permissions: Vec<String>,

    /// Organization ID
    pub organization_id: String,

    /// Resource being accessed
    pub resource: String,

    /// Action being performed (read, write, delete, etc.)
    pub action: String,

    /// IP address of the request
    pub ip_address: Option<IpAddr>,

    /// Device information
    pub device: Option<DeviceContext>,

    /// Network information
    pub network: Option<NetworkContext>,

    /// Time of request
    pub timestamp: DateTime<Utc>,

    /// Session information
    pub session: Option<SessionContext>,

    /// Risk score (0-100, higher = more risky)
    pub risk_score: Option<u8>,

    /// Additional metadata
    #[serde(flatten)]
    pub metadata: serde_json::Value,
}

impl RequestContext {
    /// Create a new request context
    pub fn new(
        user_id: &str,
        organization_id: &str,
        resource: &str,
        action: &str,
    ) -> Self {
        Self {
            user_id: user_id.to_string(),
            roles: Vec::new(),
            permissions: Vec::new(),
            organization_id: organization_id.to_string(),
            resource: resource.to_string(),
            action: action.to_string(),
            ip_address: None,
            device: None,
            network: None,
            timestamp: Utc::now(),
            session: None,
            risk_score: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Add roles to context
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    /// Add permissions to context
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }

    /// Add IP address
    pub fn with_ip_address(mut self, ip: IpAddr) -> Self {
        self.ip_address = Some(ip);
        self
    }

    /// Add device context
    pub fn with_device(mut self, device: DeviceContext) -> Self {
        self.device = Some(device);
        self
    }

    /// Add network context
    pub fn with_network(mut self, network: NetworkContext) -> Self {
        self.network = Some(network);
        self
    }

    /// Add session context
    pub fn with_session(mut self, session: SessionContext) -> Self {
        self.session = Some(session);
        self
    }

    /// Add risk score
    pub fn with_risk_score(mut self, score: u8) -> Self {
        self.risk_score = Some(score.min(100));
        self
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }

    /// Check if user has any of the given roles
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }

    /// Check if user has all of the given permissions
    pub fn has_all_permissions(&self, permissions: &[&str]) -> bool {
        permissions.iter().all(|perm| self.has_permission(perm))
    }
}

/// Device context for trust evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceContext {
    /// Device ID or fingerprint
    pub device_id: String,

    /// Device type (mobile, desktop, tablet, etc.)
    pub device_type: DeviceType,

    /// Operating system
    pub os: String,

    /// OS version
    pub os_version: String,

    /// Device is managed/enrolled
    pub is_managed: bool,

    /// Device is compliant with security policies
    pub is_compliant: bool,

    /// Device is encrypted
    pub is_encrypted: bool,

    /// Device has screen lock enabled
    pub has_screen_lock: bool,

    /// Last security update timestamp
    pub last_security_update: Option<DateTime<Utc>>,

    /// User agent string
    pub user_agent: Option<String>,
}

impl DeviceContext {
    /// Calculate device trust score (0-100)
    pub fn trust_score(&self) -> u8 {
        let mut score = 50u8; // Base score

        if self.is_managed {
            score = score.saturating_add(15);
        }
        if self.is_compliant {
            score = score.saturating_add(20);
        }
        if self.is_encrypted {
            score = score.saturating_add(10);
        }
        if self.has_screen_lock {
            score = score.saturating_add(5);
        }

        score
    }

    /// Check if device meets minimum security requirements
    pub fn meets_minimum_requirements(&self) -> bool {
        self.is_encrypted && self.has_screen_lock
    }
}

/// Device type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    /// Desktop computer
    Desktop,
    /// Laptop computer
    Laptop,
    /// Mobile phone
    Mobile,
    /// Tablet device
    Tablet,
    /// Server
    Server,
    /// IoT device
    IoT,
    /// Unknown device type
    Unknown,
}

/// Network context for trust evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkContext {
    /// Network type
    pub network_type: NetworkType,

    /// Network is corporate/trusted
    pub is_corporate_network: bool,

    /// Using VPN
    pub is_vpn: bool,

    /// Geographic location (country code)
    pub country: Option<String>,

    /// Geographic location (city)
    pub city: Option<String>,

    /// Connection is over TLS
    pub is_tls: bool,

    /// TLS version
    pub tls_version: Option<String>,
}

impl NetworkContext {
    /// Calculate network trust score (0-100)
    pub fn trust_score(&self) -> u8 {
        let mut score = 30u8; // Base score

        if self.is_corporate_network {
            score = score.saturating_add(30);
        }
        if self.is_vpn {
            score = score.saturating_add(20);
        }
        if self.is_tls {
            score = score.saturating_add(15);
        }

        match self.network_type {
            NetworkType::Corporate => score.saturating_add(5),
            NetworkType::Home => score,
            NetworkType::Public => score.saturating_sub(20),
            NetworkType::Mobile => score.saturating_sub(10),
            NetworkType::Unknown => score.saturating_sub(15),
        }

        score
    }
}

/// Network type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkType {
    /// Corporate network
    Corporate,
    /// Home network
    Home,
    /// Public WiFi
    Public,
    /// Mobile/cellular
    Mobile,
    /// Unknown network
    Unknown,
}

/// Session context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Session ID
    pub session_id: String,

    /// Session created at
    pub created_at: DateTime<Utc>,

    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,

    /// Number of requests in this session
    pub request_count: u32,

    /// MFA verified in this session
    pub mfa_verified: bool,

    /// MFA verification timestamp
    pub mfa_verified_at: Option<DateTime<Utc>>,
}

impl SessionContext {
    /// Check if session is fresh (created recently)
    pub fn is_fresh(&self, max_age_minutes: i64) -> bool {
        let age = Utc::now()
            .signed_duration_since(self.created_at)
            .num_minutes();
        age <= max_age_minutes
    }

    /// Check if MFA is fresh
    pub fn is_mfa_fresh(&self, max_age_minutes: i64) -> bool {
        if let Some(mfa_time) = self.mfa_verified_at {
            let age = Utc::now().signed_duration_since(mfa_time).num_minutes();
            age <= max_age_minutes
        } else {
            false
        }
    }
}

/// Complete security context including computed risk
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Request context
    pub request: RequestContext,

    /// Computed overall risk score
    pub overall_risk: u8,

    /// Trust level
    pub trust_level: TrustLevel,

    /// Requires additional verification
    pub requires_mfa: bool,

    /// Anomalies detected
    pub anomalies: Vec<String>,
}

impl SecurityContext {
    /// Evaluate security context from request
    pub fn evaluate(request: RequestContext) -> SecurityResult<Self> {
        let mut overall_risk = request.risk_score.unwrap_or(50);
        let mut anomalies = Vec::new();

        // Evaluate device trust
        if let Some(device) = &request.device {
            let device_score = device.trust_score();
            if device_score < 50 {
                anomalies.push("Low device trust score".to_string());
                overall_risk = overall_risk.saturating_add(10);
            }
            if !device.meets_minimum_requirements() {
                anomalies.push("Device does not meet minimum requirements".to_string());
                overall_risk = overall_risk.saturating_add(15);
            }
        }

        // Evaluate network trust
        if let Some(network) = &request.network {
            let network_score = network.trust_score();
            if network_score < 50 {
                anomalies.push("Low network trust score".to_string());
                overall_risk = overall_risk.saturating_add(10);
            }
            if network.network_type == NetworkType::Public {
                anomalies.push("Access from public network".to_string());
                overall_risk = overall_risk.saturating_add(20);
            }
        }

        // Evaluate session
        let mut requires_mfa = false;
        if let Some(session) = &request.session {
            if !session.is_fresh(60) {
                // Session older than 1 hour
                anomalies.push("Old session".to_string());
                overall_risk = overall_risk.saturating_add(5);
            }
            if !session.mfa_verified {
                requires_mfa = true;
                overall_risk = overall_risk.saturating_add(10);
            }
        } else {
            requires_mfa = true;
            overall_risk = overall_risk.saturating_add(15);
        }

        // Determine trust level
        let trust_level = match overall_risk {
            0..=30 => TrustLevel::High,
            31..=60 => TrustLevel::Medium,
            61..=80 => TrustLevel::Low,
            _ => TrustLevel::None,
        };

        Ok(Self {
            request,
            overall_risk: overall_risk.min(100),
            trust_level,
            requires_mfa,
            anomalies,
        })
    }

    /// Check if access should be allowed based on trust level
    pub fn is_trusted(&self) -> bool {
        matches!(self.trust_level, TrustLevel::High | TrustLevel::Medium)
    }

    /// Require high trust level
    pub fn requires_high_trust(&self) -> bool {
        !matches!(self.trust_level, TrustLevel::High)
    }
}

/// Trust level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TrustLevel {
    /// No trust - deny access
    None,
    /// Low trust - require additional verification
    Low,
    /// Medium trust - allow with monitoring
    Medium,
    /// High trust - allow access
    High,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_request_context_builder() {
        let context = RequestContext::new("user123", "org456", "/api/data", "read")
            .with_roles(vec!["admin".to_string()])
            .with_permissions(vec!["data:read".to_string()])
            .with_ip_address(IpAddr::from_str("192.168.1.1").unwrap());

        assert_eq!(context.user_id, "user123");
        assert!(context.has_role("admin"));
        assert!(context.has_permission("data:read"));
        assert!(context.ip_address.is_some());
    }

    #[test]
    fn test_device_trust_score() {
        let device = DeviceContext {
            device_id: "dev123".to_string(),
            device_type: DeviceType::Desktop,
            os: "Windows 11".to_string(),
            os_version: "22H2".to_string(),
            is_managed: true,
            is_compliant: true,
            is_encrypted: true,
            has_screen_lock: true,
            last_security_update: Some(Utc::now()),
            user_agent: None,
        };

        let score = device.trust_score();
        assert!(score >= 90); // Should have high trust
    }

    #[test]
    fn test_network_trust_score() {
        let network = NetworkContext {
            network_type: NetworkType::Corporate,
            is_corporate_network: true,
            is_vpn: false,
            country: Some("US".to_string()),
            city: Some("New York".to_string()),
            is_tls: true,
            tls_version: Some("1.3".to_string()),
        };

        let score = network.trust_score();
        assert!(score >= 80); // Corporate network should be trusted
    }

    #[test]
    fn test_security_context_evaluation() {
        let request = RequestContext::new("user123", "org456", "/api/sensitive", "write");

        let security = SecurityContext::evaluate(request).unwrap();
        assert!(security.overall_risk <= 100);
    }

    #[test]
    fn test_public_network_increases_risk() {
        let mut request = RequestContext::new("user123", "org456", "/api/data", "read");

        let public_network = NetworkContext {
            network_type: NetworkType::Public,
            is_corporate_network: false,
            is_vpn: false,
            country: Some("US".to_string()),
            city: None,
            is_tls: true,
            tls_version: Some("1.3".to_string()),
        };

        request = request.with_network(public_network);

        let security = SecurityContext::evaluate(request).unwrap();
        assert!(security.overall_risk > 50);
        assert!(!security.anomalies.is_empty());
    }

    #[test]
    fn test_session_freshness() {
        let session = SessionContext {
            session_id: "sess123".to_string(),
            created_at: Utc::now(),
            last_activity: Utc::now(),
            request_count: 5,
            mfa_verified: true,
            mfa_verified_at: Some(Utc::now()),
        };

        assert!(session.is_fresh(60));
        assert!(session.is_mfa_fresh(10));
    }
}
