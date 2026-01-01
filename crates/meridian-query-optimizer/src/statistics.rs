//! Table and Column Statistics
//!
//! Provides statistical information for cost estimation including:
//! - Cardinality (row counts)
//! - Histograms (value distributions)
//! - Distinct value counts
//! - NULL counts
//! - Data skew analysis

use crate::ast::{ColumnRef, DataType};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Statistics for a table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStatistics {
    /// Table name
    pub table_name: String,

    /// Total number of rows
    pub row_count: u64,

    /// Number of pages (blocks)
    pub page_count: u64,

    /// Average row size in bytes
    pub avg_row_size: usize,

    /// Statistics per column
    pub column_stats: HashMap<String, ColumnStatistics>,

    /// Last update timestamp
    pub last_updated: Option<i64>,

    /// Sampling rate used (0.0 to 1.0)
    pub sample_rate: f64,
}

impl TableStatistics {
    pub fn new(table_name: impl Into<String>, row_count: u64, page_count: u64) -> Self {
        Self {
            table_name: table_name.into(),
            row_count,
            page_count,
            avg_row_size: 100, // Default estimate
            column_stats: HashMap::new(),
            last_updated: None,
            sample_rate: 1.0,
        }
    }

    pub fn default_for_table(table_name: impl Into<String>) -> Self {
        Self::new(table_name, 10000, 100) // Default estimates
    }

    pub fn add_column_stats(&mut self, column: String, stats: ColumnStatistics) {
        self.column_stats.insert(column, stats);
    }

    pub fn get_column_stats(&self, column: &str) -> Option<&ColumnStatistics> {
        self.column_stats.get(column)
    }

    pub fn estimated_selectivity(&self, column: &str, value: &str) -> f64 {
        self.column_stats
            .get(column)
            .map(|s| s.estimate_equality_selectivity())
            .unwrap_or(0.1)
    }
}

/// Statistics for a column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnStatistics {
    /// Column name
    pub column_name: String,

    /// Data type
    pub data_type: DataType,

    /// Number of distinct values
    pub distinct_count: u64,

    /// Number of NULL values
    pub null_count: u64,

    /// Minimum value (serialized)
    pub min_value: Option<StatValue>,

    /// Maximum value (serialized)
    pub max_value: Option<StatValue>,

    /// Most common values with frequencies
    pub most_common_values: Vec<(StatValue, f64)>,

    /// Histogram for value distribution
    pub histogram: Option<Histogram>,

    /// Average width in bytes (for variable-length types)
    pub avg_width: Option<usize>,

    /// Correlation with physical row order (-1.0 to 1.0)
    pub correlation: Option<f64>,
}

impl ColumnStatistics {
    pub fn new(column_name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            column_name: column_name.into(),
            data_type,
            distinct_count: 0,
            null_count: 0,
            min_value: None,
            max_value: None,
            most_common_values: Vec::new(),
            histogram: None,
            avg_width: None,
            correlation: None,
        }
    }

    /// Estimate selectivity for equality predicate (col = value)
    pub fn estimate_equality_selectivity(&self) -> f64 {
        if self.distinct_count == 0 {
            return 0.1; // Default estimate
        }
        1.0 / self.distinct_count as f64
    }

    /// Estimate selectivity for range predicate (col > value or col < value)
    pub fn estimate_range_selectivity(&self, _value: &StatValue, _is_greater: bool) -> f64 {
        if let Some(ref hist) = self.histogram {
            // Use histogram to estimate
            hist.estimate_selectivity_range(0.0, 0.5) // Simplified
        } else {
            0.33 // Default: assume 1/3 selectivity
        }
    }

    /// Estimate selectivity for BETWEEN predicate
    pub fn estimate_between_selectivity(&self, _low: &StatValue, _high: &StatValue) -> f64 {
        if let Some(ref hist) = self.histogram {
            hist.estimate_selectivity_range(0.0, 1.0)
        } else {
            0.5 // Default
        }
    }

    /// Check if column is unique (distinct_count ≈ table rows)
    pub fn is_unique(&self, table_row_count: u64) -> bool {
        if self.distinct_count == 0 {
            return false;
        }
        let ratio = self.distinct_count as f64 / table_row_count as f64;
        ratio > 0.95
    }

    /// Check if column has high cardinality
    pub fn is_high_cardinality(&self) -> bool {
        self.distinct_count > 10000
    }

    /// Check if column has skewed distribution
    pub fn is_skewed(&self) -> bool {
        if self.most_common_values.is_empty() {
            return false;
        }
        // If top value appears in >50% of rows, it's skewed
        self.most_common_values[0].1 > 0.5
    }
}

/// Statistical value (unified representation)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StatValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(OrderedFloat<f64>),
    String(String),
    Date(String),
    Timestamp(String),
}

impl StatValue {
    pub fn from_i64(val: i64) -> Self {
        StatValue::Integer(val)
    }

    pub fn from_f64(val: f64) -> Self {
        StatValue::Float(OrderedFloat(val))
    }

    pub fn from_string(val: impl Into<String>) -> Self {
        StatValue::String(val.into())
    }
}

/// Histogram for value distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Histogram {
    /// Type of histogram
    pub histogram_type: HistogramType,

    /// Histogram buckets
    pub buckets: Vec<HistogramBucket>,

    /// Total number of values represented
    pub total_count: u64,
}

impl Histogram {
    pub fn new(histogram_type: HistogramType) -> Self {
        Self {
            histogram_type,
            buckets: Vec::new(),
            total_count: 0,
        }
    }

    pub fn equi_width(min: f64, max: f64, num_buckets: usize, values: &[f64]) -> Self {
        let mut histogram = Self::new(HistogramType::EquiWidth);

        if num_buckets == 0 || values.is_empty() {
            return histogram;
        }

        let width = (max - min) / num_buckets as f64;
        let mut buckets = vec![HistogramBucket::default(); num_buckets];

        // Initialize bucket bounds
        for (i, bucket) in buckets.iter_mut().enumerate() {
            bucket.lower_bound = StatValue::from_f64(min + i as f64 * width);
            bucket.upper_bound = StatValue::from_f64(min + (i + 1) as f64 * width);
        }

        // Populate buckets
        for &value in values {
            let bucket_idx = ((value - min) / width).floor() as usize;
            let bucket_idx = bucket_idx.min(num_buckets - 1);
            buckets[bucket_idx].count += 1;
            buckets[bucket_idx].distinct_count += 1; // Simplified
        }

        histogram.buckets = buckets;
        histogram.total_count = values.len() as u64;
        histogram
    }

    pub fn equi_depth(values: &mut [f64], num_buckets: usize) -> Self {
        let mut histogram = Self::new(HistogramType::EquiDepth);

        if num_buckets == 0 || values.is_empty() {
            return histogram;
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let bucket_size = values.len() / num_buckets;
        let mut buckets = Vec::new();

        for i in 0..num_buckets {
            let start_idx = i * bucket_size;
            let end_idx = if i == num_buckets - 1 {
                values.len()
            } else {
                (i + 1) * bucket_size
            };

            if start_idx < values.len() {
                let bucket = HistogramBucket {
                    lower_bound: StatValue::from_f64(values[start_idx]),
                    upper_bound: StatValue::from_f64(values[end_idx.min(values.len()) - 1]),
                    count: (end_idx - start_idx) as u64,
                    distinct_count: (end_idx - start_idx) as u64, // Simplified
                };
                buckets.push(bucket);
            }
        }

        histogram.buckets = buckets;
        histogram.total_count = values.len() as u64;
        histogram
    }

    /// Estimate selectivity for a range query
    pub fn estimate_selectivity_range(&self, _start_fraction: f64, _end_fraction: f64) -> f64 {
        // Simplified: would calculate based on bucket overlaps
        0.5
    }

    /// Find bucket containing value
    pub fn find_bucket(&self, _value: &StatValue) -> Option<&HistogramBucket> {
        // Would implement binary search for the appropriate bucket
        self.buckets.first()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistogramType {
    /// Equal-width buckets
    EquiWidth,
    /// Equal-depth (equal count) buckets
    EquiDepth,
    /// Height-balanced
    HeightBalanced,
    /// Compressed (for sparse data)
    Compressed,
}

/// A single histogram bucket
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistogramBucket {
    /// Lower bound (inclusive)
    pub lower_bound: StatValue,
    /// Upper bound (inclusive)
    pub upper_bound: StatValue,
    /// Number of values in bucket
    pub count: u64,
    /// Number of distinct values in bucket
    pub distinct_count: u64,
}

impl Default for StatValue {
    fn default() -> Self {
        StatValue::Null
    }
}

/// Statistics collector
pub struct StatisticsCollector {
    sample_rate: f64,
}

impl StatisticsCollector {
    pub fn new(sample_rate: f64) -> Self {
        Self { sample_rate }
    }

    pub fn with_full_scan() -> Self {
        Self::new(1.0)
    }

    pub fn with_sampling(sample_rate: f64) -> Self {
        Self::new(sample_rate.clamp(0.01, 1.0))
    }

    /// Collect statistics for integer column
    pub fn collect_integer_stats(
        &self,
        column_name: impl Into<String>,
        values: &[i64],
    ) -> ColumnStatistics {
        let mut stats = ColumnStatistics::new(column_name, DataType::BigInt);

        if values.is_empty() {
            return stats;
        }

        // Distinct count (using HashSet)
        let mut distinct_values = std::collections::HashSet::new();
        let mut null_count = 0u64;
        let mut min = i64::MAX;
        let mut max = i64::MIN;

        for &value in values {
            distinct_values.insert(value);
            min = min.min(value);
            max = max.max(value);
        }

        stats.distinct_count = distinct_values.len() as u64;
        stats.null_count = null_count;
        stats.min_value = Some(StatValue::Integer(min));
        stats.max_value = Some(StatValue::Integer(max));

        // Most common values
        stats.most_common_values = self.find_most_common_integer(values, 10);

        // Histogram
        let float_values: Vec<f64> = values.iter().map(|&v| v as f64).collect();
        stats.histogram = Some(Histogram::equi_width(
            min as f64,
            max as f64,
            20,
            &float_values,
        ));

        stats
    }

    /// Collect statistics for string column
    pub fn collect_string_stats(
        &self,
        column_name: impl Into<String>,
        values: &[String],
    ) -> ColumnStatistics {
        let mut stats = ColumnStatistics::new(column_name, DataType::Varchar(None));

        if values.is_empty() {
            return stats;
        }

        let mut distinct_values = std::collections::HashSet::new();
        let mut total_width = 0usize;

        for value in values {
            distinct_values.insert(value.clone());
            total_width += value.len();
        }

        stats.distinct_count = distinct_values.len() as u64;
        stats.avg_width = Some(total_width / values.len());

        // Most common values
        stats.most_common_values = self.find_most_common_string(values, 10);

        stats
    }

    /// Find most common integer values
    fn find_most_common_integer(&self, values: &[i64], limit: usize) -> Vec<(StatValue, f64)> {
        let mut frequency_map: HashMap<i64, u64> = HashMap::new();

        for &value in values {
            *frequency_map.entry(value).or_insert(0) += 1;
        }

        let total = values.len() as f64;
        let mut frequencies: Vec<_> = frequency_map
            .into_iter()
            .map(|(val, count)| (StatValue::Integer(val), count as f64 / total))
            .collect();

        frequencies.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        frequencies.truncate(limit);
        frequencies
    }

    /// Find most common string values
    fn find_most_common_string(&self, values: &[String], limit: usize) -> Vec<(StatValue, f64)> {
        let mut frequency_map: HashMap<String, u64> = HashMap::new();

        for value in values {
            *frequency_map.entry(value.clone()).or_insert(0) += 1;
        }

        let total = values.len() as f64;
        let mut frequencies: Vec<_> = frequency_map
            .into_iter()
            .map(|(val, count)| (StatValue::String(val), count as f64 / total))
            .collect();

        frequencies.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        frequencies.truncate(limit);
        frequencies
    }
}

/// Statistics manager
pub struct StatisticsManager {
    tables: HashMap<String, TableStatistics>,
}

impl StatisticsManager {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn add_table(&mut self, stats: TableStatistics) {
        self.tables.insert(stats.table_name.clone(), stats);
    }

    pub fn get_table(&self, table_name: &str) -> Option<&TableStatistics> {
        self.tables.get(table_name)
    }

    pub fn update_column_stats(
        &mut self,
        table_name: &str,
        column_name: String,
        stats: ColumnStatistics,
    ) {
        if let Some(table_stats) = self.tables.get_mut(table_name) {
            table_stats.add_column_stats(column_name, stats);
        }
    }

    pub fn estimate_join_cardinality(
        &self,
        left_table: &str,
        right_table: &str,
        _join_columns: &[(String, String)],
    ) -> Option<f64> {
        let left_stats = self.get_table(left_table)?;
        let right_stats = self.get_table(right_table)?;

        // Simplified: product / max(distinct values)
        let left_rows = left_stats.row_count as f64;
        let right_rows = right_stats.row_count as f64;

        // Estimate: geometric mean
        Some((left_rows * right_rows).sqrt())
    }
}

impl Default for StatisticsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_statistics() {
        let mut stats = TableStatistics::new("users", 1000, 100);
        assert_eq!(stats.row_count, 1000);
        assert_eq!(stats.page_count, 100);

        let col_stats = ColumnStatistics::new("id", DataType::BigInt);
        stats.add_column_stats("id".to_string(), col_stats);
        assert!(stats.get_column_stats("id").is_some());
    }

    #[test]
    fn test_column_statistics() {
        let mut stats = ColumnStatistics::new("age", DataType::Integer);
        stats.distinct_count = 100;

        let selectivity = stats.estimate_equality_selectivity();
        assert_eq!(selectivity, 0.01);
    }

    #[test]
    fn test_histogram_equi_width() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let hist = Histogram::equi_width(1.0, 10.0, 5, &values);

        assert_eq!(hist.buckets.len(), 5);
        assert_eq!(hist.total_count, 10);
    }

    #[test]
    fn test_statistics_collector() {
        let collector = StatisticsCollector::with_full_scan();
        let values = vec![1, 2, 3, 4, 5, 1, 2, 1];

        let stats = collector.collect_integer_stats("test_col", &values);
        assert_eq!(stats.distinct_count, 5);
        assert!(!stats.most_common_values.is_empty());
    }

    #[test]
    fn test_statistics_manager() {
        let mut manager = StatisticsManager::new();
        let table_stats = TableStatistics::new("users", 1000, 100);

        manager.add_table(table_stats);
        assert!(manager.get_table("users").is_some());
        assert!(manager.get_table("nonexistent").is_none());
    }
}
