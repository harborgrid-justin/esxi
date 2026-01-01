//! Query Optimization Rules
//!
//! Implements transformation rules for query optimization including:
//! - Predicate pushdown
//! - Join reordering
//! - Projection pruning
//! - Constant folding
//! - Expression simplification

use crate::ast::*;
use crate::plan::{LogicalNode, LogicalOp, LogicalPlan};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OptimizationError {
    #[error("Optimization failed: {0}")]
    Failed(String),

    #[error("Invalid transformation: {0}")]
    InvalidTransformation(String),
}

pub type Result<T> = std::result::Result<T, OptimizationError>;

/// Optimization rule trait
pub trait OptimizationRule {
    /// Name of the rule
    fn name(&self) -> &str;

    /// Apply the rule to a logical plan
    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan>;

    /// Check if rule is applicable
    fn is_applicable(&self, plan: &LogicalPlan) -> bool;
}

/// Rule application strategy
#[derive(Debug, Clone, Copy)]
pub enum RuleStrategy {
    /// Apply once to the entire plan
    Once,
    /// Apply repeatedly until fixpoint
    FixedPoint { max_iterations: usize },
    /// Apply top-down traversal
    TopDown,
    /// Apply bottom-up traversal
    BottomUp,
}

/// Query optimizer with rule-based optimization
pub struct RuleBasedOptimizer {
    rules: Vec<Box<dyn OptimizationRule>>,
    strategy: RuleStrategy,
}

impl RuleBasedOptimizer {
    pub fn new() -> Self {
        Self {
            rules: vec![
                Box::new(PredicatePushdownRule),
                Box::new(ProjectionPruningRule),
                Box::new(ConstantFoldingRule),
                Box::new(FilterMergeRule),
                Box::new(EliminateDistinctRule),
                Box::new(EliminateLimitRule),
                Box::new(JoinReorderingRule),
                Box::new(SubqueryDecorrelationRule),
            ],
            strategy: RuleStrategy::FixedPoint { max_iterations: 10 },
        }
    }

    pub fn with_rules(rules: Vec<Box<dyn OptimizationRule>>) -> Self {
        Self {
            rules,
            strategy: RuleStrategy::FixedPoint { max_iterations: 10 },
        }
    }

    pub fn optimize(&self, mut plan: LogicalPlan) -> Result<LogicalPlan> {
        match self.strategy {
            RuleStrategy::Once => {
                for rule in &self.rules {
                    if rule.is_applicable(&plan) {
                        plan = rule.apply(plan)?;
                    }
                }
                Ok(plan)
            }
            RuleStrategy::FixedPoint { max_iterations } => {
                for iteration in 0..max_iterations {
                    let mut changed = false;
                    for rule in &self.rules {
                        if rule.is_applicable(&plan) {
                            let new_plan = rule.apply(plan.clone())?;
                            if !plans_equal(&plan, &new_plan) {
                                plan = new_plan;
                                changed = true;
                            }
                        }
                    }
                    if !changed {
                        tracing::debug!("Optimization converged after {} iterations", iteration + 1);
                        break;
                    }
                }
                Ok(plan)
            }
            RuleStrategy::TopDown | RuleStrategy::BottomUp => {
                // Would implement tree traversal strategies
                self.optimize_fixed_point(plan, 10)
            }
        }
    }

    fn optimize_fixed_point(&self, mut plan: LogicalPlan, max_iter: usize) -> Result<LogicalPlan> {
        for _ in 0..max_iter {
            let mut changed = false;
            for rule in &self.rules {
                if rule.is_applicable(&plan) {
                    let new_plan = rule.apply(plan.clone())?;
                    if !plans_equal(&plan, &new_plan) {
                        plan = new_plan;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        Ok(plan)
    }
}

impl Default for RuleBasedOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Predicate Pushdown Rule
/// Pushes filter predicates as close to table scans as possible
pub struct PredicatePushdownRule;

impl OptimizationRule for PredicatePushdownRule {
    fn name(&self) -> &str {
        "PredicatePushdown"
    }

    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        let root = self.pushdown_node(plan.root)?;
        Ok(LogicalPlan::new(root))
    }

    fn is_applicable(&self, plan: &LogicalPlan) -> bool {
        has_filter_above_scan(&plan.root)
    }
}

impl PredicatePushdownRule {
    fn pushdown_node(&self, node: LogicalNode) -> Result<LogicalNode> {
        match &node.op {
            LogicalOp::Filter { predicates } => {
                if node.children.is_empty() {
                    return Ok(node);
                }

                let child = &node.children[0];
                match &child.op {
                    LogicalOp::Scan {
                        table,
                        alias,
                        predicates: existing_preds,
                        projection,
                    } => {
                        // Push filter into scan
                        let mut all_preds = existing_preds.clone();
                        all_preds.extend(predicates.clone());

                        Ok(LogicalNode::new(
                            LogicalOp::Scan {
                                table: table.clone(),
                                alias: alias.clone(),
                                predicates: all_preds,
                                projection: projection.clone(),
                            },
                            vec![],
                            child.schema.clone(),
                        ))
                    }
                    LogicalOp::Join { join_type, condition } => {
                        // Push predicates through join
                        let (left_preds, right_preds, join_preds) =
                            self.split_predicates_for_join(predicates, &child.children)?;

                        let mut left_child = child.children[0].clone();
                        let mut right_child = child.children[1].clone();

                        // Add predicates to children
                        if !left_preds.is_empty() {
                            left_child = LogicalNode::new(
                                LogicalOp::Filter {
                                    predicates: left_preds,
                                },
                                vec![left_child],
                                child.children[0].schema.clone(),
                            );
                        }

                        if !right_preds.is_empty() {
                            right_child = LogicalNode::new(
                                LogicalOp::Filter {
                                    predicates: right_preds,
                                },
                                vec![right_child],
                                child.children[1].schema.clone(),
                            );
                        }

                        let join_node = LogicalNode::new(
                            LogicalOp::Join {
                                join_type: *join_type,
                                condition: condition.clone(),
                            },
                            vec![left_child, right_child],
                            child.schema.clone(),
                        );

                        if !join_preds.is_empty() {
                            Ok(LogicalNode::new(
                                LogicalOp::Filter {
                                    predicates: join_preds,
                                },
                                vec![join_node],
                                child.schema.clone(),
                            ))
                        } else {
                            Ok(join_node)
                        }
                    }
                    _ => {
                        // Recursively optimize child
                        let optimized_child = self.pushdown_node(child.clone())?;
                        Ok(LogicalNode::new(
                            node.op.clone(),
                            vec![optimized_child],
                            node.schema.clone(),
                        ))
                    }
                }
            }
            _ => {
                // Recursively optimize children
                let optimized_children = node
                    .children
                    .iter()
                    .map(|c| self.pushdown_node(c.clone()))
                    .collect::<Result<Vec<_>>>()?;

                Ok(LogicalNode::new(
                    node.op.clone(),
                    optimized_children,
                    node.schema.clone(),
                ))
            }
        }
    }

    fn split_predicates_for_join(
        &self,
        predicates: &[ScalarExpr],
        join_children: &[LogicalNode],
    ) -> Result<(Vec<ScalarExpr>, Vec<ScalarExpr>, Vec<ScalarExpr>)> {
        let mut left_preds = Vec::new();
        let mut right_preds = Vec::new();
        let mut join_preds = Vec::new();

        for pred in predicates {
            let columns = extract_columns(pred);
            let left_columns = get_output_columns(&join_children[0]);
            let right_columns = get_output_columns(&join_children[1]);

            let uses_left = columns.iter().any(|c| left_columns.contains(c));
            let uses_right = columns.iter().any(|c| right_columns.contains(c));

            if uses_left && !uses_right {
                left_preds.push(pred.clone());
            } else if uses_right && !uses_left {
                right_preds.push(pred.clone());
            } else {
                join_preds.push(pred.clone());
            }
        }

        Ok((left_preds, right_preds, join_preds))
    }
}

/// Projection Pruning Rule
/// Removes unnecessary columns from projections
pub struct ProjectionPruningRule;

impl OptimizationRule for ProjectionPruningRule {
    fn name(&self) -> &str {
        "ProjectionPruning"
    }

    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        // Track which columns are actually needed
        let required_columns = self.collect_required_columns(&plan.root);
        let root = self.prune_node(plan.root, &required_columns)?;
        Ok(LogicalPlan::new(root))
    }

    fn is_applicable(&self, _plan: &LogicalPlan) -> bool {
        true // Always try to prune
    }
}

impl ProjectionPruningRule {
    fn collect_required_columns(&self, node: &LogicalNode) -> HashSet<String> {
        let mut required = HashSet::new();

        match &node.op {
            LogicalOp::Project { projections } => {
                for proj in projections {
                    required.extend(extract_column_names(&proj.expr));
                }
            }
            LogicalOp::Filter { predicates } => {
                for pred in predicates {
                    required.extend(extract_column_names(pred));
                }
            }
            LogicalOp::Join { condition, .. } => {
                if let Some(cond) = condition {
                    required.extend(extract_column_names(cond));
                }
            }
            LogicalOp::Aggregate {
                group_by,
                aggregates,
                having,
            } => {
                for expr in group_by {
                    required.extend(extract_column_names(expr));
                }
                for agg in aggregates {
                    for arg in &agg.args {
                        required.extend(extract_column_names(arg));
                    }
                }
                if let Some(h) = having {
                    required.extend(extract_column_names(h));
                }
            }
            LogicalOp::Sort { order_by } => {
                for item in order_by {
                    required.extend(extract_column_names(&item.expr));
                }
            }
            _ => {}
        }

        // Recursively collect from children
        for child in &node.children {
            required.extend(self.collect_required_columns(child));
        }

        required
    }

    fn prune_node(
        &self,
        node: LogicalNode,
        required: &HashSet<String>,
    ) -> Result<LogicalNode> {
        match &node.op {
            LogicalOp::Scan {
                table,
                alias,
                predicates,
                projection,
            } => {
                // Prune projection to only required columns
                let pruned_projection = if projection.is_none() {
                    Some(required.iter().cloned().collect())
                } else {
                    projection.clone()
                };

                Ok(LogicalNode::new(
                    LogicalOp::Scan {
                        table: table.clone(),
                        alias: alias.clone(),
                        predicates: predicates.clone(),
                        projection: pruned_projection,
                    },
                    vec![],
                    node.schema.clone(),
                ))
            }
            _ => {
                let optimized_children = node
                    .children
                    .iter()
                    .map(|c| self.prune_node(c.clone(), required))
                    .collect::<Result<Vec<_>>>()?;

                Ok(LogicalNode::new(
                    node.op.clone(),
                    optimized_children,
                    node.schema.clone(),
                ))
            }
        }
    }
}

/// Constant Folding Rule
/// Evaluates constant expressions at compile time
pub struct ConstantFoldingRule;

impl OptimizationRule for ConstantFoldingRule {
    fn name(&self) -> &str {
        "ConstantFolding"
    }

    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        let root = self.fold_node(plan.root)?;
        Ok(LogicalPlan::new(root))
    }

    fn is_applicable(&self, _plan: &LogicalPlan) -> bool {
        true
    }
}

impl ConstantFoldingRule {
    fn fold_node(&self, node: LogicalNode) -> Result<LogicalNode> {
        match &node.op {
            LogicalOp::Filter { predicates } => {
                let folded_predicates = predicates
                    .iter()
                    .map(|p| self.fold_expr(p.clone()))
                    .collect();

                let optimized_children = node
                    .children
                    .iter()
                    .map(|c| self.fold_node(c.clone()))
                    .collect::<Result<Vec<_>>>()?;

                Ok(LogicalNode::new(
                    LogicalOp::Filter {
                        predicates: folded_predicates,
                    },
                    optimized_children,
                    node.schema.clone(),
                ))
            }
            _ => {
                let optimized_children = node
                    .children
                    .iter()
                    .map(|c| self.fold_node(c.clone()))
                    .collect::<Result<Vec<_>>>()?;

                Ok(LogicalNode::new(
                    node.op.clone(),
                    optimized_children,
                    node.schema.clone(),
                ))
            }
        }
    }

    fn fold_expr(&self, expr: ScalarExpr) -> ScalarExpr {
        match expr {
            ScalarExpr::BinaryOp { left, op, right } => {
                let folded_left = self.fold_expr(*left);
                let folded_right = self.fold_expr(*right);

                // Try to evaluate if both sides are literals
                if let (ScalarExpr::Literal(l), ScalarExpr::Literal(r)) =
                    (&folded_left, &folded_right)
                {
                    if let Some(result) = self.evaluate_binary_op(l, &op, r) {
                        return ScalarExpr::Literal(result);
                    }
                }

                ScalarExpr::BinaryOp {
                    left: Box::new(folded_left),
                    op,
                    right: Box::new(folded_right),
                }
            }
            ScalarExpr::UnaryOp { op, expr } => {
                let folded = self.fold_expr(*expr);
                if let ScalarExpr::Literal(lit) = &folded {
                    if let Some(result) = self.evaluate_unary_op(&op, lit) {
                        return ScalarExpr::Literal(result);
                    }
                }
                ScalarExpr::UnaryOp {
                    op,
                    expr: Box::new(folded),
                }
            }
            _ => expr,
        }
    }

    fn evaluate_binary_op(&self, left: &Literal, op: &BinaryOp, right: &Literal) -> Option<Literal> {
        match (left, op, right) {
            (Literal::Integer(l), BinaryOp::Add, Literal::Integer(r)) => {
                Some(Literal::Integer(l + r))
            }
            (Literal::Integer(l), BinaryOp::Subtract, Literal::Integer(r)) => {
                Some(Literal::Integer(l - r))
            }
            (Literal::Integer(l), BinaryOp::Multiply, Literal::Integer(r)) => {
                Some(Literal::Integer(l * r))
            }
            (Literal::Integer(l), BinaryOp::Divide, Literal::Integer(r)) if *r != 0 => {
                Some(Literal::Integer(l / r))
            }
            (Literal::Boolean(l), BinaryOp::And, Literal::Boolean(r)) => {
                Some(Literal::Boolean(*l && *r))
            }
            (Literal::Boolean(l), BinaryOp::Or, Literal::Boolean(r)) => {
                Some(Literal::Boolean(*l || *r))
            }
            _ => None,
        }
    }

    fn evaluate_unary_op(&self, op: &UnaryOp, lit: &Literal) -> Option<Literal> {
        match (op, lit) {
            (UnaryOp::Not, Literal::Boolean(b)) => Some(Literal::Boolean(!b)),
            (UnaryOp::Negate, Literal::Integer(i)) => Some(Literal::Integer(-i)),
            (UnaryOp::Negate, Literal::Float(f)) => Some(Literal::Float(-f)),
            _ => None,
        }
    }
}

/// Filter Merge Rule
/// Combines consecutive filter operations
pub struct FilterMergeRule;

impl OptimizationRule for FilterMergeRule {
    fn name(&self) -> &str {
        "FilterMerge"
    }

    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        let root = self.merge_node(plan.root)?;
        Ok(LogicalPlan::new(root))
    }

    fn is_applicable(&self, plan: &LogicalPlan) -> bool {
        has_consecutive_filters(&plan.root)
    }
}

impl FilterMergeRule {
    fn merge_node(&self, node: LogicalNode) -> Result<LogicalNode> {
        match &node.op {
            LogicalOp::Filter { predicates } if !node.children.is_empty() => {
                let child = &node.children[0];
                if let LogicalOp::Filter {
                    predicates: child_preds,
                } = &child.op
                {
                    // Merge filters
                    let mut merged_preds = predicates.clone();
                    merged_preds.extend(child_preds.clone());

                    let grandchildren = child.children.clone();
                    return Ok(LogicalNode::new(
                        LogicalOp::Filter {
                            predicates: merged_preds,
                        },
                        grandchildren,
                        node.schema.clone(),
                    ));
                }

                let optimized_children = node
                    .children
                    .iter()
                    .map(|c| self.merge_node(c.clone()))
                    .collect::<Result<Vec<_>>>()?;

                Ok(LogicalNode::new(
                    node.op.clone(),
                    optimized_children,
                    node.schema.clone(),
                ))
            }
            _ => {
                let optimized_children = node
                    .children
                    .iter()
                    .map(|c| self.merge_node(c.clone()))
                    .collect::<Result<Vec<_>>>()?;

                Ok(LogicalNode::new(
                    node.op.clone(),
                    optimized_children,
                    node.schema.clone(),
                ))
            }
        }
    }
}

/// Eliminate Distinct Rule
/// Removes unnecessary DISTINCT operations
pub struct EliminateDistinctRule;

impl OptimizationRule for EliminateDistinctRule {
    fn name(&self) -> &str {
        "EliminateDistinct"
    }

    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        Ok(plan) // Simplified - would check if input is already unique
    }

    fn is_applicable(&self, _plan: &LogicalPlan) -> bool {
        false // Simplified
    }
}

/// Eliminate Limit Rule
/// Removes LIMIT when it's larger than cardinality
pub struct EliminateLimitRule;

impl OptimizationRule for EliminateLimitRule {
    fn name(&self) -> &str {
        "EliminateLimit"
    }

    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        Ok(plan) // Simplified
    }

    fn is_applicable(&self, _plan: &LogicalPlan) -> bool {
        false
    }
}

/// Join Reordering Rule
/// Reorders joins for optimal execution
pub struct JoinReorderingRule;

impl OptimizationRule for JoinReorderingRule {
    fn name(&self) -> &str {
        "JoinReordering"
    }

    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        // Complex - would use dynamic programming or greedy algorithm
        Ok(plan)
    }

    fn is_applicable(&self, plan: &LogicalPlan) -> bool {
        count_joins(&plan.root) > 2
    }
}

/// Subquery Decorrelation Rule
/// Converts correlated subqueries to joins
pub struct SubqueryDecorrelationRule;

impl OptimizationRule for SubqueryDecorrelationRule {
    fn name(&self) -> &str {
        "SubqueryDecorrelation"
    }

    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        // Complex transformation
        Ok(plan)
    }

    fn is_applicable(&self, _plan: &LogicalPlan) -> bool {
        false // Simplified
    }
}

// Helper functions

fn plans_equal(p1: &LogicalPlan, p2: &LogicalPlan) -> bool {
    // Simplified comparison - would need deep structural comparison
    format!("{:?}", p1.root) == format!("{:?}", p2.root)
}

fn has_filter_above_scan(node: &LogicalNode) -> bool {
    matches!(node.op, LogicalOp::Filter { .. })
        && !node.children.is_empty()
        && matches!(node.children[0].op, LogicalOp::Scan { .. })
}

fn has_consecutive_filters(node: &LogicalNode) -> bool {
    if let LogicalOp::Filter { .. } = node.op {
        if !node.children.is_empty() {
            if let LogicalOp::Filter { .. } = node.children[0].op {
                return true;
            }
        }
    }
    node.children.iter().any(has_consecutive_filters)
}

fn count_joins(node: &LogicalNode) -> usize {
    let self_count = if matches!(node.op, LogicalOp::Join { .. }) {
        1
    } else {
        0
    };
    self_count + node.children.iter().map(count_joins).sum::<usize>()
}

fn extract_columns(expr: &ScalarExpr) -> HashSet<ColumnRef> {
    let mut columns = HashSet::new();
    match expr {
        ScalarExpr::Column(col) => {
            columns.insert(col.clone());
        }
        ScalarExpr::BinaryOp { left, right, .. } => {
            columns.extend(extract_columns(left));
            columns.extend(extract_columns(right));
        }
        ScalarExpr::UnaryOp { expr, .. } => {
            columns.extend(extract_columns(expr));
        }
        ScalarExpr::Function { args, .. } => {
            for arg in args {
                columns.extend(extract_columns(arg));
            }
        }
        _ => {}
    }
    columns
}

fn extract_column_names(expr: &ScalarExpr) -> HashSet<String> {
    extract_columns(expr).into_iter().map(|c| c.name).collect()
}

fn get_output_columns(node: &LogicalNode) -> HashSet<String> {
    node.schema.column_names().into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_folding() {
        let rule = ConstantFoldingRule;
        let expr = ScalarExpr::BinaryOp {
            left: Box::new(ScalarExpr::Literal(Literal::Integer(5))),
            op: BinaryOp::Add,
            right: Box::new(ScalarExpr::Literal(Literal::Integer(3))),
        };

        let folded = rule.fold_expr(expr);
        assert!(matches!(folded, ScalarExpr::Literal(Literal::Integer(8))));
    }

    #[test]
    fn test_optimizer_creation() {
        let optimizer = RuleBasedOptimizer::new();
        assert!(!optimizer.rules.is_empty());
    }
}
