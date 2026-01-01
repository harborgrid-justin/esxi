//! Query Plan Representation
//!
//! Logical and physical query execution plans with cost estimates.

use crate::ast::*;
use crate::statistics::TableStatistics;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Logical query plan - represents what to compute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalPlan {
    pub root: LogicalNode,
    pub properties: PlanProperties,
}

impl LogicalPlan {
    pub fn new(root: LogicalNode) -> Self {
        let properties = PlanProperties::default();
        Self { root, properties }
    }

    pub fn from_query(query: QueryExpr) -> Self {
        let root = LogicalNode::from_rel_expr(*query.root);
        Self::new(root)
    }
}

/// Logical plan node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalNode {
    pub id: NodeId,
    pub op: LogicalOp,
    pub children: Vec<LogicalNode>,
    pub schema: Schema,
}

impl LogicalNode {
    pub fn new(op: LogicalOp, children: Vec<LogicalNode>, schema: Schema) -> Self {
        Self {
            id: NodeId::new(),
            op,
            children,
            schema,
        }
    }

    pub fn from_rel_expr(expr: RelExpr) -> Self {
        match expr {
            RelExpr::Scan(scan) => LogicalNode::new(
                LogicalOp::Scan {
                    table: scan.table_name.clone(),
                    alias: scan.alias.clone(),
                    predicates: scan.predicates.clone(),
                    projection: scan.projection.clone(),
                },
                vec![],
                scan.schema.clone(),
            ),

            RelExpr::Filter(filter) => {
                let child = Self::from_rel_expr(*filter.input);
                let schema = child.schema.clone();
                LogicalNode::new(
                    LogicalOp::Filter {
                        predicates: filter.predicates,
                    },
                    vec![child],
                    schema,
                )
            }

            RelExpr::Project(project) => {
                let child = Self::from_rel_expr(*project.input);
                // Schema would be derived from projections
                let schema = child.schema.clone();
                LogicalNode::new(
                    LogicalOp::Project {
                        projections: project.projections,
                    },
                    vec![child],
                    schema,
                )
            }

            RelExpr::Join(join) => {
                let left = Self::from_rel_expr(*join.left);
                let right = Self::from_rel_expr(*join.right);
                // Schema would be merged from left and right
                let schema = left.schema.clone();
                LogicalNode::new(
                    LogicalOp::Join {
                        join_type: join.join_type,
                        condition: join.condition,
                    },
                    vec![left, right],
                    schema,
                )
            }

            RelExpr::Aggregate(agg) => {
                let child = Self::from_rel_expr(*agg.input);
                let schema = child.schema.clone();
                LogicalNode::new(
                    LogicalOp::Aggregate {
                        group_by: agg.group_by,
                        aggregates: agg.aggregates,
                        having: agg.having,
                    },
                    vec![child],
                    schema,
                )
            }

            RelExpr::Sort(sort) => {
                let child = Self::from_rel_expr(*sort.input);
                let schema = child.schema.clone();
                LogicalNode::new(
                    LogicalOp::Sort {
                        order_by: sort.order_by,
                    },
                    vec![child],
                    schema,
                )
            }

            RelExpr::Limit(limit) => {
                let child = Self::from_rel_expr(*limit.input);
                let schema = child.schema.clone();
                LogicalNode::new(
                    LogicalOp::Limit {
                        limit: limit.limit,
                        offset: limit.offset,
                    },
                    vec![child],
                    schema,
                )
            }

            RelExpr::Union(union) => {
                let left = Self::from_rel_expr(*union.left);
                let right = Self::from_rel_expr(*union.right);
                let schema = left.schema.clone();
                LogicalNode::new(
                    LogicalOp::Union { all: union.all },
                    vec![left, right],
                    schema,
                )
            }

            RelExpr::Distinct(distinct) => {
                let child = Self::from_rel_expr(*distinct.input);
                let schema = child.schema.clone();
                LogicalNode::new(LogicalOp::Distinct, vec![child], schema)
            }

            RelExpr::Subquery(subquery) => {
                let child = Self::from_rel_expr(*subquery.query);
                let schema = child.schema.clone();
                LogicalNode::new(
                    LogicalOp::Subquery {
                        correlated: subquery.correlated,
                    },
                    vec![child],
                    schema,
                )
            }
        }
    }
}

/// Logical operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicalOp {
    Scan {
        table: String,
        alias: Option<String>,
        predicates: Vec<ScalarExpr>,
        projection: Option<Vec<String>>,
    },
    Filter {
        predicates: Vec<ScalarExpr>,
    },
    Project {
        projections: Vec<ProjectionItem>,
    },
    Join {
        join_type: JoinType,
        condition: Option<ScalarExpr>,
    },
    Aggregate {
        group_by: Vec<ScalarExpr>,
        aggregates: Vec<AggregateFunction>,
        having: Option<ScalarExpr>,
    },
    Sort {
        order_by: Vec<OrderByItem>,
    },
    Limit {
        limit: Option<u64>,
        offset: Option<u64>,
    },
    Union {
        all: bool,
    },
    Distinct,
    Subquery {
        correlated: bool,
    },
}

/// Physical query plan - represents how to compute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalPlan {
    pub root: PhysicalNode,
    pub properties: PlanProperties,
    pub estimated_cost: Cost,
}

impl PhysicalPlan {
    pub fn new(root: PhysicalNode, estimated_cost: Cost) -> Self {
        let properties = PlanProperties::default();
        Self {
            root,
            properties,
            estimated_cost,
        }
    }
}

/// Physical plan node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalNode {
    pub id: NodeId,
    pub op: PhysicalOp,
    pub children: Vec<PhysicalNode>,
    pub schema: Schema,
    pub cost: Cost,
    pub cardinality: Cardinality,
}

impl PhysicalNode {
    pub fn new(
        op: PhysicalOp,
        children: Vec<PhysicalNode>,
        schema: Schema,
        cost: Cost,
        cardinality: Cardinality,
    ) -> Self {
        Self {
            id: NodeId::new(),
            op,
            children,
            schema,
            cost,
            cardinality,
        }
    }
}

/// Physical operators - specific implementation algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhysicalOp {
    /// Sequential scan
    SeqScan {
        table: String,
        alias: Option<String>,
        predicates: Vec<ScalarExpr>,
        projection: Option<Vec<String>>,
    },

    /// Index scan
    IndexScan {
        table: String,
        index_name: String,
        key_conditions: Vec<ScalarExpr>,
        predicates: Vec<ScalarExpr>,
        projection: Option<Vec<String>>,
    },

    /// Bitmap index scan (multiple indexes combined)
    BitmapScan {
        table: String,
        index_names: Vec<String>,
        predicates: Vec<ScalarExpr>,
    },

    /// Filter operator
    Filter {
        predicates: Vec<ScalarExpr>,
    },

    /// Projection operator
    Project {
        projections: Vec<ProjectionItem>,
    },

    /// Nested loop join
    NestedLoopJoin {
        join_type: JoinType,
        condition: Option<ScalarExpr>,
    },

    /// Hash join
    HashJoin {
        join_type: JoinType,
        left_keys: Vec<ScalarExpr>,
        right_keys: Vec<ScalarExpr>,
        condition: Option<ScalarExpr>,
    },

    /// Merge join (sort-merge)
    MergeJoin {
        join_type: JoinType,
        left_keys: Vec<ScalarExpr>,
        right_keys: Vec<ScalarExpr>,
        condition: Option<ScalarExpr>,
    },

    /// Hash aggregation
    HashAggregate {
        group_by: Vec<ScalarExpr>,
        aggregates: Vec<AggregateFunction>,
        having: Option<ScalarExpr>,
    },

    /// Sort-based aggregation
    SortAggregate {
        group_by: Vec<ScalarExpr>,
        aggregates: Vec<AggregateFunction>,
        having: Option<ScalarExpr>,
    },

    /// External sort (disk-based)
    Sort {
        order_by: Vec<OrderByItem>,
    },

    /// Top-N sort (in-memory, limited)
    TopNSort {
        order_by: Vec<OrderByItem>,
        limit: u64,
    },

    /// Limit operator
    Limit {
        limit: Option<u64>,
        offset: Option<u64>,
    },

    /// Hash-based distinct
    HashDistinct,

    /// Sort-based distinct
    SortDistinct,

    /// Union all
    UnionAll,

    /// Hash union (deduplicated)
    HashUnion,

    /// Parallel gather (collect from parallel workers)
    Gather {
        num_workers: usize,
    },

    /// Parallel scatter (distribute to workers)
    Exchange {
        distribution: Distribution,
        num_partitions: usize,
    },

    /// Materialize intermediate results
    Materialize,
}

/// Data distribution strategy for parallel execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Distribution {
    /// Replicate to all workers
    Broadcast,
    /// Hash partition by keys
    Hash(Vec<ScalarExpr>),
    /// Range partition by keys
    Range(Vec<ScalarExpr>),
    /// Round-robin distribution
    RoundRobin,
    /// Single partition (no distribution)
    Single,
}

/// Plan properties and metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlanProperties {
    /// Whether the plan output is sorted
    pub sorted: Option<Vec<OrderByItem>>,
    /// Data distribution
    pub distribution: Option<Distribution>,
    /// Estimated selectivity
    pub selectivity: Option<f64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Cost estimation
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Cost {
    /// I/O cost (disk reads/writes)
    pub io_cost: f64,
    /// CPU cost (processing)
    pub cpu_cost: f64,
    /// Network cost (data transfer)
    pub network_cost: f64,
    /// Memory cost (RAM usage)
    pub memory_cost: f64,
    /// Total cost (weighted sum)
    pub total_cost: f64,
}

impl Cost {
    pub fn new(io: f64, cpu: f64, network: f64, memory: f64) -> Self {
        // Cost weights (tunable based on hardware profile)
        const IO_WEIGHT: f64 = 1.0;
        const CPU_WEIGHT: f64 = 0.1;
        const NETWORK_WEIGHT: f64 = 0.5;
        const MEMORY_WEIGHT: f64 = 0.2;

        let total = io * IO_WEIGHT
            + cpu * CPU_WEIGHT
            + network * NETWORK_WEIGHT
            + memory * MEMORY_WEIGHT;

        Self {
            io_cost: io,
            cpu_cost: cpu,
            network_cost: network,
            memory_cost: memory,
            total_cost: total,
        }
    }

    pub fn zero() -> Self {
        Self::default()
    }

    pub fn add(&self, other: &Cost) -> Cost {
        Cost::new(
            self.io_cost + other.io_cost,
            self.cpu_cost + other.cpu_cost,
            self.network_cost + other.network_cost,
            self.memory_cost + other.memory_cost,
        )
    }

    pub fn multiply(&self, factor: f64) -> Cost {
        Cost::new(
            self.io_cost * factor,
            self.cpu_cost * factor,
            self.network_cost * factor,
            self.memory_cost * factor,
        )
    }
}

impl fmt::Display for Cost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cost(total={:.2}, io={:.2}, cpu={:.2}, net={:.2}, mem={:.2})",
            self.total_cost, self.io_cost, self.cpu_cost, self.network_cost, self.memory_cost
        )
    }
}

/// Cardinality estimation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Cardinality {
    /// Estimated number of rows
    pub rows: f64,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
}

impl Cardinality {
    pub fn new(rows: f64) -> Self {
        Self {
            rows,
            confidence: 0.5,
        }
    }

    pub fn with_confidence(rows: f64, confidence: f64) -> Self {
        Self { rows, confidence }
    }

    pub fn unknown() -> Self {
        Self {
            rows: 1000.0, // Default estimate
            confidence: 0.0,
        }
    }
}

impl Default for Cardinality {
    fn default() -> Self {
        Self::unknown()
    }
}

/// Plan comparison for optimization
impl PhysicalPlan {
    pub fn is_better_than(&self, other: &PhysicalPlan) -> bool {
        self.estimated_cost.total_cost < other.estimated_cost.total_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculation() {
        let cost = Cost::new(100.0, 50.0, 20.0, 10.0);
        assert!(cost.total_cost > 0.0);
        assert_eq!(cost.io_cost, 100.0);
    }

    #[test]
    fn test_cost_addition() {
        let cost1 = Cost::new(100.0, 50.0, 20.0, 10.0);
        let cost2 = Cost::new(50.0, 25.0, 10.0, 5.0);
        let total = cost1.add(&cost2);
        assert_eq!(total.io_cost, 150.0);
        assert_eq!(total.cpu_cost, 75.0);
    }

    #[test]
    fn test_cardinality() {
        let card = Cardinality::new(1000.0);
        assert_eq!(card.rows, 1000.0);
        assert!(card.confidence > 0.0);
    }

    #[test]
    fn test_plan_comparison() {
        let plan1 = PhysicalPlan::new(
            PhysicalNode::new(
                PhysicalOp::SeqScan {
                    table: "users".to_string(),
                    alias: None,
                    predicates: vec![],
                    projection: None,
                },
                vec![],
                Schema::empty(),
                Cost::new(100.0, 50.0, 0.0, 10.0),
                Cardinality::new(1000.0),
            ),
            Cost::new(100.0, 50.0, 0.0, 10.0),
        );

        let plan2 = PhysicalPlan::new(
            PhysicalNode::new(
                PhysicalOp::IndexScan {
                    table: "users".to_string(),
                    index_name: "idx_id".to_string(),
                    key_conditions: vec![],
                    predicates: vec![],
                    projection: None,
                },
                vec![],
                Schema::empty(),
                Cost::new(10.0, 20.0, 0.0, 5.0),
                Cardinality::new(100.0),
            ),
            Cost::new(10.0, 20.0, 0.0, 5.0),
        );

        assert!(plan2.is_better_than(&plan1));
    }
}
