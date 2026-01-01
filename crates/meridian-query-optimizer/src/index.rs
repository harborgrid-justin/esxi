//! Index Selection and Usage Optimization
//!
//! Determines when and which indexes to use for query execution.

use crate::ast::*;
use crate::plan::*;
use crate::statistics::{ColumnStatistics, TableStatistics};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Index definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDefinition {
    pub name: String,
    pub table: String,
    pub columns: Vec<String>,
    pub index_type: IndexType,
    pub unique: bool,
    pub partial: Option<ScalarExpr>, // Partial index predicate
    pub statistics: Option<IndexStatistics>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexType {
    /// B-Tree index (default, supports range queries)
    BTree,
    /// Hash index (equality only)
    Hash,
    /// GiST (Generalized Search Tree) for spatial data
    GiST,
    /// GIN (Generalized Inverted Index) for full-text search
    GIN,
    /// BRIN (Block Range Index) for large sorted tables
    BRIN,
    /// Bitmap index
    Bitmap,
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStatistics {
    /// Number of index entries
    pub entry_count: u64,
    /// Number of index pages
    pub page_count: u64,
    /// Index tree height
    pub tree_height: u32,
    /// Index selectivity (0.0 to 1.0)
    pub selectivity: f64,
    /// Clustering factor (how well index order matches physical order)
    pub clustering_factor: f64,
}

/// Index selector
pub struct IndexSelector {
    indexes: HashMap<String, Vec<IndexDefinition>>,
    table_stats: HashMap<String, TableStatistics>,
}

impl IndexSelector {
    pub fn new() -> Self {
        Self {
            indexes: HashMap::new(),
            table_stats: HashMap::new(),
        }
    }

    pub fn add_index(&mut self, index: IndexDefinition) {
        self.indexes
            .entry(index.table.clone())
            .or_insert_with(Vec::new)
            .push(index);
    }

    pub fn add_table_stats(&mut self, table: String, stats: TableStatistics) {
        self.table_stats.insert(table, stats);
    }

    /// Select best index for a scan operation
    pub fn select_index_for_scan(
        &self,
        table: &str,
        predicates: &[ScalarExpr],
        order_by: Option<&[OrderByItem]>,
    ) -> Option<IndexSelection> {
        let table_indexes = self.indexes.get(table)?;
        let table_stats = self.table_stats.get(table);

        let mut candidates = Vec::new();

        for index in table_indexes {
            let score = self.score_index(index, predicates, order_by, table_stats);
            if score > 0.0 {
                candidates.push((index.clone(), score));
            }
        }

        // Sort by score (higher is better)
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        candidates.first().map(|(index, score)| {
            let (key_conditions, filter_conditions) =
                self.split_predicates_for_index(predicates, &index.columns);

            IndexSelection {
                index: index.clone(),
                key_conditions,
                filter_conditions,
                score: *score,
            }
        })
    }

    /// Score an index for given predicates
    fn score_index(
        &self,
        index: &IndexDefinition,
        predicates: &[ScalarExpr],
        order_by: Option<&[OrderByItem]>,
        table_stats: Option<&TableStatistics>,
    ) -> f64 {
        let mut score = 0.0;

        // Score based on predicate coverage
        let covered_columns = self.count_covered_columns(predicates, &index.columns);
        score += covered_columns as f64 * 10.0;

        // Bonus for covering all predicates
        if covered_columns == predicates.len() {
            score += 20.0;
        }

        // Score based on index selectivity
        if let Some(stats) = &index.statistics {
            // More selective indexes are better
            score += (1.0 - stats.selectivity) * 30.0;

            // Better clustering is better
            score += stats.clustering_factor * 15.0;
        }

        // Score based on ORDER BY coverage
        if let Some(order_by_items) = order_by {
            if self.index_covers_order_by(index, order_by_items) {
                score += 25.0; // Significant bonus for avoiding sort
            }
        }

        // Penalty for non-BTree indexes on range queries
        if index.index_type != IndexType::BTree && self.has_range_predicates(predicates) {
            score -= 50.0;
        }

        // Bonus for unique indexes
        if index.unique {
            score += 5.0;
        }

        score
    }

    /// Count how many columns in predicates are covered by index
    fn count_covered_columns(&self, predicates: &[ScalarExpr], index_columns: &[String]) -> usize {
        let mut covered = 0;

        for pred in predicates {
            let pred_columns = self.extract_predicate_columns(pred);
            for pred_col in pred_columns {
                if index_columns.contains(&pred_col) {
                    covered += 1;
                    break;
                }
            }
        }

        covered
    }

    /// Extract column names from predicate
    fn extract_predicate_columns(&self, expr: &ScalarExpr) -> Vec<String> {
        match expr {
            ScalarExpr::Column(col) => vec![col.name.clone()],
            ScalarExpr::BinaryOp { left, right, .. } => {
                let mut cols = self.extract_predicate_columns(left);
                cols.extend(self.extract_predicate_columns(right));
                cols
            }
            ScalarExpr::UnaryOp { expr, .. } => self.extract_predicate_columns(expr),
            _ => vec![],
        }
    }

    /// Check if index can satisfy ORDER BY without additional sort
    fn index_covers_order_by(&self, index: &IndexDefinition, order_by: &[OrderByItem]) -> bool {
        if order_by.is_empty() {
            return false;
        }

        // Index must be BTree to provide ordering
        if index.index_type != IndexType::BTree {
            return false;
        }

        // Check if ORDER BY columns match index column prefix
        if order_by.len() > index.columns.len() {
            return false;
        }

        for (i, order_item) in order_by.iter().enumerate() {
            if let ScalarExpr::Column(col) = &order_item.expr {
                if index.columns[i] != col.name {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    /// Check if predicates contain range conditions
    fn has_range_predicates(&self, predicates: &[ScalarExpr]) -> bool {
        predicates.iter().any(|pred| match pred {
            ScalarExpr::BinaryOp { op, .. } => matches!(
                op,
                BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq
            ),
            ScalarExpr::Between { .. } => true,
            _ => false,
        })
    }

    /// Split predicates into index key conditions and filter conditions
    fn split_predicates_for_index(
        &self,
        predicates: &[ScalarExpr],
        index_columns: &[String],
    ) -> (Vec<ScalarExpr>, Vec<ScalarExpr>) {
        let mut key_conditions = Vec::new();
        let mut filter_conditions = Vec::new();

        for pred in predicates {
            let pred_columns = self.extract_predicate_columns(pred);

            // Can use as key condition if:
            // 1. Uses only one column
            // 2. That column is in the index
            // 3. It's an equality or range condition
            if pred_columns.len() == 1 && index_columns.contains(&pred_columns[0]) {
                if self.is_indexable_condition(pred) {
                    key_conditions.push(pred.clone());
                    continue;
                }
            }

            filter_conditions.push(pred.clone());
        }

        (key_conditions, filter_conditions)
    }

    /// Check if condition can use index
    fn is_indexable_condition(&self, expr: &ScalarExpr) -> bool {
        matches!(
            expr,
            ScalarExpr::BinaryOp {
                op: BinaryOp::Eq
                    | BinaryOp::Lt
                    | BinaryOp::LtEq
                    | BinaryOp::Gt
                    | BinaryOp::GtEq,
                ..
            } | ScalarExpr::Between { .. }
                | ScalarExpr::In { .. }
        )
    }

    /// Select indexes for multi-column predicates
    pub fn select_bitmap_indexes(
        &self,
        table: &str,
        predicates: &[ScalarExpr],
    ) -> Vec<IndexDefinition> {
        let table_indexes = match self.indexes.get(table) {
            Some(indexes) => indexes,
            None => return vec![],
        };

        let mut selected_indexes = Vec::new();

        for pred in predicates {
            let pred_columns = self.extract_predicate_columns(pred);

            for index in table_indexes {
                // Find indexes that cover this predicate
                if pred_columns
                    .iter()
                    .all(|col| index.columns.contains(col))
                {
                    if !selected_indexes.iter().any(|idx: &IndexDefinition| idx.name == index.name) {
                        selected_indexes.push(index.clone());
                    }
                }
            }
        }

        selected_indexes
    }
}

impl Default for IndexSelector {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of index selection
#[derive(Debug, Clone)]
pub struct IndexSelection {
    pub index: IndexDefinition,
    pub key_conditions: Vec<ScalarExpr>,
    pub filter_conditions: Vec<ScalarExpr>,
    pub score: f64,
}

/// Index recommendation engine
pub struct IndexRecommender {
    query_history: Vec<QueryPattern>,
}

impl IndexRecommender {
    pub fn new() -> Self {
        Self {
            query_history: Vec::new(),
        }
    }

    pub fn add_query(&mut self, pattern: QueryPattern) {
        self.query_history.push(pattern);
    }

    /// Recommend indexes based on query patterns
    pub fn recommend_indexes(&self) -> Vec<IndexRecommendation> {
        let mut recommendations = Vec::new();

        // Analyze query patterns
        let column_usage = self.analyze_column_usage();

        // Recommend indexes for frequently used columns
        for (table, columns) in column_usage {
            for (column, usage_count) in columns {
                if usage_count > 5 {
                    // Threshold for recommendation
                    recommendations.push(IndexRecommendation {
                        table: table.clone(),
                        columns: vec![column.clone()],
                        index_type: IndexType::BTree,
                        reason: format!(
                            "Column {} used in {} queries",
                            column, usage_count
                        ),
                        estimated_benefit: usage_count as f64 * 10.0,
                    });
                }
            }
        }

        // Recommend composite indexes for common column combinations
        let composite_usage = self.analyze_composite_usage();
        for ((table, columns), usage_count) in composite_usage {
            if usage_count > 3 && columns.len() > 1 {
                recommendations.push(IndexRecommendation {
                    table: table.clone(),
                    columns: columns.clone(),
                    index_type: IndexType::BTree,
                    reason: format!(
                        "Columns {:?} frequently used together in {} queries",
                        columns, usage_count
                    ),
                    estimated_benefit: usage_count as f64 * 20.0,
                });
            }
        }

        // Sort by estimated benefit
        recommendations.sort_by(|a, b| b.estimated_benefit.partial_cmp(&a.estimated_benefit).unwrap());

        recommendations
    }

    fn analyze_column_usage(&self) -> HashMap<String, HashMap<String, usize>> {
        let mut usage = HashMap::new();

        for pattern in &self.query_history {
            for column in &pattern.filter_columns {
                usage
                    .entry(pattern.table.clone())
                    .or_insert_with(HashMap::new)
                    .entry(column.clone())
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
            }
        }

        usage
    }

    fn analyze_composite_usage(&self) -> HashMap<(String, Vec<String>), usize> {
        let mut usage = HashMap::new();

        for pattern in &self.query_history {
            if pattern.filter_columns.len() > 1 {
                let mut columns = pattern.filter_columns.clone();
                columns.sort();

                usage
                    .entry((pattern.table.clone(), columns))
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
            }
        }

        usage
    }
}

impl Default for IndexRecommender {
    fn default() -> Self {
        Self::new()
    }
}

/// Query pattern for index recommendation
#[derive(Debug, Clone)]
pub struct QueryPattern {
    pub table: String,
    pub filter_columns: Vec<String>,
    pub order_by_columns: Vec<String>,
    pub join_columns: Vec<String>,
    pub execution_count: usize,
    pub avg_execution_time_ms: f64,
}

/// Index recommendation
#[derive(Debug, Clone)]
pub struct IndexRecommendation {
    pub table: String,
    pub columns: Vec<String>,
    pub index_type: IndexType,
    pub reason: String,
    pub estimated_benefit: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_selector() {
        let mut selector = IndexSelector::new();

        let index = IndexDefinition {
            name: "idx_user_id".to_string(),
            table: "orders".to_string(),
            columns: vec!["user_id".to_string()],
            index_type: IndexType::BTree,
            unique: false,
            partial: None,
            statistics: Some(IndexStatistics {
                entry_count: 10000,
                page_count: 100,
                tree_height: 3,
                selectivity: 0.01,
                clustering_factor: 0.8,
            }),
        };

        selector.add_index(index);

        let predicates = vec![ScalarExpr::BinaryOp {
            left: Box::new(ScalarExpr::Column(ColumnRef::new("user_id"))),
            op: BinaryOp::Eq,
            right: Box::new(ScalarExpr::Literal(Literal::Integer(123))),
        }];

        let selection = selector.select_index_for_scan("orders", &predicates, None);
        assert!(selection.is_some());
    }

    #[test]
    fn test_index_recommender() {
        let mut recommender = IndexRecommender::new();

        for _ in 0..10 {
            recommender.add_query(QueryPattern {
                table: "users".to_string(),
                filter_columns: vec!["email".to_string()],
                order_by_columns: vec![],
                join_columns: vec![],
                execution_count: 1,
                avg_execution_time_ms: 100.0,
            });
        }

        let recommendations = recommender.recommend_indexes();
        assert!(!recommendations.is_empty());
    }

    #[test]
    fn test_index_type() {
        let index = IndexDefinition {
            name: "test_idx".to_string(),
            table: "test".to_string(),
            columns: vec!["col1".to_string()],
            index_type: IndexType::BTree,
            unique: true,
            partial: None,
            statistics: None,
        };

        assert_eq!(index.index_type, IndexType::BTree);
        assert!(index.unique);
    }
}
