import DOMPurify from 'dompurify';
import { ReportData, ExportOptions } from '../types';
import { formatDate, formatPercentage, getSeverityColor } from '../utils/formatting';

/**
 * HTML Report Generator
 * Creates accessible, semantic HTML reports
 */
export class HTMLGenerator {
  constructor(private options: ExportOptions, private reportData: ReportData) {}

  /**
   * Generate the complete HTML report
   */
  public async generate(): Promise<Blob> {
    const html = this.buildHTML();
    const sanitizedHTML = DOMPurify.sanitize(html);

    return new Blob([sanitizedHTML], { type: 'text/html' });
  }

  /**
   * Build complete HTML document
   */
  private buildHTML(): string {
    const { config, metrics, generatedAt } = this.reportData;

    return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="${config.subtitle || 'Accessibility Compliance Report'}">
    <title>${config.title}</title>
    ${this.getStyles()}
</head>
<body>
    <div class="report-container">
        ${this.buildHeader()}
        ${this.buildNavigationMenu()}
        <main id="main-content" class="main-content">
            ${this.buildExecutiveSummary()}
            ${this.buildMetricsSection()}
            ${this.buildIssuesSection()}
            ${this.buildTrendsSection()}
            ${this.buildRecommendationsSection()}
        </main>
        ${this.buildFooter()}
    </div>
    ${this.getScripts()}
</body>
</html>`;
  }

  /**
   * Get CSS styles
   */
  private getStyles(): string {
    const { branding } = this.reportData.config;

    return `<style>
        :root {
            --primary-color: ${branding.primaryColor};
            --secondary-color: ${branding.secondaryColor};
            --accent-color: ${branding.accentColor};
            --font-family: ${branding.fontFamily};
        }

        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: var(--font-family);
            line-height: 1.6;
            color: #333;
            background-color: #f5f5f5;
        }

        .report-container {
            max-width: 1200px;
            margin: 0 auto;
            background-color: white;
            box-shadow: 0 0 20px rgba(0, 0, 0, 0.1);
        }

        header {
            background: linear-gradient(135deg, var(--primary-color), var(--secondary-color));
            color: white;
            padding: 2rem;
        }

        .header-content {
            display: flex;
            align-items: center;
            gap: 1.5rem;
        }

        .logo {
            max-height: 60px;
            max-width: 200px;
        }

        h1 {
            font-size: 2.5rem;
            margin-bottom: 0.5rem;
        }

        .subtitle {
            font-size: 1.25rem;
            opacity: 0.9;
        }

        .metadata {
            margin-top: 1rem;
            font-size: 0.9rem;
            opacity: 0.8;
        }

        nav {
            background-color: #f9f9f9;
            padding: 1rem 2rem;
            border-bottom: 2px solid var(--primary-color);
            position: sticky;
            top: 0;
            z-index: 100;
        }

        nav ul {
            list-style: none;
            display: flex;
            gap: 2rem;
        }

        nav a {
            color: var(--primary-color);
            text-decoration: none;
            font-weight: bold;
            transition: color 0.3s;
        }

        nav a:hover,
        nav a:focus {
            color: var(--accent-color);
            text-decoration: underline;
        }

        .main-content {
            padding: 2rem;
        }

        section {
            margin-bottom: 3rem;
        }

        h2 {
            font-size: 2rem;
            color: var(--primary-color);
            border-bottom: 3px solid var(--primary-color);
            padding-bottom: 0.5rem;
            margin-bottom: 1.5rem;
        }

        h3 {
            font-size: 1.5rem;
            color: var(--secondary-color);
            margin-bottom: 1rem;
        }

        .summary-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 1.5rem;
            margin-bottom: 2rem;
        }

        .metric-card {
            background: linear-gradient(135deg, #f9f9f9, #fff);
            padding: 1.5rem;
            border-radius: 8px;
            border: 2px solid #e0e0e0;
            text-align: center;
            transition: transform 0.3s, box-shadow 0.3s;
        }

        .metric-card:hover {
            transform: translateY(-5px);
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        }

        .metric-value {
            font-size: 3rem;
            font-weight: bold;
            color: var(--primary-color);
            margin-bottom: 0.5rem;
        }

        .metric-label {
            font-size: 1rem;
            color: #666;
            font-weight: bold;
        }

        .issues-grid {
            display: grid;
            gap: 1.5rem;
        }

        .issue-card {
            background-color: #fff;
            border-left: 4px solid;
            padding: 1.5rem;
            border-radius: 4px;
            box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
        }

        .issue-card.critical {
            border-left-color: #dc3545;
        }

        .issue-card.serious {
            border-left-color: #fd7e14;
        }

        .issue-card.moderate {
            border-left-color: #ffc107;
        }

        .issue-card.minor {
            border-left-color: #28a745;
        }

        .issue-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 1rem;
        }

        .issue-title {
            font-size: 1.25rem;
            font-weight: bold;
            color: #333;
        }

        .severity-badge {
            padding: 0.25rem 0.75rem;
            border-radius: 12px;
            font-size: 0.875rem;
            font-weight: bold;
            text-transform: uppercase;
            color: white;
        }

        .severity-badge.critical {
            background-color: #dc3545;
        }

        .severity-badge.serious {
            background-color: #fd7e14;
        }

        .severity-badge.moderate {
            background-color: #ffc107;
            color: #000;
        }

        .severity-badge.minor {
            background-color: #28a745;
        }

        .issue-description {
            color: #666;
            margin-bottom: 1rem;
            line-height: 1.6;
        }

        .issue-meta {
            font-size: 0.875rem;
            color: #999;
            display: flex;
            gap: 1rem;
            flex-wrap: wrap;
        }

        table {
            width: 100%;
            border-collapse: collapse;
            margin: 1rem 0;
        }

        th,
        td {
            padding: 0.75rem;
            text-align: left;
            border-bottom: 1px solid #e0e0e0;
        }

        th {
            background-color: var(--primary-color);
            color: white;
            font-weight: bold;
        }

        tr:hover {
            background-color: #f9f9f9;
        }

        .recommendations-list {
            list-style: none;
            counter-reset: recommendations;
        }

        .recommendation-item {
            counter-increment: recommendations;
            padding: 1.5rem;
            margin-bottom: 1.5rem;
            background-color: #f9f9f9;
            border-left: 4px solid var(--primary-color);
            border-radius: 4px;
            position: relative;
        }

        .recommendation-item::before {
            content: counter(recommendations);
            position: absolute;
            left: -1.5rem;
            top: 1rem;
            background-color: var(--primary-color);
            color: white;
            width: 2.5rem;
            height: 2.5rem;
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            font-weight: bold;
            font-size: 1.25rem;
        }

        .remediation-steps {
            margin-top: 1rem;
        }

        .remediation-steps ol {
            margin-left: 1.5rem;
        }

        .remediation-steps li {
            margin-bottom: 0.5rem;
        }

        footer {
            background-color: #333;
            color: white;
            padding: 2rem;
            text-align: center;
        }

        footer p {
            margin-bottom: 0.5rem;
        }

        /* Skip link for accessibility */
        .skip-link {
            position: absolute;
            top: -40px;
            left: 0;
            background: var(--primary-color);
            color: white;
            padding: 8px;
            text-decoration: none;
            z-index: 100;
        }

        .skip-link:focus {
            top: 0;
        }

        /* Print styles */
        @media print {
            .report-container {
                box-shadow: none;
            }

            nav {
                display: none;
            }

            section {
                page-break-inside: avoid;
            }
        }

        /* Responsive design */
        @media (max-width: 768px) {
            h1 {
                font-size: 2rem;
            }

            .summary-grid {
                grid-template-columns: 1fr;
            }

            nav ul {
                flex-direction: column;
                gap: 0.5rem;
            }
        }
    </style>`;
  }

  /**
   * Build header section
   */
  private buildHeader(): string {
    const { config, generatedAt } = this.reportData;

    return `<header role="banner">
        <a href="#main-content" class="skip-link">Skip to main content</a>
        <div class="header-content">
            ${config.branding.logo ? `<img src="${config.branding.logo}" alt="${config.branding.companyName} logo" class="logo">` : ''}
            <div>
                <h1>${config.title}</h1>
                ${config.subtitle ? `<p class="subtitle">${config.subtitle}</p>` : ''}
                <div class="metadata">
                    <p>Generated: ${formatDate(generatedAt)} | ${config.branding.companyName}</p>
                    <p>Period: ${formatDate(config.dateRange.from)} - ${formatDate(config.dateRange.to)}</p>
                </div>
            </div>
        </div>
    </header>`;
  }

  /**
   * Build navigation menu
   */
  private buildNavigationMenu(): string {
    return `<nav role="navigation" aria-label="Report sections">
        <ul>
            <li><a href="#executive-summary">Executive Summary</a></li>
            <li><a href="#metrics">Metrics</a></li>
            <li><a href="#issues">Issues</a></li>
            <li><a href="#trends">Trends</a></li>
            <li><a href="#recommendations">Recommendations</a></li>
        </ul>
    </nav>`;
  }

  /**
   * Build executive summary section
   */
  private buildExecutiveSummary(): string {
    const { metrics } = this.reportData;

    return `<section id="executive-summary" aria-labelledby="summary-heading">
        <h2 id="summary-heading">Executive Summary</h2>
        <div class="summary-grid">
            <div class="metric-card">
                <div class="metric-value">${formatPercentage(metrics.complianceScore)}</div>
                <div class="metric-label">Compliance Score</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">${metrics.totalIssues}</div>
                <div class="metric-label">Total Issues</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">${metrics.criticalIssues}</div>
                <div class="metric-label">Critical Issues</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">${formatPercentage(metrics.wcagAACompliance)}</div>
                <div class="metric-label">WCAG AA Compliance</div>
            </div>
        </div>
    </section>`;
  }

  /**
   * Build metrics section
   */
  private buildMetricsSection(): string {
    const { metrics } = this.reportData;

    return `<section id="metrics" aria-labelledby="metrics-heading">
        <h2 id="metrics-heading">Compliance Metrics</h2>

        <h3>Issue Severity Breakdown</h3>
        <table>
            <thead>
                <tr>
                    <th scope="col">Severity</th>
                    <th scope="col">Count</th>
                    <th scope="col">Percentage</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>Critical</td>
                    <td>${metrics.criticalIssues}</td>
                    <td>${formatPercentage((metrics.criticalIssues / metrics.totalIssues) * 100)}</td>
                </tr>
                <tr>
                    <td>Serious</td>
                    <td>${metrics.seriousIssues}</td>
                    <td>${formatPercentage((metrics.seriousIssues / metrics.totalIssues) * 100)}</td>
                </tr>
                <tr>
                    <td>Moderate</td>
                    <td>${metrics.moderateIssues}</td>
                    <td>${formatPercentage((metrics.moderateIssues / metrics.totalIssues) * 100)}</td>
                </tr>
                <tr>
                    <td>Minor</td>
                    <td>${metrics.minorIssues}</td>
                    <td>${formatPercentage((metrics.minorIssues / metrics.totalIssues) * 100)}</td>
                </tr>
            </tbody>
        </table>

        <h3>WCAG 2.1 Compliance</h3>
        <table>
            <thead>
                <tr>
                    <th scope="col">Level</th>
                    <th scope="col">Compliance</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>Level A</td>
                    <td>${formatPercentage(metrics.wcagACompliance)}</td>
                </tr>
                <tr>
                    <td>Level AA</td>
                    <td>${formatPercentage(metrics.wcagAACompliance)}</td>
                </tr>
                <tr>
                    <td>Level AAA</td>
                    <td>${formatPercentage(metrics.wcagAAACompliance)}</td>
                </tr>
            </tbody>
        </table>
    </section>`;
  }

  /**
   * Build issues section
   */
  private buildIssuesSection(): string {
    const criticalIssues = this.reportData.issues
      .filter((i) => i.severity === 'critical')
      .slice(0, 10);

    const issuesHTML = criticalIssues
      .map(
        (issue) => `
        <article class="issue-card ${issue.severity}">
            <div class="issue-header">
                <h4 class="issue-title">${issue.title}</h4>
                <span class="severity-badge ${issue.severity}">${issue.severity}</span>
            </div>
            <p class="issue-description">${issue.description}</p>
            <div class="issue-meta">
                <span>WCAG: ${issue.wcagCriteria.join(', ')}</span>
                <span>Location: ${issue.location.url}</span>
                <span>Status: ${issue.status}</span>
            </div>
        </article>
    `
      )
      .join('');

    return `<section id="issues" aria-labelledby="issues-heading">
        <h2 id="issues-heading">Critical Issues</h2>
        <div class="issues-grid">
            ${issuesHTML}
        </div>
    </section>`;
  }

  /**
   * Build trends section
   */
  private buildTrendsSection(): string {
    const trendsRows = this.reportData.trends
      .slice(-10)
      .map(
        (trend) => `
        <tr>
            <td>${formatDate(trend.date)}</td>
            <td>${trend.totalIssues}</td>
            <td>${trend.criticalIssues}</td>
            <td>${trend.resolvedIssues}</td>
            <td>${formatPercentage(trend.complianceScore)}</td>
        </tr>
    `
      )
      .join('');

    return `<section id="trends" aria-labelledby="trends-heading">
        <h2 id="trends-heading">Trends</h2>
        <table>
            <thead>
                <tr>
                    <th scope="col">Date</th>
                    <th scope="col">Total Issues</th>
                    <th scope="col">Critical</th>
                    <th scope="col">Resolved</th>
                    <th scope="col">Compliance</th>
                </tr>
            </thead>
            <tbody>
                ${trendsRows}
            </tbody>
        </table>
    </section>`;
  }

  /**
   * Build recommendations section
   */
  private buildRecommendationsSection(): string {
    const topIssues = this.reportData.issues
      .filter((i) => i.severity === 'critical' || i.severity === 'serious')
      .slice(0, 5);

    const recommendationsHTML = topIssues
      .map(
        (issue) => `
        <li class="recommendation-item">
            <h4>${issue.title}</h4>
            <p><strong>Impact:</strong> ${issue.impact}</p>
            <div class="remediation-steps">
                <strong>Remediation Steps:</strong>
                <ol>
                    ${issue.remediation.steps.map((step) => `<li>${step}</li>`).join('')}
                </ol>
            </div>
            <p><strong>Effort:</strong> ${issue.remediation.effort}</p>
        </li>
    `
      )
      .join('');

    return `<section id="recommendations" aria-labelledby="recommendations-heading">
        <h2 id="recommendations-heading">Priority Recommendations</h2>
        <ol class="recommendations-list">
            ${recommendationsHTML}
        </ol>
    </section>`;
  }

  /**
   * Build footer
   */
  private buildFooter(): string {
    const { config } = this.reportData;

    return `<footer role="contentinfo">
        ${config.branding.footerText ? `<p>${config.branding.footerText}</p>` : ''}
        <p>Report generated by ${config.branding.companyName} | Version ${config.version}</p>
    </footer>`;
  }

  /**
   * Get JavaScript for interactivity
   */
  private getScripts(): string {
    return `<script>
        // Smooth scrolling for navigation links
        document.querySelectorAll('nav a').forEach(anchor => {
            anchor.addEventListener('click', function (e) {
                e.preventDefault();
                const target = document.querySelector(this.getAttribute('href'));
                target.scrollIntoView({ behavior: 'smooth', block: 'start' });
                target.focus();
            });
        });

        // Print functionality
        function printReport() {
            window.print();
        }
    </script>`;
  }

  /**
   * Save HTML file
   */
  public save(filename: string): void {
    const html = this.buildHTML();
    const blob = new Blob([html], { type: 'text/html' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${filename}.html`;
    a.click();
    URL.revokeObjectURL(url);
  }
}
