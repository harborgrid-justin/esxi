/**
 * HarborGrid Accessibility Scanner - TypeScript SDK
 *
 * Enterprise-grade WCAG 2.1 accessibility scanner with comprehensive DOM analysis
 *
 * @packageDocumentation
 */

// Export types
export * from './types';

// Export scanner classes
export { AccessibilityScanner, createScanner, quickScan } from './scanner/AccessibilityScanner';
export { DOMAnalyzer, ColorRGB } from './scanner/DOMAnalyzer';
export { RuleEngine, getRuleEngine } from './scanner/RuleEngine';

// Re-export commonly used items for convenience
export {
  Severity,
  WCAGLevel,
  Principle,
  ReportFormat,
  SeverityUtils,
  WCAGLevelUtils,
  createDefaultConfig,
  calculateComplianceScore,
} from './types';

/**
 * Scanner version
 */
export const VERSION = '0.1.0';

/**
 * Quick example usage:
 *
 * ```typescript
 * import { quickScan, WCAGLevel } from '@harborgrid/accessibility-scanner';
 *
 * const result = await quickScan('https://example.com', [WCAGLevel.A, WCAGLevel.AA]);
 * console.log(`Compliance Score: ${result.statistics.complianceScore}`);
 * console.log(`Total Issues: ${result.statistics.totalIssues}`);
 * ```
 */

/**
 * Advanced usage with custom configuration:
 *
 * ```typescript
 * import { AccessibilityScanner, ScanConfig, WCAGLevel } from '@harborgrid/accessibility-scanner';
 *
 * const config: ScanConfig = {
 *   targetUrl: 'https://example.com',
 *   levels: [WCAGLevel.A, WCAGLevel.AA, WCAGLevel.AAA],
 *   maxPages: 50,
 *   maxDepth: 3,
 *   includePatterns: [],
 *   excludePatterns: ['/admin/', '/private/'],
 *   timeoutSeconds: 30,
 *   followExternalLinks: false,
 *   checkImages: true,
 *   checkVideos: true,
 *   checkPdfs: true,
 *   parallelThreads: 8,
 *   incremental: true,
 *   cacheEnabled: true,
 * };
 *
 * const scanner = new AccessibilityScanner(config);
 * const result = await scanner.scan();
 *
 * // Process results
 * for (const page of result.pages) {
 *   console.log(`Page: ${page.url}`);
 *   console.log(`Issues: ${page.issues.length}`);
 *
 *   for (const issue of page.issues) {
 *     console.log(`  [${issue.severity}] ${issue.message}`);
 *     console.log(`  Element: ${issue.context.position.selector}`);
 *     console.log(`  Fix: ${issue.fixSuggestions[0]}`);
 *   }
 * }
 * ```
 */

/**
 * Get scanner information
 */
export function getScannerInfo() {
  return {
    name: '@harborgrid/accessibility-scanner',
    version: VERSION,
    description: 'Enterprise-grade WCAG 2.1 accessibility scanner',
    wcagVersion: '2.1',
    supportedLevels: ['A', 'AA', 'AAA'],
    features: [
      'Comprehensive WCAG 2.1 Support',
      'Advanced DOM Analysis',
      'Color Contrast Checking',
      'ARIA Validation',
      'Heading Structure Analysis',
      'Form Accessibility Checks',
      'Multiple Report Formats',
    ],
  };
}
