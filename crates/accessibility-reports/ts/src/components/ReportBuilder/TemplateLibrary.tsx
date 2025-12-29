import React, { useState } from 'react';
import { ReportTemplate } from '../../types';
import {
  ExecutiveSummaryTemplate,
  TechnicalReportTemplate,
  ComplianceAuditTemplate,
  RemediationGuideTemplate,
} from '../../templates';

interface TemplateLibraryProps {
  onSelectTemplate: (template: ReportTemplate) => void;
  className?: string;
}

export const TemplateLibrary: React.FC<TemplateLibraryProps> = ({
  onSelectTemplate,
  className = '',
}) => {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [filterType, setFilterType] = useState<string>('all');

  const templates: ReportTemplate[] = [
    ExecutiveSummaryTemplate,
    TechnicalReportTemplate,
    ComplianceAuditTemplate,
    RemediationGuideTemplate,
  ];

  const filteredTemplates =
    filterType === 'all'
      ? templates
      : templates.filter((t) => t.type === filterType);

  const handleSelect = (template: ReportTemplate) => {
    setSelectedId(template.id);
    onSelectTemplate(template);
  };

  const getTemplateIcon = (type: string): string => {
    const icons: Record<string, string> = {
      executive: '📊',
      technical: '🔧',
      compliance: '✓',
      remediation: '🔨',
    };
    return icons[type] || '📄';
  };

  return (
    <div className={`template-library ${className}`} role="region" aria-label="Report templates">
      {/* Filter Controls */}
      <div style={styles.filterSection}>
        <label htmlFor="template-filter" style={styles.filterLabel}>
          Filter by type:
        </label>
        <select
          id="template-filter"
          value={filterType}
          onChange={(e) => setFilterType(e.target.value)}
          style={styles.filterSelect}
          aria-label="Filter templates by type"
        >
          <option value="all">All Templates</option>
          <option value="executive">Executive Summary</option>
          <option value="technical">Technical Report</option>
          <option value="compliance">Compliance Audit</option>
          <option value="remediation">Remediation Guide</option>
        </select>
      </div>

      {/* Template Grid */}
      <div style={styles.templateGrid} role="list">
        {filteredTemplates.map((template) => (
          <article
            key={template.id}
            style={{
              ...styles.templateCard,
              ...(selectedId === template.id ? styles.templateCardSelected : {}),
            }}
            role="listitem"
            aria-label={`${template.name} template`}
          >
            <div style={styles.cardHeader}>
              <span style={styles.templateIcon} aria-hidden="true">
                {getTemplateIcon(template.type)}
              </span>
              <h3 style={styles.templateName}>{template.name}</h3>
            </div>

            <p style={styles.templateDescription}>{template.description}</p>

            <div style={styles.templateMeta}>
              <span style={styles.badge} aria-label={`Template type: ${template.type}`}>
                {template.type}
              </span>
              <span style={styles.sectionCount} aria-label={`${template.sections.length} sections`}>
                {template.sections.length} sections
              </span>
            </div>

            {/* Section Preview */}
            <details style={styles.sectionPreview}>
              <summary style={styles.sectionPreviewSummary}>
                View sections
              </summary>
              <ul style={styles.sectionList}>
                {template.sections.slice(0, 5).map((section) => (
                  <li key={section.id} style={styles.sectionListItem}>
                    {section.title}
                  </li>
                ))}
                {template.sections.length > 5 && (
                  <li style={styles.sectionListMore}>
                    +{template.sections.length - 5} more sections
                  </li>
                )}
              </ul>
            </details>

            <button
              onClick={() => handleSelect(template)}
              style={{
                ...styles.selectButton,
                ...(selectedId === template.id ? styles.selectButtonSelected : {}),
              }}
              aria-label={`Select ${template.name} template`}
              aria-pressed={selectedId === template.id}
            >
              {selectedId === template.id ? 'Selected ✓' : 'Select Template'}
            </button>
          </article>
        ))}
      </div>

      {filteredTemplates.length === 0 && (
        <div style={styles.emptyState} role="status">
          <p>No templates found matching your filter.</p>
        </div>
      )}
    </div>
  );
};

const styles = {
  filterSection: {
    marginBottom: '1.5rem',
    display: 'flex',
    alignItems: 'center',
    gap: '1rem',
  },
  filterLabel: {
    fontWeight: 'bold' as const,
    color: '#333',
  },
  filterSelect: {
    padding: '0.5rem',
    fontSize: '1rem',
    border: '1px solid #ccc',
    borderRadius: '4px',
    minWidth: '200px',
  },
  templateGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))',
    gap: '1.5rem',
  },
  templateCard: {
    padding: '1.5rem',
    border: '2px solid #e0e0e0',
    borderRadius: '8px',
    backgroundColor: '#fff',
    transition: 'all 0.3s ease',
    cursor: 'pointer',
    display: 'flex',
    flexDirection: 'column' as const,
  },
  templateCardSelected: {
    borderColor: '#0066cc',
    boxShadow: '0 4px 12px rgba(0, 102, 204, 0.2)',
    transform: 'translateY(-2px)',
  },
  cardHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.75rem',
    marginBottom: '1rem',
  },
  templateIcon: {
    fontSize: '2rem',
  },
  templateName: {
    margin: 0,
    fontSize: '1.25rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  templateDescription: {
    color: '#666',
    lineHeight: '1.6',
    marginBottom: '1rem',
    flex: 1,
  },
  templateMeta: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '1rem',
    paddingBottom: '1rem',
    borderBottom: '1px solid #e0e0e0',
  },
  badge: {
    padding: '0.25rem 0.75rem',
    backgroundColor: '#f0f7ff',
    color: '#0066cc',
    borderRadius: '12px',
    fontSize: '0.875rem',
    fontWeight: 'bold' as const,
    textTransform: 'capitalize' as const,
  },
  sectionCount: {
    fontSize: '0.875rem',
    color: '#666',
  },
  sectionPreview: {
    marginBottom: '1rem',
  },
  sectionPreviewSummary: {
    cursor: 'pointer',
    fontWeight: 'bold' as const,
    color: '#0066cc',
    fontSize: '0.875rem',
    marginBottom: '0.5rem',
  },
  sectionList: {
    listStyle: 'none',
    padding: '0.5rem 0',
    margin: 0,
  },
  sectionListItem: {
    padding: '0.25rem 0',
    fontSize: '0.875rem',
    color: '#666',
    paddingLeft: '1rem',
    position: 'relative' as const,
    '::before': {
      content: '"•"',
      position: 'absolute' as const,
      left: 0,
    },
  },
  sectionListMore: {
    padding: '0.25rem 0',
    fontSize: '0.875rem',
    color: '#999',
    fontStyle: 'italic' as const,
  },
  selectButton: {
    padding: '0.75rem',
    fontSize: '1rem',
    fontWeight: 'bold' as const,
    border: '2px solid #0066cc',
    borderRadius: '4px',
    backgroundColor: '#fff',
    color: '#0066cc',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
  },
  selectButtonSelected: {
    backgroundColor: '#0066cc',
    color: '#fff',
  },
  emptyState: {
    padding: '3rem',
    textAlign: 'center' as const,
    color: '#999',
  },
};
