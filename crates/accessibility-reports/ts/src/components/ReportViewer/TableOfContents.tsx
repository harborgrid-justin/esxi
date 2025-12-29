import React from 'react';
import { ReportSection } from '../../types';

interface TableOfContentsProps {
  sections: ReportSection[];
  activeSection: string | null;
  onSectionClick: (sectionId: string) => void;
  className?: string;
}

export const TableOfContents: React.FC<TableOfContentsProps> = ({
  sections,
  activeSection,
  onSectionClick,
  className = '',
}) => {
  const getSectionIcon = (type: ReportSection['type']): string => {
    const icons = {
      summary: '📊',
      metrics: '📈',
      issues: '⚠️',
      trends: '📉',
      recommendations: '💡',
      technical: '🔧',
      custom: '📄',
    };
    return icons[type] || '📄';
  };

  return (
    <nav
      className={`table-of-contents ${className}`}
      style={styles.container}
      aria-label="Table of contents navigation"
    >
      <h2 style={styles.title}>Contents</h2>
      <ol style={styles.list} role="list">
        {sections.map((section, index) => (
          <li key={section.id} style={styles.listItem} role="listitem">
            <button
              onClick={() => onSectionClick(section.id)}
              style={{
                ...styles.sectionButton,
                ...(activeSection === section.id ? styles.sectionButtonActive : {}),
              }}
              aria-current={activeSection === section.id ? 'location' : undefined}
              aria-label={`Go to ${section.title}`}
            >
              <span style={styles.sectionNumber}>{index + 1}.</span>
              <span style={styles.sectionIcon} aria-hidden="true">
                {getSectionIcon(section.type)}
              </span>
              <span style={styles.sectionText}>{section.title}</span>
            </button>

            {/* Subsections */}
            {section.subsections && section.subsections.length > 0 && (
              <ol style={styles.subsectionList} role="list">
                {section.subsections.map((subsection, subIndex) => (
                  <li key={subsection.id} style={styles.subsectionItem} role="listitem">
                    <button
                      onClick={() => onSectionClick(subsection.id)}
                      style={{
                        ...styles.subsectionButton,
                        ...(activeSection === subsection.id
                          ? styles.subsectionButtonActive
                          : {}),
                      }}
                      aria-current={activeSection === subsection.id ? 'location' : undefined}
                      aria-label={`Go to ${subsection.title}`}
                    >
                      <span style={styles.subsectionNumber}>
                        {index + 1}.{subIndex + 1}
                      </span>
                      <span style={styles.subsectionText}>{subsection.title}</span>
                    </button>
                  </li>
                ))}
              </ol>
            )}
          </li>
        ))}
      </ol>

      {/* Quick Stats */}
      <div style={styles.stats}>
        <h3 style={styles.statsTitle}>Quick Stats</h3>
        <ul style={styles.statsList}>
          <li style={styles.statsItem}>
            <strong>{sections.length}</strong> sections
          </li>
          <li style={styles.statsItem}>
            <strong>
              {sections.reduce(
                (acc, s) => acc + (s.subsections?.length || 0),
                0
              )}
            </strong>{' '}
            subsections
          </li>
        </ul>
      </div>

      {/* Accessibility Note */}
      <div style={styles.accessibilityNote}>
        <p style={styles.noteText}>
          Use keyboard navigation: Tab to navigate, Enter to select
        </p>
      </div>
    </nav>
  );
};

const styles = {
  container: {
    padding: '1.5rem',
    backgroundColor: '#fff',
  },
  title: {
    margin: '0 0 1rem 0',
    fontSize: '1.25rem',
    fontWeight: 'bold' as const,
    color: '#333',
    borderBottom: '2px solid #0066cc',
    paddingBottom: '0.5rem',
  },
  list: {
    listStyle: 'none',
    padding: 0,
    margin: 0,
  },
  listItem: {
    marginBottom: '0.5rem',
  },
  sectionButton: {
    width: '100%',
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    padding: '0.75rem',
    border: 'none',
    backgroundColor: 'transparent',
    textAlign: 'left' as const,
    cursor: 'pointer',
    borderRadius: '4px',
    transition: 'all 0.2s ease',
    fontSize: '1rem',
    color: '#333',
  },
  sectionButtonActive: {
    backgroundColor: '#e3f2fd',
    borderLeft: '4px solid #0066cc',
    fontWeight: 'bold' as const,
  },
  sectionNumber: {
    fontWeight: 'bold' as const,
    color: '#0066cc',
    minWidth: '30px',
  },
  sectionIcon: {
    fontSize: '1.25rem',
  },
  sectionText: {
    flex: 1,
  },
  subsectionList: {
    listStyle: 'none',
    padding: '0 0 0 2rem',
    margin: '0.25rem 0 0.5rem 0',
  },
  subsectionItem: {
    marginBottom: '0.25rem',
  },
  subsectionButton: {
    width: '100%',
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    padding: '0.5rem',
    border: 'none',
    backgroundColor: 'transparent',
    textAlign: 'left' as const,
    cursor: 'pointer',
    borderRadius: '4px',
    transition: 'all 0.2s ease',
    fontSize: '0.875rem',
    color: '#666',
  },
  subsectionButtonActive: {
    backgroundColor: '#f0f7ff',
    color: '#0066cc',
    fontWeight: 'bold' as const,
  },
  subsectionNumber: {
    fontWeight: 'bold' as const,
    minWidth: '50px',
  },
  subsectionText: {
    flex: 1,
  },
  stats: {
    marginTop: '2rem',
    padding: '1rem',
    backgroundColor: '#f9f9f9',
    borderRadius: '6px',
    border: '1px solid #e0e0e0',
  },
  statsTitle: {
    margin: '0 0 0.75rem 0',
    fontSize: '1rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  statsList: {
    listStyle: 'none',
    padding: 0,
    margin: 0,
  },
  statsItem: {
    padding: '0.25rem 0',
    fontSize: '0.875rem',
    color: '#666',
  },
  accessibilityNote: {
    marginTop: '1.5rem',
    padding: '0.75rem',
    backgroundColor: '#e3f2fd',
    borderRadius: '4px',
    borderLeft: '4px solid #0066cc',
  },
  noteText: {
    margin: 0,
    fontSize: '0.75rem',
    color: '#0066cc',
    fontStyle: 'italic' as const,
  },
};
