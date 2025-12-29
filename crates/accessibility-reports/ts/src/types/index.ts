/**
 * Comprehensive type definitions for enterprise accessibility reporting
 */

export type ExportFormat = 'pdf' | 'excel' | 'html' | 'json';

export type ComplianceLevel = 'A' | 'AA' | 'AAA';

export type SeverityLevel = 'critical' | 'serious' | 'moderate' | 'minor';

export type ReportType = 'executive' | 'technical' | 'compliance' | 'remediation';

export interface AccessibilityIssue {
  id: string;
  title: string;
  description: string;
  severity: SeverityLevel;
  wcagCriteria: string[];
  wcagLevel: ComplianceLevel;
  impact: string;
  affectedUsers: number;
  location: {
    url: string;
    selector?: string;
    screenshot?: string;
  };
  remediation: {
    effort: 'low' | 'medium' | 'high';
    priority: number;
    steps: string[];
    codeExample?: string;
  };
  detectedBy: 'automated' | 'manual' | 'user-reported';
  detectedAt: Date;
  status: 'open' | 'in-progress' | 'resolved' | 'wont-fix';
}

export interface ComplianceMetrics {
  totalIssues: number;
  criticalIssues: number;
  seriousIssues: number;
  moderateIssues: number;
  minorIssues: number;
  wcagACompliance: number;
  wcagAACompliance: number;
  wcagAAACompliance: number;
  successCriteriaPassed: number;
  successCriteriaFailed: number;
  successCriteriaTotal: number;
  complianceScore: number;
}

export interface ReportSection {
  id: string;
  title: string;
  type: 'summary' | 'metrics' | 'issues' | 'trends' | 'recommendations' | 'technical' | 'custom';
  order: number;
  enabled: boolean;
  data?: any;
  subsections?: ReportSection[];
}

export interface BrandingConfig {
  companyName: string;
  logo?: string;
  primaryColor: string;
  secondaryColor: string;
  accentColor: string;
  fontFamily: string;
  headerText?: string;
  footerText?: string;
  watermark?: string;
  includePageNumbers: boolean;
  includeDateGenerated: boolean;
}

export interface ReportTemplate {
  id: string;
  name: string;
  description: string;
  type: ReportType;
  sections: ReportSection[];
  defaultBranding?: Partial<BrandingConfig>;
}

export interface ReportConfig {
  id: string;
  title: string;
  subtitle?: string;
  description?: string;
  template: ReportTemplate;
  sections: ReportSection[];
  branding: BrandingConfig;
  dateRange: {
    from: Date;
    to: Date;
  };
  filters?: {
    severity?: SeverityLevel[];
    wcagLevel?: ComplianceLevel[];
    status?: string[];
    tags?: string[];
  };
  createdAt: Date;
  createdBy: string;
  version: string;
}

export interface ScheduledExport {
  id: string;
  reportConfig: ReportConfig;
  schedule: {
    frequency: 'daily' | 'weekly' | 'monthly' | 'quarterly';
    dayOfWeek?: number;
    dayOfMonth?: number;
    time: string;
  };
  recipients: string[];
  formats: ExportFormat[];
  enabled: boolean;
  lastRun?: Date;
  nextRun: Date;
}

export interface ExportOptions {
  format: ExportFormat;
  filename: string;
  orientation?: 'portrait' | 'landscape';
  pageSize?: 'A4' | 'Letter' | 'Legal';
  includeCharts: boolean;
  includeScreenshots: boolean;
  compression?: boolean;
  accessibility?: {
    pdfUA: boolean;
    tagged: boolean;
    altText: boolean;
    structure: boolean;
  };
}

export interface ChartData {
  type: 'bar' | 'line' | 'pie' | 'doughnut' | 'radar';
  title: string;
  labels: string[];
  datasets: {
    label: string;
    data: number[];
    backgroundColor?: string | string[];
    borderColor?: string | string[];
  }[];
  accessible: boolean;
  altText: string;
}

export interface TrendData {
  date: Date;
  totalIssues: number;
  criticalIssues: number;
  resolvedIssues: number;
  complianceScore: number;
}

export interface ReportData {
  config: ReportConfig;
  issues: AccessibilityIssue[];
  metrics: ComplianceMetrics;
  trends: TrendData[];
  charts: ChartData[];
  generatedAt: Date;
  generatedBy: string;
}
