//! Operational Transformation (OT) for collaborative editing

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::sync::VectorClock;

/// Operation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OpType {
    /// Insert operation
    Insert,
    /// Delete operation
    Delete,
    /// Update/Replace operation
    Update,
    /// Move operation
    Move,
}

/// Generic operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Operation type
    pub op_type: OpType,

    /// Position/index
    pub position: usize,

    /// Length (for delete/update)
    pub length: Option<usize>,

    /// Content (for insert/update)
    pub content: Option<Vec<u8>>,

    /// Target position (for move)
    pub target: Option<usize>,

    /// Vector clock
    pub clock: VectorClock,

    /// Operation ID
    pub id: String,

    /// User ID
    pub user_id: String,
}

impl Operation {
    /// Create insert operation
    pub fn insert(position: usize, content: Vec<u8>, clock: VectorClock, user_id: String) -> Self {
        Self {
            op_type: OpType::Insert,
            position,
            length: Some(content.len()),
            content: Some(content),
            target: None,
            clock,
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
        }
    }

    /// Create delete operation
    pub fn delete(position: usize, length: usize, clock: VectorClock, user_id: String) -> Self {
        Self {
            op_type: OpType::Delete,
            position,
            length: Some(length),
            content: None,
            target: None,
            clock,
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
        }
    }

    /// Create update operation
    pub fn update(
        position: usize,
        length: usize,
        content: Vec<u8>,
        clock: VectorClock,
        user_id: String,
    ) -> Self {
        Self {
            op_type: OpType::Update,
            position,
            length: Some(length),
            content: Some(content),
            target: None,
            clock,
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
        }
    }

    /// Create move operation
    pub fn move_op(
        position: usize,
        length: usize,
        target: usize,
        clock: VectorClock,
        user_id: String,
    ) -> Self {
        Self {
            op_type: OpType::Move,
            position,
            length: Some(length),
            content: None,
            target: Some(target),
            clock,
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
        }
    }
}

/// Operational Transformation engine
pub struct OperationalTransform {
    /// Node ID
    node_id: String,

    /// Operation history
    history: Vec<Operation>,
}

impl OperationalTransform {
    /// Create new OT engine
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            history: Vec::new(),
        }
    }

    /// Transform operation against another operation
    pub fn transform(&self, op1: &Operation, op2: &Operation) -> Result<Operation> {
        let mut transformed = op1.clone();

        match (&op1.op_type, &op2.op_type) {
            // Insert vs Insert
            (OpType::Insert, OpType::Insert) => {
                if op2.position <= op1.position {
                    // Shift position forward
                    transformed.position += op2.length.unwrap_or(0);
                }
            }

            // Insert vs Delete
            (OpType::Insert, OpType::Delete) => {
                if op2.position < op1.position {
                    // Shift position backward
                    let delete_len = op2.length.unwrap_or(0);
                    if op1.position >= op2.position + delete_len {
                        transformed.position -= delete_len;
                    } else {
                        transformed.position = op2.position;
                    }
                }
            }

            // Delete vs Insert
            (OpType::Delete, OpType::Insert) => {
                if op2.position <= op1.position {
                    // Shift position forward
                    transformed.position += op2.length.unwrap_or(0);
                }
            }

            // Delete vs Delete
            (OpType::Delete, OpType::Delete) => {
                if op2.position < op1.position {
                    let delete_len = op2.length.unwrap_or(0);
                    if op1.position >= op2.position + delete_len {
                        transformed.position -= delete_len;
                    } else {
                        transformed.position = op2.position;
                        let overlap = (op2.position + delete_len).saturating_sub(op1.position);
                        if let Some(len) = transformed.length {
                            transformed.length = Some(len.saturating_sub(overlap));
                        }
                    }
                } else if op2.position < op1.position + op1.length.unwrap_or(0) {
                    // Operations overlap
                    if let Some(len) = transformed.length {
                        let overlap = op2.length.unwrap_or(0).min(len);
                        transformed.length = Some(len.saturating_sub(overlap));
                    }
                }
            }

            // Update operations
            (OpType::Update, _) | (_, OpType::Update) => {
                // Simplify: treat update as delete + insert
                // More sophisticated logic can be added here
            }

            // Move operations
            (OpType::Move, _) | (_, OpType::Move) => {
                // Complex transformation for move operations
                // Implementation depends on specific requirements
            }
        }

        Ok(transformed)
    }

    /// Transform operation against history
    pub fn transform_against_history(&self, op: &Operation) -> Result<Operation> {
        let mut transformed = op.clone();

        for historical_op in &self.history {
            if historical_op.clock.happens_before(&op.clock) {
                transformed = self.transform(&transformed, historical_op)?;
            }
        }

        Ok(transformed)
    }

    /// Add operation to history
    pub fn add_to_history(&mut self, op: Operation) {
        self.history.push(op);
    }

    /// Get history
    pub fn history(&self) -> &[Operation] {
        &self.history
    }

    /// Clear old history (garbage collection)
    pub fn prune_history(&mut self, before_clock: &VectorClock) {
        self.history
            .retain(|op| !op.clock.happens_before(before_clock));
    }
}

/// Apply operation to a byte buffer
pub fn apply_operation(buffer: &mut Vec<u8>, op: &Operation) -> Result<()> {
    match op.op_type {
        OpType::Insert => {
            if op.position > buffer.len() {
                return Err(Error::Sync(format!(
                    "Insert position {} out of bounds (len: {})",
                    op.position,
                    buffer.len()
                )));
            }

            if let Some(content) = &op.content {
                buffer.splice(op.position..op.position, content.iter().cloned());
            }
        }

        OpType::Delete => {
            let length = op.length.unwrap_or(0);
            let end = (op.position + length).min(buffer.len());

            if op.position >= buffer.len() {
                return Err(Error::Sync(format!(
                    "Delete position {} out of bounds (len: {})",
                    op.position,
                    buffer.len()
                )));
            }

            buffer.drain(op.position..end);
        }

        OpType::Update => {
            let length = op.length.unwrap_or(0);
            let end = (op.position + length).min(buffer.len());

            if op.position >= buffer.len() {
                return Err(Error::Sync(format!(
                    "Update position {} out of bounds (len: {})",
                    op.position,
                    buffer.len()
                )));
            }

            if let Some(content) = &op.content {
                buffer.splice(op.position..end, content.iter().cloned());
            }
        }

        OpType::Move => {
            let length = op.length.unwrap_or(0);
            let target = op.target.unwrap_or(0);

            if op.position + length > buffer.len() || target > buffer.len() {
                return Err(Error::Sync("Move operation out of bounds".to_string()));
            }

            let moved_data: Vec<u8> = buffer.drain(op.position..op.position + length).collect();
            buffer.splice(target..target, moved_data.iter().cloned());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_operation() {
        let clock = VectorClock::new("node1".to_string());
        let op = Operation::insert(0, vec![1, 2, 3], clock, "user1".to_string());

        assert_eq!(op.op_type, OpType::Insert);
        assert_eq!(op.position, 0);
        assert_eq!(op.content, Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_apply_insert() {
        let mut buffer = vec![1, 2, 3];
        let clock = VectorClock::new("node1".to_string());
        let op = Operation::insert(1, vec![4, 5], clock, "user1".to_string());

        apply_operation(&mut buffer, &op).unwrap();
        assert_eq!(buffer, vec![1, 4, 5, 2, 3]);
    }

    #[test]
    fn test_apply_delete() {
        let mut buffer = vec![1, 2, 3, 4, 5];
        let clock = VectorClock::new("node1".to_string());
        let op = Operation::delete(1, 2, clock, "user1".to_string());

        apply_operation(&mut buffer, &op).unwrap();
        assert_eq!(buffer, vec![1, 4, 5]);
    }

    #[test]
    fn test_transform_insert_insert() {
        let ot = OperationalTransform::new("node1".to_string());
        let clock = VectorClock::new("node1".to_string());

        let op1 = Operation::insert(5, vec![1], clock.clone(), "user1".to_string());
        let op2 = Operation::insert(3, vec![2], clock, "user2".to_string());

        let transformed = ot.transform(&op1, &op2).unwrap();
        assert_eq!(transformed.position, 6); // Shifted forward
    }
}
