//! Query Executor with Volcano (Iterator) Model
//!
//! Executes physical query plans using the Volcano model where each operator
//! is an iterator that pulls tuples from its children.

use crate::ast::{ProjectionItem, ScalarExpr, Schema};
use crate::plan::*;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Execution result
pub type ExecutionResult<T> = Result<T, ExecutionError>;

/// Execution errors
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Execution failed: {0}")]
    Failed(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Operator not implemented: {0}")]
    NotImplemented(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

/// Row batch for vectorized execution
#[derive(Debug, Clone)]
pub struct RowBatch {
    pub rows: Vec<Row>,
    pub schema: Schema,
}

impl RowBatch {
    pub fn new(schema: Schema) -> Self {
        Self {
            rows: Vec::new(),
            schema,
        }
    }

    pub fn with_capacity(schema: Schema, capacity: usize) -> Self {
        Self {
            rows: Vec::with_capacity(capacity),
            schema,
        }
    }

    pub fn add_row(&mut self, row: Row) {
        self.rows.push(row);
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn clear(&mut self) {
        self.rows.clear();
    }
}

/// A single row of data
#[derive(Debug, Clone)]
pub struct Row {
    pub values: Vec<Value>,
}

impl Row {
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }
}

/// Column value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Binary(Vec<u8>),
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

/// Execution context
#[derive(Clone)]
pub struct ExecutionContext {
    /// Batch size for vectorized execution
    pub batch_size: usize,
    /// Maximum memory per operator (bytes)
    pub operator_memory_limit: usize,
    /// Execution timeout (milliseconds)
    pub timeout_ms: Option<u64>,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            batch_size: 1024,
            operator_memory_limit: 64 * 1024 * 1024, // 64MB
            timeout_ms: None,
        }
    }
}

/// Physical operator executor trait (Volcano model)
#[async_trait]
pub trait PhysicalOperator: Send + Sync {
    /// Initialize the operator
    async fn open(&mut self) -> ExecutionResult<()>;

    /// Get next batch of rows
    async fn next(&mut self) -> ExecutionResult<Option<RowBatch>>;

    /// Close the operator and release resources
    async fn close(&mut self) -> ExecutionResult<()>;

    /// Get output schema
    fn schema(&self) -> &Schema;

    /// Get estimated cardinality
    fn cardinality(&self) -> &Cardinality;
}

/// Query executor
pub struct QueryExecutor {
    context: ExecutionContext,
}

impl QueryExecutor {
    pub fn new(context: ExecutionContext) -> Self {
        Self { context }
    }

    pub fn with_default_context() -> Self {
        Self::new(ExecutionContext::default())
    }

    /// Execute a physical plan
    pub async fn execute(&self, plan: PhysicalPlan) -> ExecutionResult<ExecutionStats> {
        let mut operator = self.create_operator(plan.root)?;

        let start_time = std::time::Instant::now();
        let mut total_rows = 0;
        let mut total_batches = 0;

        // Open operator tree
        operator.open().await?;

        // Pull all results
        while let Some(batch) = operator.next().await? {
            total_rows += batch.len();
            total_batches += 1;
        }

        // Close operator tree
        operator.close().await?;

        let execution_time = start_time.elapsed();

        Ok(ExecutionStats {
            total_rows,
            total_batches,
            execution_time_ms: execution_time.as_millis() as u64,
            operators_executed: self.count_operators(&plan.root),
        })
    }

    /// Create operator from physical node
    fn create_operator(&self, node: PhysicalNode) -> ExecutionResult<Box<dyn PhysicalOperator>> {
        match node.op {
            PhysicalOp::SeqScan {
                table,
                alias,
                predicates,
                projection,
            } => Ok(Box::new(SeqScanOperator::new(
                table,
                alias,
                predicates,
                projection,
                node.schema,
                node.cardinality,
                self.context.clone(),
            ))),

            PhysicalOp::Filter { predicates } => {
                let child = if !node.children.is_empty() {
                    Some(self.create_operator(node.children[0].clone())?)
                } else {
                    None
                };

                Ok(Box::new(FilterOperator::new(
                    predicates,
                    child,
                    node.schema,
                    node.cardinality,
                )))
            }

            PhysicalOp::Project { projections } => {
                let child = if !node.children.is_empty() {
                    Some(self.create_operator(node.children[0].clone())?)
                } else {
                    None
                };

                Ok(Box::new(ProjectOperator::new(
                    projections,
                    child,
                    node.schema,
                    node.cardinality,
                )))
            }

            PhysicalOp::Limit { limit, offset } => {
                let child = if !node.children.is_empty() {
                    Some(self.create_operator(node.children[0].clone())?)
                } else {
                    None
                };

                Ok(Box::new(LimitOperator::new(
                    limit,
                    offset,
                    child,
                    node.schema,
                    node.cardinality,
                )))
            }

            _ => Err(ExecutionError::NotImplemented(format!(
                "Operator not implemented: {:?}",
                node.op
            ))),
        }
    }

    fn count_operators(&self, node: &PhysicalNode) -> usize {
        1 + node.children.iter().map(|c| self.count_operators(c)).sum::<usize>()
    }
}

/// Execution statistics
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    pub total_rows: usize,
    pub total_batches: usize,
    pub execution_time_ms: u64,
    pub operators_executed: usize,
}

/// Sequential scan operator
pub struct SeqScanOperator {
    table: String,
    alias: Option<String>,
    predicates: Vec<ScalarExpr>,
    projection: Option<Vec<String>>,
    schema: Schema,
    cardinality: Cardinality,
    context: ExecutionContext,
    position: usize,
    is_open: bool,
}

impl SeqScanOperator {
    pub fn new(
        table: String,
        alias: Option<String>,
        predicates: Vec<ScalarExpr>,
        projection: Option<Vec<String>>,
        schema: Schema,
        cardinality: Cardinality,
        context: ExecutionContext,
    ) -> Self {
        Self {
            table,
            alias,
            predicates,
            projection,
            schema,
            cardinality,
            context,
            position: 0,
            is_open: false,
        }
    }
}

#[async_trait]
impl PhysicalOperator for SeqScanOperator {
    async fn open(&mut self) -> ExecutionResult<()> {
        self.is_open = true;
        self.position = 0;
        Ok(())
    }

    async fn next(&mut self) -> ExecutionResult<Option<RowBatch>> {
        if !self.is_open {
            return Err(ExecutionError::InvalidState(
                "Operator not opened".to_string(),
            ));
        }

        // Simplified: would read from actual storage
        // For now, return empty batch to signal EOF
        Ok(None)
    }

    async fn close(&mut self) -> ExecutionResult<()> {
        self.is_open = false;
        Ok(())
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn cardinality(&self) -> &Cardinality {
        &self.cardinality
    }
}

/// Filter operator
pub struct FilterOperator {
    predicates: Vec<ScalarExpr>,
    child: Option<Box<dyn PhysicalOperator>>,
    schema: Schema,
    cardinality: Cardinality,
}

impl FilterOperator {
    pub fn new(
        predicates: Vec<ScalarExpr>,
        child: Option<Box<dyn PhysicalOperator>>,
        schema: Schema,
        cardinality: Cardinality,
    ) -> Self {
        Self {
            predicates,
            child,
            schema,
            cardinality,
        }
    }

    fn evaluate_predicates(&self, _row: &Row) -> bool {
        // Simplified: would evaluate actual predicates
        true
    }
}

#[async_trait]
impl PhysicalOperator for FilterOperator {
    async fn open(&mut self) -> ExecutionResult<()> {
        if let Some(ref mut child) = self.child {
            child.open().await?;
        }
        Ok(())
    }

    async fn next(&mut self) -> ExecutionResult<Option<RowBatch>> {
        if let Some(ref mut child) = self.child {
            while let Some(mut batch) = child.next().await? {
                // Filter rows
                batch.rows.retain(|row| self.evaluate_predicates(row));

                if !batch.is_empty() {
                    return Ok(Some(batch));
                }
            }
        }
        Ok(None)
    }

    async fn close(&mut self) -> ExecutionResult<()> {
        if let Some(ref mut child) = self.child {
            child.close().await?;
        }
        Ok(())
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn cardinality(&self) -> &Cardinality {
        &self.cardinality
    }
}

/// Projection operator
pub struct ProjectOperator {
    projections: Vec<ProjectionItem>,
    child: Option<Box<dyn PhysicalOperator>>,
    schema: Schema,
    cardinality: Cardinality,
}

impl ProjectOperator {
    pub fn new(
        projections: Vec<ProjectionItem>,
        child: Option<Box<dyn PhysicalOperator>>,
        schema: Schema,
        cardinality: Cardinality,
    ) -> Self {
        Self {
            projections,
            child,
            schema,
            cardinality,
        }
    }

    fn project_row(&self, row: &Row) -> Row {
        // Simplified: would evaluate projection expressions
        row.clone()
    }
}

#[async_trait]
impl PhysicalOperator for ProjectOperator {
    async fn open(&mut self) -> ExecutionResult<()> {
        if let Some(ref mut child) = self.child {
            child.open().await?;
        }
        Ok(())
    }

    async fn next(&mut self) -> ExecutionResult<Option<RowBatch>> {
        if let Some(ref mut child) = self.child {
            if let Some(mut batch) = child.next().await? {
                // Project each row
                for row in &mut batch.rows {
                    *row = self.project_row(row);
                }
                return Ok(Some(batch));
            }
        }
        Ok(None)
    }

    async fn close(&mut self) -> ExecutionResult<()> {
        if let Some(ref mut child) = self.child {
            child.close().await?;
        }
        Ok(())
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn cardinality(&self) -> &Cardinality {
        &self.cardinality
    }
}

/// Limit operator
pub struct LimitOperator {
    limit: Option<u64>,
    offset: Option<u64>,
    child: Option<Box<dyn PhysicalOperator>>,
    schema: Schema,
    cardinality: Cardinality,
    rows_returned: u64,
    rows_skipped: u64,
}

impl LimitOperator {
    pub fn new(
        limit: Option<u64>,
        offset: Option<u64>,
        child: Option<Box<dyn PhysicalOperator>>,
        schema: Schema,
        cardinality: Cardinality,
    ) -> Self {
        Self {
            limit,
            offset,
            child,
            schema,
            cardinality,
            rows_returned: 0,
            rows_skipped: 0,
        }
    }
}

#[async_trait]
impl PhysicalOperator for LimitOperator {
    async fn open(&mut self) -> ExecutionResult<()> {
        self.rows_returned = 0;
        self.rows_skipped = 0;
        if let Some(ref mut child) = self.child {
            child.open().await?;
        }
        Ok(())
    }

    async fn next(&mut self) -> ExecutionResult<Option<RowBatch>> {
        // Check if we've returned enough rows
        if let Some(limit) = self.limit {
            if self.rows_returned >= limit {
                return Ok(None);
            }
        }

        if let Some(ref mut child) = self.child {
            while let Some(mut batch) = child.next().await? {
                // Skip offset rows
                if let Some(offset) = self.offset {
                    if self.rows_skipped < offset {
                        let to_skip = (offset - self.rows_skipped).min(batch.len() as u64);
                        batch.rows.drain(0..to_skip as usize);
                        self.rows_skipped += to_skip;

                        if batch.is_empty() {
                            continue;
                        }
                    }
                }

                // Apply limit
                if let Some(limit) = self.limit {
                    let remaining = limit - self.rows_returned;
                    if (batch.len() as u64) > remaining {
                        batch.rows.truncate(remaining as usize);
                    }
                }

                self.rows_returned += batch.len() as u64;
                return Ok(Some(batch));
            }
        }

        Ok(None)
    }

    async fn close(&mut self) -> ExecutionResult<()> {
        if let Some(ref mut child) = self.child {
            child.close().await?;
        }
        Ok(())
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn cardinality(&self) -> &Cardinality {
        &self.cardinality
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ColumnDef, DataType};

    #[tokio::test]
    async fn test_executor_creation() {
        let executor = QueryExecutor::with_default_context();
        assert_eq!(executor.context.batch_size, 1024);
    }

    #[tokio::test]
    async fn test_row_batch() {
        let schema = Schema::new(vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Varchar(Some(255))),
        ]);

        let mut batch = RowBatch::new(schema);
        assert!(batch.is_empty());

        batch.add_row(Row::new(vec![Value::Integer(1), Value::String("Alice".to_string())]));
        assert_eq!(batch.len(), 1);
    }

    #[tokio::test]
    async fn test_value_types() {
        assert!(Value::Null.is_null());
        assert!(!Value::Integer(42).is_null());

        let val = Value::String("test".to_string());
        assert_eq!(val, Value::String("test".to_string()));
    }
}
