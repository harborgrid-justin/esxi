//! Data-driven style evaluation for dynamic styling based on feature properties.

use super::{ColorOrExpression, ColorUtils, NumberOrExpression};
use crate::error::{MapEngineError, Result};
use serde_json::Value;

/// Context for style evaluation.
pub struct EvaluationContext<'a> {
    /// Feature properties.
    pub properties: &'a Value,
    /// Current zoom level.
    pub zoom: f32,
    /// Additional context variables.
    pub variables: std::collections::HashMap<String, Value>,
}

impl<'a> EvaluationContext<'a> {
    /// Create a new evaluation context.
    pub fn new(properties: &'a Value, zoom: f32) -> Self {
        Self {
            properties,
            zoom,
            variables: std::collections::HashMap::new(),
        }
    }

    /// Set a context variable.
    pub fn set_variable(&mut self, key: impl Into<String>, value: Value) {
        self.variables.insert(key.into(), value);
    }

    /// Get a property value by key.
    pub fn get_property(&self, key: &str) -> Option<&Value> {
        self.properties.get(key)
    }
}

/// Style expression evaluator.
pub struct StyleEvaluator;

impl StyleEvaluator {
    /// Evaluate a color expression.
    pub fn eval_color(
        expr: &ColorOrExpression,
        context: &EvaluationContext,
    ) -> Result<[f32; 4]> {
        match expr {
            ColorOrExpression::Color(color_str) => ColorUtils::parse_color(color_str)
                .map_err(|e| MapEngineError::StyleEvaluation(e)),
            ColorOrExpression::Expression(expr) => Self::eval_color_expression(expr, context),
        }
    }

    /// Evaluate a number expression.
    pub fn eval_number(
        expr: &NumberOrExpression,
        context: &EvaluationContext,
    ) -> Result<f32> {
        match expr {
            NumberOrExpression::Number(n) => Ok(*n),
            NumberOrExpression::Expression(expr) => Self::eval_number_expression(expr, context),
        }
    }

    /// Evaluate a color expression array.
    fn eval_color_expression(
        expr: &[Value],
        context: &EvaluationContext,
    ) -> Result<[f32; 4]> {
        if expr.is_empty() {
            return Err(MapEngineError::StyleEvaluation(
                "Empty expression".to_string(),
            ));
        }

        let operator = expr[0]
            .as_str()
            .ok_or_else(|| MapEngineError::StyleEvaluation("Invalid operator".to_string()))?;

        match operator {
            "get" => {
                // ["get", "property_name"]
                if expr.len() < 2 {
                    return Err(MapEngineError::StyleEvaluation(
                        "get requires property name".to_string(),
                    ));
                }
                let prop_name = expr[1].as_str().ok_or_else(|| {
                    MapEngineError::StyleEvaluation("Property name must be string".to_string())
                })?;
                let value = context.get_property(prop_name).ok_or_else(|| {
                    MapEngineError::StyleEvaluation(format!("Property not found: {}", prop_name))
                })?;
                if let Some(color_str) = value.as_str() {
                    ColorUtils::parse_color(color_str)
                        .map_err(|e| MapEngineError::StyleEvaluation(e))
                } else {
                    Err(MapEngineError::StyleEvaluation(
                        "Property is not a color".to_string(),
                    ))
                }
            }
            "interpolate" => {
                // ["interpolate", ["linear"], ["zoom"], stop1, color1, stop2, color2, ...]
                Self::eval_interpolate_color(expr, context)
            }
            "case" => {
                // ["case", condition1, value1, condition2, value2, ..., fallback]
                Self::eval_case_color(expr, context)
            }
            "match" => {
                // ["match", input, label1, value1, label2, value2, ..., fallback]
                Self::eval_match_color(expr, context)
            }
            _ => Err(MapEngineError::StyleEvaluation(format!(
                "Unknown color operator: {}",
                operator
            ))),
        }
    }

    /// Evaluate a number expression array.
    fn eval_number_expression(
        expr: &[Value],
        context: &EvaluationContext,
    ) -> Result<f32> {
        if expr.is_empty() {
            return Err(MapEngineError::StyleEvaluation(
                "Empty expression".to_string(),
            ));
        }

        let operator = expr[0]
            .as_str()
            .ok_or_else(|| MapEngineError::StyleEvaluation("Invalid operator".to_string()))?;

        match operator {
            "get" => {
                // ["get", "property_name"]
                if expr.len() < 2 {
                    return Err(MapEngineError::StyleEvaluation(
                        "get requires property name".to_string(),
                    ));
                }
                let prop_name = expr[1].as_str().ok_or_else(|| {
                    MapEngineError::StyleEvaluation("Property name must be string".to_string())
                })?;
                let value = context.get_property(prop_name).ok_or_else(|| {
                    MapEngineError::StyleEvaluation(format!("Property not found: {}", prop_name))
                })?;
                value.as_f64().map(|v| v as f32).ok_or_else(|| {
                    MapEngineError::StyleEvaluation("Property is not a number".to_string())
                })
            }
            "zoom" => Ok(context.zoom),
            "interpolate" => {
                // ["interpolate", ["linear"], ["zoom"], stop1, value1, stop2, value2, ...]
                Self::eval_interpolate_number(expr, context)
            }
            "case" => {
                // ["case", condition1, value1, condition2, value2, ..., fallback]
                Self::eval_case_number(expr, context)
            }
            "+" | "-" | "*" | "/" => Self::eval_arithmetic(operator, expr, context),
            _ => Err(MapEngineError::StyleEvaluation(format!(
                "Unknown number operator: {}",
                operator
            ))),
        }
    }

    /// Evaluate an interpolate expression for colors.
    fn eval_interpolate_color(
        expr: &[Value],
        context: &EvaluationContext,
    ) -> Result<[f32; 4]> {
        // Simplified interpolation - just return first color
        // In production, implement proper linear interpolation
        if expr.len() >= 5 {
            if let Some(color_str) = expr[4].as_str() {
                return ColorUtils::parse_color(color_str)
                    .map_err(|e| MapEngineError::StyleEvaluation(e));
            }
        }
        Ok([1.0, 1.0, 1.0, 1.0])
    }

    /// Evaluate an interpolate expression for numbers.
    fn eval_interpolate_number(
        expr: &[Value],
        context: &EvaluationContext,
    ) -> Result<f32> {
        // ["interpolate", ["linear"], ["zoom"], stop1, value1, stop2, value2, ...]
        if expr.len() < 6 {
            return Err(MapEngineError::StyleEvaluation(
                "interpolate requires at least one stop".to_string(),
            ));
        }

        // Get input value (typically zoom)
        let input = if expr[2].is_array() {
            let input_expr = expr[2].as_array().unwrap();
            Self::eval_number_expression(input_expr, context)?
        } else {
            context.zoom
        };

        // Parse stops
        let stops: Vec<(f32, f32)> = expr[3..]
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    let stop = chunk[0].as_f64()? as f32;
                    let value = chunk[1].as_f64()? as f32;
                    Some((stop, value))
                } else {
                    None
                }
            })
            .collect();

        // Linear interpolation
        for i in 0..stops.len() - 1 {
            let (stop1, value1) = stops[i];
            let (stop2, value2) = stops[i + 1];

            if input >= stop1 && input <= stop2 {
                let t = (input - stop1) / (stop2 - stop1);
                return Ok(value1 + t * (value2 - value1));
            }
        }

        // Return first or last value if outside range
        if input < stops[0].0 {
            Ok(stops[0].1)
        } else {
            Ok(stops.last().unwrap().1)
        }
    }

    /// Evaluate a case expression for colors.
    fn eval_case_color(
        expr: &[Value],
        context: &EvaluationContext,
    ) -> Result<[f32; 4]> {
        // Simplified case evaluation - return fallback
        if let Some(fallback) = expr.last() {
            if let Some(color_str) = fallback.as_str() {
                return ColorUtils::parse_color(color_str)
                    .map_err(|e| MapEngineError::StyleEvaluation(e));
            }
        }
        Ok([1.0, 1.0, 1.0, 1.0])
    }

    /// Evaluate a case expression for numbers.
    fn eval_case_number(
        expr: &[Value],
        context: &EvaluationContext,
    ) -> Result<f32> {
        // Simplified case evaluation - return fallback
        if let Some(fallback) = expr.last() {
            if let Some(num) = fallback.as_f64() {
                return Ok(num as f32);
            }
        }
        Ok(0.0)
    }

    /// Evaluate a match expression for colors.
    fn eval_match_color(
        expr: &[Value],
        context: &EvaluationContext,
    ) -> Result<[f32; 4]> {
        // Simplified match evaluation - return fallback
        if let Some(fallback) = expr.last() {
            if let Some(color_str) = fallback.as_str() {
                return ColorUtils::parse_color(color_str)
                    .map_err(|e| MapEngineError::StyleEvaluation(e));
            }
        }
        Ok([1.0, 1.0, 1.0, 1.0])
    }

    /// Evaluate arithmetic operations.
    fn eval_arithmetic(
        operator: &str,
        expr: &[Value],
        context: &EvaluationContext,
    ) -> Result<f32> {
        if expr.len() < 3 {
            return Err(MapEngineError::StyleEvaluation(
                "Arithmetic requires at least 2 operands".to_string(),
            ));
        }

        let mut result = if expr[1].is_array() {
            Self::eval_number_expression(expr[1].as_array().unwrap(), context)?
        } else {
            expr[1].as_f64().unwrap_or(0.0) as f32
        };

        for operand in &expr[2..] {
            let value = if operand.is_array() {
                Self::eval_number_expression(operand.as_array().unwrap(), context)?
            } else {
                operand.as_f64().unwrap_or(0.0) as f32
            };

            match operator {
                "+" => result += value,
                "-" => result -= value,
                "*" => result *= value,
                "/" => result /= value,
                _ => unreachable!(),
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_eval_static_color() {
        let context = EvaluationContext::new(&json!({}), 10.0);
        let expr = ColorOrExpression::Color("#ff0000".to_string());
        let color = StyleEvaluator::eval_color(&expr, &context).unwrap();
        assert_eq!(color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_eval_static_number() {
        let context = EvaluationContext::new(&json!({}), 10.0);
        let expr = NumberOrExpression::Number(42.0);
        let num = StyleEvaluator::eval_number(&expr, &context).unwrap();
        assert_eq!(num, 42.0);
    }

    #[test]
    fn test_eval_property_number() {
        let props = json!({
            "population": 1000000
        });
        let context = EvaluationContext::new(&props, 10.0);
        let expr = NumberOrExpression::Expression(vec![
            json!("get"),
            json!("population"),
        ]);
        let num = StyleEvaluator::eval_number(&expr, &context).unwrap();
        assert_eq!(num, 1000000.0);
    }

    #[test]
    fn test_eval_zoom() {
        let context = EvaluationContext::new(&json!({}), 15.5);
        let expr = NumberOrExpression::Expression(vec![json!("zoom")]);
        let num = StyleEvaluator::eval_number(&expr, &context).unwrap();
        assert_eq!(num, 15.5);
    }
}
