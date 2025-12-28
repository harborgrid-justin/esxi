//! Tenant analytics and usage tracking.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{TenantError, TenantResult};

/// Analytics event types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    PageView,
    ApiRequest,
    MapView,
    LayerCreated,
    LayerViewed,
    LayerEdited,
    DataExport,
    DataImport,
    UserLogin,
    UserSignup,
    FeatureUsed,
    Error,
    Custom,
}

/// Analytics event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub event_type: EventType,
    pub event_name: String,
    pub user_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub properties: HashMap<String, serde_json::Value>,
    pub context: EventContext,
}

/// Context information for analytics events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContext {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub referrer: Option<String>,
    pub session_id: Option<String>,
    pub device_type: Option<DeviceType>,
    pub browser: Option<String>,
    pub os: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Desktop,
    Mobile,
    Tablet,
    Unknown,
}

impl AnalyticsEvent {
    pub fn new(
        tenant_id: Uuid,
        event_type: EventType,
        event_name: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            event_type,
            event_name: event_name.into(),
            user_id: None,
            timestamp: Utc::now(),
            properties: HashMap::new(),
            context: EventContext::default(),
        }
    }

    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_property(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.properties.insert(key.into(), value);
        self
    }

    pub fn with_context(mut self, context: EventContext) -> Self {
        self.context = context;
        self
    }
}

impl Default for EventContext {
    fn default() -> Self {
        Self {
            ip_address: None,
            user_agent: None,
            referrer: None,
            session_id: None,
            device_type: None,
            browser: None,
            os: None,
        }
    }
}

/// Time-series data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub metadata: HashMap<String, String>,
}

/// Aggregated analytics metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub tenant_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events: u64,
    pub unique_users: u64,
    pub events_by_type: HashMap<EventType, u64>,
    pub time_series: Vec<TimeSeriesPoint>,
}

/// User activity summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivitySummary {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub total_events: u64,
    pub total_sessions: u64,
    pub average_session_duration: Duration,
    pub most_used_features: Vec<(String, u64)>,
}

/// Tenant analytics manager.
pub struct AnalyticsManager {
    events: Vec<AnalyticsEvent>,
}

impl AnalyticsManager {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    /// Tracks an analytics event.
    pub fn track(&mut self, event: AnalyticsEvent) -> TenantResult<()> {
        tracing::debug!(
            "Tracking event: {} for tenant {}",
            event.event_name,
            event.tenant_id
        );

        self.events.push(event);
        Ok(())
    }

    /// Gets events for a tenant in a date range.
    pub fn get_events(
        &self,
        tenant_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&AnalyticsEvent> {
        self.events
            .iter()
            .filter(|e| {
                e.tenant_id == tenant_id && e.timestamp >= start && e.timestamp <= end
            })
            .collect()
    }

    /// Gets events by type for a tenant.
    pub fn get_events_by_type(
        &self,
        tenant_id: Uuid,
        event_type: EventType,
        limit: Option<usize>,
    ) -> Vec<&AnalyticsEvent> {
        let mut events: Vec<&AnalyticsEvent> = self
            .events
            .iter()
            .filter(|e| e.tenant_id == tenant_id && e.event_type == event_type)
            .collect();

        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            events.truncate(limit);
        }

        events
    }

    /// Generates a metrics summary for a tenant.
    pub fn generate_summary(
        &self,
        tenant_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> MetricsSummary {
        let events = self.get_events(tenant_id, start, end);

        let mut events_by_type: HashMap<EventType, u64> = HashMap::new();
        let mut unique_users = std::collections::HashSet::new();

        for event in &events {
            *events_by_type.entry(event.event_type).or_insert(0) += 1;
            if let Some(user_id) = event.user_id {
                unique_users.insert(user_id);
            }
        }

        // Generate time series (hourly buckets)
        let time_series = self.generate_time_series(tenant_id, start, end, Duration::hours(1));

        MetricsSummary {
            tenant_id,
            period_start: start,
            period_end: end,
            total_events: events.len() as u64,
            unique_users: unique_users.len() as u64,
            events_by_type,
            time_series,
        }
    }

    /// Generates time-series data.
    fn generate_time_series(
        &self,
        tenant_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        bucket_size: Duration,
    ) -> Vec<TimeSeriesPoint> {
        let mut buckets: HashMap<DateTime<Utc>, u64> = HashMap::new();

        for event in self.get_events(tenant_id, start, end) {
            let bucket_time = self.round_to_bucket(event.timestamp, bucket_size, start);
            *buckets.entry(bucket_time).or_insert(0) += 1;
        }

        let mut time_series: Vec<TimeSeriesPoint> = buckets
            .into_iter()
            .map(|(timestamp, count)| TimeSeriesPoint {
                timestamp,
                value: count as f64,
                metadata: HashMap::new(),
            })
            .collect();

        time_series.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        time_series
    }

    fn round_to_bucket(
        &self,
        timestamp: DateTime<Utc>,
        bucket_size: Duration,
        start: DateTime<Utc>,
    ) -> DateTime<Utc> {
        let elapsed = timestamp - start;
        let bucket_seconds = bucket_size.num_seconds();
        let bucket_index = elapsed.num_seconds() / bucket_seconds;
        start + Duration::seconds(bucket_index * bucket_seconds)
    }

    /// Gets user activity summary.
    pub fn get_user_activity(&self, tenant_id: Uuid, user_id: Uuid) -> Option<UserActivitySummary> {
        let user_events: Vec<&AnalyticsEvent> = self
            .events
            .iter()
            .filter(|e| e.tenant_id == tenant_id && e.user_id == Some(user_id))
            .collect();

        if user_events.is_empty() {
            return None;
        }

        let first_seen = user_events.iter().map(|e| e.timestamp).min().unwrap();
        let last_seen = user_events.iter().map(|e| e.timestamp).max().unwrap();

        // Count feature usage
        let mut feature_usage: HashMap<String, u64> = HashMap::new();
        for event in &user_events {
            if event.event_type == EventType::FeatureUsed {
                *feature_usage.entry(event.event_name.clone()).or_insert(0) += 1;
            }
        }

        let mut most_used: Vec<(String, u64)> = feature_usage.into_iter().collect();
        most_used.sort_by(|a, b| b.1.cmp(&a.1));
        most_used.truncate(10);

        Some(UserActivitySummary {
            user_id,
            tenant_id,
            first_seen,
            last_seen,
            total_events: user_events.len() as u64,
            total_sessions: 1, // Simplified
            average_session_duration: Duration::minutes(15), // Simplified
            most_used_features: most_used,
        })
    }

    /// Gets top events by frequency.
    pub fn get_top_events(&self, tenant_id: Uuid, limit: usize) -> Vec<(String, u64)> {
        let mut event_counts: HashMap<String, u64> = HashMap::new();

        for event in &self.events {
            if event.tenant_id == tenant_id {
                *event_counts.entry(event.event_name.clone()).or_insert(0) += 1;
            }
        }

        let mut top_events: Vec<(String, u64)> = event_counts.into_iter().collect();
        top_events.sort_by(|a, b| b.1.cmp(&a.1));
        top_events.truncate(limit);

        top_events
    }

    /// Calculates retention rate.
    pub fn calculate_retention(
        &self,
        tenant_id: Uuid,
        cohort_start: DateTime<Utc>,
        cohort_size_days: i64,
        retention_period_days: i64,
    ) -> f64 {
        let cohort_end = cohort_start + Duration::days(cohort_size_days);
        let retention_end = cohort_end + Duration::days(retention_period_days);

        // Users who signed up in cohort period
        let cohort_users: std::collections::HashSet<Uuid> = self
            .events
            .iter()
            .filter(|e| {
                e.tenant_id == tenant_id
                    && e.event_type == EventType::UserSignup
                    && e.timestamp >= cohort_start
                    && e.timestamp < cohort_end
            })
            .filter_map(|e| e.user_id)
            .collect();

        if cohort_users.is_empty() {
            return 0.0;
        }

        // Users who returned during retention period
        let retained_users: std::collections::HashSet<Uuid> = self
            .events
            .iter()
            .filter(|e| {
                e.tenant_id == tenant_id
                    && e.timestamp >= cohort_end
                    && e.timestamp < retention_end
            })
            .filter_map(|e| e.user_id)
            .filter(|user_id| cohort_users.contains(user_id))
            .collect();

        (retained_users.len() as f64 / cohort_users.len() as f64) * 100.0
    }
}

impl Default for AnalyticsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Funnel analysis for conversion tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Funnel {
    pub name: String,
    pub steps: Vec<FunnelStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelStep {
    pub name: String,
    pub event_type: EventType,
    pub event_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelAnalysis {
    pub funnel: Funnel,
    pub total_users: u64,
    pub step_results: Vec<FunnelStepResult>,
    pub overall_conversion_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelStepResult {
    pub step_name: String,
    pub users_count: u64,
    pub conversion_rate: f64,
    pub drop_off_rate: f64,
}

impl AnalyticsManager {
    /// Analyzes a conversion funnel.
    pub fn analyze_funnel(
        &self,
        tenant_id: Uuid,
        funnel: Funnel,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> FunnelAnalysis {
        let mut step_results = Vec::new();
        let mut previous_users: Option<std::collections::HashSet<Uuid>> = None;

        for step in &funnel.steps {
            let step_events: Vec<&AnalyticsEvent> = self
                .events
                .iter()
                .filter(|e| {
                    e.tenant_id == tenant_id
                        && e.event_type == step.event_type
                        && e.timestamp >= start
                        && e.timestamp <= end
                        && step.event_name.as_ref().map_or(true, |name| &e.event_name == name)
                })
                .collect();

            let step_users: std::collections::HashSet<Uuid> =
                step_events.iter().filter_map(|e| e.user_id).collect();

            let users_count = if let Some(prev) = &previous_users {
                // Only count users who completed previous step
                step_users.intersection(prev).count() as u64
            } else {
                step_users.len() as u64
            };

            let conversion_rate = if let Some(prev) = &previous_users {
                if prev.is_empty() {
                    0.0
                } else {
                    (users_count as f64 / prev.len() as f64) * 100.0
                }
            } else {
                100.0
            };

            let drop_off_rate = 100.0 - conversion_rate;

            step_results.push(FunnelStepResult {
                step_name: step.name.clone(),
                users_count,
                conversion_rate,
                drop_off_rate,
            });

            previous_users = Some(step_users);
        }

        let total_users = step_results.first().map(|s| s.users_count).unwrap_or(0);
        let final_users = step_results.last().map(|s| s.users_count).unwrap_or(0);

        let overall_conversion_rate = if total_users > 0 {
            (final_users as f64 / total_users as f64) * 100.0
        } else {
            0.0
        };

        FunnelAnalysis {
            funnel,
            total_users,
            step_results,
            overall_conversion_rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_event() {
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let event = AnalyticsEvent::new(tenant_id, EventType::PageView, "home_page")
            .with_user(user_id)
            .with_property("path", serde_json::json!("/home"));

        assert_eq!(event.tenant_id, tenant_id);
        assert_eq!(event.user_id, Some(user_id));
        assert!(event.properties.contains_key("path"));
    }

    #[test]
    fn test_analytics_manager() {
        let mut manager = AnalyticsManager::new();
        let tenant_id = Uuid::new_v4();

        let event = AnalyticsEvent::new(tenant_id, EventType::ApiRequest, "get_layers");
        assert!(manager.track(event).is_ok());

        let events = manager.get_events_by_type(tenant_id, EventType::ApiRequest, None);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_metrics_summary() {
        let mut manager = AnalyticsManager::new();
        let tenant_id = Uuid::new_v4();
        let start = Utc::now() - Duration::hours(1);
        let end = Utc::now();

        // Track some events
        for _ in 0..5 {
            manager.track(AnalyticsEvent::new(tenant_id, EventType::PageView, "test")).unwrap();
        }

        let summary = manager.generate_summary(tenant_id, start, end);
        assert_eq!(summary.total_events, 5);
    }
}
