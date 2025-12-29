import * as XLSX from 'xlsx';
import { ReportData, ExportOptions, AccessibilityIssue } from '../types';
import { formatDate, formatPercentage } from '../utils/formatting';

/**
 * Excel Report Generator
 * Creates comprehensive Excel workbooks with multiple sheets
 */
export class ExcelGenerator {
  private workbook: XLSX.WorkBook;

  constructor(private options: ExportOptions, private reportData: ReportData) {
    this.workbook = XLSX.utils.book_new();
  }

  /**
   * Generate the complete Excel report
   */
  public async generate(): Promise<Blob> {
    this.addSummarySheet();
    this.addMetricsSheet();
    this.addIssuesSheet();
    this.addTrendsSheet();
    this.addRecommendationsSheet();

    // Convert to blob
    const excelBuffer = XLSX.write(this.workbook, {
      type: 'array',
      bookType: 'xlsx',
      compression: this.options.compression,
    });

    return new Blob([excelBuffer], {
      type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
    });
  }

  /**
   * Add summary sheet
   */
  private addSummarySheet(): void {
    const { config, metrics, generatedAt } = this.reportData;

    const data = [
      ['Accessibility Compliance Report'],
      [],
      ['Report Information'],
      ['Title', config.title],
      ['Subtitle', config.subtitle || ''],
      ['Company', config.branding.companyName],
      ['Generated', formatDate(generatedAt)],
      ['Period From', formatDate(config.dateRange.from)],
      ['Period To', formatDate(config.dateRange.to)],
      [],
      ['Key Metrics'],
      ['Overall Compliance Score', formatPercentage(metrics.complianceScore)],
      ['Total Issues Found', metrics.totalIssues],
      ['Critical Issues', metrics.criticalIssues],
      ['Serious Issues', metrics.seriousIssues],
      ['Moderate Issues', metrics.moderateIssues],
      ['Minor Issues', metrics.minorIssues],
      [],
      ['WCAG Compliance'],
      ['Level A', formatPercentage(metrics.wcagACompliance)],
      ['Level AA', formatPercentage(metrics.wcagAACompliance)],
      ['Level AAA', formatPercentage(metrics.wcagAAACompliance)],
      [],
      ['Success Criteria'],
      ['Passed', metrics.successCriteriaPassed],
      ['Failed', metrics.successCriteriaFailed],
      ['Total', metrics.successCriteriaTotal],
    ];

    const worksheet = XLSX.utils.aoa_to_sheet(data);

    // Set column widths
    worksheet['!cols'] = [{ wch: 30 }, { wch: 30 }];

    // Style the header
    if (worksheet['A1']) {
      worksheet['A1'].s = {
        font: { bold: true, sz: 16 },
        alignment: { horizontal: 'center' },
      };
    }

    XLSX.utils.book_append_sheet(this.workbook, worksheet, 'Summary');
  }

  /**
   * Add metrics sheet
   */
  private addMetricsSheet(): void {
    const { metrics } = this.reportData;

    const data = [
      ['Metric Category', 'Metric Name', 'Value', 'Percentage'],
      [],
      ['Issue Severity', 'Critical', metrics.criticalIssues, this.calculatePercentage(metrics.criticalIssues, metrics.totalIssues)],
      ['Issue Severity', 'Serious', metrics.seriousIssues, this.calculatePercentage(metrics.seriousIssues, metrics.totalIssues)],
      ['Issue Severity', 'Moderate', metrics.moderateIssues, this.calculatePercentage(metrics.moderateIssues, metrics.totalIssues)],
      ['Issue Severity', 'Minor', metrics.minorIssues, this.calculatePercentage(metrics.minorIssues, metrics.totalIssues)],
      [],
      ['WCAG Compliance', 'Level A', metrics.wcagACompliance, `${metrics.wcagACompliance}%`],
      ['WCAG Compliance', 'Level AA', metrics.wcagAACompliance, `${metrics.wcagAACompliance}%`],
      ['WCAG Compliance', 'Level AAA', metrics.wcagAAACompliance, `${metrics.wcagAAACompliance}%`],
      [],
      ['Success Criteria', 'Passed', metrics.successCriteriaPassed, this.calculatePercentage(metrics.successCriteriaPassed, metrics.successCriteriaTotal)],
      ['Success Criteria', 'Failed', metrics.successCriteriaFailed, this.calculatePercentage(metrics.successCriteriaFailed, metrics.successCriteriaTotal)],
      ['Success Criteria', 'Total', metrics.successCriteriaTotal, '100%'],
    ];

    const worksheet = XLSX.utils.aoa_to_sheet(data);
    worksheet['!cols'] = [{ wch: 20 }, { wch: 20 }, { wch: 15 }, { wch: 15 }];

    XLSX.utils.book_append_sheet(this.workbook, worksheet, 'Metrics');
  }

  /**
   * Add issues sheet with all issues
   */
  private addIssuesSheet(): void {
    const headers = [
      'ID',
      'Title',
      'Description',
      'Severity',
      'WCAG Criteria',
      'WCAG Level',
      'Impact',
      'Affected Users',
      'URL',
      'Selector',
      'Status',
      'Detected By',
      'Detected At',
      'Priority',
      'Effort',
    ];

    const rows = this.reportData.issues.map((issue) => [
      issue.id,
      issue.title,
      issue.description,
      issue.severity,
      issue.wcagCriteria.join(', '),
      issue.wcagLevel,
      issue.impact,
      issue.affectedUsers,
      issue.location.url,
      issue.location.selector || '',
      issue.status,
      issue.detectedBy,
      formatDate(issue.detectedAt),
      issue.remediation.priority,
      issue.remediation.effort,
    ]);

    const data = [headers, ...rows];
    const worksheet = XLSX.utils.aoa_to_sheet(data);

    // Set column widths
    worksheet['!cols'] = [
      { wch: 15 }, // ID
      { wch: 30 }, // Title
      { wch: 50 }, // Description
      { wch: 12 }, // Severity
      { wch: 20 }, // WCAG Criteria
      { wch: 10 }, // WCAG Level
      { wch: 40 }, // Impact
      { wch: 15 }, // Affected Users
      { wch: 40 }, // URL
      { wch: 30 }, // Selector
      { wch: 12 }, // Status
      { wch: 12 }, // Detected By
      { wch: 12 }, // Detected At
      { wch: 10 }, // Priority
      { wch: 10 }, // Effort
    ];

    // Add autofilter
    worksheet['!autofilter'] = { ref: `A1:O${rows.length + 1}` };

    XLSX.utils.book_append_sheet(this.workbook, worksheet, 'All Issues');

    // Add separate sheets for each severity
    this.addIssuesBySeveritySheet('critical');
    this.addIssuesBySeveritySheet('serious');
    this.addIssuesBySeveritySheet('moderate');
    this.addIssuesBySeveritySheet('minor');
  }

  /**
   * Add issues by severity sheet
   */
  private addIssuesBySeveritySheet(severity: string): void {
    const filteredIssues = this.reportData.issues.filter(
      (issue) => issue.severity === severity
    );

    if (filteredIssues.length === 0) return;

    const headers = [
      'Title',
      'Description',
      'WCAG Criteria',
      'Impact',
      'URL',
      'Status',
      'Priority',
      'Remediation Steps',
    ];

    const rows = filteredIssues.map((issue) => [
      issue.title,
      issue.description,
      issue.wcagCriteria.join(', '),
      issue.impact,
      issue.location.url,
      issue.status,
      issue.remediation.priority,
      issue.remediation.steps.join(' | '),
    ]);

    const data = [headers, ...rows];
    const worksheet = XLSX.utils.aoa_to_sheet(data);

    worksheet['!cols'] = [
      { wch: 30 },
      { wch: 50 },
      { wch: 20 },
      { wch: 40 },
      { wch: 40 },
      { wch: 12 },
      { wch: 10 },
      { wch: 60 },
    ];

    const sheetName = severity.charAt(0).toUpperCase() + severity.slice(1) + ' Issues';
    XLSX.utils.book_append_sheet(this.workbook, worksheet, sheetName);
  }

  /**
   * Add trends sheet
   */
  private addTrendsSheet(): void {
    const headers = [
      'Date',
      'Total Issues',
      'Critical Issues',
      'Resolved Issues',
      'Compliance Score',
    ];

    const rows = this.reportData.trends.map((trend) => [
      formatDate(trend.date),
      trend.totalIssues,
      trend.criticalIssues,
      trend.resolvedIssues,
      `${trend.complianceScore}%`,
    ]);

    const data = [headers, ...rows];
    const worksheet = XLSX.utils.aoa_to_sheet(data);

    worksheet['!cols'] = [
      { wch: 12 },
      { wch: 15 },
      { wch: 15 },
      { wch: 15 },
      { wch: 18 },
    ];

    XLSX.utils.book_append_sheet(this.workbook, worksheet, 'Trends');
  }

  /**
   * Add recommendations sheet
   */
  private addRecommendationsSheet(): void {
    const priorityIssues = this.reportData.issues
      .filter((i) => i.severity === 'critical' || i.severity === 'serious')
      .slice(0, 20);

    const headers = [
      'Priority',
      'Title',
      'Severity',
      'Impact',
      'WCAG Criteria',
      'Effort',
      'Remediation Steps',
      'Code Example',
    ];

    const rows = priorityIssues.map((issue, index) => [
      index + 1,
      issue.title,
      issue.severity,
      issue.impact,
      issue.wcagCriteria.join(', '),
      issue.remediation.effort,
      issue.remediation.steps.join('\n'),
      issue.remediation.codeExample || '',
    ]);

    const data = [headers, ...rows];
    const worksheet = XLSX.utils.aoa_to_sheet(data);

    worksheet['!cols'] = [
      { wch: 10 },
      { wch: 30 },
      { wch: 12 },
      { wch: 40 },
      { wch: 20 },
      { wch: 10 },
      { wch: 60 },
      { wch: 40 },
    ];

    XLSX.utils.book_append_sheet(this.workbook, worksheet, 'Recommendations');
  }

  /**
   * Calculate percentage
   */
  private calculatePercentage(value: number, total: number): string {
    if (total === 0) return '0%';
    return `${((value / total) * 100).toFixed(1)}%`;
  }

  /**
   * Save Excel file
   */
  public save(filename: string): void {
    XLSX.writeFile(this.workbook, `${filename}.xlsx`);
  }
}
