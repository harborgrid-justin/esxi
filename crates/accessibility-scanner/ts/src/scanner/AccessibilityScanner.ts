/**
 * Accessibility Scanner - Main TypeScript Implementation
 */

import axios, { AxiosInstance } from 'axios';
import { v4 as uuidv4 } from 'uuid';
import {
  ScanConfig,
  ScanResult,
  PageResult,
  ScanStatistics,
  Issue,
  WCAGLevel,
  calculateComplianceScore,
} from '../types';
import { DOMAnalyzer } from './DOMAnalyzer';
import { getRuleEngine } from './RuleEngine';

/**
 * Main Accessibility Scanner class
 */
export class AccessibilityScanner {
  private config: ScanConfig;
  private httpClient: AxiosInstance;
  private ruleEngine = getRuleEngine();

  constructor(config: ScanConfig) {
    this.config = config;
    this.httpClient = axios.create({
      timeout: config.timeoutSeconds * 1000,
      headers: {
        'User-Agent': 'HarborGrid-AccessibilityScanner/0.1.0',
      },
    });
  }

  /**
   * Scan a single URL
   */
  async scanUrl(url: string): Promise<PageResult> {
    const startTime = Date.now();

    try {
      // Fetch the page
      const response = await this.httpClient.get(url);
      const html = response.data;
      const status = response.status;
      const contentType = response.headers['content-type'] || 'text/html';

      // Parse and analyze
      const analyzer = new DOMAnalyzer(html);
      const issues = this.analyzeDocument(analyzer);

      // Extract metadata
      const title = this.extractTitle(html);
      const elementsCount = this.countElements(html);

      const scanTimeMs = Date.now() - startTime;

      return {
        url,
        title,
        issues,
        elementsCount,
        scanTimeMs,
        httpStatus: status,
        contentType,
      };
    } catch (error) {
      throw new Error(`Failed to scan ${url}: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  /**
   * Scan a website (multiple pages)
   */
  async scan(): Promise<ScanResult> {
    const startedAt = new Date();
    const pages: PageResult[] = [];

    try {
      // Scan the target URL
      const page = await this.scanUrl(this.config.targetUrl);
      pages.push(page);

      // TODO: Implement crawling for multiple pages based on config
      // For now, we only scan the single target URL

    } catch (error) {
      throw new Error(`Scan failed: ${error instanceof Error ? error.message : String(error)}`);
    }

    const completedAt = new Date();

    // Calculate statistics
    const statistics = this.calculateStatistics(pages, startedAt, completedAt);

    return {
      id: uuidv4(),
      config: this.config,
      pages,
      statistics,
      startedAt,
      completedAt,
      version: '0.1.0',
    };
  }

  /**
   * Analyze a document using the DOM analyzer
   */
  private analyzeDocument(analyzer: DOMAnalyzer): Issue[] {
    const issues: Issue[] = [];

    // Run all applicable checks based on configured WCAG levels
    const checks = [
      () => analyzer.checkImageAlt(),
      () => analyzer.checkFormLabels(),
      () => analyzer.checkDocumentTitle(),
      () => analyzer.checkHtmlLang(),
      () => analyzer.checkHeadingStructure(),
      () => analyzer.checkDuplicateIds(),
      () => analyzer.checkLinkText(),
      () => analyzer.checkButtonText(),
    ];

    // Add AA-level checks if configured
    if (this.config.levels.includes(WCAGLevel.AA)) {
      checks.push(() => analyzer.analyzeColorContrast());
    }

    // Execute all checks
    for (const check of checks) {
      try {
        const checkIssues = check();
        issues.push(...checkIssues);
      } catch (error) {
        console.error('Check failed:', error);
      }
    }

    // Filter issues by configured WCAG levels
    return issues.filter(issue => this.config.levels.includes(issue.level));
  }

  /**
   * Extract page title from HTML
   */
  private extractTitle(html: string): string {
    const titleMatch = html.match(/<title[^>]*>([^<]+)<\/title>/i);
    return titleMatch ? titleMatch[1].trim() : '';
  }

  /**
   * Count elements in HTML
   */
  private countElements(html: string): number {
    // Simple element count based on opening tags
    const matches = html.match(/<[a-z][a-z0-9]*[^>]*>/gi);
    return matches ? matches.length : 0;
  }

  /**
   * Calculate scan statistics
   */
  private calculateStatistics(
    pages: PageResult[],
    startedAt: Date,
    completedAt: Date
  ): ScanStatistics {
    const stats: ScanStatistics = {
      totalIssues: 0,
      critical: 0,
      serious: 0,
      moderate: 0,
      minor: 0,
      info: 0,
      pagesScanned: pages.length,
      elementsAnalyzed: 0,
      durationMs: completedAt.getTime() - startedAt.getTime(),
      complianceScore: 100,
    };

    // Aggregate issues from all pages
    for (const page of pages) {
      stats.elementsAnalyzed += page.elementsCount;

      for (const issue of page.issues) {
        stats.totalIssues++;

        switch (issue.severity) {
          case 'critical':
            stats.critical++;
            break;
          case 'serious':
            stats.serious++;
            break;
          case 'moderate':
            stats.moderate++;
            break;
          case 'minor':
            stats.minor++;
            break;
          case 'info':
            stats.info++;
            break;
        }
      }
    }

    // Calculate compliance score
    stats.complianceScore = calculateComplianceScore(stats);

    return stats;
  }

  /**
   * Get the current configuration
   */
  getConfig(): ScanConfig {
    return { ...this.config };
  }

  /**
   * Update configuration
   */
  updateConfig(updates: Partial<ScanConfig>): void {
    this.config = { ...this.config, ...updates };
  }
}

/**
 * Create a scanner instance with default configuration
 */
export function createScanner(targetUrl: string, levels: WCAGLevel[] = [WCAGLevel.A, WCAGLevel.AA]): AccessibilityScanner {
  const config: ScanConfig = {
    targetUrl,
    levels,
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

  return new AccessibilityScanner(config);
}

/**
 * Quick scan helper function
 */
export async function quickScan(url: string, levels: WCAGLevel[] = [WCAGLevel.A, WCAGLevel.AA]): Promise<ScanResult> {
  const scanner = createScanner(url, levels);
  return await scanner.scan();
}
