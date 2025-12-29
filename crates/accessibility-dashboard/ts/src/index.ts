/**
 * Meridian Accessibility Dashboard
 * Enterprise WCAG Compliance Monitoring and Reporting
 */

// Main Components
export { AccessibilityDashboard } from './components/Dashboard/AccessibilityDashboard';
export { ComplianceOverview } from './components/Dashboard/ComplianceOverview';
export { IssueBreakdown } from './components/Dashboard/IssueBreakdown';
export { TrendAnalysis } from './components/Dashboard/TrendAnalysis';

// Chart Components
export { ComplianceGauge } from './components/Charts/ComplianceGauge';
export { IssueDistribution } from './components/Charts/IssueDistribution';
export { TrendLineChart } from './components/Charts/TrendLineChart';
export { HeatmapCalendar } from './components/Charts/HeatmapCalendar';

// Widget Components
export { ScoreCard } from './components/Widgets/ScoreCard';
export { IssueList } from './components/Widgets/IssueList';
export { PageRanking } from './components/Widgets/PageRanking';

// Filter Components
export { WCAGLevelFilter } from './components/Filters/WCAGLevelFilter';
export { SeverityFilter } from './components/Filters/SeverityFilter';
export { DateRangeFilter } from './components/Filters/DateRangeFilter';

// Context & Hooks
export { DashboardProvider, useDashboardContext } from './context/DashboardContext';
export { useCompliance } from './hooks/useCompliance';
export { useIssues } from './hooks/useIssues';

// Utilities
export * from './utils/calculations';

// Types
export type {
  WCAGLevel,
  IssueSeverity,
  IssueCategory,
  IssueStatus,
  WCAGCriterion,
  AccessibilityIssue,
  ComplianceScore,
  PageCompliance,
  TrendDataPoint,
  CategoryBreakdown,
  SeverityBreakdown,
  HeatmapDataPoint,
  DashboardFilters,
  DashboardState,
  ComplianceMetrics,
  ChartData,
  ChartDataset,
  AccessibilityReport,
} from './types';
