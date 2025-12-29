//! Attribute filtering and optimization

use crate::encoding::mvt::MvtValue;
use std::collections::{HashMap, HashSet};

/// Attribute filter for optimizing feature properties
pub struct AttributeFilter {
    /// Allowed attributes (None = allow all)
    allowed: Option<HashSet<String>>,
    /// Blocked attributes
    blocked: HashSet<String>,
    /// Maximum string length
    max_string_length: Option<usize>,
    /// Numeric precision
    numeric_precision: Option<u32>,
}

impl AttributeFilter {
    /// Create a new attribute filter
    pub fn new() -> Self {
        Self {
            allowed: None,
            blocked: HashSet::new(),
            max_string_length: None,
            numeric_precision: None,
        }
    }

    /// Set allowed attributes (whitelist)
    pub fn with_allowed(mut self, attrs: Vec<String>) -> Self {
        self.allowed = Some(attrs.into_iter().collect());
        self
    }

    /// Add blocked attributes (blacklist)
    pub fn with_blocked(mut self, attrs: Vec<String>) -> Self {
        self.blocked = attrs.into_iter().collect();
        self
    }

    /// Set maximum string length
    pub fn with_max_string_length(mut self, max: usize) -> Self {
        self.max_string_length = Some(max);
        self
    }

    /// Set numeric precision (decimal places)
    pub fn with_numeric_precision(mut self, precision: u32) -> Self {
        self.numeric_precision = Some(precision);
        self
    }

    /// Filter attributes for a feature
    pub fn filter(&self, properties: &HashMap<String, MvtValue>) -> HashMap<String, MvtValue> {
        let mut filtered = HashMap::new();

        for (key, value) in properties {
            // Check if attribute is allowed
            if !self.is_allowed(key) {
                continue;
            }

            // Process the value
            if let Some(processed) = self.process_value(value) {
                filtered.insert(key.clone(), processed);
            }
        }

        filtered
    }

    /// Check if an attribute is allowed
    fn is_allowed(&self, key: &str) -> bool {
        // Check blacklist first
        if self.blocked.contains(key) {
            return false;
        }

        // Check whitelist if present
        if let Some(ref allowed) = self.allowed {
            allowed.contains(key)
        } else {
            true
        }
    }

    /// Process a value (truncate strings, round numbers, etc.)
    fn process_value(&self, value: &MvtValue) -> Option<MvtValue> {
        match value {
            MvtValue::String(s) => {
                let s = if let Some(max_len) = self.max_string_length {
                    if s.len() > max_len {
                        s.chars().take(max_len).collect()
                    } else {
                        s.clone()
                    }
                } else {
                    s.clone()
                };
                Some(MvtValue::String(s))
            }
            MvtValue::Double(d) => {
                let d = if let Some(precision) = self.numeric_precision {
                    let factor = 10.0_f64.powi(precision as i32);
                    (d * factor).round() / factor
                } else {
                    *d
                };
                Some(MvtValue::Double(d))
            }
            MvtValue::Float(f) => {
                let f = if let Some(precision) = self.numeric_precision {
                    let factor = 10.0_f32.powi(precision as i32);
                    (f * factor).round() / factor
                } else {
                    *f
                };
                Some(MvtValue::Float(f))
            }
            _ => Some(value.clone()),
        }
    }

    /// Optimize properties by removing redundant or excessive data
    pub fn optimize(&self, properties: &HashMap<String, MvtValue>) -> HashMap<String, MvtValue> {
        self.filter(properties)
    }
}

impl Default for AttributeFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Attribute statistics for optimization analysis
#[derive(Debug, Default)]
pub struct AttributeStats {
    /// Attribute name
    pub name: String,
    /// Number of occurrences
    pub count: usize,
    /// Number of unique values
    pub unique_values: usize,
    /// Total size in bytes
    pub total_size: usize,
    /// Average size in bytes
    pub avg_size: f64,
}

/// Analyze attributes to help with optimization
pub struct AttributeAnalyzer {
    stats: HashMap<String, AttributeStats>,
}

impl AttributeAnalyzer {
    /// Create a new attribute analyzer
    pub fn new() -> Self {
        Self {
            stats: HashMap::new(),
        }
    }

    /// Analyze feature properties
    pub fn analyze(&mut self, properties: &HashMap<String, MvtValue>) {
        for (key, value) in properties {
            let stats = self.stats.entry(key.clone()).or_insert_with(|| AttributeStats {
                name: key.clone(),
                ..Default::default()
            });

            stats.count += 1;
            stats.total_size += self.value_size(value);
        }
    }

    /// Get statistics for all attributes
    pub fn get_stats(&mut self) -> Vec<AttributeStats> {
        let mut stats: Vec<_> = self.stats.values().cloned().collect();

        // Calculate averages
        for stat in &mut stats {
            stat.avg_size = stat.total_size as f64 / stat.count as f64;
        }

        // Sort by total size descending
        stats.sort_by(|a, b| b.total_size.cmp(&a.total_size));

        stats
    }

    /// Estimate size of a value in bytes
    fn value_size(&self, value: &MvtValue) -> usize {
        match value {
            MvtValue::String(s) => s.len(),
            MvtValue::Float(_) => 4,
            MvtValue::Double(_) => 8,
            MvtValue::Int(_) | MvtValue::UInt(_) | MvtValue::SInt(_) => 8,
            MvtValue::Bool(_) => 1,
        }
    }
}

impl Default for AttributeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_filter() {
        let filter = AttributeFilter::new().with_allowed(vec!["name".to_string(), "type".to_string()]);

        let mut props = HashMap::new();
        props.insert("name".to_string(), MvtValue::String("Test".to_string()));
        props.insert("type".to_string(), MvtValue::String("Feature".to_string()));
        props.insert("hidden".to_string(), MvtValue::String("Secret".to_string()));

        let filtered = filter.filter(&props);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains_key("name"));
        assert!(filtered.contains_key("type"));
        assert!(!filtered.contains_key("hidden"));
    }

    #[test]
    fn test_string_truncation() {
        let filter = AttributeFilter::new().with_max_string_length(5);

        let mut props = HashMap::new();
        props.insert(
            "name".to_string(),
            MvtValue::String("LongName".to_string()),
        );

        let filtered = filter.filter(&props);
        if let Some(MvtValue::String(s)) = filtered.get("name") {
            assert_eq!(s.len(), 5);
        }
    }

    #[test]
    fn test_numeric_precision() {
        let filter = AttributeFilter::new().with_numeric_precision(2);

        let mut props = HashMap::new();
        props.insert("value".to_string(), MvtValue::Double(3.14159));

        let filtered = filter.filter(&props);
        if let Some(MvtValue::Double(d)) = filtered.get("value") {
            assert!((d - 3.14).abs() < 1e-6);
        }
    }
}
