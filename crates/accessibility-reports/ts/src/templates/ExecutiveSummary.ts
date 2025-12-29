import { ReportTemplate } from '../types';

/**
 * Executive Summary Template
 * High-level overview for stakeholders and executives
 */
export const ExecutiveSummaryTemplate: ReportTemplate = {
  id: 'executive-summary',
  name: 'Executive Summary Report',
  description:
    'High-level accessibility overview with key metrics and executive insights. Perfect for stakeholder presentations and board meetings.',
  type: 'executive',
  sections: [
    {
      id: 'cover-page',
      title: 'Cover Page',
      type: 'custom',
      order: 0,
      enabled: true,
    },
    {
      id: 'executive-summary',
      title: 'Executive Summary',
      type: 'summary',
      order: 1,
      enabled: true,
      subsections: [
        {
          id: 'key-findings',
          title: 'Key Findings',
          type: 'summary',
          order: 0,
          enabled: true,
        },
        {
          id: 'business-impact',
          title: 'Business Impact',
          type: 'summary',
          order: 1,
          enabled: true,
        },
      ],
    },
    {
      id: 'compliance-overview',
      title: 'Compliance Overview',
      type: 'metrics',
      order: 2,
      enabled: true,
      subsections: [
        {
          id: 'wcag-compliance',
          title: 'WCAG 2.1 Compliance Status',
          type: 'metrics',
          order: 0,
          enabled: true,
        },
        {
          id: 'compliance-score',
          title: 'Overall Compliance Score',
          type: 'metrics',
          order: 1,
          enabled: true,
        },
      ],
    },
    {
      id: 'critical-issues',
      title: 'Critical Issues Summary',
      type: 'issues',
      order: 3,
      enabled: true,
    },
    {
      id: 'trends-overview',
      title: 'Trends & Progress',
      type: 'trends',
      order: 4,
      enabled: true,
      subsections: [
        {
          id: 'historical-performance',
          title: 'Historical Performance',
          type: 'trends',
          order: 0,
          enabled: true,
        },
        {
          id: 'improvement-trajectory',
          title: 'Improvement Trajectory',
          type: 'trends',
          order: 1,
          enabled: true,
        },
      ],
    },
    {
      id: 'strategic-recommendations',
      title: 'Strategic Recommendations',
      type: 'recommendations',
      order: 5,
      enabled: true,
    },
    {
      id: 'next-steps',
      title: 'Next Steps & Timeline',
      type: 'recommendations',
      order: 6,
      enabled: true,
    },
  ],
  defaultBranding: {
    companyName: 'Your Company',
    primaryColor: '#0066cc',
    secondaryColor: '#666666',
    accentColor: '#ff6b00',
    fontFamily: 'Arial, sans-serif',
    includePageNumbers: true,
    includeDateGenerated: true,
  },
};
