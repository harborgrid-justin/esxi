//! Cost Model for Query Execution
//!
//! Provides cost estimation for physical operators based on:
//! - I/O costs (sequential/random disk reads)
//! - CPU costs (processing, comparisons)
//! - Memory costs (buffer usage, sorts, hashes)
//! - Network costs (distributed queries)

use crate::ast::*;
use crate::plan::*;
use crate::statistics::{ColumnStatistics, TableStatistics};
use std::collections::HashMap;

/// System configuration for cost modeling
#[derive(Debug, Clone)]
pub struct CostConfig {
    /// Sequential I/O cost per page
    pub seq_page_cost: f64,
    /// Random I/O cost per page
    pub random_page_cost: f64,
    /// CPU cost per tuple processed
    pub cpu_tuple_cost: f64,
    /// CPU cost per operator evaluation
    pub cpu_operator_cost: f64,
    /// CPU cost per index entry
    pub cpu_index_tuple_cost: f64,
    /// Network cost per byte transferred
    pub network_byte_cost: f64,
    /// Memory cost per byte
    pub memory_byte_cost: f64,
    /// Page size in bytes
    pub page_size: usize,
    /// Available memory for operations (bytes)
    pub work_mem: usize,
    /// Effective cache size (bytes)
    pub effective_cache_size: usize,
}

impl Default for CostConfig {
    fn default() -> Self {
        Self {
            seq_page_cost: 1.0,
            random_page_cost: 4.0,
            cpu_tuple_cost: 0.01,
            cpu_operator_cost: 0.0025,
            cpu_index_tuple_cost: 0.005,
            network_byte_cost: 0.001,
            memory_byte_cost: 0.0001,
            page_size: 8192, // 8KB pages (PostgreSQL default)
            work_mem: 4 * 1024 * 1024, // 4MB
            effective_cache_size: 128 * 1024 * 1024, // 128MB
        }
    }
}

/// Cost estimator for physical plans
pub struct CostEstimator {
    config: CostConfig,
    statistics: HashMap<String, TableStatistics>,
}

impl CostEstimator {
    pub fn new(config: CostConfig) -> Self {
        Self {
            config,
            statistics: HashMap::new(),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(CostConfig::default())
    }

    pub fn add_table_stats(&mut self, table: String, stats: TableStatistics) {
        self.statistics.insert(table, stats);
    }

    /// Estimate cost for a physical operator
    pub fn estimate_operator_cost(
        &self,
        op: &PhysicalOp,
        children: &[PhysicalNode],
    ) -> (Cost, Cardinality) {
        match op {
            PhysicalOp::SeqScan {
                table,
                predicates,
                projection,
                ..
            } => self.cost_seq_scan(table, predicates, projection),

            PhysicalOp::IndexScan {
                table,
                index_name,
                key_conditions,
                predicates,
                ..
            } => self.cost_index_scan(table, index_name, key_conditions, predicates),

            PhysicalOp::BitmapScan {
                table, predicates, ..
            } => self.cost_bitmap_scan(table, predicates),

            PhysicalOp::Filter { predicates } => {
                self.cost_filter(predicates, children.first())
            }

            PhysicalOp::Project { projections } => {
                self.cost_project(projections, children.first())
            }

            PhysicalOp::NestedLoopJoin {
                join_type,
                condition,
            } => self.cost_nested_loop_join(join_type, condition, children),

            PhysicalOp::HashJoin {
                join_type,
                left_keys,
                right_keys,
                condition,
            } => self.cost_hash_join(join_type, left_keys, right_keys, condition, children),

            PhysicalOp::MergeJoin {
                join_type,
                left_keys,
                right_keys,
                condition,
            } => self.cost_merge_join(join_type, left_keys, right_keys, condition, children),

            PhysicalOp::HashAggregate {
                group_by,
                aggregates,
                ..
            } => self.cost_hash_aggregate(group_by, aggregates, children.first()),

            PhysicalOp::SortAggregate {
                group_by,
                aggregates,
                ..
            } => self.cost_sort_aggregate(group_by, aggregates, children.first()),

            PhysicalOp::Sort { order_by } => self.cost_sort(order_by, children.first()),

            PhysicalOp::TopNSort { order_by, limit } => {
                self.cost_topn_sort(order_by, *limit, children.first())
            }

            PhysicalOp::Limit { limit, offset } => {
                self.cost_limit(*limit, *offset, children.first())
            }

            PhysicalOp::HashDistinct => self.cost_hash_distinct(children.first()),

            PhysicalOp::SortDistinct => self.cost_sort_distinct(children.first()),

            PhysicalOp::UnionAll => self.cost_union_all(children),

            PhysicalOp::HashUnion => self.cost_hash_union(children),

            PhysicalOp::Gather { num_workers } => self.cost_gather(*num_workers, children.first()),

            PhysicalOp::Exchange {
                distribution,
                num_partitions,
            } => self.cost_exchange(distribution, *num_partitions, children.first()),

            PhysicalOp::Materialize => self.cost_materialize(children.first()),
        }
    }

    // Sequential Scan Cost
    fn cost_seq_scan(
        &self,
        table: &str,
        predicates: &[ScalarExpr],
        projection: &Option<Vec<String>>,
    ) -> (Cost, Cardinality) {
        let stats = self.get_table_stats(table);
        let total_rows = stats.row_count as f64;
        let total_pages = stats.page_count as f64;

        // I/O cost: read all pages sequentially
        let io_cost = total_pages * self.config.seq_page_cost;

        // CPU cost: process all tuples + evaluate predicates
        let cpu_per_tuple = self.config.cpu_tuple_cost
            + predicates.len() as f64 * self.config.cpu_operator_cost;
        let cpu_cost = total_rows * cpu_per_tuple;

        // Memory cost: minimal for scan
        let memory_cost = self.config.page_size as f64 * self.config.memory_byte_cost;

        // Estimate selectivity from predicates
        let selectivity = self.estimate_selectivity(predicates, table);
        let output_rows = total_rows * selectivity;

        let cost = Cost::new(io_cost, cpu_cost, 0.0, memory_cost);
        let cardinality = Cardinality::with_confidence(output_rows, 0.7);

        (cost, cardinality)
    }

    // Index Scan Cost
    fn cost_index_scan(
        &self,
        table: &str,
        index_name: &str,
        key_conditions: &[ScalarExpr],
        predicates: &[ScalarExpr],
    ) -> (Cost, Cardinality) {
        let stats = self.get_table_stats(table);
        let total_rows = stats.row_count as f64;

        // Estimate index selectivity
        let index_selectivity = self.estimate_index_selectivity(key_conditions, table);
        let index_rows = total_rows * index_selectivity;

        // I/O cost: index lookup + table fetch
        // Index tree height typically log(N)
        let index_height = (total_rows.log2()).max(1.0);
        let index_io = index_height * self.config.random_page_cost;

        // Random table page fetches (worst case: one per row)
        let table_io = index_rows * self.config.random_page_cost * 0.8; // Cache factor

        let io_cost = index_io + table_io;

        // CPU cost: index traversal + tuple processing
        let cpu_cost = index_rows
            * (self.config.cpu_index_tuple_cost
                + self.config.cpu_tuple_cost
                + predicates.len() as f64 * self.config.cpu_operator_cost);

        // Apply additional predicates
        let final_selectivity = index_selectivity * self.estimate_selectivity(predicates, table);
        let output_rows = total_rows * final_selectivity;

        let cost = Cost::new(io_cost, cpu_cost, 0.0, 0.0);
        let cardinality = Cardinality::with_confidence(output_rows, 0.8);

        (cost, cardinality)
    }

    // Bitmap Scan Cost
    fn cost_bitmap_scan(&self, table: &str, predicates: &[ScalarExpr]) -> (Cost, Cardinality) {
        let stats = self.get_table_stats(table);
        let total_rows = stats.row_count as f64;
        let total_pages = stats.page_count as f64;

        let selectivity = self.estimate_selectivity(predicates, table);
        let selected_rows = total_rows * selectivity;
        let selected_pages = (total_pages * selectivity).min(total_pages);

        // Bitmap creation cost
        let bitmap_cpu = total_rows * self.config.cpu_index_tuple_cost;

        // I/O: sequential scan of selected pages
        let io_cost = selected_pages * self.config.seq_page_cost;

        // CPU: tuple processing
        let cpu_cost = bitmap_cpu + selected_rows * self.config.cpu_tuple_cost;

        let cost = Cost::new(io_cost, cpu_cost, 0.0, 0.0);
        let cardinality = Cardinality::with_confidence(selected_rows, 0.7);

        (cost, cardinality)
    }

    // Filter Cost
    fn cost_filter(
        &self,
        predicates: &[ScalarExpr],
        child: Option<&PhysicalNode>,
    ) -> (Cost, Cardinality) {
        let child = child.expect("Filter requires child");
        let child_rows = child.cardinality.rows;

        // CPU cost: evaluate predicates for each tuple
        let cpu_per_tuple =
            self.config.cpu_tuple_cost + predicates.len() as f64 * self.config.cpu_operator_cost;
        let cpu_cost = child_rows * cpu_per_tuple;

        // Estimate output cardinality
        let selectivity = 0.1_f64.powi(predicates.len() as i32); // Simplified
        let output_rows = child_rows * selectivity;

        let cost = child.cost.add(&Cost::new(0.0, cpu_cost, 0.0, 0.0));
        let cardinality = Cardinality::with_confidence(output_rows, 0.6);

        (cost, cardinality)
    }

    // Project Cost
    fn cost_project(
        &self,
        projections: &[ProjectionItem],
        child: Option<&PhysicalNode>,
    ) -> (Cost, Cardinality) {
        let child = child.expect("Project requires child");
        let child_rows = child.cardinality.rows;

        // CPU cost: evaluate projections
        let cpu_cost = child_rows
            * (self.config.cpu_tuple_cost
                + projections.len() as f64 * self.config.cpu_operator_cost);

        let cost = child.cost.add(&Cost::new(0.0, cpu_cost, 0.0, 0.0));
        let cardinality = child.cardinality;

        (cost, cardinality)
    }

    // Nested Loop Join Cost
    fn cost_nested_loop_join(
        &self,
        _join_type: &JoinType,
        condition: &Option<ScalarExpr>,
        children: &[PhysicalNode],
    ) -> (Cost, Cardinality) {
        assert_eq!(children.len(), 2);
        let left = &children[0];
        let right = &children[1];

        let left_rows = left.cardinality.rows;
        let right_rows = right.cardinality.rows;

        // I/O cost: right side scanned once per left tuple
        let io_cost = left.cost.io_cost + left_rows * right.cost.io_cost;

        // CPU cost: comparisons for each tuple pair
        let comparisons = left_rows * right_rows;
        let cpu_cost = left.cost.cpu_cost
            + right.cost.cpu_cost * left_rows
            + comparisons * self.config.cpu_operator_cost;

        // Estimate join cardinality (simplified)
        let output_rows = left_rows * right_rows * 0.1; // 10% selectivity estimate

        let cost = Cost::new(io_cost, cpu_cost, 0.0, 0.0);
        let cardinality = Cardinality::with_confidence(output_rows, 0.5);

        (cost, cardinality)
    }

    // Hash Join Cost
    fn cost_hash_join(
        &self,
        _join_type: &JoinType,
        _left_keys: &[ScalarExpr],
        _right_keys: &[ScalarExpr],
        _condition: &Option<ScalarExpr>,
        children: &[PhysicalNode],
    ) -> (Cost, Cardinality) {
        assert_eq!(children.len(), 2);
        let left = &children[0];
        let right = &children[1];

        let left_rows = left.cardinality.rows;
        let right_rows = right.cardinality.rows;

        // Build hash table on smaller side
        let (build_side, probe_side) = if left_rows < right_rows {
            (left, right)
        } else {
            (right, left)
        };

        let build_rows = build_side.cardinality.rows;
        let probe_rows = probe_side.cardinality.rows;

        // I/O cost: scan both sides once
        let io_cost = left.cost.io_cost + right.cost.io_cost;

        // CPU cost: build hash table + probe
        let build_cpu = build_rows * (self.config.cpu_tuple_cost + self.config.cpu_operator_cost);
        let probe_cpu = probe_rows * (self.config.cpu_tuple_cost + self.config.cpu_operator_cost);
        let cpu_cost = left.cost.cpu_cost + right.cost.cpu_cost + build_cpu + probe_cpu;

        // Memory cost: hash table
        let avg_tuple_size = 100.0; // bytes
        let hash_table_size = build_rows * avg_tuple_size;
        let memory_cost = hash_table_size * self.config.memory_byte_cost;

        // Estimate output cardinality
        let output_rows = (left_rows * right_rows).sqrt(); // Geometric mean approximation

        let cost = Cost::new(io_cost, cpu_cost, 0.0, memory_cost);
        let cardinality = Cardinality::with_confidence(output_rows, 0.6);

        (cost, cardinality)
    }

    // Merge Join Cost
    fn cost_merge_join(
        &self,
        _join_type: &JoinType,
        _left_keys: &[ScalarExpr],
        _right_keys: &[ScalarExpr],
        _condition: &Option<ScalarExpr>,
        children: &[PhysicalNode],
    ) -> (Cost, Cardinality) {
        assert_eq!(children.len(), 2);
        let left = &children[0];
        let right = &children[1];

        let left_rows = left.cardinality.rows;
        let right_rows = right.cardinality.rows;

        // Assumes inputs are sorted
        let io_cost = left.cost.io_cost + right.cost.io_cost;

        // CPU cost: merge scan
        let cpu_cost = left.cost.cpu_cost
            + right.cost.cpu_cost
            + (left_rows + right_rows) * self.config.cpu_tuple_cost;

        // Minimal memory (just buffers)
        let memory_cost = self.config.page_size as f64 * 2.0 * self.config.memory_byte_cost;

        let output_rows = (left_rows * right_rows).sqrt();

        let cost = Cost::new(io_cost, cpu_cost, 0.0, memory_cost);
        let cardinality = Cardinality::with_confidence(output_rows, 0.6);

        (cost, cardinality)
    }

    // Hash Aggregate Cost
    fn cost_hash_aggregate(
        &self,
        group_by: &[ScalarExpr],
        aggregates: &[AggregateFunction],
        child: Option<&PhysicalNode>,
    ) -> (Cost, Cardinality) {
        let child = child.expect("Aggregate requires child");
        let input_rows = child.cardinality.rows;

        // CPU cost: hash grouping + aggregate computation
        let cpu_per_tuple = self.config.cpu_tuple_cost
            + group_by.len() as f64 * self.config.cpu_operator_cost
            + aggregates.len() as f64 * self.config.cpu_operator_cost * 2.0;

        let cpu_cost = child.cost.cpu_cost + input_rows * cpu_per_tuple;

        // Memory cost: hash table for groups
        let estimated_groups = (input_rows * 0.1).max(1.0); // 10% unique groups
        let group_size = 100.0; // bytes per group
        let memory_cost = estimated_groups * group_size * self.config.memory_byte_cost;

        let cost = Cost::new(child.cost.io_cost, cpu_cost, 0.0, memory_cost);
        let cardinality = Cardinality::with_confidence(estimated_groups, 0.5);

        (cost, cardinality)
    }

    // Sort Aggregate Cost
    fn cost_sort_aggregate(
        &self,
        group_by: &[ScalarExpr],
        aggregates: &[AggregateFunction],
        child: Option<&PhysicalNode>,
    ) -> (Cost, Cardinality) {
        let child = child.expect("Aggregate requires child");

        // Add sort cost first
        let (sort_cost, sort_card) = self.cost_sort(
            &group_by
                .iter()
                .map(|e| OrderByItem {
                    expr: e.clone(),
                    direction: SortDirection::Ascending,
                    nulls_first: false,
                })
                .collect::<Vec<_>>(),
            Some(child),
        );

        let input_rows = sort_card.rows;

        // Then aggregate (sequential scan of sorted data)
        let cpu_cost = sort_cost.cpu_cost
            + input_rows
                * (self.config.cpu_tuple_cost
                    + aggregates.len() as f64 * self.config.cpu_operator_cost);

        let estimated_groups = (input_rows * 0.1).max(1.0);

        let cost = Cost::new(sort_cost.io_cost, cpu_cost, 0.0, sort_cost.memory_cost);
        let cardinality = Cardinality::with_confidence(estimated_groups, 0.5);

        (cost, cardinality)
    }

    // Sort Cost
    fn cost_sort(
        &self,
        _order_by: &[OrderByItem],
        child: Option<&PhysicalNode>,
    ) -> (Cost, Cardinality) {
        let child = child.expect("Sort requires child");
        let input_rows = child.cardinality.rows;

        // Check if in-memory or external sort
        let tuple_size = 100.0; // bytes
        let total_size = input_rows * tuple_size;
        let fits_in_memory = total_size < self.config.work_mem as f64;

        let (io_cost, cpu_cost, memory_cost) = if fits_in_memory {
            // In-memory sort (quicksort)
            let comparisons = input_rows * input_rows.log2();
            let cpu = child.cost.cpu_cost + comparisons * self.config.cpu_operator_cost;
            let mem = total_size * self.config.memory_byte_cost;
            (child.cost.io_cost, cpu, mem)
        } else {
            // External sort (merge sort)
            let num_passes = (total_size / self.config.work_mem as f64).log2().ceil();
            let io = child.cost.io_cost + num_passes * (input_rows * tuple_size / self.config.page_size as f64) * self.config.seq_page_cost * 2.0;
            let comparisons = input_rows * input_rows.log2();
            let cpu = child.cost.cpu_cost + comparisons * self.config.cpu_operator_cost;
            let mem = self.config.work_mem as f64 * self.config.memory_byte_cost;
            (io, cpu, mem)
        };

        let cost = Cost::new(io_cost, cpu_cost, 0.0, memory_cost);
        let cardinality = child.cardinality;

        (cost, cardinality)
    }

    // Top-N Sort Cost (optimized for LIMIT)
    fn cost_topn_sort(
        &self,
        _order_by: &[OrderByItem],
        limit: u64,
        child: Option<&PhysicalNode>,
    ) -> (Cost, Cardinality) {
        let child = child.expect("TopNSort requires child");
        let input_rows = child.cardinality.rows;
        let n = limit as f64;

        // Heap-based top-N
        let cpu_cost =
            child.cost.cpu_cost + input_rows * n.log2() * self.config.cpu_operator_cost;

        let tuple_size = 100.0;
        let memory_cost = n * tuple_size * self.config.memory_byte_cost;

        let cost = Cost::new(child.cost.io_cost, cpu_cost, 0.0, memory_cost);
        let cardinality = Cardinality::with_confidence(n.min(input_rows), 0.9);

        (cost, cardinality)
    }

    // Limit Cost
    fn cost_limit(
        &self,
        limit: Option<u64>,
        offset: Option<u64>,
        child: Option<&PhysicalNode>,
    ) -> (Cost, Cardinality) {
        let child = child.expect("Limit requires child");

        let offset_rows = offset.unwrap_or(0) as f64;
        let limit_rows = limit.map(|l| l as f64).unwrap_or(child.cardinality.rows);
        let total_rows = offset_rows + limit_rows;

        // Proportional cost based on how much we actually read
        let fraction = (total_rows / child.cardinality.rows).min(1.0);
        let cost = child.cost.multiply(fraction);
        let cardinality = Cardinality::with_confidence(limit_rows.min(child.cardinality.rows), 0.95);

        (cost, cardinality)
    }

    // Hash Distinct Cost
    fn cost_hash_distinct(&self, child: Option<&PhysicalNode>) -> (Cost, Cardinality) {
        let child = child.expect("Distinct requires child");
        let input_rows = child.cardinality.rows;

        // Similar to hash aggregate
        let cpu_cost = child.cost.cpu_cost + input_rows * self.config.cpu_tuple_cost * 1.5;

        let distinct_rows = input_rows * 0.5; // 50% distinct estimate
        let memory_cost = distinct_rows * 100.0 * self.config.memory_byte_cost;

        let cost = Cost::new(child.cost.io_cost, cpu_cost, 0.0, memory_cost);
        let cardinality = Cardinality::with_confidence(distinct_rows, 0.4);

        (cost, cardinality)
    }

    // Sort Distinct Cost
    fn cost_sort_distinct(&self, child: Option<&PhysicalNode>) -> (Cost, Cardinality) {
        let child = child.expect("Distinct requires child");

        // Sort + sequential scan
        let (sort_cost, sort_card) = self.cost_sort(&[], Some(child));

        let cpu_cost = sort_cost.cpu_cost + sort_card.rows * self.config.cpu_tuple_cost;
        let distinct_rows = sort_card.rows * 0.5;

        let cost = Cost::new(sort_cost.io_cost, cpu_cost, 0.0, sort_cost.memory_cost);
        let cardinality = Cardinality::with_confidence(distinct_rows, 0.4);

        (cost, cardinality)
    }

    // Union All Cost
    fn cost_union_all(&self, children: &[PhysicalNode]) -> (Cost, Cardinality) {
        let mut total_cost = Cost::zero();
        let mut total_rows = 0.0;

        for child in children {
            total_cost = total_cost.add(&child.cost);
            total_rows += child.cardinality.rows;
        }

        (total_cost, Cardinality::with_confidence(total_rows, 0.9))
    }

    // Hash Union Cost
    fn cost_hash_union(&self, children: &[PhysicalNode]) -> (Cost, Cardinality) {
        let (union_cost, union_card) = self.cost_union_all(children);

        // Add deduplication cost
        let cpu_cost = union_cost.cpu_cost + union_card.rows * self.config.cpu_tuple_cost;
        let memory_cost = union_card.rows * 100.0 * self.config.memory_byte_cost * 0.5;

        let distinct_rows = union_card.rows * 0.7; // 70% after dedup

        let cost = Cost::new(union_cost.io_cost, cpu_cost, 0.0, memory_cost);
        let cardinality = Cardinality::with_confidence(distinct_rows, 0.6);

        (cost, cardinality)
    }

    // Gather Cost (parallel)
    fn cost_gather(&self, num_workers: usize, child: Option<&PhysicalNode>) -> (Cost, Cardinality) {
        let child = child.expect("Gather requires child");

        // Child cost is divided among workers
        let parallel_cost = child.cost.multiply(1.0 / num_workers as f64);

        // Add network cost for gathering results
        let tuple_size = 100.0;
        let network_bytes = child.cardinality.rows * tuple_size;
        let network_cost = network_bytes * self.config.network_byte_cost;

        let cost = Cost::new(
            parallel_cost.io_cost,
            parallel_cost.cpu_cost,
            network_cost,
            parallel_cost.memory_cost,
        );

        (cost, child.cardinality)
    }

    // Exchange Cost (parallel distribution)
    fn cost_exchange(
        &self,
        _distribution: &Distribution,
        num_partitions: usize,
        child: Option<&PhysicalNode>,
    ) -> (Cost, Cardinality) {
        let child = child.expect("Exchange requires child");

        // Network cost for redistributing data
        let tuple_size = 100.0;
        let network_bytes = child.cardinality.rows * tuple_size;
        let network_cost = network_bytes * self.config.network_byte_cost;

        // CPU cost for partitioning
        let cpu_cost = child.cost.cpu_cost
            + child.cardinality.rows * self.config.cpu_operator_cost * num_partitions as f64;

        let cost = Cost::new(child.cost.io_cost, cpu_cost, network_cost, 0.0);

        (cost, child.cardinality)
    }

    // Materialize Cost
    fn cost_materialize(&self, child: Option<&PhysicalNode>) -> (Cost, Cardinality) {
        let child = child.expect("Materialize requires child");

        // Write and read back data
        let tuple_size = 100.0;
        let pages = (child.cardinality.rows * tuple_size / self.config.page_size as f64).ceil();
        let io_cost = child.cost.io_cost + pages * self.config.seq_page_cost * 2.0; // write + read

        let cost = Cost::new(io_cost, child.cost.cpu_cost, 0.0, 0.0);

        (cost, child.cardinality)
    }

    // Helper: Get table statistics
    fn get_table_stats(&self, table: &str) -> TableStatistics {
        self.statistics
            .get(table)
            .cloned()
            .unwrap_or_else(|| TableStatistics::default_for_table(table))
    }

    // Helper: Estimate selectivity from predicates
    fn estimate_selectivity(&self, predicates: &[ScalarExpr], table: &str) -> f64 {
        if predicates.is_empty() {
            return 1.0;
        }

        // Simplified: assume each predicate reduces by 10%
        let per_predicate = 0.1;
        per_predicate.powi(predicates.len() as i32)
    }

    // Helper: Estimate index selectivity
    fn estimate_index_selectivity(&self, _key_conditions: &[ScalarExpr], _table: &str) -> f64 {
        // Simplified: assume index is selective
        0.01 // 1% selectivity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_config_default() {
        let config = CostConfig::default();
        assert!(config.seq_page_cost > 0.0);
        assert!(config.random_page_cost > config.seq_page_cost);
    }

    #[test]
    fn test_cost_estimator() {
        let estimator = CostEstimator::with_default_config();
        let (cost, card) = estimator.cost_seq_scan(
            "users",
            &[],
            &None,
        );
        assert!(cost.total_cost > 0.0);
        assert!(card.rows > 0.0);
    }
}
