import { ReportData, ExportOptions } from '../types';

/**
 * JSON Data Export Generator
 * Creates structured JSON data exports for API consumption
 */
export class JSONGenerator {
  constructor(private options: ExportOptions, private reportData: ReportData) {}

  /**
   * Generate the complete JSON export
   */
  public async generate(): Promise<Blob> {
    const jsonData = this.buildJSONStructure();
    const jsonString = JSON.stringify(jsonData, null, 2);

    return new Blob([jsonString], { type: 'application/json' });
  }

  /**
   * Build comprehensive JSON structure
   */
  private buildJSONStructure(): any {
    return {
      metadata: this.buildMetadata(),
      summary: this.buildSummary(),
      metrics: this.buildMetrics(),
      issues: this.buildIssues(),
      trends: this.buildTrends(),
      recommendations: this.buildRecommendations(),
      compliance: this.buildCompliance(),
    };
  }

  /**
   * Build metadata section
   */
  private buildMetadata(): any {
    const { config, generatedAt, generatedBy } = this.reportData;

    return {
      reportId: config.id,
      title: config.title,
      subtitle: config.subtitle,
      description: config.description,
      version: config.version,
      generatedAt: generatedAt.toISOString(),
      generatedBy,
      reportPeriod: {
        from: config.dateRange.from.toISOString(),
        to: config.dateRange.to.toISOString(),
      },
      branding: {
        companyName: config.branding.companyName,
        primaryColor: config.branding.primaryColor,
        secondaryColor: config.branding.secondaryColor,
        accentColor: config.branding.accentColor,
      },
      exportOptions: {
        format: this.options.format,
        filename: this.options.filename,
        includeCharts: this.options.includeCharts,
        includeScreenshots: this.options.includeScreenshots,
      },
    };
  }

  /**
   * Build summary section
   */
  private buildSummary(): any {
    const { metrics } = this.reportData;

    return {
      overallComplianceScore: metrics.complianceScore,
      totalIssues: metrics.totalIssues,
      criticalIssues: metrics.criticalIssues,
      seriousIssues: metrics.seriousIssues,
      moderateIssues: metrics.moderateIssues,
      minorIssues: metrics.minorIssues,
      wcagCompliance: {
        levelA: metrics.wcagACompliance,
        levelAA: metrics.wcagAACompliance,
        levelAAA: metrics.wcagAAACompliance,
      },
      successCriteria: {
        passed: metrics.successCriteriaPassed,
        failed: metrics.successCriteriaFailed,
        total: metrics.successCriteriaTotal,
        passRate: (metrics.successCriteriaPassed / metrics.successCriteriaTotal) * 100,
      },
    };
  }

  /**
   * Build detailed metrics
   */
  private buildMetrics(): any {
    const { metrics } = this.reportData;

    return {
      issueDistribution: {
        bySeverity: {
          critical: {
            count: metrics.criticalIssues,
            percentage: (metrics.criticalIssues / metrics.totalIssues) * 100,
          },
          serious: {
            count: metrics.seriousIssues,
            percentage: (metrics.seriousIssues / metrics.totalIssues) * 100,
          },
          moderate: {
            count: metrics.moderateIssues,
            percentage: (metrics.moderateIssues / metrics.totalIssues) * 100,
          },
          minor: {
            count: metrics.minorIssues,
            percentage: (metrics.minorIssues / metrics.totalIssues) * 100,
          },
        },
      },
      wcagCompliance: {
        levelA: {
          compliance: metrics.wcagACompliance,
          status: metrics.wcagACompliance >= 95 ? 'pass' : 'fail',
        },
        levelAA: {
          compliance: metrics.wcagAACompliance,
          status: metrics.wcagAACompliance >= 95 ? 'pass' : 'fail',
        },
        levelAAA: {
          compliance: metrics.wcagAAACompliance,
          status: metrics.wcagAAACompliance >= 95 ? 'pass' : 'fail',
        },
      },
      successCriteria: {
        passed: metrics.successCriteriaPassed,
        failed: metrics.successCriteriaFailed,
        total: metrics.successCriteriaTotal,
        passRate: (metrics.successCriteriaPassed / metrics.successCriteriaTotal) * 100,
      },
      complianceScore: {
        current: metrics.complianceScore,
        target: 100,
        gap: 100 - metrics.complianceScore,
      },
    };
  }

  /**
   * Build issues section
   */
  private buildIssues(): any {
    const issuesBySeverity = this.groupIssuesBySeverity();
    const issuesByStatus = this.groupIssuesByStatus();
    const issuesByWCAG = this.groupIssuesByWCAG();

    return {
      total: this.reportData.issues.length,
      bySeverity: issuesBySeverity,
      byStatus: issuesByStatus,
      byWCAGLevel: issuesByWCAG,
      detailedIssues: this.reportData.issues.map((issue) => ({
        id: issue.id,
        title: issue.title,
        description: issue.description,
        severity: issue.severity,
        wcagCriteria: issue.wcagCriteria,
        wcagLevel: issue.wcagLevel,
        impact: issue.impact,
        affectedUsers: issue.affectedUsers,
        location: {
          url: issue.location.url,
          selector: issue.location.selector,
          screenshotAvailable: !!issue.location.screenshot,
        },
        remediation: {
          effort: issue.remediation.effort,
          priority: issue.remediation.priority,
          steps: issue.remediation.steps,
          codeExampleAvailable: !!issue.remediation.codeExample,
        },
        detectedBy: issue.detectedBy,
        detectedAt: issue.detectedAt.toISOString(),
        status: issue.status,
      })),
    };
  }

  /**
   * Build trends section
   */
  private buildTrends(): any {
    const trends = this.reportData.trends.map((trend) => ({
      date: trend.date.toISOString().split('T')[0],
      totalIssues: trend.totalIssues,
      criticalIssues: trend.criticalIssues,
      resolvedIssues: trend.resolvedIssues,
      complianceScore: trend.complianceScore,
    }));

    const trendAnalysis = this.analyzeTrends();

    return {
      dataPoints: trends,
      analysis: trendAnalysis,
    };
  }

  /**
   * Build recommendations section
   */
  private buildRecommendations(): any {
    const priorityIssues = this.reportData.issues
      .filter((i) => i.severity === 'critical' || i.severity === 'serious')
      .sort((a, b) => b.remediation.priority - a.remediation.priority)
      .slice(0, 10);

    return {
      total: priorityIssues.length,
      recommendations: priorityIssues.map((issue, index) => ({
        priority: index + 1,
        issueId: issue.id,
        title: issue.title,
        severity: issue.severity,
        impact: issue.impact,
        wcagCriteria: issue.wcagCriteria,
        effort: issue.remediation.effort,
        estimatedTimeToFix: this.estimateTimeToFix(issue.remediation.effort),
        remediationSteps: issue.remediation.steps,
        codeExample: issue.remediation.codeExample,
      })),
    };
  }

  /**
   * Build compliance section
   */
  private buildCompliance(): any {
    const { metrics } = this.reportData;

    return {
      overall: {
        complianceScore: metrics.complianceScore,
        status: metrics.complianceScore >= 95 ? 'compliant' : 'non-compliant',
        issuesRemaining: metrics.totalIssues,
      },
      wcag21: {
        levelA: {
          compliance: metrics.wcagACompliance,
          status: metrics.wcagACompliance >= 100 ? 'compliant' : 'non-compliant',
        },
        levelAA: {
          compliance: metrics.wcagAACompliance,
          status: metrics.wcagAACompliance >= 100 ? 'compliant' : 'non-compliant',
        },
        levelAAA: {
          compliance: metrics.wcagAAACompliance,
          status: metrics.wcagAAACompliance >= 100 ? 'compliant' : 'non-compliant',
        },
      },
      successCriteria: {
        total: metrics.successCriteriaTotal,
        passed: metrics.successCriteriaPassed,
        failed: metrics.successCriteriaFailed,
        passRate: (metrics.successCriteriaPassed / metrics.successCriteriaTotal) * 100,
      },
      criticalIssuesBlocking: metrics.criticalIssues > 0,
      readyForProduction: metrics.criticalIssues === 0 && metrics.wcagAACompliance >= 100,
    };
  }

  /**
   * Group issues by severity
   */
  private groupIssuesBySeverity(): any {
    const groups: any = {
      critical: [],
      serious: [],
      moderate: [],
      minor: [],
    };

    this.reportData.issues.forEach((issue) => {
      groups[issue.severity].push(issue.id);
    });

    return {
      critical: { count: groups.critical.length, ids: groups.critical },
      serious: { count: groups.serious.length, ids: groups.serious },
      moderate: { count: groups.moderate.length, ids: groups.moderate },
      minor: { count: groups.minor.length, ids: groups.minor },
    };
  }

  /**
   * Group issues by status
   */
  private groupIssuesByStatus(): any {
    const groups: any = {
      open: [],
      'in-progress': [],
      resolved: [],
      'wont-fix': [],
    };

    this.reportData.issues.forEach((issue) => {
      if (groups[issue.status]) {
        groups[issue.status].push(issue.id);
      }
    });

    return Object.keys(groups).reduce((acc, status) => {
      acc[status] = { count: groups[status].length, ids: groups[status] };
      return acc;
    }, {} as any);
  }

  /**
   * Group issues by WCAG level
   */
  private groupIssuesByWCAG(): any {
    const groups: any = {
      A: [],
      AA: [],
      AAA: [],
    };

    this.reportData.issues.forEach((issue) => {
      groups[issue.wcagLevel].push(issue.id);
    });

    return {
      A: { count: groups.A.length, ids: groups.A },
      AA: { count: groups.AA.length, ids: groups.AA },
      AAA: { count: groups.AAA.length, ids: groups.AAA },
    };
  }

  /**
   * Analyze trends
   */
  private analyzeTrends(): any {
    if (this.reportData.trends.length < 2) {
      return {
        direction: 'insufficient-data',
        improvement: 0,
      };
    }

    const latest = this.reportData.trends[this.reportData.trends.length - 1];
    const previous = this.reportData.trends[this.reportData.trends.length - 2];

    const issueChange = latest.totalIssues - previous.totalIssues;
    const complianceChange = latest.complianceScore - previous.complianceScore;

    return {
      direction: issueChange < 0 ? 'improving' : issueChange > 0 ? 'declining' : 'stable',
      issueChange,
      complianceChange,
      complianceImprovement: complianceChange > 0,
    };
  }

  /**
   * Estimate time to fix based on effort
   */
  private estimateTimeToFix(effort: string): string {
    const estimates: Record<string, string> = {
      low: '1-2 hours',
      medium: '1-3 days',
      high: '1-2 weeks',
    };
    return estimates[effort] || 'unknown';
  }

  /**
   * Save JSON file
   */
  public save(filename: string): void {
    const jsonData = this.buildJSONStructure();
    const jsonString = JSON.stringify(jsonData, null, 2);
    const blob = new Blob([jsonString], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${filename}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }
}
