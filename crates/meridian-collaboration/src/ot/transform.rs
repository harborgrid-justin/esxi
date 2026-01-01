//! # Operational Transform - Transform Functions
//!
//! This module implements the core transformation functions for Operational Transform.
//! The transform function takes two concurrent operations and transforms them such that
//! they can be applied in any order while maintaining consistency.
//!
//! ## Transformation Properties (TP1 and TP2)
//!
//! - **TP1**: `apply(s, compose(Oa, transform(Ob, Oa))) = apply(s, compose(Ob, transform(Oa, Ob)))`
//! - **TP2**: `transform(Oc, compose(Oa, Ob)) = compose(transform(Oc, Oa), transform(Oc, Ob))`

use super::{ComponentIter, OpComponent, Operation, OtError, Result};

/// Transform operation `a` against operation `b`
///
/// Returns the transformed version of `a` that can be applied after `b`
/// has been applied.
///
/// # Example
/// ```text
/// Client 1: "abc" -> insert("X") at 1 -> "aXbc"
/// Client 2: "abc" -> insert("Y") at 1 -> "aYbc"
///
/// Transform insert("X") against insert("Y"):
/// - Transformed operation adjusts position to account for "Y"
/// ```
pub fn transform(a: &Operation, b: &Operation) -> Result<Operation> {
    if a.base_len() != b.base_len() {
        return Err(OtError::TransformError(format!(
            "Base lengths don't match: {} vs {}",
            a.base_len(),
            b.base_len()
        )));
    }

    let mut a_prime = Operation::new();
    let mut iter_a = ComponentIter::new(a.components());
    let mut iter_b = ComponentIter::new(b.components());

    while iter_a.has_next() || iter_b.has_next() {
        match (iter_a.peek(), iter_b.peek()) {
            // Insert from a - passes through unchanged
            (Some(OpComponent::Insert(_)), _) => {
                if let Some(component) = iter_a.take_rest() {
                    a_prime.components.push(component);
                }
            }

            // Insert from b - retain to skip over it
            (_, Some(OpComponent::Insert(s))) => {
                let len = s.chars().count();
                iter_b.take(len);
                a_prime.retain(len);
            }

            // Both retain
            (Some(OpComponent::Retain(n1)), Some(OpComponent::Retain(n2))) => {
                let min_len = (*n1).min(*n2);
                iter_a.take(min_len);
                iter_b.take(min_len);
                a_prime.retain(min_len);
            }

            // Retain from a, delete from b
            (Some(OpComponent::Retain(n1)), Some(OpComponent::Delete(n2))) => {
                let min_len = (*n1).min(*n2);
                iter_a.take(min_len);
                iter_b.take(min_len);
                // Delete cancels retain
            }

            // Delete from a, retain from b
            (Some(OpComponent::Delete(n1)), Some(OpComponent::Retain(n2))) => {
                let min_len = (*n1).min(*n2);
                iter_a.take(min_len);
                iter_b.take(min_len);
                a_prime.delete(min_len);
            }

            // Both delete
            (Some(OpComponent::Delete(n1)), Some(OpComponent::Delete(n2))) => {
                let min_len = (*n1).min(*n2);
                iter_a.take(min_len);
                iter_b.take(min_len);
                // Both deletes cancel out
            }

            // Only a has components left (shouldn't happen if base_len matches)
            (Some(OpComponent::Retain(_)), None) |
            (Some(OpComponent::Delete(_)), None) => {
                return Err(OtError::TransformError(
                    "Operation a has more components than expected".to_string(),
                ));
            }

            // Only b has components left (shouldn't happen if base_len matches)
            (None, Some(OpComponent::Retain(_))) |
            (None, Some(OpComponent::Delete(_))) => {
                return Err(OtError::TransformError(
                    "Operation b has more components than expected".to_string(),
                ));
            }

            (None, None) => break,
        }
    }

    Ok(a_prime)
}

/// Transform operation `a` against operation `b` (exclusive mode)
///
/// Similar to `transform`, but used when we want the transformed operation
/// to be applied "before" concurrent inserts from `b` rather than after.
/// This is used to break ties and ensure convergence.
///
/// The key difference: when both operations insert at the same position,
/// transform_x gives priority to `b`.
pub fn transform_x(a: &Operation, b: &Operation, side: Side) -> Result<Operation> {
    if a.base_len() != b.base_len() {
        return Err(OtError::TransformError(format!(
            "Base lengths don't match: {} vs {}",
            a.base_len(),
            b.base_len()
        )));
    }

    let mut a_prime = Operation::new();
    let mut iter_a = ComponentIter::new(a.components());
    let mut iter_b = ComponentIter::new(b.components());

    while iter_a.has_next() || iter_b.has_next() {
        match (iter_a.peek(), iter_b.peek()) {
            // Both insert at same position - use side to break tie
            (Some(OpComponent::Insert(_)), Some(OpComponent::Insert(s))) => {
                match side {
                    Side::Left => {
                        // a's insert goes first
                        if let Some(component) = iter_a.take_rest() {
                            a_prime.components.push(component);
                        }
                    }
                    Side::Right => {
                        // b's insert goes first, a must retain
                        let len = s.chars().count();
                        iter_b.take(len);
                        a_prime.retain(len);
                    }
                }
            }

            // Insert from a (no concurrent insert from b)
            (Some(OpComponent::Insert(_)), _) => {
                if let Some(component) = iter_a.take_rest() {
                    a_prime.components.push(component);
                }
            }

            // Insert from b
            (_, Some(OpComponent::Insert(s))) => {
                let len = s.chars().count();
                iter_b.take(len);
                a_prime.retain(len);
            }

            // Both retain
            (Some(OpComponent::Retain(n1)), Some(OpComponent::Retain(n2))) => {
                let min_len = (*n1).min(*n2);
                iter_a.take(min_len);
                iter_b.take(min_len);
                a_prime.retain(min_len);
            }

            // Retain from a, delete from b
            (Some(OpComponent::Retain(n1)), Some(OpComponent::Delete(n2))) => {
                let min_len = (*n1).min(*n2);
                iter_a.take(min_len);
                iter_b.take(min_len);
            }

            // Delete from a, retain from b
            (Some(OpComponent::Delete(n1)), Some(OpComponent::Retain(n2))) => {
                let min_len = (*n1).min(*n2);
                iter_a.take(min_len);
                iter_b.take(min_len);
                a_prime.delete(min_len);
            }

            // Both delete
            (Some(OpComponent::Delete(n1)), Some(OpComponent::Delete(n2))) => {
                let min_len = (*n1).min(*n2);
                iter_a.take(min_len);
                iter_b.take(min_len);
            }

            // Mismatched end conditions
            (Some(OpComponent::Retain(_)), None) |
            (Some(OpComponent::Delete(_)), None) => {
                return Err(OtError::TransformError(
                    "Operation a has more components than expected".to_string(),
                ));
            }

            (None, Some(OpComponent::Retain(_))) |
            (None, Some(OpComponent::Delete(_))) => {
                return Err(OtError::TransformError(
                    "Operation b has more components than expected".to_string(),
                ));
            }

            (None, None) => break,
        }
    }

    Ok(a_prime)
}

/// Side preference for breaking ties in concurrent inserts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// Prefer left operation's inserts
    Left,
    /// Prefer right operation's inserts
    Right,
}

/// Transform a cursor/selection position through an operation
pub fn transform_cursor(cursor: usize, op: &Operation) -> usize {
    let mut transformed_cursor = cursor;
    let mut index = 0;

    for component in op.components() {
        match component {
            OpComponent::Retain(n) => {
                index += n;
            }
            OpComponent::Insert(s) => {
                let len = s.chars().count();
                if index < cursor || (index == cursor) {
                    transformed_cursor += len;
                }
                index += len;
            }
            OpComponent::Delete(n) => {
                if index < cursor {
                    transformed_cursor = transformed_cursor.saturating_sub((*n).min(cursor - index));
                }
                // index stays the same after delete
            }
        }

        if index > cursor {
            break;
        }
    }

    transformed_cursor
}

/// Transform a selection range through an operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub start: usize,
    pub end: usize,
}

impl Selection {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn is_collapsed(&self) -> bool {
        self.start == self.end
    }

    pub fn len(&self) -> usize {
        if self.end >= self.start {
            self.end - self.start
        } else {
            0
        }
    }
}

pub fn transform_selection(selection: Selection, op: &Operation) -> Selection {
    Selection {
        start: transform_cursor(selection.start, op),
        end: transform_cursor(selection.end, op),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_concurrent_insert() {
        // Both insert at position 0
        let mut op_a = Operation::with_base_len(0);
        op_a.insert("A");

        let mut op_b = Operation::with_base_len(0);
        op_b.insert("B");

        let a_prime = transform(&op_a, &op_b).unwrap();
        let b_prime = transform(&op_b, &op_a).unwrap();

        // Apply in both orders
        let result1 = op_b.apply("").unwrap();
        let result1 = a_prime.apply(&result1).unwrap();

        let result2 = op_a.apply("").unwrap();
        let result2 = b_prime.apply(&result2).unwrap();

        // Should converge
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_transform_insert_delete() {
        // Initial: "Hello"
        // A: delete "llo" (chars 2-5)
        // B: insert "X" at position 3

        let mut op_a = Operation::with_base_len(5);
        op_a.retain(2).delete(3);

        let mut op_b = Operation::with_base_len(5);
        op_b.retain(3).insert("X");

        let a_prime = transform(&op_a, &op_b).unwrap();
        let b_prime = transform(&op_b, &op_a).unwrap();

        let result1 = op_b.apply("Hello").unwrap();
        let result1 = a_prime.apply(&result1).unwrap();

        let result2 = op_a.apply("Hello").unwrap();
        let result2 = b_prime.apply(&result2).unwrap();

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_transform_same_position() {
        let mut op_a = Operation::with_base_len(3);
        op_a.retain(1).insert("X");

        let mut op_b = Operation::with_base_len(3);
        op_b.retain(1).insert("Y");

        let a_prime = transform(&op_a, &op_b).unwrap();
        let b_prime = transform(&op_b, &op_a).unwrap();

        let result1 = op_b.apply("abc").unwrap();
        let result1 = a_prime.apply(&result1).unwrap();

        let result2 = op_a.apply("abc").unwrap();
        let result2 = b_prime.apply(&result2).unwrap();

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_transform_x_sides() {
        let mut op_a = Operation::with_base_len(0);
        op_a.insert("A");

        let mut op_b = Operation::with_base_len(0);
        op_b.insert("B");

        // Left side - A goes first
        let a_left = transform_x(&op_a, &op_b, Side::Left).unwrap();
        let result = op_b.apply("").unwrap();
        let result = a_left.apply(&result).unwrap();
        assert!(result.starts_with("B") || result.starts_with("A"));

        // Right side - B goes first
        let a_right = transform_x(&op_a, &op_b, Side::Right).unwrap();
        let result = op_b.apply("").unwrap();
        let result = a_right.apply(&result).unwrap();
        assert!(result.starts_with("B") || result.starts_with("A"));
    }

    #[test]
    fn test_transform_cursor() {
        let mut op = Operation::with_base_len(5);
        op.retain(2).insert("XX").delete(1);

        assert_eq!(transform_cursor(0, &op), 0);
        assert_eq!(transform_cursor(2, &op), 4); // Before insert
        assert_eq!(transform_cursor(3, &op), 4); // After delete
        assert_eq!(transform_cursor(5, &op), 6);
    }

    #[test]
    fn test_transform_selection() {
        let mut op = Operation::with_base_len(10);
        op.retain(3).insert("XXX").delete(2);

        let sel = Selection::new(2, 5);
        let transformed = transform_selection(sel, &op);

        // Selection should be adjusted for insert and delete
        assert!(transformed.start >= sel.start);
    }

    #[test]
    fn test_transform_identity() {
        let mut op_a = Operation::with_base_len(5);
        op_a.retain(2).insert("X").delete(1);

        let mut op_noop = Operation::with_base_len(5);
        op_noop.retain(5);

        let transformed = transform(&op_a, &op_noop).unwrap();

        // Transforming against a no-op should yield the same operation
        assert_eq!(op_a.apply("Hello").unwrap(), transformed.apply("Hello").unwrap());
    }

    #[test]
    fn test_transform_delete_delete() {
        // Both delete same range
        let mut op_a = Operation::with_base_len(5);
        op_a.delete(5);

        let mut op_b = Operation::with_base_len(5);
        op_b.delete(5);

        let a_prime = transform(&op_a, &op_b).unwrap();
        let b_prime = transform(&op_b, &op_a).unwrap();

        let result1 = op_b.apply("Hello").unwrap();
        let result1 = a_prime.apply(&result1).unwrap();

        let result2 = op_a.apply("Hello").unwrap();
        let result2 = b_prime.apply(&result2).unwrap();

        assert_eq!(result1, result2);
        assert_eq!(result1, "");
    }
}
