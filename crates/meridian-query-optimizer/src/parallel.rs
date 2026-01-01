//! Parallel Query Execution Planning
//!
//! Determines when and how to parallelize query execution across multiple workers.

use crate::plan::*;
use serde::{Deserialize, Serialize};

/// Parallel execution configuration
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Maximum number of worker processes
    pub max_workers: usize,
    /// Minimum table size (rows) to consider parallelization
    pub min_parallel_table_scan_size: u64,
    /// Minimum cost to consider parallelization
    pub min_parallel_cost: f64,
    /// Cost per worker process
    pub parallel_setup_cost: f64,
    /// Cost per tuple for inter-process communication
    pub parallel_tuple_cost: f64,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_workers: num_cpus::get(),
            min_parallel_table_scan_size: 100_000,
            min_parallel_cost: 1000.0,
            parallel_setup_cost: 1000.0,
            parallel_tuple_cost: 0.001,
        }
    }
}

/// Parallel query planner
pub struct ParallelPlanner {
    config: ParallelConfig,
}

impl ParallelPlanner {
    pub fn new(config: ParallelConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(ParallelConfig::default())
    }

    /// Convert a physical plan to use parallel execution where beneficial
    pub fn parallelize_plan(&self, plan: PhysicalPlan) -> PhysicalPlan {
        if !self.should_parallelize(&plan) {
            return plan;
        }

        let parallel_root = self.parallelize_node(plan.root, 1);
        let estimated_cost = self.estimate_parallel_cost(&parallel_root);

        PhysicalPlan::new(parallel_root, estimated_cost)
    }

    /// Check if plan should be parallelized
    fn should_parallelize(&self, plan: &PhysicalPlan) -> bool {
        // Don't parallelize if cost is too low
        if plan.estimated_cost.total_cost < self.config.min_parallel_cost {
            return false;
        }

        // Don't parallelize if cardinality is too low
        if plan.root.cardinality.rows < self.config.min_parallel_table_scan_size as f64 {
            return false;
        }

        true
    }

    /// Parallelize a physical node
    fn parallelize_node(&self, node: PhysicalNode, level: usize) -> PhysicalNode {
        match node.op {
            // Parallelize table scans
            PhysicalOp::SeqScan { .. } if node.cardinality.rows >= self.config.min_parallel_table_scan_size as f64 => {
                self.create_parallel_scan(node)
            }

            // Parallelize aggregates
            PhysicalOp::HashAggregate { group_by, aggregates, having } => {
                self.create_parallel_aggregate(node, group_by, aggregates, having)
            }

            // Parallelize joins
            PhysicalOp::HashJoin { join_type, left_keys, right_keys, condition } => {
                self.create_parallel_join(node, join_type, left_keys, right_keys, condition)
            }

            // Recursively handle other operators
            _ => {
                let parallel_children = node
                    .children
                    .into_iter()
                    .map(|child| self.parallelize_node(child, level + 1))
                    .collect();

                PhysicalNode {
                    children: parallel_children,
                    ..node
                }
            }
        }
    }

    /// Create parallel table scan
    fn create_parallel_scan(&self, node: PhysicalNode) -> PhysicalNode {
        let num_workers = self.calculate_optimal_workers(node.cardinality.rows);

        // Partition scan across workers
        let scan_node = node.clone();

        // Gather results from workers
        PhysicalNode::new(
            PhysicalOp::Gather { num_workers },
            vec![scan_node],
            node.schema,
            node.cost,
            node.cardinality,
        )
    }

    /// Create parallel aggregate
    fn create_parallel_aggregate(
        &self,
        node: PhysicalNode,
        group_by: Vec<ScalarExpr>,
        aggregates: Vec<AggregateFunction>,
        having: Option<ScalarExpr>,
    ) -> PhysicalNode {
        let num_workers = self.calculate_optimal_workers(node.cardinality.rows);

        if node.children.is_empty() {
            return node;
        }

        let child = node.children[0].clone();

        // Partial aggregate in workers
        let partial_agg = PhysicalNode::new(
            PhysicalOp::HashAggregate {
                group_by: group_by.clone(),
                aggregates: aggregates.clone(),
                having: None, // Apply HAVING after final aggregate
            },
            vec![child],
            node.schema.clone(),
            node.cost,
            node.cardinality,
        );

        // Gather partial results
        let gather = PhysicalNode::new(
            PhysicalOp::Gather { num_workers },
            vec![partial_agg],
            node.schema.clone(),
            node.cost,
            node.cardinality,
        );

        // Final aggregate
        PhysicalNode::new(
            PhysicalOp::HashAggregate {
                group_by,
                aggregates,
                having,
            },
            vec![gather],
            node.schema,
            node.cost,
            node.cardinality,
        )
    }

    /// Create parallel hash join
    fn create_parallel_join(
        &self,
        node: PhysicalNode,
        join_type: JoinType,
        left_keys: Vec<ScalarExpr>,
        right_keys: Vec<ScalarExpr>,
        condition: Option<ScalarExpr>,
    ) -> PhysicalNode {
        if node.children.len() != 2 {
            return node;
        }

        let num_workers = self.calculate_optimal_workers(
            node.children[0].cardinality.rows + node.children[1].cardinality.rows,
        );

        let left = node.children[0].clone();
        let right = node.children[1].clone();

        // Partition both sides on join keys
        let left_exchange = PhysicalNode::new(
            PhysicalOp::Exchange {
                distribution: Distribution::Hash(left_keys.clone()),
                num_partitions: num_workers,
            },
            vec![left],
            node.children[0].schema.clone(),
            node.children[0].cost,
            node.children[0].cardinality,
        );

        let right_exchange = PhysicalNode::new(
            PhysicalOp::Exchange {
                distribution: Distribution::Hash(right_keys.clone()),
                num_partitions: num_workers,
            },
            vec![right],
            node.children[1].schema.clone(),
            node.children[1].cost,
            node.children[1].cardinality,
        );

        // Parallel join in workers
        let parallel_join = PhysicalNode::new(
            PhysicalOp::HashJoin {
                join_type,
                left_keys,
                right_keys,
                condition,
            },
            vec![left_exchange, right_exchange],
            node.schema.clone(),
            node.cost,
            node.cardinality,
        );

        // Gather results
        PhysicalNode::new(
            PhysicalOp::Gather { num_workers },
            vec![parallel_join],
            node.schema,
            node.cost,
            node.cardinality,
        )
    }

    /// Calculate optimal number of workers
    fn calculate_optimal_workers(&self, estimated_rows: f64) -> usize {
        // Simple heuristic: more rows = more workers
        let workers = (estimated_rows / 100_000.0).sqrt().ceil() as usize;
        workers.clamp(2, self.config.max_workers)
    }

    /// Estimate cost of parallel execution
    fn estimate_parallel_cost(&self, node: &PhysicalNode) -> Cost {
        // Simplified: just return node cost
        // In production, would account for parallelization overhead
        node.cost
    }
}

/// Parallel execution strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParallelStrategy {
    /// No parallelization
    Serial,

    /// Parallel scan with gather
    ParallelScan {
        num_workers: usize,
    },

    /// Parallel aggregate
    ParallelAggregate {
        num_workers: usize,
        partial_aggregate: bool,
    },

    /// Parallel join
    ParallelJoin {
        num_workers: usize,
        distribution: Distribution,
    },

    /// Parallel union
    ParallelUnion {
        num_workers: usize,
    },
}

/// Work distribution analyzer
pub struct WorkDistributionAnalyzer {
    config: ParallelConfig,
}

impl WorkDistributionAnalyzer {
    pub fn new(config: ParallelConfig) -> Self {
        Self { config }
    }

    /// Analyze data skew in distribution
    pub fn analyze_skew(&self, cardinality: &Cardinality, num_partitions: usize) -> SkewMetrics {
        let avg_partition_size = cardinality.rows / num_partitions as f64;

        // Simplified: assume uniform distribution
        SkewMetrics {
            min_partition_size: avg_partition_size * 0.8,
            max_partition_size: avg_partition_size * 1.2,
            avg_partition_size,
            skew_factor: 1.2, // max / avg
            is_skewed: false,
        }
    }

    /// Recommend distribution strategy
    pub fn recommend_distribution(
        &self,
        op: &PhysicalOp,
        children: &[PhysicalNode],
    ) -> Distribution {
        match op {
            PhysicalOp::HashJoin { left_keys, .. } => {
                Distribution::Hash(left_keys.clone())
            }
            PhysicalOp::HashAggregate { group_by, .. } => {
                Distribution::Hash(group_by.clone())
            }
            PhysicalOp::UnionAll => Distribution::RoundRobin,
            _ => Distribution::Single,
        }
    }
}

/// Metrics for data skew analysis
#[derive(Debug, Clone)]
pub struct SkewMetrics {
    pub min_partition_size: f64,
    pub max_partition_size: f64,
    pub avg_partition_size: f64,
    pub skew_factor: f64,
    pub is_skewed: bool,
}

/// Parallel execution coordinator
pub struct ParallelCoordinator {
    num_workers: usize,
    work_queue: Vec<WorkItem>,
}

impl ParallelCoordinator {
    pub fn new(num_workers: usize) -> Self {
        Self {
            num_workers,
            work_queue: Vec::new(),
        }
    }

    pub fn add_work(&mut self, item: WorkItem) {
        self.work_queue.push(item);
    }

    pub fn schedule_work(&mut self) -> Vec<Vec<WorkItem>> {
        let mut worker_queues = vec![Vec::new(); self.num_workers];

        // Simple round-robin scheduling
        for (i, item) in self.work_queue.drain(..).enumerate() {
            let worker_id = i % self.num_workers;
            worker_queues[worker_id].push(item);
        }

        worker_queues
    }
}

/// Work item for parallel execution
#[derive(Debug, Clone)]
pub struct WorkItem {
    pub id: String,
    pub partition_id: usize,
    pub estimated_cost: f64,
    pub estimated_rows: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_config() {
        let config = ParallelConfig::default();
        assert!(config.max_workers > 0);
        assert!(config.min_parallel_table_scan_size > 0);
    }

    #[test]
    fn test_parallel_planner() {
        let planner = ParallelPlanner::with_default_config();
        assert!(planner.config.max_workers > 0);
    }

    #[test]
    fn test_calculate_optimal_workers() {
        let planner = ParallelPlanner::with_default_config();
        let workers = planner.calculate_optimal_workers(1_000_000.0);
        assert!(workers >= 2);
        assert!(workers <= planner.config.max_workers);
    }

    #[test]
    fn test_work_distribution_analyzer() {
        let analyzer = WorkDistributionAnalyzer::new(ParallelConfig::default());
        let metrics = analyzer.analyze_skew(&Cardinality::new(1000.0), 4);
        assert!(metrics.avg_partition_size > 0.0);
    }

    #[test]
    fn test_parallel_coordinator() {
        let mut coordinator = ParallelCoordinator::new(4);
        coordinator.add_work(WorkItem {
            id: "task1".to_string(),
            partition_id: 0,
            estimated_cost: 100.0,
            estimated_rows: 1000.0,
        });

        let scheduled = coordinator.schedule_work();
        assert_eq!(scheduled.len(), 4);
    }
}
