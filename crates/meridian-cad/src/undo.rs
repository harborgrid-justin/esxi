//! Undo/Redo system using the Command pattern
//!
//! This module implements a comprehensive undo/redo system with command history,
//! grouping, and memory management for CAD operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

use crate::canvas::{Canvas, Entity, Layer};
use crate::primitives::Point;
use crate::{CadError, CadResult};

/// Trait for all undoable commands
pub trait Command: std::fmt::Debug {
    /// Execute the command
    fn execute(&self, canvas: &mut Canvas) -> CadResult<()>;

    /// Undo the command
    fn undo(&self, canvas: &mut Canvas) -> CadResult<()>;

    /// Redo the command (default implementation calls execute)
    fn redo(&self, canvas: &mut Canvas) -> CadResult<()> {
        self.execute(canvas)
    }

    /// Get command description for UI
    fn description(&self) -> String;

    /// Get command ID
    fn id(&self) -> Uuid;

    /// Check if command can be merged with another
    fn can_merge(&self, other: &dyn Command) -> bool {
        false
    }

    /// Merge with another command
    fn merge(&mut self, other: &dyn Command) -> CadResult<()> {
        Err(CadError::UndoError("Command merging not supported".into()))
    }
}

/// Command to add an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEntityCommand {
    pub id: Uuid,
    pub layer_id: Uuid,
    pub entity: Entity,
    pub timestamp: DateTime<Utc>,
}

impl AddEntityCommand {
    pub fn new(layer_id: Uuid, entity: Entity) -> Self {
        Self {
            id: Uuid::new_v4(),
            layer_id,
            entity,
            timestamp: Utc::now(),
        }
    }
}

impl Command for AddEntityCommand {
    fn execute(&self, canvas: &mut Canvas) -> CadResult<()> {
        let layer = canvas.get_layer_mut(self.layer_id)?;
        layer.add_entity(self.entity.clone())?;
        Ok(())
    }

    fn undo(&self, canvas: &mut Canvas) -> CadResult<()> {
        let layer = canvas.get_layer_mut(self.layer_id)?;
        layer.remove_entity(self.entity.id())?;
        Ok(())
    }

    fn description(&self) -> String {
        format!("Add {:?}", std::mem::discriminant(&self.entity))
    }

    fn id(&self) -> Uuid {
        self.id
    }
}

/// Command to delete an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteEntityCommand {
    pub id: Uuid,
    pub layer_id: Uuid,
    pub entity: Entity,
    pub timestamp: DateTime<Utc>,
}

impl DeleteEntityCommand {
    pub fn new(layer_id: Uuid, entity: Entity) -> Self {
        Self {
            id: Uuid::new_v4(),
            layer_id,
            entity,
            timestamp: Utc::now(),
        }
    }
}

impl Command for DeleteEntityCommand {
    fn execute(&self, canvas: &mut Canvas) -> CadResult<()> {
        let layer = canvas.get_layer_mut(self.layer_id)?;
        layer.remove_entity(self.entity.id())?;
        Ok(())
    }

    fn undo(&self, canvas: &mut Canvas) -> CadResult<()> {
        let layer = canvas.get_layer_mut(self.layer_id)?;
        layer.add_entity(self.entity.clone())?;
        Ok(())
    }

    fn description(&self) -> String {
        "Delete entity".to_string()
    }

    fn id(&self) -> Uuid {
        self.id
    }
}

/// Command to modify an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyEntityCommand {
    pub id: Uuid,
    pub layer_id: Uuid,
    pub old_entity: Entity,
    pub new_entity: Entity,
    pub timestamp: DateTime<Utc>,
}

impl ModifyEntityCommand {
    pub fn new(layer_id: Uuid, old_entity: Entity, new_entity: Entity) -> Self {
        Self {
            id: Uuid::new_v4(),
            layer_id,
            old_entity,
            new_entity,
            timestamp: Utc::now(),
        }
    }
}

impl Command for ModifyEntityCommand {
    fn execute(&self, canvas: &mut Canvas) -> CadResult<()> {
        let layer = canvas.get_layer_mut(self.layer_id)?;
        layer.remove_entity(self.old_entity.id())?;
        layer.add_entity(self.new_entity.clone())?;
        Ok(())
    }

    fn undo(&self, canvas: &mut Canvas) -> CadResult<()> {
        let layer = canvas.get_layer_mut(self.layer_id)?;
        layer.remove_entity(self.new_entity.id())?;
        layer.add_entity(self.old_entity.clone())?;
        Ok(())
    }

    fn description(&self) -> String {
        "Modify entity".to_string()
    }

    fn id(&self) -> Uuid {
        self.id
    }
}

/// Command to move entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveCommand {
    pub id: Uuid,
    pub entity_ids: Vec<Uuid>,
    pub delta: Point,
    pub timestamp: DateTime<Utc>,
}

impl MoveCommand {
    pub fn new(entity_ids: Vec<Uuid>, delta: Point) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_ids,
            delta,
            timestamp: Utc::now(),
        }
    }
}

impl Command for MoveCommand {
    fn execute(&self, canvas: &mut Canvas) -> CadResult<()> {
        // Implementation would transform entities by delta
        // Simplified for brevity
        Ok(())
    }

    fn undo(&self, canvas: &mut Canvas) -> CadResult<()> {
        // Move back by -delta
        Ok(())
    }

    fn description(&self) -> String {
        format!("Move {} entities", self.entity_ids.len())
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn can_merge(&self, _other: &dyn Command) -> bool {
        // Can merge consecutive move commands
        // Note: downcasting is complex with trait objects, simplified for now
        false
    }
}

/// Command to add a layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddLayerCommand {
    pub id: Uuid,
    pub layer_id: Uuid,
    pub layer_name: String,
    pub timestamp: DateTime<Utc>,
}

impl AddLayerCommand {
    pub fn new(layer_id: Uuid, layer_name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            layer_id,
            layer_name,
            timestamp: Utc::now(),
        }
    }
}

impl Command for AddLayerCommand {
    fn execute(&self, canvas: &mut Canvas) -> CadResult<()> {
        canvas.add_layer(&self.layer_name, crate::canvas::LayerStyle::default());
        Ok(())
    }

    fn undo(&self, canvas: &mut Canvas) -> CadResult<()> {
        canvas.remove_layer(self.layer_id)?;
        Ok(())
    }

    fn description(&self) -> String {
        format!("Add layer '{}'", self.layer_name)
    }

    fn id(&self) -> Uuid {
        self.id
    }
}

/// Command group for batching multiple commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandGroup {
    pub id: Uuid,
    pub name: String,
    pub commands: Vec<CommandWrapper>,
    pub timestamp: DateTime<Utc>,
}

/// Wrapper for serializable commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandWrapper {
    AddEntity(AddEntityCommand),
    DeleteEntity(DeleteEntityCommand),
    ModifyEntity(ModifyEntityCommand),
    Move(MoveCommand),
    AddLayer(AddLayerCommand),
    Group(Box<CommandGroup>),
}

impl CommandWrapper {
    fn execute(&self, canvas: &mut Canvas) -> CadResult<()> {
        match self {
            CommandWrapper::AddEntity(cmd) => cmd.execute(canvas),
            CommandWrapper::DeleteEntity(cmd) => cmd.execute(canvas),
            CommandWrapper::ModifyEntity(cmd) => cmd.execute(canvas),
            CommandWrapper::Move(cmd) => cmd.execute(canvas),
            CommandWrapper::AddLayer(cmd) => cmd.execute(canvas),
            CommandWrapper::Group(group) => group.execute(canvas),
        }
    }

    fn undo(&self, canvas: &mut Canvas) -> CadResult<()> {
        match self {
            CommandWrapper::AddEntity(cmd) => cmd.undo(canvas),
            CommandWrapper::DeleteEntity(cmd) => cmd.undo(canvas),
            CommandWrapper::ModifyEntity(cmd) => cmd.undo(canvas),
            CommandWrapper::Move(cmd) => cmd.undo(canvas),
            CommandWrapper::AddLayer(cmd) => cmd.undo(canvas),
            CommandWrapper::Group(group) => group.undo(canvas),
        }
    }

    fn redo(&self, canvas: &mut Canvas) -> CadResult<()> {
        // Redo is typically the same as execute
        self.execute(canvas)
    }

    fn description(&self) -> String {
        match self {
            CommandWrapper::AddEntity(cmd) => cmd.description(),
            CommandWrapper::DeleteEntity(cmd) => cmd.description(),
            CommandWrapper::ModifyEntity(cmd) => cmd.description(),
            CommandWrapper::Move(cmd) => cmd.description(),
            CommandWrapper::AddLayer(cmd) => cmd.description(),
            CommandWrapper::Group(group) => group.description(),
        }
    }
}

impl CommandGroup {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            commands: Vec::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn add_command(&mut self, command: CommandWrapper) {
        self.commands.push(command);
    }

    pub fn execute(&self, canvas: &mut Canvas) -> CadResult<()> {
        for command in &self.commands {
            command.execute(canvas)?;
        }
        Ok(())
    }

    pub fn undo(&self, canvas: &mut Canvas) -> CadResult<()> {
        // Undo in reverse order
        for command in self.commands.iter().rev() {
            command.undo(canvas)?;
        }
        Ok(())
    }

    pub fn description(&self) -> String {
        format!("{} ({} commands)", self.name, self.commands.len())
    }
}

/// Command history with undo/redo stacks
#[derive(Debug, Clone)]
pub struct CommandHistory {
    undo_stack: VecDeque<CommandWrapper>,
    redo_stack: VecDeque<CommandWrapper>,
    max_history: usize,
    memory_limit_bytes: usize,
}

impl CommandHistory {
    /// Create a new command history
    pub fn new() -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_history: 100,
            memory_limit_bytes: 100 * 1024 * 1024, // 100 MB
        }
    }

    /// Create with custom limits
    pub fn with_limits(max_history: usize, memory_limit_bytes: usize) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_history,
            memory_limit_bytes,
        }
    }

    /// Push a command onto the history
    pub fn push(&mut self, command: CommandWrapper) {
        // Clear redo stack when new command is added
        self.redo_stack.clear();

        // Add to undo stack
        self.undo_stack.push_back(command);

        // Trim if exceeds limits
        self.trim_history();
    }

    /// Check if can undo
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if can redo
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get undo count
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get redo count
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Peek at next undo command description
    pub fn peek_undo(&self) -> Option<String> {
        self.undo_stack.back().map(|cmd| cmd.description())
    }

    /// Peek at next redo command description
    pub fn peek_redo(&self) -> Option<String> {
        self.redo_stack.back().map(|cmd| cmd.description())
    }

    /// Get undo history (most recent first)
    pub fn undo_history(&self, limit: usize) -> Vec<String> {
        self.undo_stack
            .iter()
            .rev()
            .take(limit)
            .map(|cmd| cmd.description())
            .collect()
    }

    /// Get redo history (most recent first)
    pub fn redo_history(&self, limit: usize) -> Vec<String> {
        self.redo_stack
            .iter()
            .rev()
            .take(limit)
            .map(|cmd| cmd.description())
            .collect()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Trim history to stay within limits
    fn trim_history(&mut self) {
        // Trim by count
        while self.undo_stack.len() > self.max_history {
            self.undo_stack.pop_front();
        }

        // Could also implement memory-based trimming here
        // by estimating command sizes
    }

    /// Estimate memory usage (simplified)
    fn estimate_memory_usage(&self) -> usize {
        // Rough estimate: each command takes about 1KB
        (self.undo_stack.len() + self.redo_stack.len()) * 1024
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Undo manager combining command execution and history
#[derive(Debug, Clone)]
pub struct UndoManager {
    history: CommandHistory,
    group_stack: Vec<CommandGroup>,
}

impl UndoManager {
    /// Create a new undo manager
    pub fn new() -> Self {
        Self {
            history: CommandHistory::new(),
            group_stack: Vec::new(),
        }
    }

    /// Execute a command and add to history
    pub fn execute(&mut self, command: CommandWrapper, canvas: &mut Canvas) -> CadResult<()> {
        // Execute the command
        command.execute(canvas)?;

        // Add to current group or history
        if let Some(group) = self.group_stack.last_mut() {
            group.add_command(command);
        } else {
            self.history.push(command);
        }

        Ok(())
    }

    /// Undo last command
    pub fn undo(&mut self, canvas: &mut Canvas) -> CadResult<()> {
        if let Some(command) = self.history.undo_stack.pop_back() {
            command.undo(canvas)?;
            self.history.redo_stack.push_back(command);
            Ok(())
        } else {
            Err(CadError::UndoError("Nothing to undo".into()))
        }
    }

    /// Redo last undone command
    pub fn redo(&mut self, canvas: &mut Canvas) -> CadResult<()> {
        if let Some(command) = self.history.redo_stack.pop_back() {
            command.redo(canvas)?;
            self.history.undo_stack.push_back(command);
            Ok(())
        } else {
            Err(CadError::UndoError("Nothing to redo".into()))
        }
    }

    /// Start a command group
    pub fn begin_group(&mut self, name: impl Into<String>) {
        self.group_stack.push(CommandGroup::new(name));
    }

    /// End current command group and add to history
    pub fn end_group(&mut self) -> CadResult<()> {
        if let Some(group) = self.group_stack.pop() {
            if !group.commands.is_empty() {
                self.history.push(CommandWrapper::Group(Box::new(group)));
            }
            Ok(())
        } else {
            Err(CadError::UndoError("No group to end".into()))
        }
    }

    /// Cancel current command group without adding to history
    pub fn cancel_group(&mut self) -> CadResult<()> {
        if self.group_stack.pop().is_some() {
            Ok(())
        } else {
            Err(CadError::UndoError("No group to cancel".into()))
        }
    }

    /// Check if currently in a group
    pub fn is_in_group(&self) -> bool {
        !self.group_stack.is_empty()
    }

    /// Get history
    pub fn history(&self) -> &CommandHistory {
        &self.history
    }

    /// Get mutable history
    pub fn history_mut(&mut self) -> &mut CommandHistory {
        &mut self.history
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.history.clear();
        self.group_stack.clear();
    }

    /// Get statistics
    pub fn stats(&self) -> UndoStats {
        UndoStats {
            undo_count: self.history.undo_count(),
            redo_count: self.history.redo_count(),
            memory_estimate: self.history.estimate_memory_usage(),
            in_group: self.is_in_group(),
        }
    }
}

impl Default for UndoManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Undo system statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoStats {
    pub undo_count: usize,
    pub redo_count: usize,
    pub memory_estimate: usize,
    pub in_group: bool,
}

/// Macro for creating command groups
#[macro_export]
macro_rules! command_group {
    ($manager:expr, $canvas:expr, $name:expr, $($command:expr),+ $(,)?) => {{
        $manager.begin_group($name);
        let result: $crate::CadResult<()> = (|| {
            $(
                $manager.execute($command, $canvas)?;
            )+
            Ok(())
        })();

        match result {
            Ok(_) => $manager.end_group(),
            Err(e) => {
                $manager.cancel_group()?;
                Err(e)
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::{Canvas, LayerStyle};
    use crate::primitives::Line;

    #[test]
    fn test_command_history() {
        let mut history = CommandHistory::new();
        assert!(!history.can_undo());
        assert!(!history.can_redo());

        let command = CommandWrapper::AddLayer(AddLayerCommand::new(Uuid::new_v4(), "Test".into()));
        history.push(command);

        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_undo_manager() {
        let mut manager = UndoManager::new();
        let mut canvas = Canvas::new("Test");

        let layer_id = canvas.add_layer("Layer1", LayerStyle::default());
        let entity = Entity::Line(Line::new(Point::new(0.0, 0.0), Point::new(10.0, 10.0)));

        let command = CommandWrapper::AddEntity(AddEntityCommand::new(layer_id, entity));
        manager.execute(command, &mut canvas).unwrap();

        assert!(manager.history().can_undo());
    }

    #[test]
    fn test_command_group() {
        let mut manager = UndoManager::new();

        manager.begin_group("Test Group");
        assert!(manager.is_in_group());

        manager.end_group().unwrap();
        assert!(!manager.is_in_group());
    }
}
