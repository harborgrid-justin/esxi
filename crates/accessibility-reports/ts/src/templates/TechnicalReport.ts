import { ReportTemplate } from '../types';

/**
 * Technical Report Template
 * Detailed technical analysis for developers and QA teams
 */
export const TechnicalReportTemplate: ReportTemplate = {
  id: 'technical-report',
  name: 'Technical Accessibility Report',
  description:
    'Comprehensive technical report with detailed issue analysis, code examples, and remediation steps. Ideal for development teams.',
  type: 'technical',
  sections: [
    {
      id: 'technical-summary',
      title: 'Technical Summary',
      type: 'summary',
      order: 0,
      enabled: true,
    },
    {
      id: 'testing-methodology',
      title: 'Testing Methodology',
      type: 'custom',
      order: 1,
      enabled: true,
      subsections: [
        {
          id: 'automated-testing',
          title: 'Automated Testing Tools',
          type: 'technical',
          order: 0,
          enabled: true,
        },
        {
          id: 'manual-testing',
          title: 'Manual Testing Procedures',
          type: 'technical',
          order: 1,
          enabled: true,
        },
        {
          id: 'assistive-technology',
          title: 'Assistive Technology Testing',
          type: 'technical',
          order: 2,
          enabled: true,
        },
      ],
    },
    {
      id: 'detailed-metrics',
      title: 'Detailed Metrics & Statistics',
      type: 'metrics',
      order: 2,
      enabled: true,
      subsections: [
        {
          id: 'wcag-breakdown',
          title: 'WCAG Success Criteria Breakdown',
          type: 'metrics',
          order: 0,
          enabled: true,
        },
        {
          id: 'component-analysis',
          title: 'Component-Level Analysis',
          type: 'metrics',
          order: 1,
          enabled: true,
        },
        {
          id: 'page-analysis',
          title: 'Page-by-Page Analysis',
          type: 'metrics',
          order: 2,
          enabled: true,
        },
      ],
    },
    {
      id: 'all-issues',
      title: 'Complete Issue Inventory',
      type: 'issues',
      order: 3,
      enabled: true,
      subsections: [
        {
          id: 'critical-issues-detail',
          title: 'Critical Issues (Detailed)',
          type: 'issues',
          order: 0,
          enabled: true,
        },
        {
          id: 'serious-issues-detail',
          title: 'Serious Issues (Detailed)',
          type: 'issues',
          order: 1,
          enabled: true,
        },
        {
          id: 'moderate-issues-detail',
          title: 'Moderate Issues (Detailed)',
          type: 'issues',
          order: 2,
          enabled: true,
        },
        {
          id: 'minor-issues-detail',
          title: 'Minor Issues (Detailed)',
          type: 'issues',
          order: 3,
          enabled: true,
        },
      ],
    },
    {
      id: 'code-examples',
      title: 'Code Examples & Solutions',
      type: 'technical',
      order: 4,
      enabled: true,
      subsections: [
        {
          id: 'html-examples',
          title: 'HTML/Semantic Markup',
          type: 'technical',
          order: 0,
          enabled: true,
        },
        {
          id: 'aria-examples',
          title: 'ARIA Implementation',
          type: 'technical',
          order: 1,
          enabled: true,
        },
        {
          id: 'javascript-examples',
          title: 'JavaScript/Keyboard Navigation',
          type: 'technical',
          order: 2,
          enabled: true,
        },
        {
          id: 'css-examples',
          title: 'CSS/Styling Considerations',
          type: 'technical',
          order: 3,
          enabled: true,
        },
      ],
    },
    {
      id: 'remediation-details',
      title: 'Detailed Remediation Guide',
      type: 'recommendations',
      order: 5,
      enabled: true,
    },
    {
      id: 'testing-checklist',
      title: 'Testing Checklist',
      type: 'technical',
      order: 6,
      enabled: true,
    },
    {
      id: 'technical-appendix',
      title: 'Technical Appendix',
      type: 'technical',
      order: 7,
      enabled: true,
      subsections: [
        {
          id: 'wcag-reference',
          title: 'WCAG 2.1 Reference',
          type: 'technical',
          order: 0,
          enabled: true,
        },
        {
          id: 'aria-reference',
          title: 'ARIA Specification Reference',
          type: 'technical',
          order: 1,
          enabled: true,
        },
        {
          id: 'tool-configuration',
          title: 'Testing Tool Configuration',
          type: 'technical',
          order: 2,
          enabled: true,
        },
      ],
    },
  ],
  defaultBranding: {
    companyName: 'Development Team',
    primaryColor: '#1e88e5',
    secondaryColor: '#424242',
    accentColor: '#ffa726',
    fontFamily: 'Courier New, monospace',
    includePageNumbers: true,
    includeDateGenerated: true,
  },
};
