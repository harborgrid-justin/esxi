//! Saga pattern for managing distributed transactions and long-running processes.

use crate::causation::{CausationContext, CorrelationId};
use crate::error::{EventError, Result};
use crate::event::StoredEvent;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

/// Saga identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SagaId(Uuid);

impl SagaId {
    /// Generate a new saga ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SagaId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SagaId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for SagaId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Saga state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SagaState {
    /// Saga is running
    Running,
    /// Saga completed successfully
    Completed,
    /// Saga failed
    Failed,
    /// Saga is being compensated (rolled back)
    Compensating,
    /// Saga compensation completed
    Compensated,
}

/// Saga metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaMetadata {
    /// Saga ID
    pub saga_id: SagaId,
    /// Correlation ID for tracking
    pub correlation_id: CorrelationId,
    /// Saga state
    pub state: SagaState,
    /// Start time
    pub started_at: DateTime<Utc>,
    /// Last updated time
    pub updated_at: DateTime<Utc>,
    /// Completion time
    pub completed_at: Option<DateTime<Utc>>,
    /// Timeout duration
    pub timeout: Option<Duration>,
}

impl SagaMetadata {
    /// Create new saga metadata.
    pub fn new(correlation_id: CorrelationId) -> Self {
        let now = Utc::now();
        Self {
            saga_id: SagaId::new(),
            correlation_id,
            state: SagaState::Running,
            started_at: now,
            updated_at: now,
            completed_at: None,
            timeout: None,
        }
    }

    /// Set the timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Check if the saga has timed out.
    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.timeout {
            Utc::now() - self.started_at > timeout
        } else {
            false
        }
    }

    /// Update the state.
    pub fn update_state(&mut self, state: SagaState) {
        self.state = state;
        self.updated_at = Utc::now();

        if matches!(
            state,
            SagaState::Completed | SagaState::Failed | SagaState::Compensated
        ) {
            self.completed_at = Some(Utc::now());
        }
    }
}

/// Saga step result.
#[derive(Debug, Clone)]
pub enum StepResult {
    /// Step succeeded
    Success,
    /// Step failed
    Failure(String),
    /// Step needs retry
    Retry,
}

/// Trait for saga steps.
#[async_trait]
pub trait SagaStep: Send + Sync {
    /// Execute the step.
    async fn execute(&self, context: &CausationContext) -> Result<StepResult>;

    /// Compensate (rollback) the step.
    async fn compensate(&self, context: &CausationContext) -> Result<()>;

    /// Get the step name.
    fn name(&self) -> &str;
}

/// Saga definition.
pub struct Saga {
    metadata: SagaMetadata,
    steps: Vec<Box<dyn SagaStep>>,
    current_step: usize,
    completed_steps: Vec<String>,
}

impl Saga {
    /// Create a new saga.
    pub fn new(correlation_id: CorrelationId) -> Self {
        Self {
            metadata: SagaMetadata::new(correlation_id),
            steps: Vec::new(),
            current_step: 0,
            completed_steps: Vec::new(),
        }
    }

    /// Add a step to the saga.
    pub fn add_step(&mut self, step: Box<dyn SagaStep>) {
        self.steps.push(step);
    }

    /// Get the saga ID.
    pub fn id(&self) -> SagaId {
        self.metadata.saga_id
    }

    /// Get the metadata.
    pub fn metadata(&self) -> &SagaMetadata {
        &self.metadata
    }

    /// Get the current state.
    pub fn state(&self) -> SagaState {
        self.metadata.state
    }

    /// Set timeout.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.metadata.timeout = Some(timeout);
    }

    /// Execute the next step.
    pub async fn execute_next_step(&mut self, context: &CausationContext) -> Result<bool> {
        if self.current_step >= self.steps.len() {
            self.metadata.update_state(SagaState::Completed);
            return Ok(false);
        }

        let step = &self.steps[self.current_step];
        let result = step.execute(context).await?;

        match result {
            StepResult::Success => {
                self.completed_steps.push(step.name().to_string());
                self.current_step += 1;
                Ok(true)
            }
            StepResult::Failure(reason) => {
                self.metadata.update_state(SagaState::Failed);
                Err(EventError::Saga(format!("Step failed: {}", reason)))
            }
            StepResult::Retry => Ok(true),
        }
    }

    /// Compensate all completed steps.
    pub async fn compensate(&mut self, context: &CausationContext) -> Result<()> {
        self.metadata.update_state(SagaState::Compensating);

        // Compensate in reverse order
        for i in (0..self.current_step).rev() {
            let step = &self.steps[i];
            step.compensate(context).await?;
        }

        self.metadata.update_state(SagaState::Compensated);
        Ok(())
    }

    /// Check if saga is complete.
    pub fn is_complete(&self) -> bool {
        matches!(
            self.metadata.state,
            SagaState::Completed | SagaState::Failed | SagaState::Compensated
        )
    }

    /// Check if saga has timed out.
    pub fn is_timed_out(&self) -> bool {
        self.metadata.is_timed_out()
    }
}

/// Saga manager for coordinating multiple sagas.
pub struct SagaManager {
    sagas: Arc<RwLock<HashMap<SagaId, Saga>>>,
}

impl Default for SagaManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SagaManager {
    /// Create a new saga manager.
    pub fn new() -> Self {
        Self {
            sagas: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a new saga.
    pub fn start_saga(&self, saga: Saga) -> SagaId {
        let id = saga.id();
        self.sagas.write().insert(id, saga);
        id
    }

    /// Get a saga by ID.
    pub fn get_saga(&self, id: &SagaId) -> Option<Saga> {
        self.sagas.read().get(id).cloned()
    }

    /// Execute the next step of a saga.
    pub async fn execute_next(
        &self,
        id: &SagaId,
        context: &CausationContext,
    ) -> Result<bool> {
        let mut sagas = self.sagas.write();
        let saga = sagas.get_mut(id).ok_or_else(|| {
            EventError::Saga(format!("Saga not found: {}", id))
        })?;

        saga.execute_next_step(context).await
    }

    /// Compensate a saga.
    pub async fn compensate(&self, id: &SagaId, context: &CausationContext) -> Result<()> {
        let mut sagas = self.sagas.write();
        let saga = sagas.get_mut(id).ok_or_else(|| {
            EventError::Saga(format!("Saga not found: {}", id))
        })?;

        saga.compensate(context).await
    }

    /// Remove a completed saga.
    pub fn remove_saga(&self, id: &SagaId) -> Option<Saga> {
        self.sagas.write().remove(id)
    }

    /// Get all active sagas.
    pub fn active_sagas(&self) -> Vec<SagaId> {
        self.sagas
            .read()
            .iter()
            .filter(|(_, saga)| !saga.is_complete())
            .map(|(id, _)| *id)
            .collect()
    }

    /// Check for timed out sagas.
    pub async fn check_timeouts(&self, context: &CausationContext) -> Result<Vec<SagaId>> {
        let mut timed_out = Vec::new();

        for id in self.active_sagas() {
            if let Some(saga) = self.sagas.read().get(&id) {
                if saga.is_timed_out() {
                    timed_out.push(id);
                }
            }
        }

        // Compensate timed out sagas
        for id in &timed_out {
            self.compensate(id, context).await?;
        }

        Ok(timed_out)
    }
}

impl Clone for Saga {
    fn clone(&self) -> Self {
        Self {
            metadata: self.metadata.clone(),
            steps: Vec::new(), // Steps can't be cloned
            current_step: self.current_step,
            completed_steps: self.completed_steps.clone(),
        }
    }
}

/// Process manager for handling events and coordinating sagas.
#[async_trait]
pub trait ProcessManager: Send + Sync {
    /// Handle an event.
    async fn handle_event(&self, event: &StoredEvent) -> Result<()>;

    /// Get the process manager name.
    fn name(&self) -> &str;
}

/// Simple saga step implementation.
pub struct SimpleSagaStep<F, C>
where
    F: Fn(&CausationContext) -> Result<StepResult> + Send + Sync,
    C: Fn(&CausationContext) -> Result<()> + Send + Sync,
{
    name: String,
    execute_fn: F,
    compensate_fn: C,
}

impl<F, C> SimpleSagaStep<F, C>
where
    F: Fn(&CausationContext) -> Result<StepResult> + Send + Sync,
    C: Fn(&CausationContext) -> Result<()> + Send + Sync,
{
    /// Create a new simple saga step.
    pub fn new(name: impl Into<String>, execute_fn: F, compensate_fn: C) -> Self {
        Self {
            name: name.into(),
            execute_fn,
            compensate_fn,
        }
    }
}

#[async_trait]
impl<F, C> SagaStep for SimpleSagaStep<F, C>
where
    F: Fn(&CausationContext) -> Result<StepResult> + Send + Sync,
    C: Fn(&CausationContext) -> Result<()> + Send + Sync,
{
    async fn execute(&self, context: &CausationContext) -> Result<StepResult> {
        (self.execute_fn)(context)
    }

    async fn compensate(&self, context: &CausationContext) -> Result<()> {
        (self.compensate_fn)(context)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saga_id_generation() {
        let id1 = SagaId::new();
        let id2 = SagaId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_saga_metadata() {
        let correlation_id = CorrelationId::new();
        let mut metadata = SagaMetadata::new(correlation_id);

        assert_eq!(metadata.state, SagaState::Running);
        assert!(metadata.completed_at.is_none());

        metadata.update_state(SagaState::Completed);
        assert_eq!(metadata.state, SagaState::Completed);
        assert!(metadata.completed_at.is_some());
    }

    #[test]
    fn test_saga_timeout() {
        let correlation_id = CorrelationId::new();
        let metadata = SagaMetadata::new(correlation_id)
            .with_timeout(Duration::milliseconds(-1000));

        assert!(metadata.is_timed_out());
    }

    #[tokio::test]
    async fn test_saga_execution() {
        let mut saga = Saga::new(CorrelationId::new());

        let step = SimpleSagaStep::new(
            "test-step",
            |_| Ok(StepResult::Success),
            |_| Ok(()),
        );

        saga.add_step(Box::new(step));

        let context = CausationContext::new();
        let has_more = saga.execute_next_step(&context).await.unwrap();

        assert!(!has_more);
        assert_eq!(saga.state(), SagaState::Completed);
    }

    #[tokio::test]
    async fn test_saga_compensation() {
        let mut saga = Saga::new(CorrelationId::new());

        let step1 = SimpleSagaStep::new(
            "step1",
            |_| Ok(StepResult::Success),
            |_| Ok(()),
        );

        let step2 = SimpleSagaStep::new(
            "step2",
            |_| Ok(StepResult::Failure("Test failure".to_string())),
            |_| Ok(()),
        );

        saga.add_step(Box::new(step1));
        saga.add_step(Box::new(step2));

        let context = CausationContext::new();

        // Execute first step (success)
        saga.execute_next_step(&context).await.unwrap();

        // Execute second step (failure)
        let result = saga.execute_next_step(&context).await;
        assert!(result.is_err());

        // Compensate
        saga.compensate(&context).await.unwrap();
        assert_eq!(saga.state(), SagaState::Compensated);
    }

    #[tokio::test]
    async fn test_saga_manager() {
        let manager = SagaManager::new();
        let saga = Saga::new(CorrelationId::new());
        let id = saga.id();

        manager.start_saga(saga);

        assert!(manager.get_saga(&id).is_some());
        assert_eq!(manager.active_sagas().len(), 1);

        let removed = manager.remove_saga(&id);
        assert!(removed.is_some());
        assert_eq!(manager.active_sagas().len(), 0);
    }
}
