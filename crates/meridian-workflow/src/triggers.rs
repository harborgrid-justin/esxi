//! Event-triggered workflow execution.

use crate::dag::WorkflowId;
use crate::error::{WorkflowError, WorkflowResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// A unique identifier for a trigger.
pub type TriggerId = Uuid;

/// Event that can trigger a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event ID.
    pub id: Uuid,

    /// Event type/name.
    pub event_type: String,

    /// Event payload.
    pub payload: Value,

    /// Event timestamp.
    pub timestamp: DateTime<Utc>,

    /// Event source.
    pub source: Option<String>,

    /// Event metadata.
    pub metadata: HashMap<String, String>,
}

impl Event {
    /// Creates a new event.
    pub fn new(event_type: impl Into<String>, payload: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: event_type.into(),
            payload,
            timestamp: Utc::now(),
            source: None,
            metadata: HashMap::new(),
        }
    }

    /// Sets the event source.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Adds metadata.
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Trigger condition for evaluating whether to execute a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerCondition {
    /// Always trigger (no condition).
    Always,

    /// Trigger if event type matches.
    EventType { event_type: String },

    /// Trigger if event source matches.
    EventSource { source: String },

    /// Trigger if JSONPath expression matches.
    JsonPath { path: String, expected: Value },

    /// Trigger if custom expression evaluates to true.
    Expression { expression: String },

    /// Trigger if all conditions are met (AND).
    All { conditions: Vec<TriggerCondition> },

    /// Trigger if any condition is met (OR).
    Any { conditions: Vec<TriggerCondition> },

    /// Trigger if condition is not met (NOT).
    Not { condition: Box<TriggerCondition> },
}

impl TriggerCondition {
    /// Evaluates the condition against an event.
    pub fn evaluate(&self, event: &Event) -> bool {
        match self {
            TriggerCondition::Always => true,

            TriggerCondition::EventType { event_type } => &event.event_type == event_type,

            TriggerCondition::EventSource { source } => {
                event.source.as_ref().map(|s| s == source).unwrap_or(false)
            }

            TriggerCondition::JsonPath { path, expected } => {
                // Simplified JSONPath evaluation (in production, use a proper library)
                self.evaluate_json_path(&event.payload, path, expected)
            }

            TriggerCondition::Expression { expression } => {
                // Simplified expression evaluation (in production, use a proper evaluator)
                self.evaluate_expression(event, expression)
            }

            TriggerCondition::All { conditions } => {
                conditions.iter().all(|c| c.evaluate(event))
            }

            TriggerCondition::Any { conditions } => {
                conditions.iter().any(|c| c.evaluate(event))
            }

            TriggerCondition::Not { condition } => !condition.evaluate(event),
        }
    }

    /// Simplified JSONPath evaluation.
    fn evaluate_json_path(&self, payload: &Value, path: &str, expected: &Value) -> bool {
        // Simple dot-notation path evaluation
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = payload;

        for part in parts {
            match current {
                Value::Object(map) => {
                    if let Some(value) = map.get(part) {
                        current = value;
                    } else {
                        return false;
                    }
                }
                _ => return false,
            }
        }

        current == expected
    }

    /// Simplified expression evaluation.
    fn evaluate_expression(&self, event: &Event, _expression: &str) -> bool {
        // In production, use a proper expression evaluator
        // For now, just check if the expression contains the event type
        true
    }
}

/// Workflow trigger configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    /// Trigger ID.
    pub id: TriggerId,

    /// Trigger name.
    pub name: String,

    /// Workflow to trigger.
    pub workflow_id: WorkflowId,

    /// Trigger condition.
    pub condition: TriggerCondition,

    /// Whether the trigger is enabled.
    pub enabled: bool,

    /// Input mapping (extracts data from event to pass to workflow).
    pub input_mapping: Option<HashMap<String, String>>,

    /// Maximum number of concurrent executions.
    pub max_concurrent: Option<usize>,

    /// Cooldown period in seconds (minimum time between triggers).
    pub cooldown_secs: Option<u64>,

    /// Last trigger time.
    pub last_triggered_at: Option<DateTime<Utc>>,

    /// Trigger metadata.
    pub metadata: HashMap<String, String>,
}

impl Trigger {
    /// Creates a new trigger.
    pub fn new(
        name: impl Into<String>,
        workflow_id: WorkflowId,
        condition: TriggerCondition,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            workflow_id,
            condition,
            enabled: true,
            input_mapping: None,
            max_concurrent: None,
            cooldown_secs: None,
            last_triggered_at: None,
            metadata: HashMap::new(),
        }
    }

    /// Sets the input mapping.
    pub fn with_input_mapping(mut self, mapping: HashMap<String, String>) -> Self {
        self.input_mapping = Some(mapping);
        self
    }

    /// Sets the maximum concurrent executions.
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = Some(max);
        self
    }

    /// Sets the cooldown period.
    pub fn with_cooldown(mut self, cooldown_secs: u64) -> Self {
        self.cooldown_secs = Some(cooldown_secs);
        self
    }

    /// Checks if the trigger should fire for the given event.
    pub fn should_trigger(&self, event: &Event) -> bool {
        if !self.enabled {
            return false;
        }

        // Check cooldown
        if let (Some(cooldown), Some(last_triggered)) = (self.cooldown_secs, self.last_triggered_at) {
            let elapsed = Utc::now()
                .signed_duration_since(last_triggered)
                .num_seconds() as u64;
            if elapsed < cooldown {
                debug!(
                    "Trigger {} in cooldown ({}s remaining)",
                    self.id,
                    cooldown - elapsed
                );
                return false;
            }
        }

        // Evaluate condition
        self.condition.evaluate(event)
    }

    /// Extracts workflow inputs from the event using the input mapping.
    pub fn extract_inputs(&self, event: &Event) -> Value {
        if let Some(ref mapping) = self.input_mapping {
            let mut inputs = serde_json::Map::new();

            for (key, path) in mapping {
                // Simple extraction (in production, use proper JSONPath)
                if let Some(value) = self.extract_value(&event.payload, path) {
                    inputs.insert(key.clone(), value);
                }
            }

            Value::Object(inputs)
        } else {
            event.payload.clone()
        }
    }

    /// Extracts a value from the payload using a simple path.
    fn extract_value(&self, payload: &Value, path: &str) -> Option<Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = payload;

        for part in parts {
            match current {
                Value::Object(map) => {
                    current = map.get(part)?;
                }
                _ => return None,
            }
        }

        Some(current.clone())
    }
}

/// Callback for workflow execution when triggered.
#[async_trait]
pub trait TriggerCallback: Send + Sync {
    /// Called when a trigger fires.
    async fn on_trigger(
        &self,
        workflow_id: WorkflowId,
        inputs: Value,
        event: &Event,
    ) -> WorkflowResult<String>;
}

/// Trigger manager for handling event-driven workflows.
pub struct TriggerManager {
    /// Registered triggers.
    triggers: Arc<RwLock<HashMap<TriggerId, Trigger>>>,

    /// Triggers by workflow ID.
    workflow_triggers: Arc<RwLock<HashMap<WorkflowId, Vec<TriggerId>>>>,

    /// Execution callback.
    callback: Arc<dyn TriggerCallback>,

    /// Event history (limited size).
    event_history: Arc<RwLock<Vec<Event>>>,

    /// Maximum event history size.
    max_event_history: usize,
}

impl TriggerManager {
    /// Creates a new trigger manager.
    pub fn new(callback: Arc<dyn TriggerCallback>) -> Self {
        Self {
            triggers: Arc::new(RwLock::new(HashMap::new())),
            workflow_triggers: Arc::new(RwLock::new(HashMap::new())),
            callback,
            event_history: Arc::new(RwLock::new(Vec::new())),
            max_event_history: 1000,
        }
    }

    /// Registers a trigger.
    pub async fn register_trigger(&self, trigger: Trigger) -> WorkflowResult<TriggerId> {
        let trigger_id = trigger.id;
        let workflow_id = trigger.workflow_id;

        info!(
            "Registering trigger {} for workflow {}",
            trigger.name, workflow_id
        );

        let mut triggers = self.triggers.write().await;
        triggers.insert(trigger_id, trigger);

        let mut workflow_triggers = self.workflow_triggers.write().await;
        workflow_triggers
            .entry(workflow_id)
            .or_insert_with(Vec::new)
            .push(trigger_id);

        Ok(trigger_id)
    }

    /// Removes a trigger.
    pub async fn remove_trigger(&self, trigger_id: TriggerId) -> WorkflowResult<()> {
        let mut triggers = self.triggers.write().await;
        let trigger = triggers.remove(&trigger_id).ok_or_else(|| {
            WorkflowError::NotFound(format!("Trigger {} not found", trigger_id))
        })?;

        let mut workflow_triggers = self.workflow_triggers.write().await;
        if let Some(trigger_list) = workflow_triggers.get_mut(&trigger.workflow_id) {
            trigger_list.retain(|&id| id != trigger_id);
        }

        info!("Removed trigger {}", trigger_id);
        Ok(())
    }

    /// Enables a trigger.
    pub async fn enable_trigger(&self, trigger_id: TriggerId) -> WorkflowResult<()> {
        let mut triggers = self.triggers.write().await;
        let trigger = triggers.get_mut(&trigger_id).ok_or_else(|| {
            WorkflowError::NotFound(format!("Trigger {} not found", trigger_id))
        })?;

        trigger.enabled = true;
        info!("Enabled trigger {}", trigger_id);
        Ok(())
    }

    /// Disables a trigger.
    pub async fn disable_trigger(&self, trigger_id: TriggerId) -> WorkflowResult<()> {
        let mut triggers = self.triggers.write().await;
        let trigger = triggers.get_mut(&trigger_id).ok_or_else(|| {
            WorkflowError::NotFound(format!("Trigger {} not found", trigger_id))
        })?;

        trigger.enabled = false;
        info!("Disabled trigger {}", trigger_id);
        Ok(())
    }

    /// Processes an event and triggers matching workflows.
    pub async fn process_event(&self, event: Event) -> WorkflowResult<Vec<String>> {
        info!("Processing event: {} ({})", event.event_type, event.id);

        // Store event in history
        {
            let mut history = self.event_history.write().await;
            history.push(event.clone());

            // Limit history size
            if history.len() > self.max_event_history {
                history.remove(0);
            }
        }

        let mut execution_ids = Vec::new();
        let triggers = self.triggers.read().await;

        // Find matching triggers
        let mut matching_triggers = Vec::new();
        for trigger in triggers.values() {
            if trigger.should_trigger(&event) {
                matching_triggers.push(trigger.clone());
            }
        }

        drop(triggers);

        // Execute workflows for matching triggers
        for mut trigger in matching_triggers {
            info!(
                "Trigger {} matched event {}",
                trigger.name, event.event_type
            );

            // Extract inputs
            let inputs = trigger.extract_inputs(&event);

            // Execute workflow
            match self
                .callback
                .on_trigger(trigger.workflow_id, inputs, &event)
                .await
            {
                Ok(execution_id) => {
                    execution_ids.push(execution_id);

                    // Update last triggered time
                    trigger.last_triggered_at = Some(Utc::now());
                    let mut triggers = self.triggers.write().await;
                    triggers.insert(trigger.id, trigger);
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to trigger workflow {} for trigger {}: {}",
                        trigger.workflow_id,
                        trigger.id,
                        e
                    );
                }
            }
        }

        Ok(execution_ids)
    }

    /// Gets a trigger by ID.
    pub async fn get_trigger(&self, trigger_id: TriggerId) -> Option<Trigger> {
        let triggers = self.triggers.read().await;
        triggers.get(&trigger_id).cloned()
    }

    /// Lists all triggers.
    pub async fn list_triggers(&self) -> Vec<Trigger> {
        let triggers = self.triggers.read().await;
        triggers.values().cloned().collect()
    }

    /// Lists triggers for a specific workflow.
    pub async fn list_workflow_triggers(&self, workflow_id: WorkflowId) -> Vec<Trigger> {
        let workflow_triggers = self.workflow_triggers.read().await;
        let trigger_ids = workflow_triggers.get(&workflow_id);

        if let Some(ids) = trigger_ids {
            let triggers = self.triggers.read().await;
            ids.iter()
                .filter_map(|id| triggers.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Gets the event history.
    pub async fn get_event_history(&self, limit: usize) -> Vec<Event> {
        let history = self.event_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Clears the event history.
    pub async fn clear_event_history(&self) {
        let mut history = self.event_history.write().await;
        history.clear();
        info!("Cleared event history");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCallback;

    #[async_trait]
    impl TriggerCallback for MockCallback {
        async fn on_trigger(
            &self,
            _workflow_id: WorkflowId,
            _inputs: Value,
            _event: &Event,
        ) -> WorkflowResult<String> {
            Ok("mock-execution-id".to_string())
        }
    }

    #[test]
    fn test_trigger_condition_event_type() {
        let condition = TriggerCondition::EventType {
            event_type: "user.created".to_string(),
        };

        let event1 = Event::new("user.created", serde_json::json!({}));
        assert!(condition.evaluate(&event1));

        let event2 = Event::new("user.updated", serde_json::json!({}));
        assert!(!condition.evaluate(&event2));
    }

    #[test]
    fn test_trigger_condition_all() {
        let condition = TriggerCondition::All {
            conditions: vec![
                TriggerCondition::EventType {
                    event_type: "user.created".to_string(),
                },
                TriggerCondition::EventSource {
                    source: "api".to_string(),
                },
            ],
        };

        let event1 = Event::new("user.created", serde_json::json!({}))
            .with_source("api");
        assert!(condition.evaluate(&event1));

        let event2 = Event::new("user.created", serde_json::json!({}))
            .with_source("webhook");
        assert!(!condition.evaluate(&event2));
    }

    #[tokio::test]
    async fn test_trigger_manager() {
        let callback = Arc::new(MockCallback);
        let manager = TriggerManager::new(callback);

        let workflow_id = Uuid::new_v4();
        let trigger = Trigger::new(
            "test_trigger",
            workflow_id,
            TriggerCondition::EventType {
                event_type: "test.event".to_string(),
            },
        );

        manager.register_trigger(trigger).await.unwrap();

        let event = Event::new("test.event", serde_json::json!({"key": "value"}));
        let executions = manager.process_event(event).await.unwrap();

        assert_eq!(executions.len(), 1);
    }

    #[test]
    fn test_input_extraction() {
        let mut mapping = HashMap::new();
        mapping.insert("user_id".to_string(), "data.user.id".to_string());
        mapping.insert("email".to_string(), "data.user.email".to_string());

        let trigger = Trigger::new(
            "test",
            Uuid::new_v4(),
            TriggerCondition::Always,
        )
        .with_input_mapping(mapping);

        let event = Event::new(
            "user.created",
            serde_json::json!({
                "data": {
                    "user": {
                        "id": 123,
                        "email": "test@example.com"
                    }
                }
            }),
        );

        let inputs = trigger.extract_inputs(&event);
        assert_eq!(inputs["user_id"], serde_json::json!(123));
        assert_eq!(inputs["email"], serde_json::json!("test@example.com"));
    }
}
