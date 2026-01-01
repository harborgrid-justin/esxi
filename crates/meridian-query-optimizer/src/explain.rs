//! EXPLAIN Output Generation
//!
//! Generates human-readable query execution plans with cost breakdowns.

use crate::plan::*;
use std::fmt::Write;

/// EXPLAIN output format
#[derive(Debug, Clone, Copy)]
pub enum ExplainFormat {
    /// Text format (traditional)
    Text,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// Graphviz DOT format
    Dot,
}

/// EXPLAIN options
#[derive(Debug, Clone)]
pub struct ExplainOptions {
    pub format: ExplainFormat,
    pub verbose: bool,
    pub costs: bool,
    pub buffers: bool,
    pub timing: bool,
    pub analyze: bool,
}

impl Default for ExplainOptions {
    fn default() -> Self {
        Self {
            format: ExplainFormat::Text,
            verbose: false,
            costs: true,
            buffers: false,
            timing: false,
            analyze: false,
        }
    }
}

/// EXPLAIN plan formatter
pub struct ExplainFormatter {
    options: ExplainOptions,
}

impl ExplainFormatter {
    pub fn new(options: ExplainOptions) -> Self {
        Self { options }
    }

    pub fn with_default_options() -> Self {
        Self::new(ExplainOptions::default())
    }

    /// Format a physical plan as EXPLAIN output
    pub fn format_plan(&self, plan: &PhysicalPlan) -> String {
        match self.options.format {
            ExplainFormat::Text => self.format_text(plan),
            ExplainFormat::Json => self.format_json(plan),
            ExplainFormat::Yaml => self.format_yaml(plan),
            ExplainFormat::Dot => self.format_dot(plan),
        }
    }

    /// Format as text (tree structure)
    fn format_text(&self, plan: &PhysicalPlan) -> String {
        let mut output = String::new();

        if self.options.costs {
            writeln!(
                &mut output,
                "Query Cost: {}",
                plan.estimated_cost
            )
            .unwrap();
            writeln!(&mut output).unwrap();
        }

        self.format_node_text(&plan.root, 0, &mut output);
        output
    }

    fn format_node_text(&self, node: &PhysicalNode, indent: usize, output: &mut String) {
        let indent_str = "  ".repeat(indent);

        // Node operator
        write!(output, "{}", indent_str).unwrap();
        self.write_operator(node, output);

        if self.options.costs {
            write!(output, " (cost={})", node.cost).unwrap();
        }

        if self.options.verbose {
            write!(
                output,
                " [rows={:.0}, confidence={:.2}]",
                node.cardinality.rows, node.cardinality.confidence
            )
            .unwrap();
        }

        writeln!(output).unwrap();

        // Schema (if verbose)
        if self.options.verbose && !node.schema.columns.is_empty() {
            writeln!(
                &mut *output,
                "{}  Output: {}",
                indent_str,
                node.schema
                    .columns
                    .iter()
                    .map(|c| c.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .unwrap();
        }

        // Recurse on children
        for child in &node.children {
            self.format_node_text(child, indent + 1, output);
        }
    }

    fn write_operator(&self, node: &PhysicalNode, output: &mut String) {
        match &node.op {
            PhysicalOp::SeqScan {
                table,
                alias,
                predicates,
                projection,
            } => {
                write!(output, "Seq Scan on {}", table).unwrap();
                if let Some(a) = alias {
                    write!(output, " as {}", a).unwrap();
                }
                if !predicates.is_empty() && self.options.verbose {
                    write!(output, " [Filter: {} predicates]", predicates.len()).unwrap();
                }
            }

            PhysicalOp::IndexScan {
                table,
                index_name,
                key_conditions,
                predicates,
                ..
            } => {
                write!(output, "Index Scan using {} on {}", index_name, table).unwrap();
                if !key_conditions.is_empty() && self.options.verbose {
                    write!(output, " [Index Cond: {} conditions]", key_conditions.len()).unwrap();
                }
            }

            PhysicalOp::BitmapScan { table, .. } => {
                write!(output, "Bitmap Heap Scan on {}", table).unwrap();
            }

            PhysicalOp::Filter { predicates } => {
                write!(output, "Filter").unwrap();
                if self.options.verbose {
                    write!(output, " [{} predicates]", predicates.len()).unwrap();
                }
            }

            PhysicalOp::Project { projections } => {
                write!(output, "Project").unwrap();
                if self.options.verbose {
                    write!(output, " [{} columns]", projections.len()).unwrap();
                }
            }

            PhysicalOp::NestedLoopJoin { join_type, .. } => {
                write!(output, "Nested Loop {:?} Join", join_type).unwrap();
            }

            PhysicalOp::HashJoin { join_type, left_keys, .. } => {
                write!(output, "Hash {:?} Join", join_type).unwrap();
                if self.options.verbose && !left_keys.is_empty() {
                    write!(output, " [Hash Keys: {} keys]", left_keys.len()).unwrap();
                }
            }

            PhysicalOp::MergeJoin { join_type, left_keys, .. } => {
                write!(output, "Merge {:?} Join", join_type).unwrap();
                if self.options.verbose && !left_keys.is_empty() {
                    write!(output, " [Merge Keys: {} keys]", left_keys.len()).unwrap();
                }
            }

            PhysicalOp::HashAggregate { group_by, aggregates, .. } => {
                write!(output, "Hash Aggregate").unwrap();
                if self.options.verbose {
                    write!(
                        output,
                        " [Group: {} keys, Agg: {} functions]",
                        group_by.len(),
                        aggregates.len()
                    )
                    .unwrap();
                }
            }

            PhysicalOp::SortAggregate { group_by, aggregates, .. } => {
                write!(output, "Sort Aggregate").unwrap();
                if self.options.verbose {
                    write!(
                        output,
                        " [Group: {} keys, Agg: {} functions]",
                        group_by.len(),
                        aggregates.len()
                    )
                    .unwrap();
                }
            }

            PhysicalOp::Sort { order_by } => {
                write!(output, "Sort").unwrap();
                if self.options.verbose {
                    write!(output, " [{} keys]", order_by.len()).unwrap();
                }
            }

            PhysicalOp::TopNSort { order_by, limit } => {
                write!(output, "Top-N Sort [limit={}]", limit).unwrap();
            }

            PhysicalOp::Limit { limit, offset } => {
                write!(output, "Limit").unwrap();
                if let Some(l) = limit {
                    write!(output, " [limit={}]", l).unwrap();
                }
                if let Some(o) = offset {
                    write!(output, " [offset={}]", o).unwrap();
                }
            }

            PhysicalOp::HashDistinct => {
                write!(output, "Hash Distinct").unwrap();
            }

            PhysicalOp::SortDistinct => {
                write!(output, "Sort Distinct").unwrap();
            }

            PhysicalOp::UnionAll => {
                write!(output, "Union All").unwrap();
            }

            PhysicalOp::HashUnion => {
                write!(output, "Hash Union").unwrap();
            }

            PhysicalOp::Gather { num_workers } => {
                write!(output, "Gather [workers={}]", num_workers).unwrap();
            }

            PhysicalOp::Exchange { distribution, num_partitions } => {
                write!(output, "Exchange [partitions={}, dist={:?}]", num_partitions, distribution).unwrap();
            }

            PhysicalOp::Materialize => {
                write!(output, "Materialize").unwrap();
            }
        }
    }

    /// Format as JSON
    fn format_json(&self, plan: &PhysicalPlan) -> String {
        serde_json::to_string_pretty(plan).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format as YAML
    fn format_yaml(&self, plan: &PhysicalPlan) -> String {
        serde_yaml::to_string(plan).unwrap_or_else(|_| "---".to_string())
    }

    /// Format as Graphviz DOT
    fn format_dot(&self, plan: &PhysicalPlan) -> String {
        let mut output = String::new();
        writeln!(&mut output, "digraph QueryPlan {{").unwrap();
        writeln!(&mut output, "  rankdir=BT;").unwrap();
        writeln!(&mut output, "  node [shape=box];").unwrap();
        writeln!(&mut output).unwrap();

        self.format_node_dot(&plan.root, &mut output);

        writeln!(&mut output, "}}").unwrap();
        output
    }

    fn format_node_dot(&self, node: &PhysicalNode, output: &mut String) {
        let node_id = format!("node_{}", node.id.0);

        // Node definition
        let label = self.get_operator_label(node);
        writeln!(
            output,
            "  {} [label=\"{}\"];",
            node_id, label
        )
        .unwrap();

        // Edges to children
        for child in &node.children {
            let child_id = format!("node_{}", child.id.0);
            writeln!(output, "  {} -> {};", child_id, node_id).unwrap();
            self.format_node_dot(child, output);
        }
    }

    fn get_operator_label(&self, node: &PhysicalNode) -> String {
        match &node.op {
            PhysicalOp::SeqScan { table, .. } => format!("Seq Scan\\n{}", table),
            PhysicalOp::IndexScan { table, index_name, .. } => {
                format!("Index Scan\\n{}\\n({})", table, index_name)
            }
            PhysicalOp::HashJoin { join_type, .. } => format!("Hash Join\\n({:?})", join_type),
            PhysicalOp::Sort { .. } => "Sort".to_string(),
            PhysicalOp::Limit { limit, .. } => format!("Limit\\n({})", limit.unwrap_or(0)),
            _ => format!("{:?}", node.op).split_whitespace().next().unwrap_or("Unknown").to_string(),
        }
    }
}

/// Cost breakdown analyzer
pub struct CostBreakdown {
    pub total_cost: f64,
    pub scan_cost: f64,
    pub join_cost: f64,
    pub aggregate_cost: f64,
    pub sort_cost: f64,
    pub other_cost: f64,
}

impl CostBreakdown {
    pub fn analyze(plan: &PhysicalPlan) -> Self {
        let mut breakdown = Self {
            total_cost: plan.estimated_cost.total_cost,
            scan_cost: 0.0,
            join_cost: 0.0,
            aggregate_cost: 0.0,
            sort_cost: 0.0,
            other_cost: 0.0,
        };

        breakdown.analyze_node(&plan.root);
        breakdown
    }

    fn analyze_node(&mut self, node: &PhysicalNode) {
        match &node.op {
            PhysicalOp::SeqScan { .. }
            | PhysicalOp::IndexScan { .. }
            | PhysicalOp::BitmapScan { .. } => {
                self.scan_cost += node.cost.total_cost;
            }

            PhysicalOp::NestedLoopJoin { .. }
            | PhysicalOp::HashJoin { .. }
            | PhysicalOp::MergeJoin { .. } => {
                self.join_cost += node.cost.total_cost;
            }

            PhysicalOp::HashAggregate { .. } | PhysicalOp::SortAggregate { .. } => {
                self.aggregate_cost += node.cost.total_cost;
            }

            PhysicalOp::Sort { .. } | PhysicalOp::TopNSort { .. } => {
                self.sort_cost += node.cost.total_cost;
            }

            _ => {
                self.other_cost += node.cost.total_cost;
            }
        }

        for child in &node.children {
            self.analyze_node(child);
        }
    }

    pub fn format(&self) -> String {
        format!(
            "Cost Breakdown:\n\
             Total:      {:.2}\n\
             Scan:       {:.2} ({:.1}%)\n\
             Join:       {:.2} ({:.1}%)\n\
             Aggregate:  {:.2} ({:.1}%)\n\
             Sort:       {:.2} ({:.1}%)\n\
             Other:      {:.2} ({:.1}%)",
            self.total_cost,
            self.scan_cost,
            self.scan_cost / self.total_cost * 100.0,
            self.join_cost,
            self.join_cost / self.total_cost * 100.0,
            self.aggregate_cost,
            self.aggregate_cost / self.total_cost * 100.0,
            self.sort_cost,
            self.sort_cost / self.total_cost * 100.0,
            self.other_cost,
            self.other_cost / self.total_cost * 100.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Schema;

    fn create_test_plan() -> PhysicalPlan {
        let node = PhysicalNode::new(
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
        );

        PhysicalPlan::new(node, Cost::new(100.0, 50.0, 0.0, 10.0))
    }

    #[test]
    fn test_explain_text_format() {
        let formatter = ExplainFormatter::with_default_options();
        let plan = create_test_plan();
        let output = formatter.format_plan(&plan);

        assert!(output.contains("Seq Scan"));
        assert!(output.contains("users"));
    }

    #[test]
    fn test_explain_verbose() {
        let options = ExplainOptions {
            verbose: true,
            ..Default::default()
        };
        let formatter = ExplainFormatter::new(options);
        let plan = create_test_plan();
        let output = formatter.format_plan(&plan);

        assert!(output.contains("rows="));
    }

    #[test]
    fn test_cost_breakdown() {
        let plan = create_test_plan();
        let breakdown = CostBreakdown::analyze(&plan);

        assert!(breakdown.total_cost > 0.0);
        assert!(breakdown.scan_cost > 0.0);

        let formatted = breakdown.format();
        assert!(formatted.contains("Cost Breakdown"));
    }
}
