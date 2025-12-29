/**
 * Analyzes ARIA live regions
 */

import type {
  AccessibilityNode,
  LiveRegionStructure,
  LiveRegionInfo,
  LiveRegionIssue,
} from '../types';

export class LiveRegionAnalyzer {
  /**
   * Analyze live regions
   */
  public analyze(root: AccessibilityNode): LiveRegionStructure {
    const regions = this.extractLiveRegions(root);
    const issues = this.detectIssues(regions);
    const score = this.calculateScore(regions, issues);

    return { regions, issues, score };
  }

  /**
   * Extract live regions from tree
   */
  private extractLiveRegions(root: AccessibilityNode): LiveRegionInfo[] {
    const regions: LiveRegionInfo[] = [];

    const traverse = (node: AccessibilityNode) => {
      if (node.live && node.live !== 'off') {
        const region: LiveRegionInfo = {
          node,
          live: node.live,
          atomic: node.atomic || false,
          relevant: this.parseRelevant(node.relevant),
          busy: node.busy || false,
          label: node.name,
        };

        regions.push(region);
      }

      node.children.forEach(traverse);
    };

    traverse(root);

    return regions;
  }

  /**
   * Parse aria-relevant attribute
   */
  private parseRelevant(relevant: string | undefined): string[] {
    if (!relevant) {
      return ['additions', 'text'];
    }

    return relevant.split(/\s+/).filter(Boolean);
  }

  /**
   * Detect live region issues
   */
  private detectIssues(regions: LiveRegionInfo[]): LiveRegionIssue[] {
    const issues: LiveRegionIssue[] = [];

    regions.forEach(region => {
      // Missing role for status/alert
      if (!this.hasProperRole(region)) {
        issues.push({
          type: 'missing-role',
          severity: 'moderate',
          region,
          description: 'Live region should use role="status" or role="alert"',
          remediation: 'Use role="status" for polite announcements or role="alert" for important messages',
        });
      }

      // Improper politeness
      if (this.hasImproperPoliteness(region)) {
        issues.push({
          type: 'improper-politeness',
          severity: 'moderate',
          region,
          description: 'Live region politeness may not match content importance',
          remediation: 'Use aria-live="polite" for non-critical updates, "assertive" only for critical alerts',
        });
      }

      // Missing label for multiple regions
      if (this.needsLabel(regions, region) && !region.label) {
        issues.push({
          type: 'missing-label',
          severity: 'minor',
          region,
          description: 'Live region without label when multiple regions exist',
          remediation: 'Add aria-label to help users identify different live regions',
        });
      }
    });

    // Too frequent updates (requires monitoring over time)
    // This is a placeholder - actual implementation would need to track update frequency
    const frequentUpdateIssues = this.detectFrequentUpdates(regions);
    issues.push(...frequentUpdateIssues);

    return issues;
  }

  /**
   * Check if live region has proper role
   */
  private hasProperRole(region: LiveRegionInfo): boolean {
    const role = region.node.role;
    const liveRoles = ['alert', 'status', 'log', 'timer', 'marquee'];

    return liveRoles.includes(role);
  }

  /**
   * Check if politeness level matches content
   */
  private hasImproperPoliteness(region: LiveRegionInfo): boolean {
    const role = region.node.role;

    // Alert role should use assertive
    if (role === 'alert' && region.live !== 'assertive') {
      return true;
    }

    // Status role should use polite
    if (role === 'status' && region.live !== 'polite') {
      return true;
    }

    // Assertive should be rare
    if (region.live === 'assertive' && role !== 'alert') {
      // Might be appropriate in some cases, but worth flagging
      return true;
    }

    return false;
  }

  /**
   * Check if region needs a label
   */
  private needsLabel(regions: LiveRegionInfo[], region: LiveRegionInfo): boolean {
    // If multiple regions of same type, they need labels
    const sameTypeRegions = regions.filter(r => r.node.role === region.node.role);
    return sameTypeRegions.length > 1;
  }

  /**
   * Detect regions with too frequent updates
   */
  private detectFrequentUpdates(regions: LiveRegionInfo[]): LiveRegionIssue[] {
    const issues: LiveRegionIssue[] = [];

    // This is a placeholder implementation
    // Real implementation would need to monitor DOM changes over time
    regions.forEach(region => {
      // Check for busy state
      if (region.busy) {
        issues.push({
          type: 'too-frequent-updates',
          severity: 'minor',
          region,
          description: 'Live region is marked busy, may be updating too frequently',
          remediation: 'Batch updates and use aria-busy to prevent announcement spam',
        });
      }
    });

    return issues;
  }

  /**
   * Calculate score
   */
  private calculateScore(regions: LiveRegionInfo[], issues: LiveRegionIssue[]): number {
    if (regions.length === 0) {
      return 100; // No live regions is fine
    }

    let score = 100;

    // Deduct for issues
    issues.forEach(issue => {
      switch (issue.severity) {
        case 'critical':
          score -= 30;
          break;
        case 'serious':
          score -= 20;
          break;
        case 'moderate':
          score -= 10;
          break;
        case 'minor':
          score -= 5;
          break;
      }
    });

    // Bonus for proper implementation
    const properRoles = regions.filter(r => this.hasProperRole(r)).length;
    const roleScore = (properRoles / regions.length) * 10;
    score += roleScore;

    return Math.max(0, Math.min(100, Math.round(score)));
  }

  /**
   * Monitor live region updates
   */
  public monitorUpdates(
    region: LiveRegionInfo,
    callback: (text: string) => void
  ): () => void {
    const element = region.node.element;

    const observer = new MutationObserver(mutations => {
      mutations.forEach(mutation => {
        if (mutation.type === 'characterData' || mutation.type === 'childList') {
          const text = element.textContent?.trim() || '';
          if (text) {
            callback(text);
          }
        }
      });
    });

    observer.observe(element, {
      characterData: true,
      childList: true,
      subtree: region.atomic === false,
    });

    return () => observer.disconnect();
  }
}
