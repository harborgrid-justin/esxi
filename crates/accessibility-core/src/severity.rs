//! Severity levels for accessibility issues

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Severity level for accessibility issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeverityLevel {
    /// Critical accessibility barrier that prevents access for many users
    Critical,
    /// Serious accessibility issue that significantly impacts user experience
    Serious,
    /// Moderate accessibility issue that affects some users
    Moderate,
    /// Minor accessibility improvement that would enhance user experience
    Minor,
    /// Informational note about accessibility best practices
    Info,
}

impl SeverityLevel {
    /// Get priority value (higher = more severe)
    pub fn priority(&self) -> u8 {
        match self {
            SeverityLevel::Critical => 5,
            SeverityLevel::Serious => 4,
            SeverityLevel::Moderate => 3,
            SeverityLevel::Minor => 2,
            SeverityLevel::Info => 1,
        }
    }

    /// Get description of the severity level
    pub fn description(&self) -> &'static str {
        match self {
            SeverityLevel::Critical => {
                "Critical accessibility barrier that prevents access for many users"
            }
            SeverityLevel::Serious => {
                "Serious accessibility issue that significantly impacts user experience"
            }
            SeverityLevel::Moderate => "Moderate accessibility issue that affects some users",
            SeverityLevel::Minor => {
                "Minor accessibility improvement that would enhance user experience"
            }
            SeverityLevel::Info => "Informational note about accessibility best practices",
        }
    }

    /// Get display label
    pub fn label(&self) -> &'static str {
        match self {
            SeverityLevel::Critical => "Critical",
            SeverityLevel::Serious => "Serious",
            SeverityLevel::Moderate => "Moderate",
            SeverityLevel::Minor => "Minor",
            SeverityLevel::Info => "Info",
        }
    }

    /// Get icon representation
    pub fn icon(&self) -> &'static str {
        match self {
            SeverityLevel::Critical => "🔴",
            SeverityLevel::Serious => "🟠",
            SeverityLevel::Moderate => "🟡",
            SeverityLevel::Minor => "🔵",
            SeverityLevel::Info => "🟢",
        }
    }

    /// Get color scheme for UI display
    pub fn colors(&self) -> SeverityColors {
        match self {
            SeverityLevel::Critical => SeverityColors {
                bg: "#FEE2E2",
                text: "#991B1B",
                border: "#DC2626",
            },
            SeverityLevel::Serious => SeverityColors {
                bg: "#FED7AA",
                text: "#9A3412",
                border: "#EA580C",
            },
            SeverityLevel::Moderate => SeverityColors {
                bg: "#FEF3C7",
                text: "#92400E",
                border: "#F59E0B",
            },
            SeverityLevel::Minor => SeverityColors {
                bg: "#DBEAFE",
                text: "#1E40AF",
                border: "#3B82F6",
            },
            SeverityLevel::Info => SeverityColors {
                bg: "#E0E7FF",
                text: "#3730A3",
                border: "#6366F1",
            },
        }
    }

    /// Check if severity is at least as severe as threshold
    pub fn is_at_least(&self, threshold: SeverityLevel) -> bool {
        self.priority() >= threshold.priority()
    }

    /// Get all severity levels ordered from most to least severe
    pub fn all_ordered() -> Vec<SeverityLevel> {
        vec![
            SeverityLevel::Critical,
            SeverityLevel::Serious,
            SeverityLevel::Moderate,
            SeverityLevel::Minor,
            SeverityLevel::Info,
        ]
    }

    /// Get severity level from priority value
    pub fn from_priority(priority: u8) -> Option<SeverityLevel> {
        match priority {
            5 => Some(SeverityLevel::Critical),
            4 => Some(SeverityLevel::Serious),
            3 => Some(SeverityLevel::Moderate),
            2 => Some(SeverityLevel::Minor),
            1 => Some(SeverityLevel::Info),
            _ => None,
        }
    }
}

impl std::fmt::Display for SeverityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl PartialOrd for SeverityLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SeverityLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority().cmp(&other.priority())
    }
}

/// Color scheme for severity level
#[derive(Debug, Clone, Copy)]
pub struct SeverityColors {
    /// Background color (hex)
    pub bg: &'static str,
    /// Text color (hex)
    pub text: &'static str,
    /// Border color (hex)
    pub border: &'static str,
}

/// Compare two severity levels
pub fn compare_severity(a: SeverityLevel, b: SeverityLevel) -> Ordering {
    a.priority().cmp(&b.priority())
}

/// Sort severity levels from most to least severe
pub fn sort_severities(mut severities: Vec<SeverityLevel>) -> Vec<SeverityLevel> {
    severities.sort_by(|a, b| b.priority().cmp(&a.priority()));
    severities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_priority() {
        assert_eq!(SeverityLevel::Critical.priority(), 5);
        assert_eq!(SeverityLevel::Serious.priority(), 4);
        assert_eq!(SeverityLevel::Moderate.priority(), 3);
        assert_eq!(SeverityLevel::Minor.priority(), 2);
        assert_eq!(SeverityLevel::Info.priority(), 1);
    }

    #[test]
    fn test_severity_comparison() {
        assert!(SeverityLevel::Critical > SeverityLevel::Serious);
        assert!(SeverityLevel::Serious > SeverityLevel::Moderate);
        assert!(SeverityLevel::Moderate > SeverityLevel::Minor);
        assert!(SeverityLevel::Minor > SeverityLevel::Info);
    }

    #[test]
    fn test_is_at_least() {
        assert!(SeverityLevel::Critical.is_at_least(SeverityLevel::Moderate));
        assert!(!SeverityLevel::Minor.is_at_least(SeverityLevel::Serious));
        assert!(SeverityLevel::Serious.is_at_least(SeverityLevel::Serious));
    }

    #[test]
    fn test_from_priority() {
        assert_eq!(
            SeverityLevel::from_priority(5),
            Some(SeverityLevel::Critical)
        );
        assert_eq!(SeverityLevel::from_priority(1), Some(SeverityLevel::Info));
        assert_eq!(SeverityLevel::from_priority(0), None);
    }

    #[test]
    fn test_sort_severities() {
        let severities = vec![
            SeverityLevel::Info,
            SeverityLevel::Critical,
            SeverityLevel::Moderate,
        ];

        let sorted = sort_severities(severities);

        assert_eq!(sorted[0], SeverityLevel::Critical);
        assert_eq!(sorted[1], SeverityLevel::Moderate);
        assert_eq!(sorted[2], SeverityLevel::Info);
    }
}
