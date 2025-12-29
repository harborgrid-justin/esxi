/**
 * PDF Accessibility Remediator
 * Provides remediation suggestions and automated fixes for PDF accessibility issues
 */

import type {
  AccessibilityIssue,
  RemediationSuggestion,
  PDFUARequirement
} from '../types/index.js';

export class PDFRemediator {
  /**
   * Generate remediation plan for PDF issues
   */
  generateRemediationPlan(issues: AccessibilityIssue[]): RemediationPlan {
    const plan: RemediationPlan = {
      critical: [],
      high: [],
      medium: [],
      low: [],
      estimatedHours: 0,
      toolsRequired: new Set<string>(),
      steps: []
    };

    // Sort issues by severity and priority
    const sortedIssues = this.sortIssuesByPriority(issues);

    sortedIssues.forEach(issue => {
      const priority = this.determinePriority(issue);
      const effort = this.estimateEffort(issue);

      const remediation = issue.remediation || this.generateRemediationSuggestion(issue);

      const task: RemediationTask = {
        issue,
        remediation,
        priority,
        estimatedHours: effort
      };

      // Add to appropriate priority bucket
      switch (priority) {
        case 'critical':
          plan.critical.push(task);
          break;
        case 'high':
          plan.high.push(task);
          break;
        case 'medium':
          plan.medium.push(task);
          break;
        case 'low':
          plan.low.push(task);
          break;
      }

      plan.estimatedHours += effort;

      if (remediation.toolsRequired) {
        remediation.toolsRequired.forEach(tool => plan.toolsRequired.add(tool));
      }
    });

    // Generate step-by-step plan
    plan.steps = this.generateStepByStepPlan(plan);

    return plan;
  }

  /**
   * Generate remediation suggestion for an issue
   */
  private generateRemediationSuggestion(issue: AccessibilityIssue): RemediationSuggestion {
    const suggestions: Record<string, RemediationSuggestion> = {
      pdf_not_tagged: {
        action: 'Add tags to PDF',
        description: 'Create tagged PDF structure using Adobe Acrobat Pro or similar tool.',
        steps: [
          'Open PDF in Adobe Acrobat Pro',
          'Select Accessibility > Add Tags to Document',
          'Review auto-generated tags in Tags panel',
          'Correct any tagging errors',
          'Run Accessibility Checker to verify'
        ],
        automated: false,
        estimatedEffort: 'high',
        priority: 1,
        toolsRequired: ['Adobe Acrobat Pro', 'PAC 2021']
      },
      missing_title: {
        action: 'Add document title',
        description: 'Set title in PDF metadata and enable DisplayDocTitle.',
        steps: [
          'Open File > Properties',
          'Enter descriptive title in Description tab',
          'Go to Initial View tab',
          'Set "Show" to "Document Title"',
          'Save document'
        ],
        automated: false,
        estimatedEffort: 'low',
        priority: 2,
        toolsRequired: ['Adobe Acrobat Pro']
      },
      missing_language: {
        action: 'Set document language',
        description: 'Specify the primary language of the PDF.',
        steps: [
          'Open File > Properties > Advanced',
          'Set Language field (e.g., "en-US")',
          'For multi-language content, set language on specific tags',
          'Save document'
        ],
        automated: false,
        estimatedEffort: 'low',
        priority: 2,
        toolsRequired: ['Adobe Acrobat Pro']
      },
      image_missing_alt: {
        action: 'Add alternative text to images',
        description: 'Provide descriptive alt text for all images.',
        steps: [
          'Open Tags panel',
          'Find Figure tags',
          'Right-click and select Properties',
          'Enter descriptive alternative text',
          'For decorative images, mark as artifact'
        ],
        automated: false,
        estimatedEffort: 'medium',
        priority: 1,
        toolsRequired: ['Adobe Acrobat Pro']
      },
      form_missing_label: {
        action: 'Add labels to form fields',
        description: 'Ensure all form fields have accessible labels.',
        steps: [
          'Select Prepare Form tool',
          'Click on form field',
          'In Properties, set Tooltip text',
          'Or use proper form field labels in Tags panel',
          'Verify with screen reader'
        ],
        automated: false,
        estimatedEffort: 'low',
        priority: 2,
        toolsRequired: ['Adobe Acrobat Pro']
      }
    };

    return suggestions[issue.type] || {
      action: 'Review and fix issue',
      description: issue.description,
      steps: ['Identify the issue', 'Apply appropriate fix', 'Test with assistive technology'],
      automated: false,
      estimatedEffort: 'medium',
      priority: 3
    };
  }

  /**
   * Sort issues by priority
   */
  private sortIssuesByPriority(issues: AccessibilityIssue[]): AccessibilityIssue[] {
    return [...issues].sort((a, b) => {
      // Sort by severity first
      const severityOrder = { critical: 0, error: 1, warning: 2, info: 3 };
      const severityDiff = severityOrder[a.severity] - severityOrder[b.severity];

      if (severityDiff !== 0) return severityDiff;

      // Then by WCAG level
      const wcagOrder = { A: 0, AA: 1, AAA: 2 };
      const aLevel = a.wcagLevel || 'AAA';
      const bLevel = b.wcagLevel || 'AAA';

      return wcagOrder[aLevel] - wcagOrder[bLevel];
    });
  }

  /**
   * Determine remediation priority
   */
  private determinePriority(issue: AccessibilityIssue): 'critical' | 'high' | 'medium' | 'low' {
    if (issue.severity === 'critical') return 'critical';
    if (issue.severity === 'error' && issue.wcagLevel === 'A') return 'high';
    if (issue.severity === 'error') return 'medium';
    return 'low';
  }

  /**
   * Estimate effort in hours
   */
  private estimateEffort(issue: AccessibilityIssue): number {
    const effortMap = {
      low: 0.25,
      medium: 1,
      high: 4
    };

    const effort = issue.remediation?.estimatedEffort || 'medium';
    return effortMap[effort];
  }

  /**
   * Generate step-by-step remediation plan
   */
  private generateStepByStepPlan(plan: RemediationPlan): string[] {
    const steps: string[] = [
      '1. Setup and Preparation',
      '   - Install required tools: ' + Array.from(plan.toolsRequired).join(', '),
      '   - Make a backup copy of the original PDF',
      '   - Create a working copy for remediation',
      '',
      '2. Critical Issues (Must Fix)',
      ...plan.critical.map((task, i) => `   ${i + 1}. ${task.remediation.action}`),
      '',
      '3. High Priority Issues',
      ...plan.high.map((task, i) => `   ${i + 1}. ${task.remediation.action}`),
      '',
      '4. Medium Priority Issues',
      ...plan.medium.map((task, i) => `   ${i + 1}. ${task.remediation.action}`),
      '',
      '5. Low Priority Issues (Optional)',
      ...plan.low.map((task, i) => `   ${i + 1}. ${task.remediation.action}`),
      '',
      '6. Testing and Validation',
      '   - Run Adobe Acrobat Accessibility Checker',
      '   - Test with PAC 2021 for PDF/UA compliance',
      '   - Test with screen reader (NVDA or JAWS)',
      '   - Verify reading order is logical',
      '   - Test keyboard navigation for forms and links',
      '',
      '7. Documentation',
      '   - Document changes made',
      '   - Update accessibility statement if applicable',
      '   - Save final version with descriptive filename'
    ];

    return steps.filter(s => s.trim().length > 0 || s === '');
  }

  /**
   * Generate detailed remediation report
   */
  generateReport(plan: RemediationPlan): string {
    let report = 'PDF ACCESSIBILITY REMEDIATION PLAN\n';
    report += '=====================================\n\n';

    report += `Total Issues: ${plan.critical.length + plan.high.length + plan.medium.length + plan.low.length}\n`;
    report += `Estimated Time: ${plan.estimatedHours.toFixed(1)} hours\n`;
    report += `Tools Required: ${Array.from(plan.toolsRequired).join(', ')}\n\n`;

    report += `Issue Breakdown:\n`;
    report += `  Critical: ${plan.critical.length}\n`;
    report += `  High: ${plan.high.length}\n`;
    report += `  Medium: ${plan.medium.length}\n`;
    report += `  Low: ${plan.low.length}\n\n`;

    report += `REMEDIATION STEPS\n`;
    report += `=================\n\n`;
    report += plan.steps.join('\n');

    report += '\n\nDETAILED ISSUE LIST\n';
    report += '===================\n\n';

    const allTasks = [...plan.critical, ...plan.high, ...plan.medium, ...plan.low];
    allTasks.forEach((task, index) => {
      report += `${index + 1}. ${task.issue.title} [${task.priority.toUpperCase()}]\n`;
      report += `   ${task.issue.description}\n`;
      report += `   Action: ${task.remediation.action}\n`;
      report += `   Estimated time: ${task.estimatedHours} hours\n`;
      if (task.issue.pageNumber) {
        report += `   Page: ${task.issue.pageNumber}\n`;
      }
      report += '\n';
    });

    return report;
  }
}

export interface RemediationPlan {
  critical: RemediationTask[];
  high: RemediationTask[];
  medium: RemediationTask[];
  low: RemediationTask[];
  estimatedHours: number;
  toolsRequired: Set<string>;
  steps: string[];
}

export interface RemediationTask {
  issue: AccessibilityIssue;
  remediation: RemediationSuggestion;
  priority: 'critical' | 'high' | 'medium' | 'low';
  estimatedHours: number;
}
