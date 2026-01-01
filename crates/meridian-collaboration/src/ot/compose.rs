//! # Operational Transform - Compose Functions
//!
//! This module implements operation composition. Composing two sequential operations
//! creates a single operation that has the same effect as applying them one after another.
//!
//! ## Composition
//!
//! Given operations `A` and `B` where `A.target_len == B.base_len`:
//! ```text
//! compose(A, B) = C
//! apply(s, A) -> s'
//! apply(s', B) -> s''
//! apply(s, C) -> s''
//! ```

use super::{ComponentIter, OpComponent, Operation, OtError, Result};

/// Compose two sequential operations into a single operation
///
/// # Arguments
/// * `a` - First operation to apply
/// * `b` - Second operation to apply (must have base_len == a.target_len)
///
/// # Returns
/// A single operation that has the same effect as applying `a` then `b`
///
/// # Example
/// ```text
/// op1: insert("Hello") at 0
/// op2: insert(" World") at 5
/// composed: insert("Hello World") at 0
/// ```
pub fn compose(a: &Operation, b: &Operation) -> Result<Operation> {
    if a.target_len() != b.base_len() {
        return Err(OtError::ComposeError(format!(
            "Target length of first operation ({}) doesn't match base length of second ({})",
            a.target_len(),
            b.base_len()
        )));
    }

    let mut composed = Operation::new();
    let mut iter_a = ComponentIter::new(a.components());
    let mut iter_b = ComponentIter::new(b.components());

    while iter_a.has_next() || iter_b.has_next() {
        match (iter_a.peek(), iter_b.peek()) {
            // Delete from b
            (_, Some(OpComponent::Delete(n))) => {
                let len = *n;
                iter_b.take(len);
                composed.delete(len);
            }

            // Insert from a
            (Some(OpComponent::Insert(_)), _) => {
                if let Some(component) = iter_a.take_rest() {
                    match component {
                        OpComponent::Insert(s) => {
                            // Check if b deletes or retains this insert
                            match iter_b.peek() {
                                Some(OpComponent::Delete(_)) => {
                                    let len = s.chars().count();
                                    iter_b.take(len);
                                    // Delete cancels insert
                                }
                                Some(OpComponent::Retain(_)) => {
                                    let len = s.chars().count();
                                    iter_b.take(len);
                                    composed.insert(s);
                                }
                                Some(OpComponent::Insert(_)) => {
                                    // b inserts at this position
                                    composed.insert(s);
                                }
                                None => {
                                    composed.insert(s);
                                }
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            }

            // Insert from b
            (_, Some(OpComponent::Insert(s))) => {
                let text = s.clone();
                let len = text.chars().count();
                iter_b.take(len);
                composed.insert(text);
            }

            // Both retain
            (Some(OpComponent::Retain(n1)), Some(OpComponent::Retain(n2))) => {
                let min_len = (*n1).min(*n2);
                iter_a.take(min_len);
                iter_b.take(min_len);
                composed.retain(min_len);
            }

            // Delete from a, retain from b
            (Some(OpComponent::Delete(n1)), Some(OpComponent::Retain(n2))) => {
                let min_len = (*n1).min(*n2);
                iter_a.take(min_len);
                iter_b.take(min_len);
                composed.delete(min_len);
            }

            // Retain from a, delete from b (already handled above)
            (Some(OpComponent::Retain(n1)), Some(OpComponent::Delete(n2))) => {
                let min_len = (*n1).min(*n2);
                iter_a.take(min_len);
                iter_b.take(min_len);
                composed.delete(min_len);
            }

            // Delete from a, delete from b
            (Some(OpComponent::Delete(n1)), Some(OpComponent::Delete(n2))) => {
                let len_a = *n1;
                iter_a.take(len_a);
                composed.delete(len_a);
            }

            // Only a remaining
            (Some(OpComponent::Retain(n)), None) => {
                let len = *n;
                iter_a.take(len);
                composed.retain(len);
            }

            (Some(OpComponent::Delete(n)), None) => {
                let len = *n;
                iter_a.take(len);
                composed.delete(len);
            }

            // Only b remaining
            (None, Some(OpComponent::Retain(n))) => {
                let len = *n;
                iter_b.take(len);
                composed.retain(len);
            }

            (None, None) => break,
        }
    }

    Ok(composed)
}

/// Compose a list of operations sequentially
pub fn compose_all(ops: &[Operation]) -> Result<Operation> {
    if ops.is_empty() {
        return Ok(Operation::new());
    }

    let mut result = ops[0].clone();
    for op in &ops[1..] {
        result = compose(&result, op)?;
    }

    Ok(result)
}

/// Check if two operations can be composed
pub fn can_compose(a: &Operation, b: &Operation) -> bool {
    a.target_len() == b.base_len()
}

/// Optimize an operation by merging consecutive components
pub fn optimize(op: &Operation) -> Operation {
    let mut optimized = Operation::new();

    for component in op.components() {
        match component {
            OpComponent::Retain(n) => { optimized.retain(*n); },
            OpComponent::Insert(s) => { optimized.insert(s.clone()); },
            OpComponent::Delete(n) => { optimized.delete(*n); },
        };
    }

    optimized
}

/// Split an operation at a given position
///
/// Returns (before, after) where:
/// - before: operation up to position
/// - after: operation from position onwards
pub fn split_at(op: &Operation, pos: usize) -> Result<(Operation, Operation)> {
    if pos > op.base_len() {
        return Err(OtError::InvalidIndex(pos));
    }

    let mut before = Operation::new();
    let mut after = Operation::new();
    let mut current_pos = 0;
    let mut in_after = false;

    for component in op.components() {
        match component {
            OpComponent::Retain(n) => {
                if current_pos + n <= pos {
                    // Entirely in 'before'
                    before.retain(*n);
                    current_pos += n;
                } else if current_pos >= pos {
                    // Entirely in 'after'
                    after.retain(*n);
                } else {
                    // Spans the split point
                    let before_len = pos - current_pos;
                    let after_len = n - before_len;
                    before.retain(before_len);
                    after.retain(after_len);
                    current_pos = pos;
                    in_after = true;
                }
            }
            OpComponent::Insert(s) => {
                if in_after || current_pos >= pos {
                    after.insert(s.clone());
                } else {
                    before.insert(s.clone());
                }
            }
            OpComponent::Delete(n) => {
                if current_pos + n <= pos {
                    before.delete(*n);
                    current_pos += n;
                } else if current_pos >= pos {
                    after.delete(*n);
                } else {
                    let before_len = pos - current_pos;
                    let after_len = n - before_len;
                    before.delete(before_len);
                    after.delete(after_len);
                    current_pos = pos;
                    in_after = true;
                }
            }
        }
    }

    Ok((before, after))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_inserts() {
        let mut op1 = Operation::new();
        op1.insert("Hello");

        let mut op2 = Operation::with_base_len(5);
        op2.retain(5).insert(" World");

        let composed = compose(&op1, &op2).unwrap();

        assert_eq!(composed.base_len(), 0);
        assert_eq!(composed.target_len(), 11);

        let result = composed.apply("").unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_compose_delete_insert() {
        let mut op1 = Operation::with_base_len(5);
        op1.delete(5);

        let mut op2 = Operation::with_base_len(0);
        op2.insert("World");

        let composed = compose(&op1, &op2).unwrap();

        let result = composed.apply("Hello").unwrap();
        assert_eq!(result, "World");
    }

    #[test]
    fn test_compose_retain() {
        let mut op1 = Operation::with_base_len(5);
        op1.retain(2).insert("X");

        let mut op2 = Operation::with_base_len(6);
        op2.retain(3).insert("Y");

        let composed = compose(&op1, &op2).unwrap();

        let result1 = op1.apply("Hello").unwrap();
        let result1 = op2.apply(&result1).unwrap();

        let result2 = composed.apply("Hello").unwrap();

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_compose_all() {
        let mut op1 = Operation::new();
        op1.insert("A");

        let mut op2 = Operation::with_base_len(1);
        op2.retain(1).insert("B");

        let mut op3 = Operation::with_base_len(2);
        op3.retain(2).insert("C");

        let composed = compose_all(&[op1, op2, op3]).unwrap();

        let result = composed.apply("").unwrap();
        assert_eq!(result, "ABC");
    }

    #[test]
    fn test_compose_insert_delete() {
        let mut op1 = Operation::with_base_len(5);
        op1.retain(2).insert("XX").retain(3);

        let mut op2 = Operation::with_base_len(7);
        op2.retain(2).delete(2).retain(3);

        let composed = compose(&op1, &op2).unwrap();

        let result1 = op1.apply("Hello").unwrap();
        let result1 = op2.apply(&result1).unwrap();

        let result2 = composed.apply("Hello").unwrap();

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_compose_error() {
        let mut op1 = Operation::new();
        op1.insert("Hello");

        let mut op2 = Operation::with_base_len(10); // Wrong base length
        op2.retain(10);

        assert!(compose(&op1, &op2).is_err());
    }

    #[test]
    fn test_can_compose() {
        let mut op1 = Operation::new();
        op1.insert("Hello");

        let mut op2 = Operation::with_base_len(5);
        op2.retain(5);

        assert!(can_compose(&op1, &op2));

        let mut op3 = Operation::with_base_len(10);
        op3.retain(10);

        assert!(!can_compose(&op1, &op3));
    }

    #[test]
    fn test_optimize() {
        let mut op = Operation::new();
        op.retain(5).insert("X").retain(3);

        let optimized = optimize(&op);

        assert_eq!(op.components().len(), optimized.components().len());
        assert_eq!(op.apply("Hello!!!").unwrap(), optimized.apply("Hello!!!").unwrap());
    }

    #[test]
    fn test_split_at() {
        let mut op = Operation::with_base_len(10);
        op.retain(3).insert("XX").delete(2).retain(5);

        let (before, after) = split_at(&op, 5).unwrap();

        assert_eq!(before.base_len(), 5);
        // After should handle the remaining base length
    }

    #[test]
    fn test_compose_identity() {
        let mut op = Operation::with_base_len(5);
        op.retain(2).insert("X").delete(1);

        let mut identity = Operation::with_base_len(6);
        identity.retain(6);

        let composed = compose(&op, &identity).unwrap();

        assert_eq!(op.apply("Hello").unwrap(), composed.apply("Hello").unwrap());
    }

    #[test]
    fn test_compose_complex() {
        let mut op1 = Operation::with_base_len(10);
        op1.retain(3).insert("ABC").delete(2).retain(5);

        let mut op2 = Operation::with_base_len(11);
        op2.retain(5).delete(3).insert("XY").retain(3);

        let composed = compose(&op1, &op2).unwrap();

        let text = "0123456789";
        let result1 = op1.apply(text).unwrap();
        let result1 = op2.apply(&result1).unwrap();

        let result2 = composed.apply(text).unwrap();

        assert_eq!(result1, result2);
    }
}
