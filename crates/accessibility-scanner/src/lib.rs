/*!
# Accessibility Scanner

Enterprise-grade WCAG 2.1 accessibility scanner with advanced DOM analysis.

## Features

- **Comprehensive WCAG 2.1 Support**: Implements 50+ rules across Level A, AA, and AAA
- **Parallel Scanning**: Uses Rayon for high-performance parallel processing
- **Incremental Analysis**: Supports incremental scanning for large websites
- **Multiple Report Formats**: JSON, HTML, CSV, and text summary reports
- **Advanced Analysis**: Color contrast checking, heading structure validation, ARIA verification
- **Production Ready**: Built for enterprise use with proper error handling and logging

## Example

```rust
use accessibility_scanner::{AccessibilityScanner, ScanConfig, WCAGLevel};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = ScanConfig::default();
    config.target_url = "https://example.com".to_string();
    config.levels = vec![WCAGLevel::A, WCAGLevel::AA];

    let scanner = AccessibilityScanner::new(config);
    let result = scanner.scan().await?;

    println!("Compliance Score: {:.1}", result.statistics.compliance_score);
    println!("Total Issues: {}", result.statistics.total_issues);

    Ok(())
}
```
*/

pub mod analysis;
pub mod report;
pub mod rules;
pub mod scanner;
pub mod types;

// Re-export commonly used types
pub use scanner::{AccessibilityScanner, HTMLParser, WCAGRuleEngine};
pub use report::ReportGenerator;
pub use types::{
    Issue, IssueContext, PageResult, Position, Principle, Rule, ScanConfig, ScanResult,
    ScanStatistics, Severity, ScannerError, WCAGLevel, Result,
};

/// Current version of the accessibility scanner
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the scanner with tracing
pub fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "accessibility_scanner=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_default_config() {
        let config = ScanConfig::default();
        assert_eq!(config.levels.len(), 2);
        assert!(config.levels.contains(&WCAGLevel::A));
        assert!(config.levels.contains(&WCAGLevel::AA));
    }

    #[tokio::test]
    async fn test_scan_basic_html() {
        let html = r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <title>Test Page</title>
            </head>
            <body>
                <h1>Test</h1>
                <img src="test.jpg">
            </body>
            </html>
        "#;

        let parser = HTMLParser::new(html, "http://example.com".to_string()).unwrap();
        let engine = WCAGRuleEngine::new(parser, vec![WCAGLevel::A]);
        let issues = engine.execute();

        // Should find at least the missing alt text issue
        assert!(!issues.is_empty());
    }
}
