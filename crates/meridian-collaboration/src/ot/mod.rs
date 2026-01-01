//! # Operational Transform (OT) Module
//!
//! Operational Transform is a technology for supporting real-time collaborative editing.
//! This module implements OT for text documents with support for:
//!
//! - Insert, Delete, and Retain operations
//! - Operation transformation for concurrent editing
//! - Operation composition for efficiency
//! - Undo/Redo support
//!
//! ## Algorithm
//!
//! OT works by transforming operations against each other to maintain consistency:
//! 1. Two concurrent operations are transformed against each other
//! 2. The transformed operations produce the same result when applied in any order
//! 3. This ensures convergence across all clients

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use thiserror::Error;

pub mod transform;
pub mod compose;

pub use transform::{transform, transform_x};
pub use compose::compose;

/// Errors that can occur during OT operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OtError {
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Cannot compose operations: {0}")]
    ComposeError(String),

    #[error("Cannot transform operations: {0}")]
    TransformError(String),

    #[error("Operation out of bounds: expected length {expected}, got {actual}")]
    OutOfBounds { expected: usize, actual: usize },

    #[error("Invalid index: {0}")]
    InvalidIndex(usize),
}

pub type Result<T> = std::result::Result<T, OtError>;

/// A single component of an operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpComponent {
    /// Retain n characters from the current position
    Retain(usize),

    /// Insert a string at the current position
    Insert(String),

    /// Delete n characters at the current position
    Delete(usize),
}

impl OpComponent {
    /// Get the length affected by this component
    pub fn len(&self) -> usize {
        match self {
            OpComponent::Retain(n) => *n,
            OpComponent::Insert(s) => s.chars().count(),
            OpComponent::Delete(n) => *n,
        }
    }

    /// Check if this component is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if this component is a retain
    pub fn is_retain(&self) -> bool {
        matches!(self, OpComponent::Retain(_))
    }

    /// Check if this component is an insert
    pub fn is_insert(&self) -> bool {
        matches!(self, OpComponent::Insert(_))
    }

    /// Check if this component is a delete
    pub fn is_delete(&self) -> bool {
        matches!(self, OpComponent::Delete(_))
    }
}

/// An operation on a text document
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Operation {
    /// Components of this operation
    components: Vec<OpComponent>,

    /// Base length - the length of the document before applying this operation
    base_len: usize,

    /// Target length - the length of the document after applying this operation
    target_len: usize,
}

impl Operation {
    /// Create a new empty operation
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            base_len: 0,
            target_len: 0,
        }
    }

    /// Create an operation with a specific base length
    pub fn with_base_len(base_len: usize) -> Self {
        Self {
            components: Vec::new(),
            base_len,
            target_len: base_len,
        }
    }

    /// Add a retain component
    pub fn retain(&mut self, n: usize) -> &mut Self {
        if n == 0 {
            return self;
        }

        self.base_len += n;
        self.target_len += n;

        // Merge with previous retain
        if let Some(OpComponent::Retain(prev_n)) = self.components.last_mut() {
            *prev_n += n;
        } else {
            self.components.push(OpComponent::Retain(n));
        }

        self
    }

    /// Add an insert component
    pub fn insert(&mut self, s: impl Into<String>) -> &mut Self {
        let s = s.into();
        if s.is_empty() {
            return self;
        }

        let len = s.chars().count();
        self.target_len += len;

        // Merge with previous insert
        if let Some(OpComponent::Insert(prev_s)) = self.components.last_mut() {
            prev_s.push_str(&s);
        } else {
            self.components.push(OpComponent::Insert(s));
        }

        self
    }

    /// Add a delete component
    pub fn delete(&mut self, n: usize) -> &mut Self {
        if n == 0 {
            return self;
        }

        self.base_len += n;

        // Merge with previous delete
        if let Some(OpComponent::Delete(prev_n)) = self.components.last_mut() {
            *prev_n += n;
        } else {
            self.components.push(OpComponent::Delete(n));
        }

        self
    }

    /// Get the base length
    pub fn base_len(&self) -> usize {
        self.base_len
    }

    /// Get the target length
    pub fn target_len(&self) -> usize {
        self.target_len
    }

    /// Get the components
    pub fn components(&self) -> &[OpComponent] {
        &self.components
    }

    /// Check if this operation is empty (no-op)
    pub fn is_noop(&self) -> bool {
        self.components.is_empty() ||
        (self.components.len() == 1 && matches!(self.components[0], OpComponent::Retain(_)))
    }

    /// Apply this operation to a string
    pub fn apply(&self, s: &str) -> Result<String> {
        let chars: Vec<char> = s.chars().collect();

        if chars.len() != self.base_len {
            return Err(OtError::OutOfBounds {
                expected: self.base_len,
                actual: chars.len(),
            });
        }

        let mut result = String::new();
        let mut index = 0;

        for component in &self.components {
            match component {
                OpComponent::Retain(n) => {
                    if index + n > chars.len() {
                        return Err(OtError::InvalidOperation(
                            "Retain exceeds string length".to_string(),
                        ));
                    }
                    result.extend(&chars[index..index + n]);
                    index += n;
                }
                OpComponent::Insert(s) => {
                    result.push_str(s);
                }
                OpComponent::Delete(n) => {
                    if index + n > chars.len() {
                        return Err(OtError::InvalidOperation(
                            "Delete exceeds string length".to_string(),
                        ));
                    }
                    index += n;
                }
            }
        }

        // Ensure we've consumed the entire input
        if index != chars.len() {
            return Err(OtError::InvalidOperation(
                "Operation did not consume entire string".to_string(),
            ));
        }

        Ok(result)
    }

    /// Invert this operation (for undo)
    pub fn invert(&self, s: &str) -> Result<Operation> {
        let chars: Vec<char> = s.chars().collect();

        if chars.len() != self.base_len {
            return Err(OtError::OutOfBounds {
                expected: self.base_len,
                actual: chars.len(),
            });
        }

        let mut inverted = Operation::new();
        let mut index = 0;

        for component in &self.components {
            match component {
                OpComponent::Retain(n) => {
                    inverted.retain(*n);
                    index += n;
                }
                OpComponent::Insert(s) => {
                    inverted.delete(s.chars().count());
                }
                OpComponent::Delete(n) => {
                    let deleted: String = chars[index..index + n].iter().collect();
                    inverted.insert(deleted);
                    index += n;
                }
            }
        }

        Ok(inverted)
    }

    /// Get an iterator over components
    pub fn iter(&self) -> std::slice::Iter<'_, OpComponent> {
        self.components.iter()
    }
}

impl Default for Operation {
    fn default() -> Self {
        Self::new()
    }
}

impl FromIterator<OpComponent> for Operation {
    fn from_iter<T: IntoIterator<Item = OpComponent>>(iter: T) -> Self {
        let mut op = Operation::new();
        for component in iter {
            match component {
                OpComponent::Retain(n) => { op.retain(n); }
                OpComponent::Insert(s) => { op.insert(s); }
                OpComponent::Delete(n) => { op.delete(n); }
            }
        }
        op
    }
}

/// Iterator for operation components
#[derive(Debug)]
pub struct ComponentIter<'a> {
    components: &'a [OpComponent],
    index: usize,
    offset: usize,
}

impl<'a> ComponentIter<'a> {
    /// Create a new component iterator
    pub fn new(components: &'a [OpComponent]) -> Self {
        Self {
            components,
            index: 0,
            offset: 0,
        }
    }

    /// Peek at the next component without consuming it
    pub fn peek(&self) -> Option<&OpComponent> {
        self.components.get(self.index)
    }

    /// Check if there are more components
    pub fn has_next(&self) -> bool {
        self.index < self.components.len()
    }

    /// Take up to n from the current component
    pub fn take(&mut self, n: usize) -> Option<OpComponent> {
        if let Some(component) = self.components.get(self.index) {
            let remaining = component.len() - self.offset;
            let taken = n.min(remaining);

            let result = match component {
                OpComponent::Retain(_) => OpComponent::Retain(taken),
                OpComponent::Insert(s) => {
                    let chars: Vec<char> = s.chars().collect();
                    let chunk: String = chars[self.offset..self.offset + taken].iter().collect();
                    OpComponent::Insert(chunk)
                }
                OpComponent::Delete(_) => OpComponent::Delete(taken),
            };

            self.offset += taken;
            if self.offset >= component.len() {
                self.index += 1;
                self.offset = 0;
            }

            Some(result)
        } else {
            None
        }
    }

    /// Take the rest of the current component
    pub fn take_rest(&mut self) -> Option<OpComponent> {
        if let Some(component) = self.components.get(self.index) {
            let remaining = component.len() - self.offset;
            self.take(remaining)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_basic() {
        let mut op = Operation::new();
        op.insert("Hello");

        assert_eq!(op.base_len(), 0);
        assert_eq!(op.target_len(), 5);
        assert_eq!(op.components().len(), 1);
    }

    #[test]
    fn test_operation_apply() {
        let mut op = Operation::new();
        op.retain(5).insert(" World");

        let result = op.apply("Hello").unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_operation_delete() {
        let mut op = Operation::new();
        op.delete(5);

        let result = op.apply("Hello").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_operation_complex() {
        let mut op = Operation::new();
        op.retain(6).delete(5).insert("World");

        let result = op.apply("Hello World").unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_operation_invert() {
        let mut op = Operation::new();
        op.retain(5).insert(" World");

        let inverted = op.invert("Hello").unwrap();

        let result = op.apply("Hello").unwrap();
        let reverted = inverted.apply(&result).unwrap();

        assert_eq!(reverted, "Hello");
    }

    #[test]
    fn test_component_iter() {
        let mut op = Operation::new();
        op.retain(5).insert("XY").delete(3);

        let mut iter = ComponentIter::new(op.components());

        assert_eq!(iter.take(3), Some(OpComponent::Retain(3)));
        assert_eq!(iter.take(2), Some(OpComponent::Retain(2)));
        assert_eq!(iter.take(1), Some(OpComponent::Insert("X".to_string())));
        assert_eq!(iter.take(1), Some(OpComponent::Insert("Y".to_string())));
    }

    #[test]
    fn test_operation_merge() {
        let mut op = Operation::new();
        op.retain(5);
        op.retain(3);

        assert_eq!(op.components().len(), 1);
        assert_eq!(op.components()[0], OpComponent::Retain(8));
    }
}
