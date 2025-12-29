/**
 * Office Document Accessibility Remediator
 * Provides remediation suggestions for Word, Excel, PowerPoint documents
 */

import type {
  AccessibilityIssue,
  RemediationSuggestion,
  DocumentType
} from '../types/index.js';

export class OfficeRemediator {
  /**
   * Generate remediation plan for Office documents
   */
  generateRemediationPlan(issues: AccessibilityIssue[], documentType: DocumentType): RemediationPlan {
    const plan: RemediationPlan = {
      critical: [],
      high: [],
      medium: [],
      low: [],
      estimatedHours: 0,
      steps: []
    };

    // Sort and categorize issues
    const sortedIssues = this.sortIssuesByPriority(issues);

    sortedIssues.forEach(issue => {
      const priority = this.determinePriority(issue);
      const effort = this.estimateEffort(issue);

      const remediation = issue.remediation || this.generateRemediationSuggestion(issue, documentType);

      const task: RemediationTask = {
        issue,
        remediation,
        priority,
        estimatedHours: effort
      };

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
    });

    plan.steps = this.generateStepByStepPlan(plan, documentType);

    return plan;
  }

  /**
   * Generate remediation suggestion based on issue and document type
   */
  private generateRemediationSuggestion(
    issue: AccessibilityIssue,
    documentType: DocumentType
  ): RemediationSuggestion {
    const typePrefix = documentType.toLowerCase();

    const suggestions: Record<string, RemediationSuggestion> = {
      [`${typePrefix}_missing_title`]: {
        action: 'Add document title',
        description: 'Set a descriptive title in document properties.',
        steps: [
          'Go to File > Info',
          'Click Properties > Advanced Properties',
          'Enter a descriptive title in the Summary tab',
          'Click OK to save'
        ],
        automated: false,
        estimatedEffort: 'low',
        priority: 2
      },
      [`${typePrefix}_missing_language`]: {
        action: 'Set document language',
        description: 'Specify the language for proper text-to-speech.',
        steps: [
          'Go to Review > Language > Set Proofing Language',
          'Select the appropriate language',
          'Click "Set As Default" if needed',
          'Click OK'
        ],
        automated: false,
        estimatedEffort: 'low',
        priority: 2
      },
      word_no_headings: {
        action: 'Add heading structure',
        description: 'Use built-in Heading styles to structure content.',
        steps: [
          'Identify major sections in document',
          'Select text for main sections',
          'Apply Heading 1 style from Home > Styles',
          'Apply Heading 2-6 for subsections',
          'Maintain logical hierarchy'
        ],
        automated: false,
        estimatedEffort: 'medium',
        priority: 2
      },
      word_image_no_alt: {
        action: 'Add alternative text to images',
        description: 'Provide alt text for all meaningful images.',
        steps: [
          'Right-click image',
          'Select "Edit Alt Text"',
          'Enter descriptive text',
          'For decorative images, check "Mark as decorative"',
          'Repeat for all images'
        ],
        automated: false,
        estimatedEffort: 'medium',
        priority: 1
      },
      word_table_no_headers: {
        action: 'Add table header row',
        description: 'Designate the first row as a header row.',
        steps: [
          'Click anywhere in the table',
          'Go to Table Design tab',
          'Check "Header Row" in Table Style Options',
          'Verify headers are descriptive'
        ],
        automated: false,
        estimatedEffort: 'low',
        priority: 2
      },
      excel_image_no_alt: {
        action: 'Add alt text to charts and images',
        description: 'Describe the meaning and key data of charts.',
        steps: [
          'Right-click chart or image',
          'Select "Edit Alt Text"',
          'Describe key trends and data points',
          'Click outside to save'
        ],
        automated: false,
        estimatedEffort: 'medium',
        priority: 1
      },
      excel_merged_cells: {
        action: 'Remove merged cells',
        description: 'Restructure layout to avoid merged cells.',
        steps: [
          'Select merged cell',
          'Go to Home > Merge & Center > Unmerge Cells',
          'Restructure data using proper tables',
          'Use cell borders for visual organization'
        ],
        automated: false,
        estimatedEffort: 'medium',
        priority: 3
      },
      ppt_slide_no_title: {
        action: 'Add slide titles',
        description: 'Every slide needs a unique, descriptive title.',
        steps: [
          'Click in title placeholder',
          'Enter descriptive title',
          'For slides without visible title, add off-screen title',
          'Ensure each title is unique'
        ],
        automated: false,
        estimatedEffort: 'low',
        priority: 1
      },
      ppt_image_no_alt: {
        action: 'Add alt text to images',
        description: 'Describe what each image conveys.',
        steps: [
          'Right-click image',
          'Select "Edit Alt Text"',
          'Enter description of image content',
          'Mark decorative images as decorative'
        ],
        automated: false,
        estimatedEffort: 'medium',
        priority: 1
      },
      ppt_reading_order: {
        action: 'Fix reading order',
        description: 'Ensure logical reading order for screen readers.',
        steps: [
          'Go to Home > Arrange > Selection Pane',
          'View object order (bottom to top is reading order)',
          'Drag objects to reorder as needed',
          'Test with screen reader'
        ],
        automated: false,
        estimatedEffort: 'low',
        priority: 3
      }
    };

    return suggestions[issue.type] || {
      action: 'Fix accessibility issue',
      description: issue.description,
      steps: ['Review the issue', 'Apply appropriate fix', 'Test with accessibility checker'],
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
      const severityOrder = { critical: 0, error: 1, warning: 2, info: 3 };
      return severityOrder[a.severity] - severityOrder[b.severity];
    });
  }

  /**
   * Determine remediation priority
   */
  private determinePriority(issue: AccessibilityIssue): 'critical' | 'high' | 'medium' | 'low' {
    if (issue.severity === 'critical') return 'critical';
    if (issue.severity === 'error') return 'high';
    if (issue.severity === 'warning') return 'medium';
    return 'low';
  }

  /**
   * Estimate effort in hours
   */
  private estimateEffort(issue: AccessibilityIssue): number {
    const effortMap = { low: 0.25, medium: 0.5, high: 2 };
    const effort = issue.remediation?.estimatedEffort || 'medium';
    return effortMap[effort];
  }

  /**
   * Generate step-by-step plan
   */
  private generateStepByStepPlan(plan: RemediationPlan, documentType: DocumentType): string[] {
    const docTypeName = this.getDocumentTypeName(documentType);

    const steps: string[] = [
      `1. Preparation`,
      `   - Make a backup copy of the original ${docTypeName}`,
      `   - Open document in Microsoft ${docTypeName}`,
      `   - Enable "Check Accessibility" in Review tab`,
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
      '5. Low Priority Issues',
      ...plan.low.map((task, i) => `   ${i + 1}. ${task.remediation.action}`),
      '',
      '6. Final Checks',
      '   - Run built-in Accessibility Checker',
      '   - Review and fix any remaining issues',
      '   - Test with screen reader if possible',
      '   - Save document',
      '',
      '7. Documentation',
      '   - Note any issues that cannot be fixed',
      '   - Update accessibility statement if needed'
    ];

    return steps.filter(s => s.trim().length > 0 || s === '');
  }

  /**
   * Get document type name
   */
  private getDocumentTypeName(documentType: DocumentType): string {
    const names: Record<DocumentType, string> = {
      [DocumentType.WORD]: 'Word',
      [DocumentType.EXCEL]: 'Excel',
      [DocumentType.POWERPOINT]: 'PowerPoint',
      [DocumentType.PDF]: 'PDF',
      [DocumentType.EPUB]: 'EPUB',
      [DocumentType.UNKNOWN]: 'Document'
    };

    return names[documentType];
  }

  /**
   * Generate detailed report
   */
  generateReport(plan: RemediationPlan, documentType: DocumentType): string {
    const docTypeName = this.getDocumentTypeName(documentType);

    let report = `${docTypeName.toUpperCase()} ACCESSIBILITY REMEDIATION PLAN\n`;
    report += '='.repeat(docTypeName.length + 35) + '\n\n';

    const totalIssues = plan.critical.length + plan.high.length + plan.medium.length + plan.low.length;
    report += `Total Issues: ${totalIssues}\n`;
    report += `Estimated Time: ${plan.estimatedHours.toFixed(1)} hours\n\n`;

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
      report += `   Time: ${task.estimatedHours} hours\n\n`;
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
  steps: string[];
}

export interface RemediationTask {
  issue: AccessibilityIssue;
  remediation: RemediationSuggestion;
  priority: 'critical' | 'high' | 'medium' | 'low';
  estimatedHours: number;
}
