//! Join Algorithm Selection and Optimization
//!
//! Implements different join algorithms:
//! - Nested Loop Join (simple, works for any condition)
//! - Hash Join (fast for equi-joins)
//! - Merge Join (efficient for sorted inputs)
//! - Index Nested Loop Join (uses indexes on inner table)

use crate::ast::*;
use crate::cost::{CostConfig, CostEstimator};
use crate::plan::*;
use crate::statistics::TableStatistics;
use std::collections::HashMap;

/// Join algorithm selector
pub struct JoinOptimizer {
    cost_estimator: CostEstimator,
}

impl JoinOptimizer {
    pub fn new(cost_estimator: CostEstimator) -> Self {
        Self { cost_estimator }
    }

    pub fn with_default_config() -> Self {
        Self::new(CostEstimator::with_default_config())
    }

    /// Select the best join algorithm for a join operation
    pub fn select_join_algorithm(
        &self,
        join_type: JoinType,
        condition: &Option<ScalarExpr>,
        left: &PhysicalNode,
        right: &PhysicalNode,
    ) -> PhysicalOp {
        // Extract equi-join keys if available
        let (left_keys, right_keys) = self.extract_join_keys(condition);

        // Generate candidate algorithms
        let mut candidates = Vec::new();

        // Nested Loop Join - always viable
        candidates.push((
            PhysicalOp::NestedLoopJoin {
                join_type,
                condition: condition.clone(),
            },
            self.estimate_nested_loop_cost(join_type, condition, left, right),
        ));

        // Hash Join - only for equi-joins
        if !left_keys.is_empty() && !right_keys.is_empty() {
            candidates.push((
                PhysicalOp::HashJoin {
                    join_type,
                    left_keys: left_keys.clone(),
                    right_keys: right_keys.clone(),
                    condition: condition.clone(),
                },
                self.estimate_hash_join_cost(
                    join_type,
                    &left_keys,
                    &right_keys,
                    condition,
                    left,
                    right,
                ),
            ));
        }

        // Merge Join - for equi-joins on sorted inputs
        if !left_keys.is_empty() && !right_keys.is_empty() {
            if self.is_sorted_on(left, &left_keys) && self.is_sorted_on(right, &right_keys) {
                candidates.push((
                    PhysicalOp::MergeJoin {
                        join_type,
                        left_keys: left_keys.clone(),
                        right_keys: right_keys.clone(),
                        condition: condition.clone(),
                    },
                    self.estimate_merge_join_cost(
                        join_type,
                        &left_keys,
                        &right_keys,
                        condition,
                        left,
                        right,
                    ),
                ));
            }
        }

        // Select algorithm with lowest cost
        candidates
            .into_iter()
            .min_by(|(_, cost1), (_, cost2)| {
                cost1
                    .0
                    .total_cost
                    .partial_cmp(&cost2.0.total_cost)
                    .unwrap()
            })
            .map(|(op, _)| op)
            .unwrap_or(PhysicalOp::NestedLoopJoin {
                join_type,
                condition: condition.clone(),
            })
    }

    /// Extract equi-join keys from condition
    fn extract_join_keys(&self, condition: &Option<ScalarExpr>) -> (Vec<ScalarExpr>, Vec<ScalarExpr>) {
        let mut left_keys = Vec::new();
        let mut right_keys = Vec::new();

        if let Some(cond) = condition {
            self.extract_keys_recursive(cond, &mut left_keys, &mut right_keys);
        }

        (left_keys, right_keys)
    }

    fn extract_keys_recursive(
        &self,
        expr: &ScalarExpr,
        left_keys: &mut Vec<ScalarExpr>,
        right_keys: &mut Vec<ScalarExpr>,
    ) {
        match expr {
            ScalarExpr::BinaryOp {
                left,
                op: BinaryOp::Eq,
                right,
            } => {
                // Check if this is a column = column comparison
                if matches!(**left, ScalarExpr::Column(_))
                    && matches!(**right, ScalarExpr::Column(_))
                {
                    left_keys.push(*left.clone());
                    right_keys.push(*right.clone());
                }
            }
            ScalarExpr::BinaryOp {
                left,
                op: BinaryOp::And,
                right,
            } => {
                // Recurse on AND conditions
                self.extract_keys_recursive(left, left_keys, right_keys);
                self.extract_keys_recursive(right, left_keys, right_keys);
            }
            _ => {}
        }
    }

    /// Check if input is sorted on given keys
    fn is_sorted_on(&self, node: &PhysicalNode, _keys: &[ScalarExpr]) -> bool {
        // Check if node is a sort or index scan that produces sorted output
        matches!(
            node.op,
            PhysicalOp::Sort { .. }
                | PhysicalOp::TopNSort { .. }
                | PhysicalOp::IndexScan { .. }
                | PhysicalOp::SortAggregate { .. }
        )
    }

    // Cost estimation helpers
    fn estimate_nested_loop_cost(
        &self,
        join_type: JoinType,
        condition: &Option<ScalarExpr>,
        left: &PhysicalNode,
        right: &PhysicalNode,
    ) -> (Cost, Cardinality) {
        self.cost_estimator
            .estimate_operator_cost(&PhysicalOp::NestedLoopJoin { join_type, condition: condition.clone() }, &[left.clone(), right.clone()])
    }

    fn estimate_hash_join_cost(
        &self,
        join_type: JoinType,
        left_keys: &[ScalarExpr],
        right_keys: &[ScalarExpr],
        condition: &Option<ScalarExpr>,
        left: &PhysicalNode,
        right: &PhysicalNode,
    ) -> (Cost, Cardinality) {
        self.cost_estimator.estimate_operator_cost(
            &PhysicalOp::HashJoin {
                join_type,
                left_keys: left_keys.to_vec(),
                right_keys: right_keys.to_vec(),
                condition: condition.clone(),
            },
            &[left.clone(), right.clone()],
        )
    }

    fn estimate_merge_join_cost(
        &self,
        join_type: JoinType,
        left_keys: &[ScalarExpr],
        right_keys: &[ScalarExpr],
        condition: &Option<ScalarExpr>,
        left: &PhysicalNode,
        right: &PhysicalNode,
    ) -> (Cost, Cardinality) {
        self.cost_estimator.estimate_operator_cost(
            &PhysicalOp::MergeJoin {
                join_type,
                left_keys: left_keys.to_vec(),
                right_keys: right_keys.to_vec(),
                condition: condition.clone(),
            },
            &[left.clone(), right.clone()],
        )
    }
}

/// Join order optimizer using dynamic programming
pub struct JoinOrderOptimizer {
    cost_estimator: CostEstimator,
    table_stats: HashMap<String, TableStatistics>,
}

impl JoinOrderOptimizer {
    pub fn new(cost_estimator: CostEstimator) -> Self {
        Self {
            cost_estimator,
            table_stats: HashMap::new(),
        }
    }

    pub fn add_table_stats(&mut self, table: String, stats: TableStatistics) {
        self.table_stats.insert(table, stats);
    }

    /// Optimize join order for multiple tables using dynamic programming
    pub fn optimize_join_order(&self, tables: Vec<JoinNode>) -> Option<JoinNode> {
        if tables.is_empty() {
            return None;
        }

        if tables.len() == 1 {
            return Some(tables[0].clone());
        }

        if tables.len() == 2 {
            return Some(self.join_two_tables(&tables[0], &tables[1]));
        }

        // Dynamic programming for larger joins
        self.dp_join_order(tables)
    }

    /// Dynamic programming algorithm for join ordering
    fn dp_join_order(&self, tables: Vec<JoinNode>) -> Option<JoinNode> {
        let n = tables.len();

        // dp[subset] = best plan for joining tables in subset
        let mut dp: HashMap<Vec<bool>, (JoinNode, Cost)> = HashMap::new();

        // Base case: single tables
        for (i, table) in tables.iter().enumerate() {
            let mut subset = vec![false; n];
            subset[i] = true;
            let cost = self.estimate_base_cost(table);
            dp.insert(subset, (table.clone(), cost));
        }

        // Build up subsets of increasing size
        for subset_size in 2..=n {
            let subsets = self.generate_subsets(n, subset_size);

            for subset in subsets {
                let mut best_plan = None;
                let mut best_cost = Cost::new(f64::MAX, f64::MAX, f64::MAX, f64::MAX);

                // Try all ways to split this subset
                for split_size in 1..subset_size {
                    let splits = self.generate_splits(&subset, split_size);

                    for (left_subset, right_subset) in splits {
                        if let (Some((left_plan, left_cost)), Some((right_plan, right_cost))) =
                            (dp.get(&left_subset), dp.get(&right_subset))
                        {
                            let join_node = self.join_two_nodes(left_plan, right_plan);
                            let join_cost = self.estimate_join_cost_from_nodes(left_plan, right_plan);
                            let total_cost = left_cost.add(&right_cost).add(&join_cost);

                            if total_cost.total_cost < best_cost.total_cost {
                                best_cost = total_cost;
                                best_plan = Some(join_node);
                            }
                        }
                    }
                }

                if let Some(plan) = best_plan {
                    dp.insert(subset, (plan, best_cost));
                }
            }
        }

        // Final result: all tables joined
        let all_tables = vec![true; n];
        dp.get(&all_tables).map(|(plan, _)| plan.clone())
    }

    /// Generate all subsets of given size
    fn generate_subsets(&self, n: usize, size: usize) -> Vec<Vec<bool>> {
        let mut subsets = Vec::new();
        let mut subset = vec![false; n];
        self.generate_subsets_helper(&mut subset, 0, size, &mut subsets);
        subsets
    }

    fn generate_subsets_helper(
        &self,
        subset: &mut Vec<bool>,
        start: usize,
        remaining: usize,
        result: &mut Vec<Vec<bool>>,
    ) {
        if remaining == 0 {
            result.push(subset.clone());
            return;
        }

        for i in start..subset.len() {
            subset[i] = true;
            self.generate_subsets_helper(subset, i + 1, remaining - 1, result);
            subset[i] = false;
        }
    }

    /// Generate all ways to split a subset
    fn generate_splits(&self, subset: &[bool], left_size: usize) -> Vec<(Vec<bool>, Vec<bool>)> {
        let mut splits = Vec::new();

        // Find indices that are true
        let true_indices: Vec<usize> = subset
            .iter()
            .enumerate()
            .filter(|(_, &v)| v)
            .map(|(i, _)| i)
            .collect();

        // Generate combinations for left side
        let mut left = vec![false; subset.len()];
        self.generate_split_helper(&true_indices, &mut left, 0, left_size, subset, &mut splits);

        splits
    }

    fn generate_split_helper(
        &self,
        indices: &[usize],
        left: &mut Vec<bool>,
        start: usize,
        remaining: usize,
        original: &[bool],
        result: &mut Vec<(Vec<bool>, Vec<bool>)>,
    ) {
        if remaining == 0 {
            let mut right = original.to_vec();
            for i in 0..left.len() {
                if left[i] {
                    right[i] = false;
                }
            }
            result.push((left.clone(), right));
            return;
        }

        for i in start..indices.len() {
            left[indices[i]] = true;
            self.generate_split_helper(indices, left, i + 1, remaining - 1, original, result);
            left[indices[i]] = false;
        }
    }

    fn estimate_base_cost(&self, node: &JoinNode) -> Cost {
        // Estimate cost of scanning base table
        Cost::new(100.0, 50.0, 0.0, 0.0) // Simplified
    }

    fn estimate_join_cost_from_nodes(&self, _left: &JoinNode, _right: &JoinNode) -> Cost {
        // Estimate cost of joining two nodes
        Cost::new(50.0, 100.0, 0.0, 20.0) // Simplified
    }

    fn join_two_tables(&self, left: &JoinNode, right: &JoinNode) -> JoinNode {
        JoinNode::Join {
            left: Box::new(left.clone()),
            right: Box::new(right.clone()),
            join_type: JoinType::Inner,
            condition: None,
        }
    }

    fn join_two_nodes(&self, left: &JoinNode, right: &JoinNode) -> JoinNode {
        self.join_two_tables(left, right)
    }
}

/// Simplified join tree representation for optimization
#[derive(Debug, Clone)]
pub enum JoinNode {
    Table {
        name: String,
        alias: Option<String>,
    },
    Join {
        left: Box<JoinNode>,
        right: Box<JoinNode>,
        join_type: JoinType,
        condition: Option<ScalarExpr>,
    },
}

/// Join hint to guide optimizer
#[derive(Debug, Clone)]
pub enum JoinHint {
    /// Force nested loop join
    ForceNestedLoop,
    /// Force hash join
    ForceHashJoin,
    /// Force merge join
    ForceMergeJoin,
    /// Suggest join order
    JoinOrder(Vec<String>),
    /// Use specific index
    UseIndex(String),
}

/// Join strategy selector with hints
pub struct JoinStrategySelector {
    optimizer: JoinOptimizer,
    hints: Vec<JoinHint>,
}

impl JoinStrategySelector {
    pub fn new(optimizer: JoinOptimizer) -> Self {
        Self {
            optimizer,
            hints: Vec::new(),
        }
    }

    pub fn with_hints(mut self, hints: Vec<JoinHint>) -> Self {
        self.hints = hints;
        self
    }

    pub fn select_strategy(
        &self,
        join_type: JoinType,
        condition: &Option<ScalarExpr>,
        left: &PhysicalNode,
        right: &PhysicalNode,
    ) -> PhysicalOp {
        // Check for forced hints
        for hint in &self.hints {
            match hint {
                JoinHint::ForceNestedLoop => {
                    return PhysicalOp::NestedLoopJoin {
                        join_type,
                        condition: condition.clone(),
                    };
                }
                JoinHint::ForceHashJoin => {
                    let (left_keys, right_keys) = self.optimizer.extract_join_keys(condition);
                    if !left_keys.is_empty() {
                        return PhysicalOp::HashJoin {
                            join_type,
                            left_keys,
                            right_keys,
                            condition: condition.clone(),
                        };
                    }
                }
                JoinHint::ForceMergeJoin => {
                    let (left_keys, right_keys) = self.optimizer.extract_join_keys(condition);
                    if !left_keys.is_empty() {
                        return PhysicalOp::MergeJoin {
                            join_type,
                            left_keys,
                            right_keys,
                            condition: condition.clone(),
                        };
                    }
                }
                _ => {}
            }
        }

        // Default: use cost-based selection
        self.optimizer
            .select_join_algorithm(join_type, condition, left, right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_optimizer_creation() {
        let optimizer = JoinOptimizer::with_default_config();
        // Just verify it's created
    }

    #[test]
    fn test_extract_join_keys() {
        let optimizer = JoinOptimizer::with_default_config();

        let condition = ScalarExpr::BinaryOp {
            left: Box::new(ScalarExpr::Column(ColumnRef::new("a"))),
            op: BinaryOp::Eq,
            right: Box::new(ScalarExpr::Column(ColumnRef::new("b"))),
        };

        let (left_keys, right_keys) = optimizer.extract_join_keys(&Some(condition));
        assert_eq!(left_keys.len(), 1);
        assert_eq!(right_keys.len(), 1);
    }

    #[test]
    fn test_join_order_optimizer() {
        let cost_estimator = CostEstimator::with_default_config();
        let optimizer = JoinOrderOptimizer::new(cost_estimator);

        let tables = vec![
            JoinNode::Table {
                name: "users".to_string(),
                alias: None,
            },
            JoinNode::Table {
                name: "orders".to_string(),
                alias: None,
            },
        ];

        let result = optimizer.optimize_join_order(tables);
        assert!(result.is_some());
    }

    #[test]
    fn test_join_strategy_selector() {
        let optimizer = JoinOptimizer::with_default_config();
        let selector = JoinStrategySelector::new(optimizer)
            .with_hints(vec![JoinHint::ForceNestedLoop]);

        // Would test with actual physical nodes
    }
}
