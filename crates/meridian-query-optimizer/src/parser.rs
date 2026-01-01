//! SQL Parser - Wrapper around sqlparser with dialect support
//!
//! Converts SQL text into our internal AST representation for optimization.

use crate::ast::*;
use sqlparser::ast as sql;
use sqlparser::dialect::{Dialect, GenericDialect, PostgreSqlDialect, MySqlDialect, SQLiteDialect};
use sqlparser::parser::Parser;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("SQL parsing error: {0}")]
    SqlParser(#[from] sqlparser::parser::ParserError),

    #[error("Unsupported SQL feature: {0}")]
    UnsupportedFeature(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Type conversion error: {0}")]
    TypeConversion(String),
}

pub type Result<T> = std::result::Result<T, ParseError>;

/// SQL dialect support
#[derive(Debug, Clone, Copy)]
pub enum SqlDialect {
    Generic,
    PostgreSQL,
    MySQL,
    SQLite,
}

impl SqlDialect {
    fn to_dialect(&self) -> Box<dyn Dialect> {
        match self {
            SqlDialect::Generic => Box::new(GenericDialect {}),
            SqlDialect::PostgreSQL => Box::new(PostgreSqlDialect {}),
            SqlDialect::MySQL => Box::new(MySqlDialect {}),
            SqlDialect::SQLite => Box::new(SQLiteDialect {}),
        }
    }
}

/// SQL Query Parser
pub struct QueryParser {
    dialect: SqlDialect,
}

impl QueryParser {
    pub fn new(dialect: SqlDialect) -> Self {
        Self { dialect }
    }

    pub fn with_generic_dialect() -> Self {
        Self::new(SqlDialect::Generic)
    }

    pub fn with_postgres_dialect() -> Self {
        Self::new(SqlDialect::PostgreSQL)
    }

    /// Parse SQL query string into our AST
    pub fn parse(&self, sql: &str) -> Result<QueryExpr> {
        let dialect = self.dialect.to_dialect();
        let statements = Parser::parse_sql(dialect.as_ref(), sql)?;

        if statements.is_empty() {
            return Err(ParseError::InvalidQuery("Empty query".to_string()));
        }

        if statements.len() > 1 {
            return Err(ParseError::InvalidQuery(
                "Multiple statements not supported".to_string(),
            ));
        }

        let statement = &statements[0];
        self.convert_statement(statement)
    }

    fn convert_statement(&self, stmt: &sql::Statement) -> Result<QueryExpr> {
        match stmt {
            sql::Statement::Query(query) => self.convert_query(query),
            _ => Err(ParseError::UnsupportedFeature(
                "Only SELECT queries are supported".to_string(),
            )),
        }
    }

    fn convert_query(&self, query: &sql::Query) -> Result<QueryExpr> {
        let rel_expr = self.convert_query_body(&query.body)?;

        // Apply ORDER BY if present
        let rel_expr = if !query.order_by.is_empty() {
            let order_by = query
                .order_by
                .iter()
                .map(|o| self.convert_order_by(o))
                .collect::<Result<Vec<_>>>()?;

            Box::new(RelExpr::Sort(SortExpr {
                id: NodeId::new(),
                input: rel_expr,
                order_by,
            }))
        } else {
            rel_expr
        };

        // Apply LIMIT/OFFSET if present
        let rel_expr = if query.limit.is_some() || query.offset.is_some() {
            let limit = query
                .limit
                .as_ref()
                .and_then(|l| self.extract_limit_value(l));
            let offset = query
                .offset
                .as_ref()
                .and_then(|o| self.extract_offset_value(o));

            Box::new(RelExpr::Limit(LimitExpr {
                id: NodeId::new(),
                input: rel_expr,
                limit,
                offset,
            }))
        } else {
            rel_expr
        };

        // Determine output schema (simplified - in production, would infer from projection)
        let output_schema = Schema::empty();

        Ok(QueryExpr {
            id: NodeId::new(),
            root: rel_expr,
            output_schema,
        })
    }

    fn convert_query_body(&self, body: &sql::SetExpr) -> Result<Box<RelExpr>> {
        match body {
            sql::SetExpr::Select(select) => self.convert_select(select),
            sql::SetExpr::SetOperation {
                op,
                set_quantifier,
                left,
                right,
            } => self.convert_set_operation(op, set_quantifier, left, right),
            _ => Err(ParseError::UnsupportedFeature(
                "Complex query body".to_string(),
            )),
        }
    }

    fn convert_select(&self, select: &sql::Select) -> Result<Box<RelExpr>> {
        // Start with FROM clause
        let mut rel_expr = if select.from.is_empty() {
            // No FROM clause - create a dummy scan
            return Err(ParseError::InvalidQuery(
                "SELECT without FROM not yet supported".to_string(),
            ));
        } else {
            self.convert_from(&select.from)?
        };

        // Apply WHERE clause
        if let Some(ref selection) = select.selection {
            let predicates = self.convert_where_clause(selection)?;
            rel_expr = Box::new(RelExpr::Filter(FilterExpr {
                id: NodeId::new(),
                input: rel_expr,
                predicates,
            }));
        }

        // Apply GROUP BY and aggregates
        if !select.group_by.is_empty() || self.has_aggregates(&select.projection) {
            let group_by = select
                .group_by
                .iter()
                .map(|e| self.convert_expr(e))
                .collect::<Result<Vec<_>>>()?;

            let aggregates = self.extract_aggregates(&select.projection)?;

            let having = if let Some(ref having_expr) = select.having {
                Some(self.convert_expr(having_expr)?)
            } else {
                None
            };

            rel_expr = Box::new(RelExpr::Aggregate(AggregateExpr {
                id: NodeId::new(),
                input: rel_expr,
                group_by,
                aggregates,
                having,
            }));
        }

        // Apply DISTINCT
        if select.distinct.is_some() {
            rel_expr = Box::new(RelExpr::Distinct(DistinctExpr {
                id: NodeId::new(),
                input: rel_expr,
            }));
        }

        // Apply projection
        let projections = self.convert_projection(&select.projection)?;
        if !projections.is_empty() {
            rel_expr = Box::new(RelExpr::Project(ProjectExpr {
                id: NodeId::new(),
                input: rel_expr,
                projections,
            }));
        }

        Ok(rel_expr)
    }

    fn convert_from(&self, from: &[sql::TableWithJoins]) -> Result<Box<RelExpr>> {
        if from.is_empty() {
            return Err(ParseError::InvalidQuery("Empty FROM clause".to_string()));
        }

        let mut result = self.convert_table_factor(&from[0].relation)?;

        // Process joins
        for join in &from[0].joins {
            result = self.convert_join(result, join)?;
        }

        // Process additional tables (implicit cross joins)
        for table_with_joins in &from[1..] {
            let right = self.convert_table_factor(&table_with_joins.relation)?;
            result = Box::new(RelExpr::Join(JoinExpr {
                id: NodeId::new(),
                left: result,
                right,
                join_type: JoinType::Cross,
                condition: None,
            }));

            for join in &table_with_joins.joins {
                result = self.convert_join(result, join)?;
            }
        }

        Ok(result)
    }

    fn convert_table_factor(&self, factor: &sql::TableFactor) -> Result<Box<RelExpr>> {
        match factor {
            sql::TableFactor::Table { name, alias, .. } => {
                let table_name = name.to_string();
                let alias_name = alias.as_ref().map(|a| a.name.to_string());

                Ok(Box::new(RelExpr::Scan(ScanExpr {
                    id: NodeId::new(),
                    table_name,
                    alias: alias_name,
                    schema: Schema::empty(), // Would be populated from catalog
                    predicates: vec![],
                    projection: None,
                })))
            }
            sql::TableFactor::Derived {
                subquery, alias, ..
            } => {
                let query_expr = self.convert_query(subquery)?;
                Ok(Box::new(RelExpr::Subquery(SubqueryExpr {
                    id: NodeId::new(),
                    query: query_expr.root,
                    correlated: false,
                })))
            }
            _ => Err(ParseError::UnsupportedFeature(
                "Unsupported table factor".to_string(),
            )),
        }
    }

    fn convert_join(&self, left: Box<RelExpr>, join: &sql::Join) -> Result<Box<RelExpr>> {
        let right = self.convert_table_factor(&join.relation)?;

        let join_type = match &join.join_operator {
            sql::JoinOperator::Inner(_) => JoinType::Inner,
            sql::JoinOperator::LeftOuter(_) => JoinType::Left,
            sql::JoinOperator::RightOuter(_) => JoinType::Right,
            sql::JoinOperator::FullOuter(_) => JoinType::Full,
            sql::JoinOperator::CrossJoin => JoinType::Cross,
            _ => {
                return Err(ParseError::UnsupportedFeature(
                    "Unsupported join type".to_string(),
                ))
            }
        };

        let condition = match &join.join_operator {
            sql::JoinOperator::Inner(sql::JoinConstraint::On(expr))
            | sql::JoinOperator::LeftOuter(sql::JoinConstraint::On(expr))
            | sql::JoinOperator::RightOuter(sql::JoinConstraint::On(expr))
            | sql::JoinOperator::FullOuter(sql::JoinConstraint::On(expr)) => {
                Some(self.convert_expr(expr)?)
            }
            _ => None,
        };

        Ok(Box::new(RelExpr::Join(JoinExpr {
            id: NodeId::new(),
            left,
            right,
            join_type,
            condition,
        })))
    }

    fn convert_where_clause(&self, expr: &sql::Expr) -> Result<Vec<ScalarExpr>> {
        // Split AND predicates into separate items for optimization
        let scalar_expr = self.convert_expr(expr)?;
        Ok(self.split_conjunctions(scalar_expr))
    }

    fn split_conjunctions(&self, expr: ScalarExpr) -> Vec<ScalarExpr> {
        match expr {
            ScalarExpr::BinaryOp {
                left,
                op: BinaryOp::And,
                right,
            } => {
                let mut result = self.split_conjunctions(*left);
                result.extend(self.split_conjunctions(*right));
                result
            }
            _ => vec![expr],
        }
    }

    fn convert_expr(&self, expr: &sql::Expr) -> Result<ScalarExpr> {
        match expr {
            sql::Expr::Identifier(ident) => Ok(ScalarExpr::Column(ColumnRef::new(ident.to_string()))),

            sql::Expr::CompoundIdentifier(idents) => {
                if idents.len() == 2 {
                    Ok(ScalarExpr::Column(ColumnRef::with_table(
                        idents[0].to_string(),
                        idents[1].to_string(),
                    )))
                } else {
                    Err(ParseError::UnsupportedFeature(
                        "Complex identifiers".to_string(),
                    ))
                }
            }

            sql::Expr::Value(value) => Ok(ScalarExpr::Literal(self.convert_value(value)?)),

            sql::Expr::BinaryOp { left, op, right } => {
                let left_expr = self.convert_expr(left)?;
                let right_expr = self.convert_expr(right)?;
                let binary_op = self.convert_binary_op(op)?;

                Ok(ScalarExpr::BinaryOp {
                    left: Box::new(left_expr),
                    op: binary_op,
                    right: Box::new(right_expr),
                })
            }

            sql::Expr::UnaryOp { op, expr } => {
                let inner_expr = self.convert_expr(expr)?;
                let unary_op = self.convert_unary_op(op)?;

                Ok(ScalarExpr::UnaryOp {
                    op: unary_op,
                    expr: Box::new(inner_expr),
                })
            }

            sql::Expr::Function(func) => {
                let name = func.name.to_string();
                let args = func
                    .args
                    .iter()
                    .map(|arg| match arg {
                        sql::FunctionArg::Unnamed(sql::FunctionArgExpr::Expr(e)) => {
                            self.convert_expr(e)
                        }
                        _ => Err(ParseError::UnsupportedFeature(
                            "Complex function arguments".to_string(),
                        )),
                    })
                    .collect::<Result<Vec<_>>>()?;

                Ok(ScalarExpr::Function { name, args })
            }

            sql::Expr::InList {
                expr,
                list,
                negated,
            } => {
                let expr = self.convert_expr(expr)?;
                let list = list
                    .iter()
                    .map(|e| self.convert_expr(e))
                    .collect::<Result<Vec<_>>>()?;

                Ok(ScalarExpr::In {
                    expr: Box::new(expr),
                    list,
                    negated: *negated,
                })
            }

            sql::Expr::Between {
                expr,
                negated,
                low,
                high,
            } => {
                let expr = self.convert_expr(expr)?;
                let low = self.convert_expr(low)?;
                let high = self.convert_expr(high)?;

                Ok(ScalarExpr::Between {
                    expr: Box::new(expr),
                    low: Box::new(low),
                    high: Box::new(high),
                    negated: *negated,
                })
            }

            sql::Expr::Case {
                operand,
                conditions,
                results,
                else_result,
            } => {
                let operand = operand
                    .as_ref()
                    .map(|e| self.convert_expr(e))
                    .transpose()?
                    .map(Box::new);

                let when_clauses = conditions
                    .iter()
                    .zip(results.iter())
                    .map(|(cond, res)| {
                        let cond_expr = self.convert_expr(cond)?;
                        let res_expr = self.convert_expr(res)?;
                        Ok((cond_expr, res_expr))
                    })
                    .collect::<Result<Vec<_>>>()?;

                let else_clause = else_result
                    .as_ref()
                    .map(|e| self.convert_expr(e))
                    .transpose()?
                    .map(Box::new);

                Ok(ScalarExpr::Case {
                    operand,
                    when_clauses,
                    else_clause,
                })
            }

            sql::Expr::IsNull(expr) => {
                let inner = self.convert_expr(expr)?;
                Ok(ScalarExpr::UnaryOp {
                    op: UnaryOp::IsNull,
                    expr: Box::new(inner),
                })
            }

            sql::Expr::IsNotNull(expr) => {
                let inner = self.convert_expr(expr)?;
                Ok(ScalarExpr::UnaryOp {
                    op: UnaryOp::IsNotNull,
                    expr: Box::new(inner),
                })
            }

            _ => Err(ParseError::UnsupportedFeature(format!(
                "Expression type: {:?}",
                expr
            ))),
        }
    }

    fn convert_value(&self, value: &sql::Value) -> Result<Literal> {
        match value {
            sql::Value::Null => Ok(Literal::Null),
            sql::Value::Boolean(b) => Ok(Literal::Boolean(*b)),
            sql::Value::Number(n, _) => {
                if let Ok(i) = n.parse::<i64>() {
                    Ok(Literal::Integer(i))
                } else if let Ok(f) = n.parse::<f64>() {
                    Ok(Literal::Float(f))
                } else {
                    Err(ParseError::TypeConversion(format!(
                        "Invalid number: {}",
                        n
                    )))
                }
            }
            sql::Value::SingleQuotedString(s) | sql::Value::DoubleQuotedString(s) => {
                Ok(Literal::String(s.clone()))
            }
            _ => Err(ParseError::UnsupportedFeature(format!(
                "Value type: {:?}",
                value
            ))),
        }
    }

    fn convert_binary_op(&self, op: &sql::BinaryOperator) -> Result<BinaryOp> {
        match op {
            sql::BinaryOperator::Plus => Ok(BinaryOp::Add),
            sql::BinaryOperator::Minus => Ok(BinaryOp::Subtract),
            sql::BinaryOperator::Multiply => Ok(BinaryOp::Multiply),
            sql::BinaryOperator::Divide => Ok(BinaryOp::Divide),
            sql::BinaryOperator::Modulo => Ok(BinaryOp::Modulo),
            sql::BinaryOperator::Eq => Ok(BinaryOp::Eq),
            sql::BinaryOperator::NotEq => Ok(BinaryOp::NotEq),
            sql::BinaryOperator::Lt => Ok(BinaryOp::Lt),
            sql::BinaryOperator::LtEq => Ok(BinaryOp::LtEq),
            sql::BinaryOperator::Gt => Ok(BinaryOp::Gt),
            sql::BinaryOperator::GtEq => Ok(BinaryOp::GtEq),
            sql::BinaryOperator::And => Ok(BinaryOp::And),
            sql::BinaryOperator::Or => Ok(BinaryOp::Or),
            sql::BinaryOperator::Like => Ok(BinaryOp::Like),
            _ => Err(ParseError::UnsupportedFeature(format!(
                "Binary operator: {:?}",
                op
            ))),
        }
    }

    fn convert_unary_op(&self, op: &sql::UnaryOperator) -> Result<UnaryOp> {
        match op {
            sql::UnaryOperator::Not => Ok(UnaryOp::Not),
            sql::UnaryOperator::Minus => Ok(UnaryOp::Negate),
            _ => Err(ParseError::UnsupportedFeature(format!(
                "Unary operator: {:?}",
                op
            ))),
        }
    }

    fn convert_projection(&self, projection: &[sql::SelectItem]) -> Result<Vec<ProjectionItem>> {
        let mut items = Vec::new();

        for item in projection {
            match item {
                sql::SelectItem::UnnamedExpr(expr) => {
                    let expr = self.convert_expr(expr)?;
                    items.push(ProjectionItem { expr, alias: None });
                }
                sql::SelectItem::ExprWithAlias { expr, alias } => {
                    let expr = self.convert_expr(expr)?;
                    items.push(ProjectionItem {
                        expr,
                        alias: Some(alias.to_string()),
                    });
                }
                sql::SelectItem::Wildcard(_) => {
                    // Wildcard - would expand based on input schema
                    // For now, skip
                }
                _ => {}
            }
        }

        Ok(items)
    }

    fn convert_order_by(&self, order_by: &sql::OrderByExpr) -> Result<OrderByItem> {
        let expr = self.convert_expr(&order_by.expr)?;
        let direction = if order_by.asc.unwrap_or(true) {
            SortDirection::Ascending
        } else {
            SortDirection::Descending
        };
        let nulls_first = order_by.nulls_first.unwrap_or(false);

        Ok(OrderByItem {
            expr,
            direction,
            nulls_first,
        })
    }

    fn has_aggregates(&self, projection: &[sql::SelectItem]) -> bool {
        // Simplified check - in production would deeply traverse expressions
        projection.iter().any(|item| match item {
            sql::SelectItem::UnnamedExpr(expr) | sql::SelectItem::ExprWithAlias { expr, .. } => {
                matches!(expr, sql::Expr::Function(_))
            }
            _ => false,
        })
    }

    fn extract_aggregates(&self, projection: &[sql::SelectItem]) -> Result<Vec<AggregateFunction>> {
        // Simplified - would extract actual aggregate functions
        Ok(vec![])
    }

    fn extract_limit_value(&self, expr: &sql::Expr) -> Option<u64> {
        match expr {
            sql::Expr::Value(sql::Value::Number(n, _)) => n.parse().ok(),
            _ => None,
        }
    }

    fn extract_offset_value(&self, offset: &sql::Offset) -> Option<u64> {
        match &offset.value {
            sql::Expr::Value(sql::Value::Number(n, _)) => n.parse().ok(),
            _ => None,
        }
    }

    fn convert_set_operation(
        &self,
        op: &sql::SetOperator,
        quantifier: &sql::SetQuantifier,
        left: &sql::SetExpr,
        right: &sql::SetExpr,
    ) -> Result<Box<RelExpr>> {
        let left_expr = self.convert_query_body(left)?;
        let right_expr = self.convert_query_body(right)?;

        match op {
            sql::SetOperator::Union => {
                let all = matches!(quantifier, sql::SetQuantifier::All);
                Ok(Box::new(RelExpr::Union(UnionExpr {
                    id: NodeId::new(),
                    left: left_expr,
                    right: right_expr,
                    all,
                })))
            }
            _ => Err(ParseError::UnsupportedFeature(
                "Set operation not supported".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_select() {
        let parser = QueryParser::with_generic_dialect();
        let result = parser.parse("SELECT id, name FROM users WHERE id > 10");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_join() {
        let parser = QueryParser::with_generic_dialect();
        let result = parser.parse(
            "SELECT u.id, o.total FROM users u INNER JOIN orders o ON u.id = o.user_id",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with_limit() {
        let parser = QueryParser::with_generic_dialect();
        let result = parser.parse("SELECT * FROM users ORDER BY id DESC LIMIT 10 OFFSET 20");
        assert!(result.is_ok());
    }
}
