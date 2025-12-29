import React, { useState } from 'react';
import { ReportSection } from '../../types';

interface SectionSelectorProps {
  sections: ReportSection[];
  onSectionsChange: (sections: ReportSection[]) => void;
  className?: string;
}

export const SectionSelector: React.FC<SectionSelectorProps> = ({
  sections,
  onSectionsChange,
  className = '',
}) => {
  const [expandedSections, setExpandedSections] = useState<Set<string>>(new Set());

  const toggleSection = (sectionId: string) => {
    const updatedSections = sections.map((section) =>
      section.id === sectionId
        ? { ...section, enabled: !section.enabled }
        : section
    );
    onSectionsChange(updatedSections);
  };

  const reorderSection = (sectionId: string, direction: 'up' | 'down') => {
    const currentIndex = sections.findIndex((s) => s.id === sectionId);
    if (
      (direction === 'up' && currentIndex === 0) ||
      (direction === 'down' && currentIndex === sections.length - 1)
    ) {
      return;
    }

    const newSections = [...sections];
    const targetIndex = direction === 'up' ? currentIndex - 1 : currentIndex + 1;
    [newSections[currentIndex], newSections[targetIndex]] = [
      newSections[targetIndex],
      newSections[currentIndex],
    ];

    // Update order values
    newSections.forEach((section, index) => {
      section.order = index;
    });

    onSectionsChange(newSections);
  };

  const toggleExpanded = (sectionId: string) => {
    const newExpanded = new Set(expandedSections);
    if (newExpanded.has(sectionId)) {
      newExpanded.delete(sectionId);
    } else {
      newExpanded.add(sectionId);
    }
    setExpandedSections(newExpanded);
  };

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
    <div className={`section-selector ${className}`} role="region" aria-label="Report sections">
      <div style={styles.header}>
        <p style={styles.description}>
          Select and reorder the sections to include in your report. Toggle sections on or off,
          and use the arrows to change their order.
        </p>
        <div style={styles.stats}>
          <span>
            <strong>{sections.filter((s) => s.enabled).length}</strong> of{' '}
            <strong>{sections.length}</strong> sections enabled
          </span>
        </div>
      </div>

      <ul style={styles.sectionList} role="list">
        {sections.map((section, index) => (
          <li
            key={section.id}
            style={{
              ...styles.sectionItem,
              ...(section.enabled ? styles.sectionItemEnabled : styles.sectionItemDisabled),
            }}
            aria-label={`${section.title} section`}
          >
            <div style={styles.sectionHeader}>
              <div style={styles.sectionInfo}>
                <span style={styles.sectionIcon} aria-hidden="true">
                  {getSectionIcon(section.type)}
                </span>
                <div style={styles.sectionTitle}>
                  <h3 style={styles.titleText}>{section.title}</h3>
                  <span style={styles.sectionType}>{section.type}</span>
                </div>
              </div>

              <div style={styles.sectionControls}>
                {/* Reorder buttons */}
                <div style={styles.reorderButtons} role="group" aria-label="Reorder section">
                  <button
                    onClick={() => reorderSection(section.id, 'up')}
                    disabled={index === 0}
                    style={{
                      ...styles.controlButton,
                      ...(index === 0 ? styles.controlButtonDisabled : {}),
                    }}
                    aria-label={`Move ${section.title} up`}
                    title="Move up"
                  >
                    ↑
                  </button>
                  <button
                    onClick={() => reorderSection(section.id, 'down')}
                    disabled={index === sections.length - 1}
                    style={{
                      ...styles.controlButton,
                      ...(index === sections.length - 1 ? styles.controlButtonDisabled : {}),
                    }}
                    aria-label={`Move ${section.title} down`}
                    title="Move down"
                  >
                    ↓
                  </button>
                </div>

                {/* Toggle button */}
                <label style={styles.toggleLabel}>
                  <input
                    type="checkbox"
                    checked={section.enabled}
                    onChange={() => toggleSection(section.id)}
                    style={styles.checkbox}
                    aria-label={`Include ${section.title} section`}
                  />
                  <span style={styles.toggleText}>
                    {section.enabled ? 'Enabled' : 'Disabled'}
                  </span>
                </label>

                {/* Expand button for subsections */}
                {section.subsections && section.subsections.length > 0 && (
                  <button
                    onClick={() => toggleExpanded(section.id)}
                    style={styles.expandButton}
                    aria-expanded={expandedSections.has(section.id)}
                    aria-label={`${expandedSections.has(section.id) ? 'Collapse' : 'Expand'} ${section.title} subsections`}
                  >
                    {expandedSections.has(section.id) ? '▼' : '▶'}
                  </button>
                )}
              </div>
            </div>

            {/* Subsections */}
            {section.subsections &&
              section.subsections.length > 0 &&
              expandedSections.has(section.id) && (
                <ul style={styles.subsectionList} role="list" aria-label="Subsections">
                  {section.subsections.map((subsection) => (
                    <li key={subsection.id} style={styles.subsectionItem}>
                      <span style={styles.subsectionIcon} aria-hidden="true">
                        {getSectionIcon(subsection.type)}
                      </span>
                      <span>{subsection.title}</span>
                    </li>
                  ))}
                </ul>
              )}
          </li>
        ))}
      </ul>
    </div>
  );
};

const styles = {
  header: {
    marginBottom: '1.5rem',
  },
  description: {
    color: '#666',
    lineHeight: '1.6',
    marginBottom: '1rem',
  },
  stats: {
    padding: '0.75rem',
    backgroundColor: '#f0f7ff',
    borderRadius: '4px',
    borderLeft: '4px solid #0066cc',
  },
  sectionList: {
    listStyle: 'none',
    padding: 0,
    margin: 0,
  },
  sectionItem: {
    marginBottom: '0.75rem',
    padding: '1rem',
    border: '1px solid #e0e0e0',
    borderRadius: '6px',
    transition: 'all 0.2s ease',
  },
  sectionItemEnabled: {
    backgroundColor: '#fff',
    borderColor: '#0066cc',
  },
  sectionItemDisabled: {
    backgroundColor: '#f5f5f5',
    opacity: 0.7,
  },
  sectionHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  sectionInfo: {
    display: 'flex',
    alignItems: 'center',
    gap: '1rem',
    flex: 1,
  },
  sectionIcon: {
    fontSize: '1.5rem',
  },
  sectionTitle: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '0.25rem',
  },
  titleText: {
    margin: 0,
    fontSize: '1rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  sectionType: {
    fontSize: '0.875rem',
    color: '#666',
    textTransform: 'capitalize' as const,
  },
  sectionControls: {
    display: 'flex',
    alignItems: 'center',
    gap: '1rem',
  },
  reorderButtons: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '0.25rem',
  },
  controlButton: {
    padding: '0.25rem 0.5rem',
    border: '1px solid #ccc',
    borderRadius: '3px',
    backgroundColor: '#fff',
    cursor: 'pointer',
    fontSize: '0.875rem',
    minWidth: '30px',
  },
  controlButtonDisabled: {
    opacity: 0.3,
    cursor: 'not-allowed',
  },
  toggleLabel: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    cursor: 'pointer',
  },
  checkbox: {
    width: '18px',
    height: '18px',
    cursor: 'pointer',
  },
  toggleText: {
    fontSize: '0.875rem',
    fontWeight: 'bold' as const,
  },
  expandButton: {
    padding: '0.5rem',
    border: 'none',
    backgroundColor: 'transparent',
    cursor: 'pointer',
    fontSize: '0.875rem',
  },
  subsectionList: {
    listStyle: 'none',
    padding: '1rem 0 0 3rem',
    margin: '0.75rem 0 0 0',
    borderTop: '1px solid #e0e0e0',
  },
  subsectionItem: {
    padding: '0.5rem',
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    color: '#666',
  },
  subsectionIcon: {
    fontSize: '1rem',
  },
};
