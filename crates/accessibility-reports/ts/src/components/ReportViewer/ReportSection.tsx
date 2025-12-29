import React from 'react';
import { ReportSection as ReportSectionType, ReportData, BrandingConfig } from '../../types';

interface ReportSectionProps {
  section: ReportSectionType;
  data: ReportData;
  branding: BrandingConfig;
  className?: string;
}

export const ReportSection: React.FC<ReportSectionProps> = ({
  section,
  data,
  branding,
  className = '',
}) => {
  const renderMetricsSection = () => (
    <div style={styles.metricsContainer}>
      <div style={styles.metricsGrid}>
        <div style={styles.metricCard}>
          <h4 style={styles.metricTitle}>Issue Breakdown</h4>
          <ul style={styles.metricList}>
            <li style={styles.metricItem}>
              <span style={{ ...styles.severityDot, ...styles.criticalDot }} aria-hidden="true" />
              <span style={styles.metricLabel}>Critical:</span>
              <span style={styles.metricValue}>{data.metrics.criticalIssues}</span>
            </li>
            <li style={styles.metricItem}>
              <span style={{ ...styles.severityDot, ...styles.seriousDot }} aria-hidden="true" />
              <span style={styles.metricLabel}>Serious:</span>
              <span style={styles.metricValue}>{data.metrics.seriousIssues}</span>
            </li>
            <li style={styles.metricItem}>
              <span style={{ ...styles.severityDot, ...styles.moderateDot }} aria-hidden="true" />
              <span style={styles.metricLabel}>Moderate:</span>
              <span style={styles.metricValue}>{data.metrics.moderateIssues}</span>
            </li>
            <li style={styles.metricItem}>
              <span style={{ ...styles.severityDot, ...styles.minorDot }} aria-hidden="true" />
              <span style={styles.metricLabel}>Minor:</span>
              <span style={styles.metricValue}>{data.metrics.minorIssues}</span>
            </li>
          </ul>
        </div>

        <div style={styles.metricCard}>
          <h4 style={styles.metricTitle}>WCAG Compliance</h4>
          <ul style={styles.metricList}>
            <li style={styles.metricItem}>
              <span style={styles.metricLabel}>Level A:</span>
              <span style={styles.metricValue}>{data.metrics.wcagACompliance}%</span>
            </li>
            <li style={styles.metricItem}>
              <span style={styles.metricLabel}>Level AA:</span>
              <span style={styles.metricValue}>{data.metrics.wcagAACompliance}%</span>
            </li>
            <li style={styles.metricItem}>
              <span style={styles.metricLabel}>Level AAA:</span>
              <span style={styles.metricValue}>{data.metrics.wcagAAACompliance}%</span>
            </li>
          </ul>
        </div>

        <div style={styles.metricCard}>
          <h4 style={styles.metricTitle}>Success Criteria</h4>
          <ul style={styles.metricList}>
            <li style={styles.metricItem}>
              <span style={styles.metricLabel}>Passed:</span>
              <span style={styles.metricValue}>{data.metrics.successCriteriaPassed}</span>
            </li>
            <li style={styles.metricItem}>
              <span style={styles.metricLabel}>Failed:</span>
              <span style={styles.metricValue}>{data.metrics.successCriteriaFailed}</span>
            </li>
            <li style={styles.metricItem}>
              <span style={styles.metricLabel}>Total:</span>
              <span style={styles.metricValue}>{data.metrics.successCriteriaTotal}</span>
            </li>
          </ul>
        </div>
      </div>
    </div>
  );

  const renderIssuesSection = () => {
    const issuesByStatus = data.issues.reduce((acc, issue) => {
      if (!acc[issue.severity]) {
        acc[issue.severity] = [];
      }
      acc[issue.severity].push(issue);
      return acc;
    }, {} as Record<string, typeof data.issues>);

    return (
      <div style={styles.issuesContainer}>
        {(['critical', 'serious', 'moderate', 'minor'] as const).map((severity) => {
          const issues = issuesByStatus[severity] || [];
          if (issues.length === 0) return null;

          return (
            <div key={severity} style={styles.issueGroup}>
              <h4 style={styles.issueGroupTitle}>
                {severity.charAt(0).toUpperCase() + severity.slice(1)} Issues ({issues.length})
              </h4>
              <ul style={styles.issueList}>
                {issues.slice(0, 10).map((issue) => (
                  <li key={issue.id} style={styles.issueItem}>
                    <div style={styles.issueHeader}>
                      <h5 style={styles.issueTitle}>{issue.title}</h5>
                      <span
                        style={{
                          ...styles.severityBadge,
                          ...getSeverityStyle(severity),
                        }}
                      >
                        {severity}
                      </span>
                    </div>
                    <p style={styles.issueDescription}>{issue.description}</p>
                    <div style={styles.issueMeta}>
                      <span>WCAG: {issue.wcagCriteria.join(', ')}</span>
                      <span>Location: {issue.location.url}</span>
                      <span>Status: {issue.status}</span>
                    </div>
                  </li>
                ))}
              </ul>
              {issues.length > 10 && (
                <p style={styles.moreIssues}>
                  And {issues.length - 10} more {severity} issues...
                </p>
              )}
            </div>
          );
        })}
      </div>
    );
  };

  const renderTrendsSection = () => (
    <div style={styles.trendsContainer}>
      <p style={styles.trendsDescription}>
        Trends over the past {data.trends.length} reporting periods:
      </p>
      <div style={styles.trendsTable}>
        <table style={styles.table}>
          <thead>
            <tr>
              <th style={styles.tableHeader}>Date</th>
              <th style={styles.tableHeader}>Total Issues</th>
              <th style={styles.tableHeader}>Critical</th>
              <th style={styles.tableHeader}>Resolved</th>
              <th style={styles.tableHeader}>Compliance</th>
            </tr>
          </thead>
          <tbody>
            {data.trends.slice(-10).map((trend, index) => (
              <tr key={index} style={styles.tableRow}>
                <td style={styles.tableCell}>
                  {trend.date.toLocaleDateString()}
                </td>
                <td style={styles.tableCell}>{trend.totalIssues}</td>
                <td style={styles.tableCell}>{trend.criticalIssues}</td>
                <td style={styles.tableCell}>{trend.resolvedIssues}</td>
                <td style={styles.tableCell}>{trend.complianceScore}%</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );

  const renderRecommendationsSection = () => {
    const topIssues = data.issues
      .filter((i) => i.severity === 'critical' || i.severity === 'serious')
      .slice(0, 5);

    return (
      <div style={styles.recommendationsContainer}>
        <h4 style={styles.recommendationsTitle}>Priority Recommendations</h4>
        <ol style={styles.recommendationsList}>
          {topIssues.map((issue) => (
            <li key={issue.id} style={styles.recommendationItem}>
              <h5 style={styles.recommendationTitle}>{issue.title}</h5>
              <p style={styles.recommendationImpact}>
                <strong>Impact:</strong> {issue.impact}
              </p>
              <div style={styles.remediationSteps}>
                <strong>Remediation Steps:</strong>
                <ol style={styles.stepsList}>
                  {issue.remediation.steps.map((step, idx) => (
                    <li key={idx} style={styles.stepItem}>
                      {step}
                    </li>
                  ))}
                </ol>
              </div>
              <div style={styles.effortBadge}>
                Effort: {issue.remediation.effort}
              </div>
            </li>
          ))}
        </ol>
      </div>
    );
  };

  const renderSectionContent = () => {
    switch (section.type) {
      case 'metrics':
        return renderMetricsSection();
      case 'issues':
        return renderIssuesSection();
      case 'trends':
        return renderTrendsSection();
      case 'recommendations':
        return renderRecommendationsSection();
      default:
        return (
          <p style={styles.placeholderText}>
            Content for {section.type} section will be rendered here.
          </p>
        );
    }
  };

  return (
    <section
      className={`report-section ${className}`}
      style={styles.container}
      aria-labelledby={`section-title-${section.id}`}
    >
      <h3
        id={`section-title-${section.id}`}
        style={{
          ...styles.sectionTitle,
          borderBottomColor: branding.primaryColor,
        }}
      >
        {section.title}
      </h3>
      {renderSectionContent()}
    </section>
  );
};

const getSeverityStyle = (severity: string) => {
  const styles = {
    critical: { backgroundColor: '#dc3545', color: '#fff' },
    serious: { backgroundColor: '#fd7e14', color: '#fff' },
    moderate: { backgroundColor: '#ffc107', color: '#000' },
    minor: { backgroundColor: '#28a745', color: '#fff' },
  };
  return styles[severity as keyof typeof styles] || {};
};

const styles = {
  container: {
    backgroundColor: '#fff',
    padding: '2rem',
    borderRadius: '8px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
  },
  sectionTitle: {
    margin: '0 0 1.5rem 0',
    fontSize: '1.75rem',
    fontWeight: 'bold' as const,
    color: '#333',
    borderBottom: '3px solid',
    paddingBottom: '0.75rem',
  },
  metricsContainer: {
    marginTop: '1rem',
  },
  metricsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
    gap: '1.5rem',
  },
  metricCard: {
    padding: '1.5rem',
    backgroundColor: '#f9f9f9',
    borderRadius: '6px',
    border: '1px solid #e0e0e0',
  },
  metricTitle: {
    margin: '0 0 1rem 0',
    fontSize: '1.125rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  metricList: {
    listStyle: 'none',
    padding: 0,
    margin: 0,
  },
  metricItem: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.75rem',
    padding: '0.5rem 0',
    borderBottom: '1px solid #e0e0e0',
  },
  metricLabel: {
    flex: 1,
    fontWeight: 'bold' as const,
    color: '#666',
  },
  metricValue: {
    fontWeight: 'bold' as const,
    fontSize: '1.125rem',
    color: '#333',
  },
  severityDot: {
    width: '12px',
    height: '12px',
    borderRadius: '50%',
    flexShrink: 0,
  },
  criticalDot: { backgroundColor: '#dc3545' },
  seriousDot: { backgroundColor: '#fd7e14' },
  moderateDot: { backgroundColor: '#ffc107' },
  minorDot: { backgroundColor: '#28a745' },
  issuesContainer: {
    marginTop: '1rem',
  },
  issueGroup: {
    marginBottom: '2rem',
  },
  issueGroupTitle: {
    margin: '0 0 1rem 0',
    fontSize: '1.25rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  issueList: {
    listStyle: 'none',
    padding: 0,
    margin: 0,
  },
  issueItem: {
    padding: '1rem',
    marginBottom: '1rem',
    backgroundColor: '#f9f9f9',
    border: '1px solid #e0e0e0',
    borderRadius: '6px',
  },
  issueHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '0.5rem',
  },
  issueTitle: {
    margin: 0,
    fontSize: '1.125rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  severityBadge: {
    padding: '0.25rem 0.75rem',
    borderRadius: '12px',
    fontSize: '0.875rem',
    fontWeight: 'bold' as const,
    textTransform: 'uppercase' as const,
  },
  issueDescription: {
    margin: '0.5rem 0',
    color: '#666',
    lineHeight: '1.6',
  },
  issueMeta: {
    display: 'flex',
    gap: '1rem',
    fontSize: '0.875rem',
    color: '#999',
    marginTop: '0.5rem',
  },
  moreIssues: {
    fontStyle: 'italic' as const,
    color: '#666',
    marginTop: '1rem',
  },
  trendsContainer: {
    marginTop: '1rem',
  },
  trendsDescription: {
    marginBottom: '1rem',
    color: '#666',
  },
  trendsTable: {
    overflowX: 'auto' as const,
  },
  table: {
    width: '100%',
    borderCollapse: 'collapse' as const,
  },
  tableHeader: {
    padding: '0.75rem',
    backgroundColor: '#f0f0f0',
    textAlign: 'left' as const,
    fontWeight: 'bold' as const,
    borderBottom: '2px solid #ddd',
  },
  tableRow: {
    borderBottom: '1px solid #e0e0e0',
  },
  tableCell: {
    padding: '0.75rem',
  },
  recommendationsContainer: {
    marginTop: '1rem',
  },
  recommendationsTitle: {
    margin: '0 0 1rem 0',
    fontSize: '1.25rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  recommendationsList: {
    paddingLeft: '1.5rem',
  },
  recommendationItem: {
    marginBottom: '1.5rem',
    padding: '1rem',
    backgroundColor: '#f9f9f9',
    borderLeft: '4px solid #0066cc',
    borderRadius: '4px',
  },
  recommendationTitle: {
    margin: '0 0 0.5rem 0',
    fontSize: '1.125rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  recommendationImpact: {
    margin: '0.5rem 0',
    color: '#666',
  },
  remediationSteps: {
    margin: '1rem 0',
  },
  stepsList: {
    marginTop: '0.5rem',
    paddingLeft: '1.5rem',
  },
  stepItem: {
    marginBottom: '0.5rem',
    color: '#666',
  },
  effortBadge: {
    display: 'inline-block',
    padding: '0.25rem 0.75rem',
    backgroundColor: '#e3f2fd',
    color: '#0066cc',
    borderRadius: '12px',
    fontSize: '0.875rem',
    fontWeight: 'bold' as const,
    textTransform: 'capitalize' as const,
  },
  placeholderText: {
    color: '#999',
    fontStyle: 'italic' as const,
  },
};
