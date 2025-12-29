/**
 * Analyzes landmark structure and detects issues
 */

import type {
  AccessibilityNode,
  LandmarkStructure,
  LandmarkInfo,
  LandmarkIssue,
  SeverityLevel,
} from '../types';

export class LandmarkAnalyzer {
  private readonly landmarkRoles = [
    'banner',
    'navigation',
    'main',
    'complementary',
    'contentinfo',
    'region',
    'search',
    'form',
  ];

  /**
   * Analyze landmark structure
   */
  public analyze(root: AccessibilityNode): LandmarkStructure {
    const landmarks = this.extractLandmarks(root);
    const issues = this.detectIssues(landmarks, root);
    const score = this.calculateScore(landmarks, issues);

    return { landmarks, issues, score };
  }

  /**
   * Extract landmarks from tree
   */
  private extractLandmarks(root: AccessibilityNode): LandmarkInfo[] {
    const landmarks: LandmarkInfo[] = [];

    const traverse = (node: AccessibilityNode, level: number = 0) => {
      if (this.landmarkRoles.includes(node.role)) {
        const landmark: LandmarkInfo = {
          node,
          role: node.role,
          label: node.name,
          level,
          duplicateLabels: false,
          missingLabel: false,
          children: [],
        };

        // Find child landmarks
        node.children.forEach(child => {
          const childLandmarks = this.extractLandmarksFromNode(child, level + 1);
          landmark.children.push(...childLandmarks);
        });

        landmarks.push(landmark);
      } else {
        // Continue traversing for landmarks
        node.children.forEach(child => traverse(child, level));
      }
    };

    traverse(root);

    // Mark duplicate labels
    this.markDuplicateLabels(landmarks);

    return landmarks;
  }

  /**
   * Extract landmarks from a single node
   */
  private extractLandmarksFromNode(node: AccessibilityNode, level: number): LandmarkInfo[] {
    const landmarks: LandmarkInfo[] = [];

    const traverse = (n: AccessibilityNode, l: number) => {
      if (this.landmarkRoles.includes(n.role)) {
        const landmark: LandmarkInfo = {
          node: n,
          role: n.role,
          label: n.name,
          level: l,
          duplicateLabels: false,
          missingLabel: false,
          children: [],
        };

        n.children.forEach(child => {
          const childLandmarks = this.extractLandmarksFromNode(child, l + 1);
          landmark.children.push(...childLandmarks);
        });

        landmarks.push(landmark);
      } else {
        n.children.forEach(child => traverse(child, l));
      }
    };

    traverse(node, level);
    return landmarks;
  }

  /**
   * Mark landmarks with duplicate labels
   */
  private markDuplicateLabels(landmarks: LandmarkInfo[]): void {
    const labelCounts = new Map<string, number>();

    const countLabels = (landmark: LandmarkInfo) => {
      const key = `${landmark.role}:${landmark.label || ''}`;
      labelCounts.set(key, (labelCounts.get(key) || 0) + 1);
      landmark.children.forEach(countLabels);
    };

    landmarks.forEach(countLabels);

    const markDuplicates = (landmark: LandmarkInfo) => {
      const key = `${landmark.role}:${landmark.label || ''}`;
      const count = labelCounts.get(key) || 0;

      if (count > 1 && landmark.label) {
        landmark.duplicateLabels = true;
      }

      if (count > 1 && !landmark.label) {
        landmark.missingLabel = true;
      }

      landmark.children.forEach(markDuplicates);
    };

    landmarks.forEach(markDuplicates);
  }

  /**
   * Detect landmark issues
   */
  private detectIssues(landmarks: LandmarkInfo[], root: AccessibilityNode): LandmarkIssue[] {
    const issues: LandmarkIssue[] = [];

    // Missing main landmark
    const mainLandmarks = this.findLandmarksByRole(landmarks, 'main');
    if (mainLandmarks.length === 0) {
      issues.push({
        type: 'missing-main',
        severity: 'serious',
        node: root,
        description: 'Page is missing a main landmark',
        remediation: 'Add a <main> element or role="main" to identify the primary content area',
      });
    }

    // Multiple main landmarks
    if (mainLandmarks.length > 1) {
      mainLandmarks.slice(1).forEach(landmark => {
        issues.push({
          type: 'multiple-main',
          severity: 'serious',
          node: landmark.node,
          description: 'Multiple main landmarks found on page',
          remediation: 'Use only one main landmark per page to identify the primary content',
        });
      });
    }

    // Missing labels on multiple landmarks of same type
    const allLandmarks = this.flattenLandmarks(landmarks);
    allLandmarks.forEach(landmark => {
      if (landmark.missingLabel) {
        issues.push({
          type: 'missing-label',
          severity: 'serious',
          node: landmark.node,
          description: `${landmark.role} landmark is missing a label when multiple ${landmark.role} landmarks exist`,
          remediation: `Add aria-label or aria-labelledby to distinguish this ${landmark.role} from others`,
        });
      }

      if (landmark.duplicateLabels) {
        issues.push({
          type: 'duplicate-label',
          severity: 'moderate',
          node: landmark.node,
          description: `Multiple ${landmark.role} landmarks have the same label "${landmark.label}"`,
          remediation: 'Use unique labels for landmarks of the same type to help users distinguish between them',
        });
      }
    });

    // Improper nesting
    const nestingIssues = this.detectImproperNesting(landmarks);
    issues.push(...nestingIssues);

    // Redundant landmarks
    const redundantIssues = this.detectRedundantLandmarks(landmarks);
    issues.push(...redundantIssues);

    return issues;
  }

  /**
   * Find landmarks by role
   */
  private findLandmarksByRole(landmarks: LandmarkInfo[], role: string): LandmarkInfo[] {
    const results: LandmarkInfo[] = [];

    const traverse = (landmark: LandmarkInfo) => {
      if (landmark.role === role) {
        results.push(landmark);
      }
      landmark.children.forEach(traverse);
    };

    landmarks.forEach(traverse);
    return results;
  }

  /**
   * Flatten landmark tree
   */
  private flattenLandmarks(landmarks: LandmarkInfo[]): LandmarkInfo[] {
    const results: LandmarkInfo[] = [];

    const traverse = (landmark: LandmarkInfo) => {
      results.push(landmark);
      landmark.children.forEach(traverse);
    };

    landmarks.forEach(traverse);
    return results;
  }

  /**
   * Detect improper nesting
   */
  private detectImproperNesting(landmarks: LandmarkInfo[]): LandmarkIssue[] {
    const issues: LandmarkIssue[] = [];

    const checkNesting = (landmark: LandmarkInfo, parentRole?: string) => {
      // Banner shouldn't be nested in other landmarks
      if (landmark.role === 'banner' && parentRole) {
        issues.push({
          type: 'nested-incorrectly',
          severity: 'moderate',
          node: landmark.node,
          description: 'Banner landmark should not be nested within other landmarks',
          remediation: 'Move banner to the top level of the page structure',
        });
      }

      // Contentinfo shouldn't be nested in other landmarks
      if (landmark.role === 'contentinfo' && parentRole) {
        issues.push({
          type: 'nested-incorrectly',
          severity: 'moderate',
          node: landmark.node,
          description: 'Contentinfo landmark should not be nested within other landmarks',
          remediation: 'Move contentinfo to the top level of the page structure',
        });
      }

      // Main shouldn't be nested in other landmarks (except region in rare cases)
      if (landmark.role === 'main' && parentRole && parentRole !== 'region') {
        issues.push({
          type: 'nested-incorrectly',
          severity: 'moderate',
          node: landmark.node,
          description: 'Main landmark should not be nested within other landmarks',
          remediation: 'Move main to the top level of the page structure',
        });
      }

      landmark.children.forEach(child => checkNesting(child, landmark.role));
    };

    landmarks.forEach(landmark => checkNesting(landmark));
    return issues;
  }

  /**
   * Detect redundant landmarks
   */
  private detectRedundantLandmarks(landmarks: LandmarkInfo[]): LandmarkIssue[] {
    const issues: LandmarkIssue[] = [];

    // Region landmarks without labels are redundant
    const regionLandmarks = this.findLandmarksByRole(landmarks, 'region');
    regionLandmarks.forEach(region => {
      if (!region.label) {
        issues.push({
          type: 'redundant-landmark',
          severity: 'minor',
          node: region.node,
          description: 'Region landmark without a label provides no additional value',
          remediation: 'Add aria-label to describe the region, or remove role="region" if not needed',
        });
      }
    });

    // Form landmarks without labels are redundant if only one form
    const formLandmarks = this.findLandmarksByRole(landmarks, 'form');
    if (formLandmarks.length === 1 && !formLandmarks[0].label) {
      issues.push({
        type: 'redundant-landmark',
        severity: 'minor',
        node: formLandmarks[0].node,
        description: 'Single form landmark without a label provides limited value',
        remediation: 'Add aria-label to describe the form purpose',
      });
    }

    return issues;
  }

  /**
   * Calculate score
   */
  private calculateScore(landmarks: LandmarkInfo[], issues: LandmarkIssue[]): number {
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

    // Bonus for having good landmark structure
    const allLandmarks = this.flattenLandmarks(landmarks);
    const hasMain = this.findLandmarksByRole(landmarks, 'main').length === 1;
    const hasNav = this.findLandmarksByRole(landmarks, 'navigation').length > 0;

    if (hasMain) score += 5;
    if (hasNav) score += 5;
    if (allLandmarks.length >= 3) score += 5;

    return Math.max(0, Math.min(100, Math.round(score)));
  }

  /**
   * Get landmark navigation list
   */
  public getLandmarkNavigationList(landmarks: LandmarkInfo[]): string[] {
    const list: string[] = [];

    const traverse = (landmark: LandmarkInfo, indent: number = 0) => {
      const prefix = '  '.repeat(indent);
      const label = landmark.label ? `"${landmark.label}"` : '(unlabeled)';
      list.push(`${prefix}${landmark.role} ${label}`);
      landmark.children.forEach(child => traverse(child, indent + 1));
    };

    landmarks.forEach(landmark => traverse(landmark));
    return list;
  }
}
