/**
 * Compliance Score Calculation Utilities
 * Enterprise-grade algorithms for WCAG compliance metrics
 */

import type {
  ComplianceScore,
  AccessibilityIssue,
  IssueSeverity,
  IssueCategory,
  CategoryBreakdown,
  SeverityBreakdown,
  TrendDataPoint,
  WCAGLevel,
} from '../types';

/**
 * Calculate overall compliance score based on issues
 */
export function calculateComplianceScore(
  totalTests: number,
  passedTests: number,
  failedTests: number,
  warningTests: number,
  level: WCAGLevel = 'AA'
): ComplianceScore {
  const complianceRate = totalTests > 0 ? passedTests / totalTests : 0;
  const overall = Math.round(complianceRate * 100);

  return {
    overall,
    level,
    totalTests,
    passedTests,
    failedTests,
    warningTests,
    complianceRate,
    timestamp: new Date(),
  };
}

/**
 * Calculate weighted compliance score considering issue severity
 */
export function calculateWeightedScore(issues: AccessibilityIssue[]): number {
  if (issues.length === 0) return 100;

  const severityWeights: Record<IssueSeverity, number> = {
    critical: 10,
    serious: 5,
    moderate: 2,
    minor: 1,
  };

  const totalWeight = issues.reduce((sum, issue) => {
    return sum + severityWeights[issue.severity];
  }, 0);

  // Normalize to 0-100 scale (assuming max 50 critical issues = 0 score)
  const maxWeight = 500;
  const score = Math.max(0, 100 - (totalWeight / maxWeight) * 100);

  return Math.round(score);
}

/**
 * Break down issues by category
 */
export function calculateCategoryBreakdown(
  issues: AccessibilityIssue[]
): CategoryBreakdown[] {
  const categories: IssueCategory[] = [
    'perceivable',
    'operable',
    'understandable',
    'robust',
  ];

  const total = issues.length;

  return categories.map((category) => {
    const categoryIssues = issues.filter(
      (issue) => issue.criterion.category === category
    );
    const criticalCount = categoryIssues.filter(
      (issue) => issue.severity === 'critical'
    ).length;

    return {
      category,
      count: categoryIssues.length,
      criticalCount,
      percentage: total > 0 ? (categoryIssues.length / total) * 100 : 0,
    };
  });
}

/**
 * Break down issues by severity
 */
export function calculateSeverityBreakdown(
  issues: AccessibilityIssue[]
): SeverityBreakdown[] {
  const severities: IssueSeverity[] = ['critical', 'serious', 'moderate', 'minor'];
  const total = issues.length;

  return severities.map((severity) => {
    const count = issues.filter((issue) => issue.severity === severity).length;

    return {
      severity,
      count,
      percentage: total > 0 ? (count / total) * 100 : 0,
    };
  });
}

/**
 * Calculate trend direction
 */
export function calculateTrend(
  current: number,
  previous: number
): 'improving' | 'declining' | 'stable' {
  const threshold = 2; // 2% threshold for stability

  if (Math.abs(current - previous) < threshold) {
    return 'stable';
  }

  return current > previous ? 'improving' : 'declining';
}

/**
 * Calculate percentage change
 */
export function calculateChange(current: number, previous: number): number {
  if (previous === 0) return current > 0 ? 100 : 0;

  return ((current - previous) / previous) * 100;
}

/**
 * Generate trend data points from historical scores
 */
export function generateTrendData(
  scores: ComplianceScore[],
  issuesByDate: Map<string, AccessibilityIssue[]>
): TrendDataPoint[] {
  return scores.map((score) => {
    const dateKey = score.timestamp.toISOString().split('T')[0];
    const issues = issuesByDate.get(dateKey) || [];
    const criticalCount = issues.filter(
      (issue) => issue.severity === 'critical'
    ).length;

    return {
      date: score.timestamp,
      score: score.overall,
      issueCount: issues.length,
      criticalCount,
    };
  });
}

/**
 * Calculate compliance rate for a specific WCAG level
 */
export function calculateLevelCompliance(
  issues: AccessibilityIssue[],
  level: WCAGLevel
): number {
  const relevantIssues = issues.filter((issue) => {
    const criterionLevel = issue.criterion.level;

    // A includes only A criteria
    if (level === 'A') return criterionLevel === 'A';

    // AA includes A and AA criteria
    if (level === 'AA')
      return criterionLevel === 'A' || criterionLevel === 'AA';

    // AAA includes all criteria
    return true;
  });

  return calculateWeightedScore(relevantIssues);
}

/**
 * Calculate average score from multiple scores
 */
export function calculateAverageScore(scores: number[]): number {
  if (scores.length === 0) return 0;

  const sum = scores.reduce((acc, score) => acc + score, 0);
  return Math.round(sum / scores.length);
}

/**
 * Group issues by date
 */
export function groupIssuesByDate(
  issues: AccessibilityIssue[]
): Map<string, AccessibilityIssue[]> {
  const grouped = new Map<string, AccessibilityIssue[]>();

  issues.forEach((issue) => {
    const dateKey = issue.detectedAt.toISOString().split('T')[0];
    const existing = grouped.get(dateKey) || [];
    grouped.set(dateKey, [...existing, issue]);
  });

  return grouped;
}

/**
 * Calculate issue resolution rate
 */
export function calculateResolutionRate(issues: AccessibilityIssue[]): number {
  if (issues.length === 0) return 100;

  const resolved = issues.filter((issue) => issue.status === 'resolved').length;
  return (resolved / issues.length) * 100;
}

/**
 * Calculate mean time to resolution (in days)
 */
export function calculateMTTR(issues: AccessibilityIssue[]): number {
  const resolvedIssues = issues.filter((issue) => issue.status === 'resolved');

  if (resolvedIssues.length === 0) return 0;

  const totalDays = resolvedIssues.reduce((sum, issue) => {
    const days =
      (issue.updatedAt.getTime() - issue.detectedAt.getTime()) /
      (1000 * 60 * 60 * 24);
    return sum + days;
  }, 0);

  return Math.round(totalDays / resolvedIssues.length);
}

/**
 * Format percentage for display
 */
export function formatPercentage(value: number, decimals: number = 1): string {
  return `${value.toFixed(decimals)}%`;
}

/**
 * Get color for score (accessible color scheme)
 */
export function getScoreColor(score: number): string {
  if (score >= 90) return '#10b981'; // green-500
  if (score >= 70) return '#f59e0b'; // amber-500
  if (score >= 50) return '#f97316'; // orange-500
  return '#ef4444'; // red-500
}

/**
 * Get severity color (WCAG AA compliant contrast)
 */
export function getSeverityColor(severity: IssueSeverity): string {
  const colors: Record<IssueSeverity, string> = {
    critical: '#dc2626', // red-600
    serious: '#ea580c', // orange-600
    moderate: '#d97706', // amber-600
    minor: '#2563eb', // blue-600
  };

  return colors[severity];
}

/**
 * Get category color
 */
export function getCategoryColor(category: IssueCategory): string {
  const colors: Record<IssueCategory, string> = {
    perceivable: '#8b5cf6', // violet-500
    operable: '#3b82f6', // blue-500
    understandable: '#06b6d4', // cyan-500
    robust: '#10b981', // green-500
  };

  return colors[category];
}
