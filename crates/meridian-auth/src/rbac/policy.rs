//! Policy engine for attribute-based access control

use crate::error::AuthResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Policy decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyDecision {
    /// Access allowed
    Allow,
    /// Access denied
    Deny,
}

/// Policy context for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyContext {
    /// User attributes
    pub user: HashMap<String, serde_json::Value>,
    /// Resource attributes
    pub resource: HashMap<String, serde_json::Value>,
    /// Environment attributes
    pub environment: HashMap<String, serde_json::Value>,
    /// Action being performed
    pub action: String,
}

impl PolicyContext {
    /// Create a new policy context
    pub fn new(action: impl Into<String>) -> Self {
        Self {
            user: HashMap::new(),
            resource: HashMap::new(),
            environment: HashMap::new(),
            action: action.into(),
        }
    }

    /// Set user attribute
    pub fn set_user_attr(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.user.insert(key.into(), value);
    }

    /// Set resource attribute
    pub fn set_resource_attr(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.resource.insert(key.into(), value);
    }

    /// Set environment attribute
    pub fn set_env_attr(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.environment.insert(key.into(), value);
    }

    /// Get user attribute
    pub fn get_user_attr(&self, key: &str) -> Option<&serde_json::Value> {
        self.user.get(key)
    }

    /// Get resource attribute
    pub fn get_resource_attr(&self, key: &str) -> Option<&serde_json::Value> {
        self.resource.get(key)
    }

    /// Get environment attribute
    pub fn get_env_attr(&self, key: &str) -> Option<&serde_json::Value> {
        self.environment.get(key)
    }
}

/// Policy condition type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PolicyCondition {
    /// Check if two attributes are equal
    Equals {
        left: AttributeRef,
        right: AttributeRef,
    },
    /// Check if attribute contains value
    Contains {
        attribute: AttributeRef,
        value: serde_json::Value,
    },
    /// Check if attribute is in a list
    In {
        attribute: AttributeRef,
        values: Vec<serde_json::Value>,
    },
    /// Check if attribute matches regex
    Matches {
        attribute: AttributeRef,
        pattern: String,
    },
    /// Logical AND of conditions
    And {
        conditions: Vec<PolicyCondition>,
    },
    /// Logical OR of conditions
    Or {
        conditions: Vec<PolicyCondition>,
    },
    /// Logical NOT of condition
    Not {
        condition: Box<PolicyCondition>,
    },
    /// Resource ownership check
    IsOwner,
    /// Check if user has role
    HasRole {
        role: String,
    },
    /// Time-based condition
    TimeRange {
        start_hour: u32,
        end_hour: u32,
    },
}

/// Attribute reference in policy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum AttributeRef {
    /// User attribute
    User { key: String },
    /// Resource attribute
    Resource { key: String },
    /// Environment attribute
    Environment { key: String },
    /// Literal value
    Literal { value: serde_json::Value },
}

impl AttributeRef {
    /// Resolve attribute value from context
    pub fn resolve<'a>(&'a self, context: &'a PolicyContext) -> Option<&'a serde_json::Value> {
        match self {
            AttributeRef::User { key } => context.get_user_attr(key),
            AttributeRef::Resource { key } => context.get_resource_attr(key),
            AttributeRef::Environment { key } => context.get_env_attr(key),
            AttributeRef::Literal { value } => Some(value),
        }
    }
}

/// Policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: Option<String>,
    /// Effect of the rule
    pub effect: PolicyDecision,
    /// Actions this rule applies to
    pub actions: Vec<String>,
    /// Resources this rule applies to
    pub resources: Vec<String>,
    /// Conditions that must be met
    pub conditions: Option<PolicyCondition>,
}

impl PolicyRule {
    /// Create a new policy rule
    pub fn new(name: impl Into<String>, effect: PolicyDecision) -> Self {
        Self {
            name: name.into(),
            description: None,
            effect,
            actions: vec!["*".to_string()],
            resources: vec!["*".to_string()],
            conditions: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set actions
    pub fn with_actions(mut self, actions: Vec<String>) -> Self {
        self.actions = actions;
        self
    }

    /// Set resources
    pub fn with_resources(mut self, resources: Vec<String>) -> Self {
        self.resources = resources;
        self
    }

    /// Set conditions
    pub fn with_conditions(mut self, conditions: PolicyCondition) -> Self {
        self.conditions = Some(conditions);
        self
    }

    /// Check if rule applies to the given action and resource
    pub fn applies_to(&self, action: &str, resource: &str) -> bool {
        let action_matches = self.actions.iter().any(|a| a == "*" || a == action);
        let resource_matches = self.resources.iter().any(|r| r == "*" || r == resource);
        action_matches && resource_matches
    }

    /// Evaluate rule conditions
    pub fn evaluate(&self, context: &PolicyContext) -> AuthResult<bool> {
        match &self.conditions {
            Some(condition) => evaluate_condition(condition, context),
            None => Ok(true), // No conditions means always true
        }
    }
}

/// Policy document containing multiple rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Policy name
    pub name: String,
    /// Policy version
    pub version: String,
    /// Policy rules
    pub rules: Vec<PolicyRule>,
}

impl Policy {
    /// Create a new policy
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "1.0".to_string(),
            rules: Vec::new(),
        }
    }

    /// Add a rule to the policy
    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
    }

    /// Evaluate policy against context
    pub fn evaluate(&self, context: &PolicyContext, resource: &str) -> AuthResult<PolicyDecision> {
        let mut decision = PolicyDecision::Deny; // Default deny

        // Evaluate all applicable rules
        for rule in &self.rules {
            if rule.applies_to(&context.action, resource)
                && rule.evaluate(context)? {
                    decision = rule.effect;
                    // If we find an explicit deny, stop and deny
                    if decision == PolicyDecision::Deny {
                        break;
                    }
                }
        }

        Ok(decision)
    }
}

/// Evaluate a policy condition
fn evaluate_condition(condition: &PolicyCondition, context: &PolicyContext) -> AuthResult<bool> {
    match condition {
        PolicyCondition::Equals { left, right } => {
            let left_val = left.resolve(context);
            let right_val = right.resolve(context);
            Ok(left_val == right_val)
        }

        PolicyCondition::Contains { attribute, value } => {
            if let Some(attr_val) = attribute.resolve(context) {
                if let Some(array) = attr_val.as_array() {
                    Ok(array.contains(value))
                } else if let Some(string) = attr_val.as_str() {
                    if let Some(substr) = value.as_str() {
                        Ok(string.contains(substr))
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }

        PolicyCondition::In { attribute, values } => {
            if let Some(attr_val) = attribute.resolve(context) {
                Ok(values.contains(attr_val))
            } else {
                Ok(false)
            }
        }

        PolicyCondition::Matches { attribute, pattern } => {
            if let Some(attr_val) = attribute.resolve(context) {
                if let Some(string) = attr_val.as_str() {
                    // Simple pattern matching (could be enhanced with regex)
                    Ok(string.contains(pattern))
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }

        PolicyCondition::And { conditions } => {
            for cond in conditions {
                if !evaluate_condition(cond, context)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }

        PolicyCondition::Or { conditions } => {
            for cond in conditions {
                if evaluate_condition(cond, context)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }

        PolicyCondition::Not { condition } => {
            Ok(!evaluate_condition(condition, context)?)
        }

        PolicyCondition::IsOwner => {
            // Check if user_id matches resource owner_id
            let user_id = context.get_user_attr("id");
            let owner_id = context.get_resource_attr("owner_id");
            Ok(user_id.is_some() && user_id == owner_id)
        }

        PolicyCondition::HasRole { role } => {
            if let Some(roles) = context.get_user_attr("roles") {
                if let Some(roles_array) = roles.as_array() {
                    Ok(roles_array.iter().any(|r| {
                        r.as_str().map(|s| s == role).unwrap_or(false)
                    }))
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }

        PolicyCondition::TimeRange { start_hour, end_hour } => {
            if let Some(current_hour) = context.get_env_attr("current_hour") {
                if let Some(hour) = current_hour.as_u64() {
                    let hour = hour as u32;
                    Ok(hour >= *start_hour && hour < *end_hour)
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }
    }
}

/// Create default GIS platform policies
pub fn default_policies() -> Vec<Policy> {
    let mut policies = Vec::new();

    // Resource ownership policy
    let mut ownership_policy = Policy::new("resource_ownership");
    ownership_policy.add_rule(
        PolicyRule::new("allow_owner_edit", PolicyDecision::Allow)
            .with_description("Resource owners can edit their own resources")
            .with_actions(vec!["update".to_string(), "delete".to_string()])
            .with_conditions(PolicyCondition::IsOwner),
    );
    policies.push(ownership_policy);

    // Time-based access policy
    let mut time_policy = Policy::new("business_hours");
    time_policy.add_rule(
        PolicyRule::new("business_hours_only", PolicyDecision::Allow)
            .with_description("Allow access only during business hours (9-17)")
            .with_conditions(PolicyCondition::TimeRange {
                start_hour: 9,
                end_hour: 17,
            }),
    );
    policies.push(time_policy);

    policies
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_context() {
        let mut context = PolicyContext::new("read");
        context.set_user_attr("id", serde_json::json!("user123"));
        context.set_resource_attr("owner_id", serde_json::json!("user123"));

        assert_eq!(
            context.get_user_attr("id"),
            Some(&serde_json::json!("user123"))
        );
    }

    #[test]
    fn test_equals_condition() {
        let mut context = PolicyContext::new("read");
        context.set_user_attr("id", serde_json::json!("user123"));
        context.set_resource_attr("owner_id", serde_json::json!("user123"));

        let condition = PolicyCondition::Equals {
            left: AttributeRef::User {
                key: "id".to_string(),
            },
            right: AttributeRef::Resource {
                key: "owner_id".to_string(),
            },
        };

        assert!(evaluate_condition(&condition, &context).unwrap());
    }

    #[test]
    fn test_is_owner_condition() {
        let mut context = PolicyContext::new("update");
        context.set_user_attr("id", serde_json::json!("user123"));
        context.set_resource_attr("owner_id", serde_json::json!("user123"));

        let condition = PolicyCondition::IsOwner;
        assert!(evaluate_condition(&condition, &context).unwrap());

        // Different owner
        context.set_resource_attr("owner_id", serde_json::json!("user456"));
        assert!(!evaluate_condition(&condition, &context).unwrap());
    }

    #[test]
    fn test_has_role_condition() {
        let mut context = PolicyContext::new("read");
        context.set_user_attr("roles", serde_json::json!(["editor", "viewer"]));

        let condition = PolicyCondition::HasRole {
            role: "editor".to_string(),
        };
        assert!(evaluate_condition(&condition, &context).unwrap());

        let condition = PolicyCondition::HasRole {
            role: "admin".to_string(),
        };
        assert!(!evaluate_condition(&condition, &context).unwrap());
    }

    #[test]
    fn test_and_condition() {
        let mut context = PolicyContext::new("update");
        context.set_user_attr("id", serde_json::json!("user123"));
        context.set_user_attr("roles", serde_json::json!(["editor"]));
        context.set_resource_attr("owner_id", serde_json::json!("user123"));

        let condition = PolicyCondition::And {
            conditions: vec![
                PolicyCondition::IsOwner,
                PolicyCondition::HasRole {
                    role: "editor".to_string(),
                },
            ],
        };

        assert!(evaluate_condition(&condition, &context).unwrap());
    }

    #[test]
    fn test_or_condition() {
        let mut context = PolicyContext::new("read");
        context.set_user_attr("roles", serde_json::json!(["viewer"]));

        let condition = PolicyCondition::Or {
            conditions: vec![
                PolicyCondition::HasRole {
                    role: "admin".to_string(),
                },
                PolicyCondition::HasRole {
                    role: "viewer".to_string(),
                },
            ],
        };

        assert!(evaluate_condition(&condition, &context).unwrap());
    }

    #[test]
    fn test_policy_evaluation() {
        let mut policy = Policy::new("test_policy");
        policy.add_rule(
            PolicyRule::new("allow_owner", PolicyDecision::Allow)
                .with_actions(vec!["update".to_string()])
                .with_resources(vec!["layer".to_string()])
                .with_conditions(PolicyCondition::IsOwner),
        );

        let mut context = PolicyContext::new("update");
        context.set_user_attr("id", serde_json::json!("user123"));
        context.set_resource_attr("owner_id", serde_json::json!("user123"));

        let decision = policy.evaluate(&context, "layer").unwrap();
        assert_eq!(decision, PolicyDecision::Allow);
    }

    #[test]
    fn test_policy_deny() {
        let mut policy = Policy::new("test_policy");
        policy.add_rule(
            PolicyRule::new("deny_non_owner", PolicyDecision::Deny)
                .with_actions(vec!["delete".to_string()])
                .with_conditions(PolicyCondition::Not {
                    condition: Box::new(PolicyCondition::IsOwner),
                }),
        );

        let mut context = PolicyContext::new("delete");
        context.set_user_attr("id", serde_json::json!("user123"));
        context.set_resource_attr("owner_id", serde_json::json!("user456"));

        let decision = policy.evaluate(&context, "layer").unwrap();
        assert_eq!(decision, PolicyDecision::Deny);
    }

    #[test]
    fn test_time_range_condition() {
        let mut context = PolicyContext::new("read");
        context.set_env_attr("current_hour", serde_json::json!(10));

        let condition = PolicyCondition::TimeRange {
            start_hour: 9,
            end_hour: 17,
        };
        assert!(evaluate_condition(&condition, &context).unwrap());

        context.set_env_attr("current_hour", serde_json::json!(20));
        assert!(!evaluate_condition(&condition, &context).unwrap());
    }

    #[test]
    fn test_contains_condition() {
        let mut context = PolicyContext::new("read");
        context.set_user_attr("tags", serde_json::json!(["admin", "power_user"]));

        let condition = PolicyCondition::Contains {
            attribute: AttributeRef::User {
                key: "tags".to_string(),
            },
            value: serde_json::json!("admin"),
        };
        assert!(evaluate_condition(&condition, &context).unwrap());
    }

    #[test]
    fn test_in_condition() {
        let mut context = PolicyContext::new("read");
        context.set_user_attr("department", serde_json::json!("engineering"));

        let condition = PolicyCondition::In {
            attribute: AttributeRef::User {
                key: "department".to_string(),
            },
            values: vec![
                serde_json::json!("engineering"),
                serde_json::json!("sales"),
            ],
        };
        assert!(evaluate_condition(&condition, &context).unwrap());
    }
}
