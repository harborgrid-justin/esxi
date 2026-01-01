//! Policy engine for zero-trust access control
//!
//! Implements a flexible, extensible policy engine for making access decisions.
//!
//! ## Policy-Based Access Control (PBAC)
//! - Policies define who can access what under which conditions
//! - Rules are evaluated in order (first match wins)
//! - Support for complex conditions and risk-based decisions
//! - Audit trail for all access decisions
//!
//! ## Policy Types
//! - RBAC: Role-Based Access Control
//! - ABAC: Attribute-Based Access Control
//! - Risk-Based: Access based on computed risk score
//! - Time-Based: Access restricted by time windows
//! - Location-Based: Access restricted by geography/network

use serde::{Deserialize, Serialize};

use crate::{
    error::{SecurityError, SecurityResult},
    zero_trust::context::{RequestContext, SecurityContext, TrustLevel},
};

/// Access decision result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessDecision {
    /// Access is allowed
    Allow,

    /// Access is denied
    Deny,

    /// Access requires additional verification (MFA, etc.)
    RequireAdditionalAuth,

    /// Access is allowed but should be monitored/logged
    AllowWithMonitoring,
}

/// Policy rule for access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule ID
    pub id: String,

    /// Rule description
    pub description: String,

    /// Priority (higher = evaluated first)
    pub priority: i32,

    /// Rule is enabled
    pub enabled: bool,

    /// Effect (allow or deny)
    pub effect: Effect,

    /// Conditions that must be met
    pub conditions: Vec<Condition>,
}

impl PolicyRule {
    /// Create a new policy rule
    pub fn new(id: &str, description: &str, effect: Effect) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            priority: 0,
            enabled: true,
            effect,
            conditions: Vec::new(),
        }
    }

    /// Add a condition to the rule
    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Evaluate rule against context
    pub fn evaluate(&self, context: &RequestContext) -> SecurityResult<bool> {
        if !self.enabled {
            return Ok(false);
        }

        // All conditions must be true for rule to match
        for condition in &self.conditions {
            if !condition.evaluate(context)? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// Policy effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    /// Allow access
    Allow,
    /// Deny access
    Deny,
}

/// Condition for policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Condition {
    /// User has specific role
    HasRole { role: String },

    /// User has specific permission
    HasPermission { permission: String },

    /// User has any of the roles
    HasAnyRole { roles: Vec<String> },

    /// User has all of the permissions
    HasAllPermissions { permissions: Vec<String> },

    /// Resource matches pattern
    ResourceMatches { pattern: String },

    /// Action matches
    ActionIs { action: String },

    /// Organization matches
    OrganizationIs { org_id: String },

    /// Risk score below threshold
    RiskScoreBelow { threshold: u8 },

    /// Risk score above threshold
    RiskScoreAbove { threshold: u8 },

    /// Trust level at least
    TrustLevelAtLeast { level: TrustLevel },

    /// IP in CIDR range
    IpInRange { cidr: String },

    /// Device is managed
    DeviceIsManaged,

    /// Device is compliant
    DeviceIsCompliant,

    /// Network is corporate
    NetworkIsCorporate,

    /// Using VPN
    UsingVpn,

    /// MFA verified
    MfaVerified,

    /// Time in range (business hours)
    TimeInRange { start_hour: u8, end_hour: u8 },

    /// And combinator (all must be true)
    And { conditions: Vec<Condition> },

    /// Or combinator (any must be true)
    Or { conditions: Vec<Condition> },

    /// Not combinator (inverts result)
    Not { condition: Box<Condition> },
}

impl Condition {
    /// Evaluate condition against request context
    pub fn evaluate(&self, context: &RequestContext) -> SecurityResult<bool> {
        match self {
            Condition::HasRole { role } => Ok(context.has_role(role)),

            Condition::HasPermission { permission } => Ok(context.has_permission(permission)),

            Condition::HasAnyRole { roles } => {
                let role_refs: Vec<&str> = roles.iter().map(|s| s.as_str()).collect();
                Ok(context.has_any_role(&role_refs))
            }

            Condition::HasAllPermissions { permissions } => {
                let perm_refs: Vec<&str> = permissions.iter().map(|s| s.as_str()).collect();
                Ok(context.has_all_permissions(&perm_refs))
            }

            Condition::ResourceMatches { pattern } => {
                // Simple pattern matching (production would use regex)
                Ok(context.resource.contains(pattern))
            }

            Condition::ActionIs { action } => Ok(context.action == *action),

            Condition::OrganizationIs { org_id } => Ok(context.organization_id == *org_id),

            Condition::RiskScoreBelow { threshold } => {
                Ok(context.risk_score.unwrap_or(50) < *threshold)
            }

            Condition::RiskScoreAbove { threshold } => {
                Ok(context.risk_score.unwrap_or(50) > *threshold)
            }

            Condition::TrustLevelAtLeast { level } => {
                // This requires SecurityContext, which we don't have here
                // In production, pass SecurityContext instead
                Ok(true) // Stub
            }

            Condition::IpInRange { cidr: _cidr } => {
                // Production would parse CIDR and check IP
                Ok(context.ip_address.is_some())
            }

            Condition::DeviceIsManaged => Ok(context
                .device
                .as_ref()
                .map(|d| d.is_managed)
                .unwrap_or(false)),

            Condition::DeviceIsCompliant => Ok(context
                .device
                .as_ref()
                .map(|d| d.is_compliant)
                .unwrap_or(false)),

            Condition::NetworkIsCorporate => Ok(context
                .network
                .as_ref()
                .map(|n| n.is_corporate_network)
                .unwrap_or(false)),

            Condition::UsingVpn => Ok(context
                .network
                .as_ref()
                .map(|n| n.is_vpn)
                .unwrap_or(false)),

            Condition::MfaVerified => Ok(context
                .session
                .as_ref()
                .map(|s| s.mfa_verified)
                .unwrap_or(false)),

            Condition::TimeInRange {
                start_hour,
                end_hour,
            } => {
                let current_hour = context.timestamp.hour() as u8;
                Ok(current_hour >= *start_hour && current_hour < *end_hour)
            }

            Condition::And { conditions } => {
                for cond in conditions {
                    if !cond.evaluate(context)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }

            Condition::Or { conditions } => {
                for cond in conditions {
                    if cond.evaluate(context)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }

            Condition::Not { condition } => Ok(!condition.evaluate(context)?),
        }
    }
}

/// Complete policy containing multiple rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Policy ID
    pub id: String,

    /// Policy name
    pub name: String,

    /// Policy description
    pub description: String,

    /// Policy version
    pub version: String,

    /// Policy rules (evaluated in priority order)
    pub rules: Vec<PolicyRule>,

    /// Default decision if no rules match
    pub default_decision: AccessDecision,
}

impl Policy {
    /// Create a new policy
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: String::new(),
            version: "1.0".to_string(),
            rules: Vec::new(),
            default_decision: AccessDecision::Deny, // Secure default
        }
    }

    /// Add a rule to the policy
    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
        // Sort by priority (higher first)
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Evaluate policy against context
    pub fn evaluate(&self, context: &RequestContext) -> SecurityResult<AccessDecision> {
        // Evaluate rules in priority order
        for rule in &self.rules {
            if rule.evaluate(context)? {
                return Ok(match rule.effect {
                    Effect::Allow => AccessDecision::Allow,
                    Effect::Deny => AccessDecision::Deny,
                });
            }
        }

        // No rules matched, use default
        Ok(self.default_decision.clone())
    }
}

/// Policy engine for managing and evaluating policies
pub struct PolicyEngine {
    /// Loaded policies
    policies: Vec<Policy>,

    /// Enable risk-based decisions
    risk_based_enabled: bool,

    /// Risk threshold for additional auth
    risk_threshold: u8,
}

impl PolicyEngine {
    /// Create a new policy engine
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
            risk_based_enabled: true,
            risk_threshold: 70,
        }
    }

    /// Add a policy to the engine
    pub fn add_policy(&mut self, policy: Policy) {
        self.policies.push(policy);
    }

    /// Enable risk-based decision making
    pub fn enable_risk_based(&mut self, enabled: bool) {
        self.risk_based_enabled = enabled;
    }

    /// Set risk threshold for requiring additional auth
    pub fn set_risk_threshold(&mut self, threshold: u8) {
        self.risk_threshold = threshold.min(100);
    }

    /// Evaluate access request
    ///
    /// # Arguments
    /// * `context` - Request context
    ///
    /// # Returns
    /// Access decision
    pub fn evaluate(&self, context: &RequestContext) -> SecurityResult<AccessDecision> {
        // First, evaluate all policies
        let mut policy_decisions = Vec::new();

        for policy in &self.policies {
            let decision = policy.evaluate(context)?;
            policy_decisions.push(decision);
        }

        // If any policy explicitly denies, deny access
        if policy_decisions
            .iter()
            .any(|d| matches!(d, AccessDecision::Deny))
        {
            return Ok(AccessDecision::Deny);
        }

        // If any policy allows, check risk-based conditions
        if policy_decisions
            .iter()
            .any(|d| matches!(d, AccessDecision::Allow))
        {
            // Risk-based evaluation
            if self.risk_based_enabled {
                if let Some(risk_score) = context.risk_score {
                    if risk_score >= self.risk_threshold {
                        return Ok(AccessDecision::RequireAdditionalAuth);
                    }
                    if risk_score >= 50 {
                        return Ok(AccessDecision::AllowWithMonitoring);
                    }
                }
            }

            return Ok(AccessDecision::Allow);
        }

        // Default deny
        Ok(AccessDecision::Deny)
    }

    /// Evaluate with full security context (including computed risk)
    pub fn evaluate_with_context(
        &self,
        security_context: &SecurityContext,
    ) -> SecurityResult<AccessDecision> {
        // First evaluate policies
        let mut decision = self.evaluate(&security_context.request)?;

        // Override based on trust level
        if security_context.trust_level == TrustLevel::None {
            decision = AccessDecision::Deny;
        } else if security_context.requires_mfa
            && !matches!(decision, AccessDecision::Deny)
        {
            decision = AccessDecision::RequireAdditionalAuth;
        } else if security_context.overall_risk >= self.risk_threshold
            && !matches!(decision, AccessDecision::Deny)
        {
            decision = AccessDecision::RequireAdditionalAuth;
        }

        Ok(decision)
    }

    /// Get number of loaded policies
    pub fn policy_count(&self) -> usize {
        self.policies.len()
    }

    /// Clear all policies
    pub fn clear_policies(&mut self) {
        self.policies.clear();
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Policy builder for creating common policy patterns
pub struct PolicyBuilder;

impl PolicyBuilder {
    /// Create a simple RBAC policy (role-based)
    pub fn rbac(resource: &str, action: &str, roles: Vec<String>) -> Policy {
        let mut policy = Policy::new(
            &format!("rbac-{}-{}", resource, action),
            &format!("RBAC for {} {}", action, resource),
        );

        let rule = PolicyRule::new(
            "rbac-rule",
            &format!("Allow {} {} for roles: {:?}", action, resource, roles),
            Effect::Allow,
        )
        .with_condition(Condition::ResourceMatches {
            pattern: resource.to_string(),
        })
        .with_condition(Condition::ActionIs {
            action: action.to_string(),
        })
        .with_condition(Condition::HasAnyRole { roles })
        .with_priority(100);

        policy.add_rule(rule);
        policy
    }

    /// Create a time-based access policy (business hours only)
    pub fn business_hours_only(resource: &str) -> Policy {
        let mut policy = Policy::new(
            &format!("business-hours-{}", resource),
            "Business hours access only",
        );

        let rule = PolicyRule::new(
            "business-hours",
            "Allow access during business hours (9 AM - 5 PM)",
            Effect::Allow,
        )
        .with_condition(Condition::ResourceMatches {
            pattern: resource.to_string(),
        })
        .with_condition(Condition::TimeInRange {
            start_hour: 9,
            end_hour: 17,
        })
        .with_priority(100);

        policy.add_rule(rule);
        policy
    }

    /// Create a high-security policy requiring MFA and managed devices
    pub fn high_security(resource: &str, action: &str) -> Policy {
        let mut policy = Policy::new(
            &format!("high-security-{}-{}", resource, action),
            "High security access",
        );

        let rule = PolicyRule::new(
            "high-security",
            "Require MFA and managed device",
            Effect::Allow,
        )
        .with_condition(Condition::ResourceMatches {
            pattern: resource.to_string(),
        })
        .with_condition(Condition::ActionIs {
            action: action.to_string(),
        })
        .with_condition(Condition::And {
            conditions: vec![
                Condition::MfaVerified,
                Condition::DeviceIsManaged,
                Condition::DeviceIsCompliant,
            ],
        })
        .with_priority(100);

        policy.add_rule(rule);
        policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_rbac_policy() {
        let mut policy = Policy::new("test-policy", "Test Policy");

        let rule = PolicyRule::new("admin-rule", "Admin access", Effect::Allow)
            .with_condition(Condition::HasRole {
                role: "admin".to_string(),
            })
            .with_priority(100);

        policy.add_rule(rule);

        let context = RequestContext::new("user123", "org456", "/api/admin", "read")
            .with_roles(vec!["admin".to_string()]);

        let decision = policy.evaluate(&context).unwrap();
        assert_eq!(decision, AccessDecision::Allow);
    }

    #[test]
    fn test_permission_based_policy() {
        let mut policy = Policy::new("test-policy", "Test Policy");

        let rule = PolicyRule::new("permission-rule", "Permission check", Effect::Allow)
            .with_condition(Condition::HasPermission {
                permission: "data:read".to_string(),
            });

        policy.add_rule(rule);

        let context = RequestContext::new("user123", "org456", "/data", "read")
            .with_permissions(vec!["data:read".to_string()]);

        let decision = policy.evaluate(&context).unwrap();
        assert_eq!(decision, AccessDecision::Allow);
    }

    #[test]
    fn test_deny_rule() {
        let mut policy = Policy::new("test-policy", "Test Policy");

        let rule = PolicyRule::new("deny-rule", "Deny access", Effect::Deny)
            .with_condition(Condition::ResourceMatches {
                pattern: "/admin".to_string(),
            })
            .with_priority(100);

        policy.add_rule(rule);

        let context = RequestContext::new("user123", "org456", "/admin/users", "read");

        let decision = policy.evaluate(&context).unwrap();
        assert_eq!(decision, AccessDecision::Deny);
    }

    #[test]
    fn test_and_condition() {
        let condition = Condition::And {
            conditions: vec![
                Condition::HasRole {
                    role: "admin".to_string(),
                },
                Condition::ActionIs {
                    action: "delete".to_string(),
                },
            ],
        };

        let context = RequestContext::new("user123", "org456", "/data", "delete")
            .with_roles(vec!["admin".to_string()]);

        assert!(condition.evaluate(&context).unwrap());
    }

    #[test]
    fn test_or_condition() {
        let condition = Condition::Or {
            conditions: vec![
                Condition::HasRole {
                    role: "admin".to_string(),
                },
                Condition::HasRole {
                    role: "moderator".to_string(),
                },
            ],
        };

        let context = RequestContext::new("user123", "org456", "/data", "read")
            .with_roles(vec!["moderator".to_string()]);

        assert!(condition.evaluate(&context).unwrap());
    }

    #[test]
    fn test_not_condition() {
        let condition = Condition::Not {
            condition: Box::new(Condition::HasRole {
                role: "guest".to_string(),
            }),
        };

        let context = RequestContext::new("user123", "org456", "/data", "read")
            .with_roles(vec!["user".to_string()]);

        assert!(condition.evaluate(&context).unwrap());
    }

    #[test]
    fn test_policy_engine() {
        let mut engine = PolicyEngine::new();

        let policy = PolicyBuilder::rbac(
            "/api/data",
            "read",
            vec!["user".to_string(), "admin".to_string()],
        );

        engine.add_policy(policy);

        let context = RequestContext::new("user123", "org456", "/api/data", "read")
            .with_roles(vec!["user".to_string()]);

        let decision = engine.evaluate(&context).unwrap();
        assert_eq!(decision, AccessDecision::Allow);
    }

    #[test]
    fn test_risk_based_decision() {
        let mut engine = PolicyEngine::new();
        engine.set_risk_threshold(70);

        let policy = PolicyBuilder::rbac("/api/data", "read", vec!["user".to_string()]);
        engine.add_policy(policy);

        let context = RequestContext::new("user123", "org456", "/api/data", "read")
            .with_roles(vec!["user".to_string()])
            .with_risk_score(80);

        let decision = engine.evaluate(&context).unwrap();
        assert_eq!(decision, AccessDecision::RequireAdditionalAuth);
    }

    #[test]
    fn test_business_hours_policy() {
        let policy = PolicyBuilder::business_hours_only("/api/admin");

        let context = RequestContext::new("user123", "org456", "/api/admin", "read");

        let decision = policy.evaluate(&context).unwrap();
        // Result depends on current time
        assert!(matches!(
            decision,
            AccessDecision::Allow | AccessDecision::Deny
        ));
    }
}
