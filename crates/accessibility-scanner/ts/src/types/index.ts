/**
 * Type definitions for Accessibility Scanner
 * Matches the Rust implementation types
 */

/**
 * Severity level of an accessibility issue
 */
export enum Severity {
  Critical = 'critical',
  Serious = 'serious',
  Moderate = 'moderate',
  Minor = 'minor',
  Info = 'info',
}

/**
 * WCAG 2.1 conformance level
 */
export enum WCAGLevel {
  A = 'A',
  AA = 'AA',
  AAA = 'AAA',
}

/**
 * WCAG 2.1 principle categories
 */
export enum Principle {
  Perceivable = 'Perceivable',
  Operable = 'Operable',
  Understandable = 'Understandable',
  Robust = 'Robust',
}

/**
 * A WCAG rule definition
 */
export interface Rule {
  id: string;
  name: string;
  description: string;
  level: WCAGLevel;
  principle: Principle;
  guideline: string;
  successCriterion: string;
  tags: string[];
}

/**
 * Position of an issue in the HTML
 */
export interface Position {
  line: number;
  column: number;
  xpath: string;
  selector: string;
}

/**
 * Context information about where an issue occurred
 */
export interface IssueContext {
  html: string;
  outerHtml: string;
  position: Position;
  attributes: Record<string, string>;
  computedStyles?: Record<string, string>;
}

/**
 * An accessibility issue found during scanning
 */
export interface Issue {
  id: string;
  ruleId: string;
  ruleName: string;
  severity: Severity;
  level: WCAGLevel;
  principle: Principle;
  message: string;
  help: string;
  helpUrl: string;
  context: IssueContext;
  fixSuggestions: string[];
  impactDescription: string;
  wcagReference: string;
}

/**
 * Statistics about a scan
 */
export interface ScanStatistics {
  totalIssues: number;
  critical: number;
  serious: number;
  moderate: number;
  minor: number;
  info: number;
  pagesScanned: number;
  elementsAnalyzed: number;
  durationMs: number;
  complianceScore: number;
}

/**
 * Configuration for a scan
 */
export interface ScanConfig {
  targetUrl: string;
  levels: WCAGLevel[];
  maxPages?: number;
  maxDepth?: number;
  includePatterns: string[];
  excludePatterns: string[];
  timeoutSeconds: number;
  followExternalLinks: boolean;
  checkImages: boolean;
  checkVideos: boolean;
  checkPdfs: boolean;
  parallelThreads: number;
  incremental: boolean;
  cacheEnabled: boolean;
}

/**
 * Result of a single page scan
 */
export interface PageResult {
  url: string;
  title: string;
  issues: Issue[];
  elementsCount: number;
  scanTimeMs: number;
  httpStatus: number;
  contentType: string;
}

/**
 * Complete scan result
 */
export interface ScanResult {
  id: string;
  config: ScanConfig;
  pages: PageResult[];
  statistics: ScanStatistics;
  startedAt: Date;
  completedAt: Date;
  version: string;
}

/**
 * Report format options
 */
export enum ReportFormat {
  JSON = 'json',
  HTML = 'html',
  CSV = 'csv',
  Summary = 'summary',
}

/**
 * Severity mapping utilities
 */
export class SeverityUtils {
  static getScore(severity: Severity): number {
    switch (severity) {
      case Severity.Critical:
        return 100;
      case Severity.Serious:
        return 75;
      case Severity.Moderate:
        return 50;
      case Severity.Minor:
        return 25;
      case Severity.Info:
        return 10;
      default:
        return 0;
    }
  }

  static fromString(value: string): Severity | undefined {
    const normalized = value.toLowerCase();
    return Object.values(Severity).find(s => s === normalized);
  }
}

/**
 * WCAG Level utilities
 */
export class WCAGLevelUtils {
  static fromString(value: string): WCAGLevel | undefined {
    const normalized = value.toUpperCase();
    return Object.values(WCAGLevel).find(l => l === normalized);
  }

  static includes(level: WCAGLevel, targetLevel: WCAGLevel): boolean {
    const levels = [WCAGLevel.A, WCAGLevel.AA, WCAGLevel.AAA];
    const levelIndex = levels.indexOf(level);
    const targetIndex = levels.indexOf(targetLevel);
    return levelIndex >= targetIndex;
  }
}

/**
 * Default scan configuration
 */
export const DEFAULT_SCAN_CONFIG: Partial<ScanConfig> = {
  levels: [WCAGLevel.A, WCAGLevel.AA],
  maxPages: 100,
  maxDepth: 3,
  includePatterns: [],
  excludePatterns: [],
  timeoutSeconds: 30,
  followExternalLinks: false,
  checkImages: true,
  checkVideos: true,
  checkPdfs: false,
  parallelThreads: 4,
  incremental: true,
  cacheEnabled: true,
};

/**
 * Create a default scan configuration
 */
export function createDefaultConfig(targetUrl: string): ScanConfig {
  return {
    targetUrl,
    ...DEFAULT_SCAN_CONFIG,
  } as ScanConfig;
}

/**
 * Calculate compliance score from statistics
 */
export function calculateComplianceScore(stats: ScanStatistics): number {
  const penalty =
    stats.critical * 100 +
    stats.serious * 75 +
    stats.moderate * 50 +
    stats.minor * 25 +
    stats.info * 10;

  const maxScore = 100;
  const elementsAnalyzed = Math.max(stats.elementsAnalyzed, 1);
  return Math.max(0, Math.min(100, maxScore - penalty / elementsAnalyzed));
}
