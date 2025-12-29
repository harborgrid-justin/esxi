//! Temporal feature extraction for time series data

use crate::error::{MlError, Result};
use crate::features::{FeatureExtractor, FeatureInput, FeatureSet};
use chrono::{DateTime, Datelike, Timelike, Utc};
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

/// Temporal features extracted from time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalFeatures {
    /// Feature matrix
    pub features: Array2<f64>,

    /// Feature names
    pub names: Vec<String>,

    /// Timestamps
    pub timestamps: Vec<DateTime<Utc>>,
}

/// Temporal feature extractor
pub struct TemporalFeatureExtractor {
    /// Feature types to extract
    feature_types: Vec<TemporalFeatureType>,

    /// Lag values for autoregressive features
    lags: Vec<usize>,

    /// Window size for rolling statistics
    window_size: usize,
}

/// Types of temporal features
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalFeatureType {
    /// Year
    Year,

    /// Month (1-12)
    Month,

    /// Day of month (1-31)
    DayOfMonth,

    /// Day of week (0-6)
    DayOfWeek,

    /// Hour of day (0-23)
    Hour,

    /// Minute of hour (0-59)
    Minute,

    /// Day of year (1-366)
    DayOfYear,

    /// Quarter (1-4)
    Quarter,

    /// Is weekend (0 or 1)
    IsWeekend,

    /// Sin-encoded hour (for cyclical patterns)
    HourSin,

    /// Cos-encoded hour
    HourCos,

    /// Sin-encoded day of week
    DayOfWeekSin,

    /// Cos-encoded day of week
    DayOfWeekCos,

    /// Sin-encoded month
    MonthSin,

    /// Cos-encoded month
    MonthCos,

    /// Rolling mean
    RollingMean,

    /// Rolling std
    RollingStd,

    /// Rolling min
    RollingMin,

    /// Rolling max
    RollingMax,

    /// Trend
    Trend,

    /// Lag values
    Lag,
}

impl TemporalFeatureType {
    /// Get the feature name
    pub fn name(&self) -> &str {
        match self {
            Self::Year => "year",
            Self::Month => "month",
            Self::DayOfMonth => "day_of_month",
            Self::DayOfWeek => "day_of_week",
            Self::Hour => "hour",
            Self::Minute => "minute",
            Self::DayOfYear => "day_of_year",
            Self::Quarter => "quarter",
            Self::IsWeekend => "is_weekend",
            Self::HourSin => "hour_sin",
            Self::HourCos => "hour_cos",
            Self::DayOfWeekSin => "dow_sin",
            Self::DayOfWeekCos => "dow_cos",
            Self::MonthSin => "month_sin",
            Self::MonthCos => "month_cos",
            Self::RollingMean => "rolling_mean",
            Self::RollingStd => "rolling_std",
            Self::RollingMin => "rolling_min",
            Self::RollingMax => "rolling_max",
            Self::Trend => "trend",
            Self::Lag => "lag",
        }
    }

    /// Get calendar-based features
    pub fn calendar() -> Vec<Self> {
        vec![
            Self::Year,
            Self::Month,
            Self::DayOfMonth,
            Self::DayOfWeek,
            Self::Hour,
        ]
    }

    /// Get cyclical features
    pub fn cyclical() -> Vec<Self> {
        vec![
            Self::HourSin,
            Self::HourCos,
            Self::DayOfWeekSin,
            Self::DayOfWeekCos,
            Self::MonthSin,
            Self::MonthCos,
        ]
    }
}

impl TemporalFeatureExtractor {
    /// Create a new temporal feature extractor
    pub fn new() -> Self {
        Self {
            feature_types: TemporalFeatureType::calendar(),
            lags: vec![1, 7, 30],
            window_size: 7,
        }
    }

    /// Set feature types
    pub fn with_features(mut self, types: Vec<TemporalFeatureType>) -> Self {
        self.feature_types = types;
        self
    }

    /// Set lag values
    pub fn with_lags(mut self, lags: Vec<usize>) -> Self {
        self.lags = lags;
        self
    }

    /// Set window size for rolling statistics
    pub fn with_window_size(mut self, size: usize) -> Self {
        self.window_size = size;
        self
    }

    /// Extract features from a time series
    pub fn extract_from_series(
        &self,
        timestamps: &[DateTime<Utc>],
        values: &[f64],
    ) -> Result<FeatureSet> {
        if timestamps.len() != values.len() {
            return Err(MlError::InvalidInput(
                "Timestamps and values must have same length".to_string(),
            ));
        }

        if timestamps.is_empty() {
            return Err(MlError::EmptyDataset);
        }

        let n_samples = timestamps.len();
        let mut feature_list = Vec::new();

        for (i, ts) in timestamps.iter().enumerate() {
            let mut features = Vec::new();

            for feature_type in &self.feature_types {
                let value = match feature_type {
                    TemporalFeatureType::Year => ts.year() as f64,
                    TemporalFeatureType::Month => ts.month() as f64,
                    TemporalFeatureType::DayOfMonth => ts.day() as f64,
                    TemporalFeatureType::DayOfWeek => ts.weekday().num_days_from_monday() as f64,
                    TemporalFeatureType::Hour => ts.hour() as f64,
                    TemporalFeatureType::Minute => ts.minute() as f64,
                    TemporalFeatureType::DayOfYear => ts.ordinal() as f64,
                    TemporalFeatureType::Quarter => ((ts.month() - 1) / 3 + 1) as f64,
                    TemporalFeatureType::IsWeekend => {
                        let dow = ts.weekday().num_days_from_monday();
                        if dow >= 5 {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    TemporalFeatureType::HourSin => {
                        let hour = ts.hour() as f64;
                        (2.0 * std::f64::consts::PI * hour / 24.0).sin()
                    }
                    TemporalFeatureType::HourCos => {
                        let hour = ts.hour() as f64;
                        (2.0 * std::f64::consts::PI * hour / 24.0).cos()
                    }
                    TemporalFeatureType::DayOfWeekSin => {
                        let dow = ts.weekday().num_days_from_monday() as f64;
                        (2.0 * std::f64::consts::PI * dow / 7.0).sin()
                    }
                    TemporalFeatureType::DayOfWeekCos => {
                        let dow = ts.weekday().num_days_from_monday() as f64;
                        (2.0 * std::f64::consts::PI * dow / 7.0).cos()
                    }
                    TemporalFeatureType::MonthSin => {
                        let month = ts.month() as f64;
                        (2.0 * std::f64::consts::PI * month / 12.0).sin()
                    }
                    TemporalFeatureType::MonthCos => {
                        let month = ts.month() as f64;
                        (2.0 * std::f64::consts::PI * month / 12.0).cos()
                    }
                    TemporalFeatureType::RollingMean => {
                        self.rolling_mean(values, i)
                    }
                    TemporalFeatureType::RollingStd => {
                        self.rolling_std(values, i)
                    }
                    TemporalFeatureType::RollingMin => {
                        self.rolling_min(values, i)
                    }
                    TemporalFeatureType::RollingMax => {
                        self.rolling_max(values, i)
                    }
                    TemporalFeatureType::Trend => i as f64,
                    TemporalFeatureType::Lag => 0.0, // Will be handled separately
                };
                features.push(value);
            }

            // Add lag features
            for &lag in &self.lags {
                if i >= lag {
                    features.push(values[i - lag]);
                } else {
                    features.push(0.0); // Padding for early samples
                }
            }

            feature_list.push(Array1::from_vec(features));
        }

        let n_features = feature_list[0].len();
        let mut feature_matrix = Array2::zeros((n_samples, n_features));

        for (i, feat) in feature_list.iter().enumerate() {
            feature_matrix.row_mut(i).assign(feat);
        }

        let mut names: Vec<String> = self
            .feature_types
            .iter()
            .map(|t| t.name().to_string())
            .collect();

        for lag in &self.lags {
            names.push(format!("lag_{}", lag));
        }

        Ok(FeatureSet::new(feature_matrix, names))
    }

    /// Calculate rolling mean
    fn rolling_mean(&self, values: &[f64], index: usize) -> f64 {
        let start = index.saturating_sub(self.window_size - 1);
        let window = &values[start..=index];
        window.iter().sum::<f64>() / window.len() as f64
    }

    /// Calculate rolling standard deviation
    fn rolling_std(&self, values: &[f64], index: usize) -> f64 {
        let start = index.saturating_sub(self.window_size - 1);
        let window = &values[start..=index];
        let mean = window.iter().sum::<f64>() / window.len() as f64;
        let variance = window
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / window.len() as f64;
        variance.sqrt()
    }

    /// Calculate rolling minimum
    fn rolling_min(&self, values: &[f64], index: usize) -> f64 {
        let start = index.saturating_sub(self.window_size - 1);
        let window = &values[start..=index];
        window.iter().cloned().fold(f64::INFINITY, f64::min)
    }

    /// Calculate rolling maximum
    fn rolling_max(&self, values: &[f64], index: usize) -> f64 {
        let start = index.saturating_sub(self.window_size - 1);
        let window = &values[start..=index];
        window.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }
}

impl Default for TemporalFeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureExtractor for TemporalFeatureExtractor {
    fn extract(&self, input: &FeatureInput) -> Result<FeatureSet> {
        match input {
            FeatureInput::TimeSeries(data) => {
                let (timestamps, values): (Vec<_>, Vec<_>) = data.iter().cloned().unzip();
                self.extract_from_series(&timestamps, &values)
            }
            _ => Err(MlError::FeatureExtraction(
                "Expected time series input".to_string(),
            )),
        }
    }

    fn num_features(&self) -> usize {
        self.feature_types.len() + self.lags.len()
    }

    fn feature_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .feature_types
            .iter()
            .map(|t| t.name().to_string())
            .collect();

        for lag in &self.lags {
            names.push(format!("lag_{}", lag));
        }

        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_temporal_feature_extractor() {
        let extractor = TemporalFeatureExtractor::new();
        assert_eq!(extractor.window_size, 7);
    }

    #[test]
    fn test_feature_extraction() {
        let timestamps = vec![
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap(),
        ];
        let values = vec![1.0, 2.0, 3.0];

        let extractor = TemporalFeatureExtractor::new();
        let features = extractor.extract_from_series(&timestamps, &values).unwrap();

        assert_eq!(features.num_samples(), 3);
        assert!(features.num_features() > 0);
    }
}
