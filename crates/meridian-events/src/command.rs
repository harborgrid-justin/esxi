//! Command handling and validation for CQRS.

use crate::aggregate::{Aggregate, AggregateId, AggregateRepository, AggregateRoot};
use crate::causation::CausationContext;
use crate::error::{EventError, Result};
use async_trait::async_trait;
use std::fmt;

/// Trait for commands in the CQRS pattern.
pub trait Command: Send + Sync + fmt::Debug {
    /// Get the command name.
    fn command_name(&self) -> &str;

    /// Validate the command.
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// Result of command execution.
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Success status
    pub success: bool,
    /// Aggregate ID that was affected
    pub aggregate_id: AggregateId,
    /// New version of the aggregate
    pub version: u64,
    /// Optional message
    pub message: Option<String>,
}

impl CommandResult {
    /// Create a successful result.
    pub fn success(aggregate_id: AggregateId, version: u64) -> Self {
        Self {
            success: true,
            aggregate_id,
            version,
            message: None,
        }
    }

    /// Create a successful result with a message.
    pub fn success_with_message(
        aggregate_id: AggregateId,
        version: u64,
        message: impl Into<String>,
    ) -> Self {
        Self {
            success: true,
            aggregate_id,
            version,
            message: Some(message.into()),
        }
    }

    /// Create a failure result.
    pub fn failure(aggregate_id: AggregateId, message: impl Into<String>) -> Self {
        Self {
            success: false,
            aggregate_id,
            version: 0,
            message: Some(message.into()),
        }
    }
}

/// Trait for command handlers.
#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    /// Handle the command.
    async fn handle(&self, command: C, context: CausationContext) -> Result<CommandResult>;
}

/// Base command handler for aggregates.
pub struct AggregateCommandHandler<A, R>
where
    A: AggregateRoot,
    R: AggregateRepository<A>,
{
    repository: R,
    _phantom: std::marker::PhantomData<A>,
}

impl<A, R> AggregateCommandHandler<A, R>
where
    A: AggregateRoot,
    R: AggregateRepository<A>,
{
    /// Create a new aggregate command handler.
    pub fn new(repository: R) -> Self {
        Self {
            repository,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the repository.
    pub fn repository(&self) -> &R {
        &self.repository
    }
}

/// Trait for handling commands on aggregates.
#[async_trait]
pub trait AggregateCommandExecutor<C: Command>: Send + Sync {
    /// Execute the command on the aggregate.
    fn execute(&self, aggregate: &mut Aggregate<Self>, command: &C) -> Result<()>
    where
        Self: AggregateRoot;
}

/// Command dispatcher for routing commands to handlers.
#[derive(Default)]
pub struct CommandDispatcher {
    // In a real implementation, this would store command handlers
    // For now, we'll keep it simple
}

impl CommandDispatcher {
    /// Create a new command dispatcher.
    pub fn new() -> Self {
        Self::default()
    }

    /// Dispatch a command (generic method).
    pub async fn dispatch<C, H>(
        &self,
        command: C,
        handler: &H,
        context: CausationContext,
    ) -> Result<CommandResult>
    where
        C: Command,
        H: CommandHandler<C>,
    {
        // Validate command
        command.validate()?;

        // Handle the command
        handler.handle(command, context).await
    }
}

/// Validation trait for commands.
pub trait Validator: Send + Sync {
    /// Validate a value.
    fn validate(&self, value: &dyn std::any::Any) -> Result<()>;
}

/// Validation rules for commands.
pub struct ValidationRules {
    validators: Vec<Box<dyn Validator>>,
}

impl ValidationRules {
    /// Create new validation rules.
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    /// Add a validator.
    pub fn add_validator(&mut self, validator: Box<dyn Validator>) {
        self.validators.push(validator);
    }

    /// Validate a value against all rules.
    pub fn validate(&self, value: &dyn std::any::Any) -> Result<()> {
        for validator in &self.validators {
            validator.validate(value)?;
        }
        Ok(())
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self::new()
    }
}

/// String length validator.
pub struct StringLengthValidator {
    min: Option<usize>,
    max: Option<usize>,
}

impl StringLengthValidator {
    /// Create a new string length validator.
    pub fn new(min: Option<usize>, max: Option<usize>) -> Self {
        Self { min, max }
    }
}

impl Validator for StringLengthValidator {
    fn validate(&self, value: &dyn std::any::Any) -> Result<()> {
        if let Some(s) = value.downcast_ref::<String>() {
            if let Some(min) = self.min {
                if s.len() < min {
                    return Err(EventError::CommandValidation(format!(
                        "String length {} is less than minimum {}",
                        s.len(),
                        min
                    )));
                }
            }
            if let Some(max) = self.max {
                if s.len() > max {
                    return Err(EventError::CommandValidation(format!(
                        "String length {} exceeds maximum {}",
                        s.len(),
                        max
                    )));
                }
            }
        }
        Ok(())
    }
}

/// Range validator for numeric values.
pub struct RangeValidator<T: PartialOrd> {
    min: Option<T>,
    max: Option<T>,
}

impl<T: PartialOrd> RangeValidator<T> {
    /// Create a new range validator.
    pub fn new(min: Option<T>, max: Option<T>) -> Self {
        Self { min, max }
    }
}

impl<T: PartialOrd + fmt::Display + Send + Sync + 'static> Validator for RangeValidator<T> {
    fn validate(&self, value: &dyn std::any::Any) -> Result<()> {
        if let Some(v) = value.downcast_ref::<T>() {
            if let Some(ref min) = self.min {
                if v < min {
                    return Err(EventError::CommandValidation(format!(
                        "Value {} is less than minimum {}",
                        v, min
                    )));
                }
            }
            if let Some(ref max) = self.max {
                if v > max {
                    return Err(EventError::CommandValidation(format!(
                        "Value {} exceeds maximum {}",
                        v, max
                    )));
                }
            }
        }
        Ok(())
    }
}

/// Command middleware for cross-cutting concerns.
#[async_trait]
pub trait CommandMiddleware: Send + Sync {
    /// Execute before command handling.
    async fn before(&self, command: &dyn Command, context: &CausationContext) -> Result<()>;

    /// Execute after command handling.
    async fn after(
        &self,
        command: &dyn Command,
        context: &CausationContext,
        result: &CommandResult,
    ) -> Result<()>;
}

/// Logging middleware.
pub struct LoggingMiddleware;

#[async_trait]
impl CommandMiddleware for LoggingMiddleware {
    async fn before(&self, command: &dyn Command, context: &CausationContext) -> Result<()> {
        tracing::info!(
            command = command.command_name(),
            correlation_id = %context.correlation_id,
            "Executing command"
        );
        Ok(())
    }

    async fn after(
        &self,
        command: &dyn Command,
        context: &CausationContext,
        result: &CommandResult,
    ) -> Result<()> {
        tracing::info!(
            command = command.command_name(),
            correlation_id = %context.correlation_id,
            success = result.success,
            version = result.version,
            "Command executed"
        );
        Ok(())
    }
}

/// Validation middleware.
pub struct ValidationMiddleware;

#[async_trait]
impl CommandMiddleware for ValidationMiddleware {
    async fn before(&self, command: &dyn Command, _context: &CausationContext) -> Result<()> {
        command.validate()
    }

    async fn after(
        &self,
        _command: &dyn Command,
        _context: &CausationContext,
        _result: &CommandResult,
    ) -> Result<()> {
        Ok(())
    }
}

/// Command pipeline with middleware support.
pub struct CommandPipeline {
    middlewares: Vec<Box<dyn CommandMiddleware>>,
}

impl CommandPipeline {
    /// Create a new command pipeline.
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    /// Add middleware to the pipeline.
    pub fn add_middleware(&mut self, middleware: Box<dyn CommandMiddleware>) {
        self.middlewares.push(middleware);
    }

    /// Execute a command through the pipeline.
    pub async fn execute<C, H>(
        &self,
        command: C,
        handler: &H,
        context: CausationContext,
    ) -> Result<CommandResult>
    where
        C: Command + Clone,
        H: CommandHandler<C>,
    {
        // Execute before middleware
        for middleware in &self.middlewares {
            middleware.before(&command, &context).await?;
        }

        // Execute the command
        let result = handler.handle(command.clone(), context.clone()).await?;

        // Execute after middleware
        for middleware in &self.middlewares {
            middleware.after(&command as &dyn Command, &context, &result).await?;
        }

        Ok(result)
    }
}

impl Default for CommandPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestCommand {
        value: String,
    }

    impl Command for TestCommand {
        fn command_name(&self) -> &str {
            "TestCommand"
        }

        fn validate(&self) -> Result<()> {
            if self.value.is_empty() {
                return Err(EventError::CommandValidation(
                    "Value cannot be empty".to_string(),
                ));
            }
            Ok(())
        }
    }

    #[test]
    fn test_command_validation() {
        let valid_cmd = TestCommand {
            value: "test".to_string(),
        };
        assert!(valid_cmd.validate().is_ok());

        let invalid_cmd = TestCommand {
            value: String::new(),
        };
        assert!(invalid_cmd.validate().is_err());
    }

    #[test]
    fn test_command_result() {
        let result = CommandResult::success(AggregateId::new("test-1"), 5);
        assert!(result.success);
        assert_eq!(result.version, 5);

        let failure = CommandResult::failure(AggregateId::new("test-2"), "Error occurred");
        assert!(!failure.success);
        assert!(failure.message.is_some());
    }

    #[test]
    fn test_string_length_validator() {
        let validator = StringLengthValidator::new(Some(3), Some(10));

        let valid = "hello".to_string();
        assert!(validator.validate(&valid).is_ok());

        let too_short = "hi".to_string();
        assert!(validator.validate(&too_short).is_err());

        let too_long = "this is way too long".to_string();
        assert!(validator.validate(&too_long).is_err());
    }

    #[test]
    fn test_range_validator() {
        let validator = RangeValidator::new(Some(0), Some(100));

        let valid = 50;
        assert!(validator.validate(&valid).is_ok());

        let too_small = -10;
        assert!(validator.validate(&too_small).is_err());

        let too_large = 150;
        assert!(validator.validate(&too_large).is_err());
    }
}
