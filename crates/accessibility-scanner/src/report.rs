use crate::types::*;
use serde_json;
use std::collections::HashMap;

/// Report generator for scan results
pub struct ReportGenerator;

impl ReportGenerator {
    /// Get embedded CSS for HTML reports
    fn get_report_css() -> &'static str {
        include_str!("../assets/report.css")
    }

    /// Generate JSON report
    pub fn generate_json(result: &ScanResult) -> Result<String> {
        serde_json::to_string_pretty(result)
            .map_err(|e| ScannerError::ParseError(format!("Failed to generate JSON: {}", e)))
    }

    /// Generate HTML report
    pub fn generate_html(result: &ScanResult) -> Result<String> {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n");
        html.push_str("<head>\n");
        html.push_str("  <meta charset=\"UTF-8\">\n");
        html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str(&format!("  <title>Accessibility Scan Report - {}</title>\n", result.config.target_url));
        html.push_str("  <style>\n");
        html.push_str(Self::get_report_css());
        html.push_str("  </style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");

        // Header
        html.push_str("  <header class=\"report-header\">\n");
        html.push_str("    <h1>Accessibility Scan Report</h1>\n");
        html.push_str(&format!("    <p class=\"url\">{}</p>\n", result.config.target_url));
        html.push_str(&format!("    <p class=\"timestamp\">Scanned: {}</p>\n", result.completed_at.format("%Y-%m-%d %H:%M:%S UTC")));
        html.push_str("  </header>\n");

        // Summary
        html.push_str("  <section class=\"summary\">\n");
        html.push_str("    <h2>Summary</h2>\n");
        html.push_str("    <div class=\"stats-grid\">\n");
        html.push_str(&format!("      <div class=\"stat-card\"><div class=\"stat-value\">{:.1}</div><div class=\"stat-label\">Compliance Score</div></div>\n", result.statistics.compliance_score));
        html.push_str(&format!("      <div class=\"stat-card\"><div class=\"stat-value\">{}</div><div class=\"stat-label\">Total Issues</div></div>\n", result.statistics.total_issues));
        html.push_str(&format!("      <div class=\"stat-card critical\"><div class=\"stat-value\">{}</div><div class=\"stat-label\">Critical</div></div>\n", result.statistics.critical));
        html.push_str(&format!("      <div class=\"stat-card serious\"><div class=\"stat-value\">{}</div><div class=\"stat-label\">Serious</div></div>\n", result.statistics.serious));
        html.push_str(&format!("      <div class=\"stat-card moderate\"><div class=\"stat-value\">{}</div><div class=\"stat-label\">Moderate</div></div>\n", result.statistics.moderate));
        html.push_str(&format!("      <div class=\"stat-card minor\"><div class=\"stat-value\">{}</div><div class=\"stat-label\">Minor</div></div>\n", result.statistics.minor));
        html.push_str(&format!("      <div class=\"stat-card\"><div class=\"stat-value\">{}</div><div class=\"stat-label\">Pages Scanned</div></div>\n", result.statistics.pages_scanned));
        html.push_str(&format!("      <div class=\"stat-card\"><div class=\"stat-value\">{}</div><div class=\"stat-label\">Elements Analyzed</div></div>\n", result.statistics.elements_analyzed));
        html.push_str("    </div>\n");
        html.push_str("  </section>\n");

        // Issues by severity
        html.push_str("  <section class=\"issues\">\n");
        html.push_str("    <h2>Issues by Severity</h2>\n");

        for page in &result.pages {
            html.push_str(&format!("    <div class=\"page-section\">\n"));
            html.push_str(&format!("      <h3>{}</h3>\n", page.url));
            html.push_str(&format!("      <p class=\"page-info\">{} issues found in {} elements</p>\n", page.issues.len(), page.elements_count));

            // Group issues by severity
            let mut by_severity: HashMap<Severity, Vec<&Issue>> = HashMap::new();
            for issue in &page.issues {
                by_severity.entry(issue.severity).or_insert_with(Vec::new).push(issue);
            }

            for severity in &[Severity::Critical, Severity::Serious, Severity::Moderate, Severity::Minor, Severity::Info] {
                if let Some(issues) = by_severity.get(severity) {
                    html.push_str(&format!("      <div class=\"severity-group {}\">\n", severity.as_str()));
                    html.push_str(&format!("        <h4>{} ({} issues)</h4>\n", severity.as_str().to_uppercase(), issues.len()));

                    for issue in issues {
                        html.push_str("        <div class=\"issue-card\">\n");
                        html.push_str(&format!("          <div class=\"issue-header\">\n"));
                        html.push_str(&format!("            <span class=\"issue-rule\">{}</span>\n", issue.rule_name));
                        html.push_str(&format!("            <span class=\"issue-wcag\">{}</span>\n", issue.wcag_reference));
                        html.push_str("          </div>\n");
                        html.push_str(&format!("          <p class=\"issue-message\">{}</p>\n", html_escape(&issue.message)));
                        html.push_str(&format!("          <p class=\"issue-help\">{}</p>\n", html_escape(&issue.help)));

                        if !issue.fix_suggestions.is_empty() {
                            html.push_str("          <div class=\"issue-suggestions\">\n");
                            html.push_str("            <strong>How to fix:</strong>\n");
                            html.push_str("            <ul>\n");
                            for suggestion in &issue.fix_suggestions {
                                html.push_str(&format!("              <li>{}</li>\n", html_escape(suggestion)));
                            }
                            html.push_str("            </ul>\n");
                            html.push_str("          </div>\n");
                        }

                        html.push_str("          <div class=\"issue-context\">\n");
                        html.push_str(&format!("            <strong>Element:</strong> <code>{}</code>\n", html_escape(&issue.context.position.selector)));
                        html.push_str(&format!("            <pre><code>{}</code></pre>\n", html_escape(&issue.context.outer_html)));
                        html.push_str("          </div>\n");

                        html.push_str(&format!("          <a href=\"{}\" class=\"issue-link\" target=\"_blank\">Learn more about this rule</a>\n", issue.help_url));
                        html.push_str("        </div>\n");
                    }

                    html.push_str("      </div>\n");
                }
            }

            html.push_str("    </div>\n");
        }

        html.push_str("  </section>\n");

        // Footer
        html.push_str("  <footer class=\"report-footer\">\n");
        html.push_str(&format!("    <p>Generated by Accessibility Scanner v{}</p>\n", result.version));
        html.push_str(&format!("    <p>Scan duration: {}ms</p>\n", result.statistics.duration_ms));
        html.push_str("    <p>WCAG 2.1 Conformance Levels: ");
        for level in &result.config.levels {
            html.push_str(&format!("{} ", level.as_str()));
        }
        html.push_str("</p>\n");
        html.push_str("  </footer>\n");

        html.push_str("</body>\n");
        html.push_str("</html>\n");

        Ok(html)
    }

    /// Generate CSV report
    pub fn generate_csv(result: &ScanResult) -> Result<String> {
        let mut csv = String::new();

        // Header
        csv.push_str("URL,Rule ID,Rule Name,Severity,WCAG Level,Message,Element,Fix Suggestion\n");

        // Data
        for page in &result.pages {
            for issue in &page.issues {
                let fix = if issue.fix_suggestions.is_empty() {
                    String::new()
                } else {
                    issue.fix_suggestions[0].clone()
                };

                csv.push_str(&format!(
                    "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                    csv_escape(&page.url),
                    csv_escape(&issue.rule_id),
                    csv_escape(&issue.rule_name),
                    issue.severity.as_str(),
                    issue.level.as_str(),
                    csv_escape(&issue.message),
                    csv_escape(&issue.context.position.selector),
                    csv_escape(&fix)
                ));
            }
        }

        Ok(csv)
    }

    /// Generate text summary report
    pub fn generate_summary(result: &ScanResult) -> Result<String> {
        let mut text = String::new();

        text.push_str("═══════════════════════════════════════════════════════════════\n");
        text.push_str("              ACCESSIBILITY SCAN REPORT\n");
        text.push_str("═══════════════════════════════════════════════════════════════\n\n");

        text.push_str(&format!("Target URL:        {}\n", result.config.target_url));
        text.push_str(&format!("Scan completed:    {}\n", result.completed_at.format("%Y-%m-%d %H:%M:%S UTC")));
        text.push_str(&format!("Duration:          {}ms\n", result.statistics.duration_ms));
        text.push_str(&format!("Scanner version:   {}\n\n", result.version));

        text.push_str("───────────────────────────────────────────────────────────────\n");
        text.push_str("                          SUMMARY\n");
        text.push_str("───────────────────────────────────────────────────────────────\n\n");

        text.push_str(&format!("Compliance Score:  {:.1}/100\n\n", result.statistics.compliance_score));

        text.push_str(&format!("Total Issues:      {}\n", result.statistics.total_issues));
        text.push_str(&format!("  Critical:        {}\n", result.statistics.critical));
        text.push_str(&format!("  Serious:         {}\n", result.statistics.serious));
        text.push_str(&format!("  Moderate:        {}\n", result.statistics.moderate));
        text.push_str(&format!("  Minor:           {}\n", result.statistics.minor));
        text.push_str(&format!("  Info:            {}\n\n", result.statistics.info));

        text.push_str(&format!("Pages scanned:     {}\n", result.statistics.pages_scanned));
        text.push_str(&format!("Elements analyzed: {}\n\n", result.statistics.elements_analyzed));

        text.push_str("WCAG Levels tested: ");
        for (i, level) in result.config.levels.iter().enumerate() {
            if i > 0 {
                text.push_str(", ");
            }
            text.push_str(level.as_str());
        }
        text.push_str("\n\n");

        // Issue breakdown by page
        for page in &result.pages {
            text.push_str("───────────────────────────────────────────────────────────────\n");
            text.push_str(&format!("PAGE: {}\n", page.url));
            text.push_str("───────────────────────────────────────────────────────────────\n\n");

            text.push_str(&format!("Title:         {}\n", page.title));
            text.push_str(&format!("HTTP Status:   {}\n", page.http_status));
            text.push_str(&format!("Issues found:  {}\n", page.issues.len()));
            text.push_str(&format!("Elements:      {}\n", page.elements_count));
            text.push_str(&format!("Scan time:     {}ms\n\n", page.scan_time_ms));

            if !page.issues.is_empty() {
                text.push_str("ISSUES:\n\n");

                for (i, issue) in page.issues.iter().enumerate() {
                    text.push_str(&format!("{}. [{} - {}] {}\n",
                        i + 1,
                        issue.severity.as_str().to_uppercase(),
                        issue.level.as_str(),
                        issue.rule_name
                    ));
                    text.push_str(&format!("   Message: {}\n", issue.message));
                    text.push_str(&format!("   Element: {}\n", issue.context.position.selector));
                    if !issue.fix_suggestions.is_empty() {
                        text.push_str(&format!("   Fix:     {}\n", issue.fix_suggestions[0]));
                    }
                    text.push_str("\n");
                }
            }
        }

        text.push_str("═══════════════════════════════════════════════════════════════\n");
        text.push_str("                       END OF REPORT\n");
        text.push_str("═══════════════════════════════════════════════════════════════\n");

        Ok(text)
    }
}

/// Escape HTML special characters
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Escape CSV special characters
fn csv_escape(text: &str) -> String {
    text.replace('"', "\"\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<div>"), "&lt;div&gt;");
        assert_eq!(html_escape("AT&T"), "AT&amp;T");
    }

    #[test]
    fn test_csv_escape() {
        assert_eq!(csv_escape("test\"value"), "test\"\"value");
    }

    #[test]
    fn test_generate_json() {
        let config = ScanConfig::default();
        let result = ScanResult::new(config);

        let json = ReportGenerator::generate_json(&result).unwrap();
        assert!(json.contains("\"id\":"));
        assert!(json.contains("\"statistics\":"));
    }

    #[test]
    fn test_generate_summary() {
        let config = ScanConfig::default();
        let result = ScanResult::new(config);

        let summary = ReportGenerator::generate_summary(&result).unwrap();
        assert!(summary.contains("ACCESSIBILITY SCAN REPORT"));
        assert!(summary.contains("Compliance Score"));
    }
}
