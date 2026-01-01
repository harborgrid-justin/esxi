//! Query Abstract Syntax Tree (AST) representation
//!
//! Provides a normalized, optimizable representation of SQL queries
//! independent of specific SQL dialects.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for query nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

/// Query expression representing a complete query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryExpr {
    pub id: NodeId,
    pub root: Box<RelExpr>,
    pub output_schema: Schema,
}

/// Relational expression - core algebra operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelExpr {
    /// Table scan
    Scan(ScanExpr),

    /// Selection (filter)
    Filter(FilterExpr),

    /// Projection
    Project(ProjectExpr),

    /// Join operations
    Join(JoinExpr),

    /// Aggregation
    Aggregate(AggregateExpr),

    /// Sort
    Sort(SortExpr),

    /// Limit/Offset
    Limit(LimitExpr),

    /// Union
    Union(UnionExpr),

    /// Distinct
    Distinct(DistinctExpr),

    /// Subquery
    Subquery(SubqueryExpr),
}

impl RelExpr {
    pub fn node_id(&self) -> NodeId {
        match self {
            RelExpr::Scan(e) => e.id,
            RelExpr::Filter(e) => e.id,
            RelExpr::Project(e) => e.id,
            RelExpr::Join(e) => e.id,
            RelExpr::Aggregate(e) => e.id,
            RelExpr::Sort(e) => e.id,
            RelExpr::Limit(e) => e.id,
            RelExpr::Union(e) => e.id,
            RelExpr::Distinct(e) => e.id,
            RelExpr::Subquery(e) => e.id,
        }
    }

    pub fn children(&self) -> Vec<&RelExpr> {
        match self {
            RelExpr::Scan(_) => vec![],
            RelExpr::Filter(e) => vec![&e.input],
            RelExpr::Project(e) => vec![&e.input],
            RelExpr::Join(e) => vec![&e.left, &e.right],
            RelExpr::Aggregate(e) => vec![&e.input],
            RelExpr::Sort(e) => vec![&e.input],
            RelExpr::Limit(e) => vec![&e.input],
            RelExpr::Union(e) => vec![&e.left, &e.right],
            RelExpr::Distinct(e) => vec![&e.input],
            RelExpr::Subquery(e) => vec![&e.query],
        }
    }
}

/// Table scan expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanExpr {
    pub id: NodeId,
    pub table_name: String,
    pub alias: Option<String>,
    pub schema: Schema,
    /// Optional filter predicates pushed down to scan
    pub predicates: Vec<ScalarExpr>,
    /// Columns to scan (None = all columns)
    pub projection: Option<Vec<String>>,
}

/// Filter expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterExpr {
    pub id: NodeId,
    pub input: Box<RelExpr>,
    pub predicates: Vec<ScalarExpr>,
}

/// Projection expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectExpr {
    pub id: NodeId,
    pub input: Box<RelExpr>,
    pub projections: Vec<ProjectionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionItem {
    pub expr: ScalarExpr,
    pub alias: Option<String>,
}

/// Join expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinExpr {
    pub id: NodeId,
    pub left: Box<RelExpr>,
    pub right: Box<RelExpr>,
    pub join_type: JoinType,
    pub condition: Option<ScalarExpr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
    Semi,      // Left semi join
    AntiSemi,  // Left anti join
}

/// Aggregation expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateExpr {
    pub id: NodeId,
    pub input: Box<RelExpr>,
    pub group_by: Vec<ScalarExpr>,
    pub aggregates: Vec<AggregateFunction>,
    pub having: Option<ScalarExpr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateFunction {
    pub func: AggFunc,
    pub args: Vec<ScalarExpr>,
    pub distinct: bool,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggFunc {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    StdDev,
    Variance,
    ArrayAgg,
    StringAgg,
}

/// Sort expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortExpr {
    pub id: NodeId,
    pub input: Box<RelExpr>,
    pub order_by: Vec<OrderByItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderByItem {
    pub expr: ScalarExpr,
    pub direction: SortDirection,
    pub nulls_first: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Limit/Offset expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitExpr {
    pub id: NodeId,
    pub input: Box<RelExpr>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

/// Union expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionExpr {
    pub id: NodeId,
    pub left: Box<RelExpr>,
    pub right: Box<RelExpr>,
    pub all: bool, // UNION ALL vs UNION
}

/// Distinct expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistinctExpr {
    pub id: NodeId,
    pub input: Box<RelExpr>,
}

/// Subquery expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubqueryExpr {
    pub id: NodeId,
    pub query: Box<RelExpr>,
    pub correlated: bool,
}

/// Scalar expression (column references, literals, operations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalarExpr {
    /// Column reference
    Column(ColumnRef),

    /// Literal value
    Literal(Literal),

    /// Binary operation
    BinaryOp {
        left: Box<ScalarExpr>,
        op: BinaryOp,
        right: Box<ScalarExpr>,
    },

    /// Unary operation
    UnaryOp {
        op: UnaryOp,
        expr: Box<ScalarExpr>,
    },

    /// Function call
    Function {
        name: String,
        args: Vec<ScalarExpr>,
    },

    /// CASE expression
    Case {
        operand: Option<Box<ScalarExpr>>,
        when_clauses: Vec<(ScalarExpr, ScalarExpr)>,
        else_clause: Option<Box<ScalarExpr>>,
    },

    /// IN expression
    In {
        expr: Box<ScalarExpr>,
        list: Vec<ScalarExpr>,
        negated: bool,
    },

    /// BETWEEN expression
    Between {
        expr: Box<ScalarExpr>,
        low: Box<ScalarExpr>,
        high: Box<ScalarExpr>,
        negated: bool,
    },

    /// Subquery expression
    Subquery(Box<RelExpr>),
}

/// Column reference
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColumnRef {
    pub table: Option<String>,
    pub name: String,
}

impl ColumnRef {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            table: None,
            name: name.into(),
        }
    }

    pub fn with_table(table: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            table: Some(table.into()),
            name: name.into(),
        }
    }
}

/// Literal value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Date(String),
    Timestamp(String),
    Interval(String),
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    // Comparison
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,

    // Logical
    And,
    Or,

    // String
    Like,
    NotLike,
    ILike,     // Case-insensitive LIKE
    NotILike,

    // Pattern matching
    RegexMatch,
    RegexNotMatch,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,
    Negate,
    IsNull,
    IsNotNull,
}

/// Schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub columns: Vec<ColumnDef>,
}

impl Schema {
    pub fn new(columns: Vec<ColumnDef>) -> Self {
        Self { columns }
    }

    pub fn empty() -> Self {
        Self { columns: vec![] }
    }

    pub fn find_column(&self, name: &str) -> Option<&ColumnDef> {
        self.columns.iter().find(|c| c.name == name)
    }

    pub fn column_names(&self) -> Vec<String> {
        self.columns.iter().map(|c| c.name.clone()).collect()
    }
}

/// Column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

impl ColumnDef {
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable: true,
        }
    }

    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }
}

/// Data types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Boolean,
    TinyInt,
    SmallInt,
    Integer,
    BigInt,
    Float,
    Double,
    Decimal { precision: u8, scale: u8 },
    Char(u32),
    Varchar(Option<u32>),
    Text,
    Binary(u32),
    Varbinary(Option<u32>),
    Date,
    Time,
    Timestamp,
    TimestampTz,
    Interval,
    Json,
    Jsonb,
    Uuid,
    Array(Box<DataType>),
    Struct(Vec<ColumnDef>),
}

impl DataType {
    /// Estimate size in bytes for cost estimation
    pub fn estimated_size(&self) -> usize {
        match self {
            DataType::Boolean | DataType::TinyInt => 1,
            DataType::SmallInt => 2,
            DataType::Integer | DataType::Float => 4,
            DataType::BigInt | DataType::Double => 8,
            DataType::Decimal { .. } => 16,
            DataType::Char(n) => *n as usize,
            DataType::Varchar(Some(n)) => *n as usize,
            DataType::Varchar(None) | DataType::Text => 64, // average estimate
            DataType::Binary(n) => *n as usize,
            DataType::Varbinary(Some(n)) => *n as usize,
            DataType::Varbinary(None) => 64,
            DataType::Date => 4,
            DataType::Time => 8,
            DataType::Timestamp | DataType::TimestampTz => 8,
            DataType::Interval => 16,
            DataType::Json | DataType::Jsonb => 256, // average estimate
            DataType::Uuid => 16,
            DataType::Array(inner) => inner.estimated_size() * 10, // estimate 10 elements
            DataType::Struct(fields) => fields.iter().map(|f| f.data_type.estimated_size()).sum(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_generation() {
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_column_ref() {
        let col = ColumnRef::new("id");
        assert_eq!(col.name, "id");
        assert_eq!(col.table, None);

        let col = ColumnRef::with_table("users", "id");
        assert_eq!(col.name, "id");
        assert_eq!(col.table, Some("users".to_string()));
    }

    #[test]
    fn test_schema() {
        let schema = Schema::new(vec![
            ColumnDef::new("id", DataType::BigInt).not_null(),
            ColumnDef::new("name", DataType::Varchar(Some(255))),
        ]);

        assert_eq!(schema.columns.len(), 2);
        assert!(schema.find_column("id").is_some());
        assert!(schema.find_column("nonexistent").is_none());
    }

    #[test]
    fn test_data_type_size_estimation() {
        assert_eq!(DataType::Integer.estimated_size(), 4);
        assert_eq!(DataType::BigInt.estimated_size(), 8);
        assert_eq!(DataType::Varchar(Some(100)).estimated_size(), 100);
    }
}
