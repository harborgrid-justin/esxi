/**
 * WCAG Compliance Dashboard Types
 * Production-ready type definitions for enterprise accessibility monitoring
 */

export type WCAGLevel = 'A' | 'AA' | 'AAA';

export type IssueSeverity = 'critical' | 'serious' | 'moderate' | 'minor';

export type IssueCategory =
  | 'perceivable'
  | 'operable'
  | 'understandable'
  | 'robust';

export type IssueStatus = 'open' | 'in-progress' | 'resolved' | 'wont-fix';

export interface WCAGCriterion {
  id: string;
  code: string; // e.g., "1.1.1"
  name: string;
  level: WCAGLevel;
  category: IssueCategory;
  description: string;
}

export interface AccessibilityIssue {
  id: string;
  criterion: WCAGCriterion;
  severity: IssueSeverity;
  status: IssueStatus;
  pageUrl: string;
  element: string;
  xpath?: string;
  description: string;
  recommendation: string;
  detectedAt: Date;
  updatedAt: Date;
  assignee?: string;
  impact: number; // 0-100 score
}

export interface ComplianceScore {
  overall: number; // 0-100
  level: WCAGLevel;
  totalTests: number;
  passedTests: number;
  failedTests: number;
  warningTests: number;
  complianceRate: number; // 0-1
  timestamp: Date;
}

export interface PageCompliance {
  url: string;
  title: string;
  score: ComplianceScore;
  issueCount: number;
  criticalIssues: number;
  lastScanned: Date;
}

export interface TrendDataPoint {
  date: Date;
  score: number;
  issueCount: number;
  criticalCount: number;
}

export interface CategoryBreakdown {
  category: IssueCategory;
  count: number;
  criticalCount: number;
  percentage: number;
}

export interface SeverityBreakdown {
  severity: IssueSeverity;
  count: number;
  percentage: number;
}

export interface HeatmapDataPoint {
  date: Date;
  value: number; // 0-100 compliance score
  issueCount: number;
}

export interface DashboardFilters {
  wcagLevels: WCAGLevel[];
  severities: IssueSeverity[];
  categories: IssueCategory[];
  statuses: IssueStatus[];
  dateRange: {
    start: Date | null;
    end: Date | null;
  };
  searchQuery: string;
}

export interface DashboardState {
  filters: DashboardFilters;
  selectedPage: string | null;
  selectedIssue: string | null;
  viewMode: 'overview' | 'detailed' | 'trends';
  dateRangePreset: 'week' | 'month' | 'quarter' | 'year' | 'custom';
}

export interface ComplianceMetrics {
  currentScore: ComplianceScore;
  previousScore: ComplianceScore;
  change: number;
  trend: 'improving' | 'declining' | 'stable';
  issues: {
    total: number;
    byCategory: CategoryBreakdown[];
    bySeverity: SeverityBreakdown[];
  };
  pages: {
    total: number;
    compliant: number;
    nonCompliant: number;
    avgScore: number;
  };
}

export interface ChartDataset {
  label: string;
  data: number[];
  backgroundColor?: string | string[];
  borderColor?: string | string[];
  borderWidth?: number;
  fill?: boolean;
}

export interface ChartData {
  labels: string[];
  datasets: ChartDataset[];
}

export interface AccessibilityReport {
  id: string;
  generatedAt: Date;
  scope: {
    pageCount: number;
    urlPattern?: string;
  };
  metrics: ComplianceMetrics;
  issues: AccessibilityIssue[];
  recommendations: string[];
  exportFormat?: 'pdf' | 'csv' | 'json' | 'html';
}
