import React, { useState, useRef } from 'react';
import { ReportData } from '../../types';
import { ReportSection } from './ReportSection';
import { TableOfContents } from './TableOfContents';

interface ReportViewerProps {
  reportData: ReportData;
  onExport?: () => void;
  className?: string;
}

export const ReportViewer: React.FC<ReportViewerProps> = ({
  reportData,
  onExport,
  className = '',
}) => {
  const [showTOC, setShowTOC] = useState(true);
  const [activeSection, setActiveSection] = useState<string | null>(null);
  const mainContentRef = useRef<HTMLDivElement>(null);

  const scrollToSection = (sectionId: string) => {
    const element = document.getElementById(`section-${sectionId}`);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth', block: 'start' });
      setActiveSection(sectionId);
    }
  };

  const enabledSections = reportData.config.sections.filter((s) => s.enabled);

  return (
    <div
      className={`report-viewer ${className}`}
      style={styles.container}
      role="main"
      aria-label="Report viewer"
    >
      {/* Header */}
      <header style={styles.header}>
        <div style={styles.headerContent}>
          {reportData.config.branding.logo && (
            <img
              src={reportData.config.branding.logo}
              alt={`${reportData.config.branding.companyName} logo`}
              style={styles.logo}
            />
          )}
          <div style={styles.headerText}>
            <h1 style={styles.title}>{reportData.config.title}</h1>
            {reportData.config.subtitle && (
              <p style={styles.subtitle}>{reportData.config.subtitle}</p>
            )}
          </div>
        </div>
        <div style={styles.headerMeta}>
          <span>
            Generated: {reportData.generatedAt.toLocaleDateString('en-US', {
              year: 'numeric',
              month: 'long',
              day: 'numeric',
            })}
          </span>
          <span>By: {reportData.generatedBy}</span>
        </div>
        <div style={styles.headerActions}>
          <button
            onClick={() => setShowTOC(!showTOC)}
            style={styles.toggleButton}
            aria-expanded={showTOC}
            aria-label={showTOC ? 'Hide table of contents' : 'Show table of contents'}
          >
            {showTOC ? 'Hide' : 'Show'} Table of Contents
          </button>
          {onExport && (
            <button
              onClick={onExport}
              style={{ ...styles.toggleButton, ...styles.exportButton }}
              aria-label="Export report"
            >
              Export Report
            </button>
          )}
        </div>
      </header>

      {/* Main Content Area */}
      <div style={styles.mainLayout}>
        {/* Table of Contents Sidebar */}
        {showTOC && (
          <aside
            style={styles.sidebar}
            role="navigation"
            aria-label="Table of contents"
          >
            <TableOfContents
              sections={enabledSections}
              activeSection={activeSection}
              onSectionClick={scrollToSection}
            />
          </aside>
        )}

        {/* Report Content */}
        <div
          ref={mainContentRef}
          style={{
            ...styles.content,
            ...(showTOC ? styles.contentWithSidebar : {}),
          }}
          role="article"
        >
          {/* Executive Summary (if enabled) */}
          {enabledSections.find((s) => s.type === 'summary') && (
            <section style={styles.executiveSummary}>
              <h2 style={styles.summaryTitle}>Executive Summary</h2>
              <div style={styles.summaryGrid}>
                <div style={styles.summaryCard}>
                  <div style={styles.summaryNumber}>
                    {reportData.metrics.complianceScore}%
                  </div>
                  <div style={styles.summaryLabel}>Compliance Score</div>
                </div>
                <div style={styles.summaryCard}>
                  <div style={styles.summaryNumber}>
                    {reportData.metrics.totalIssues}
                  </div>
                  <div style={styles.summaryLabel}>Total Issues</div>
                </div>
                <div style={{ ...styles.summaryCard, ...styles.criticalCard }}>
                  <div style={styles.summaryNumber}>
                    {reportData.metrics.criticalIssues}
                  </div>
                  <div style={styles.summaryLabel}>Critical Issues</div>
                </div>
                <div style={styles.summaryCard}>
                  <div style={styles.summaryNumber}>
                    {reportData.metrics.wcagAACompliance}%
                  </div>
                  <div style={styles.summaryLabel}>WCAG AA Compliance</div>
                </div>
              </div>
            </section>
          )}

          {/* Report Sections */}
          {enabledSections.map((section) => (
            <div
              key={section.id}
              id={`section-${section.id}`}
              style={styles.sectionContainer}
            >
              <ReportSection
                section={section}
                data={reportData}
                branding={reportData.config.branding}
              />
            </div>
          ))}

          {/* Footer */}
          <footer style={styles.footer}>
            {reportData.config.branding.footerText && (
              <p>{reportData.config.branding.footerText}</p>
            )}
            <p style={styles.footerMeta}>
              Report generated by {reportData.config.branding.companyName} |{' '}
              Version {reportData.config.version}
            </p>
          </footer>
        </div>
      </div>

      {/* Watermark */}
      {reportData.config.branding.watermark && (
        <div style={styles.watermark} aria-hidden="true">
          {reportData.config.branding.watermark}
        </div>
      )}
    </div>
  );
};

const styles = {
  container: {
    position: 'relative' as const,
    backgroundColor: '#f5f5f5',
    minHeight: '100vh',
    fontFamily: 'Arial, sans-serif',
  },
  header: {
    backgroundColor: '#fff',
    padding: '2rem',
    borderBottom: '3px solid #0066cc',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
  },
  headerContent: {
    display: 'flex',
    alignItems: 'center',
    gap: '1.5rem',
    marginBottom: '1rem',
  },
  logo: {
    maxHeight: '60px',
    maxWidth: '200px',
    objectFit: 'contain' as const,
  },
  headerText: {
    flex: 1,
  },
  title: {
    margin: 0,
    fontSize: '2rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  subtitle: {
    margin: '0.5rem 0 0 0',
    fontSize: '1.25rem',
    color: '#666',
  },
  headerMeta: {
    display: 'flex',
    gap: '2rem',
    fontSize: '0.875rem',
    color: '#666',
    marginBottom: '1rem',
  },
  headerActions: {
    display: 'flex',
    gap: '1rem',
  },
  toggleButton: {
    padding: '0.5rem 1rem',
    fontSize: '0.875rem',
    border: '1px solid #ccc',
    borderRadius: '4px',
    backgroundColor: '#fff',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
  },
  exportButton: {
    backgroundColor: '#0066cc',
    color: '#fff',
    borderColor: '#0066cc',
  },
  mainLayout: {
    display: 'flex',
    position: 'relative' as const,
  },
  sidebar: {
    width: '280px',
    backgroundColor: '#fff',
    borderRight: '1px solid #e0e0e0',
    position: 'sticky' as const,
    top: 0,
    height: '100vh',
    overflowY: 'auto' as const,
    flexShrink: 0,
  },
  content: {
    flex: 1,
    padding: '2rem',
    maxWidth: '100%',
  },
  contentWithSidebar: {
    maxWidth: 'calc(100% - 280px)',
  },
  executiveSummary: {
    backgroundColor: '#fff',
    padding: '2rem',
    borderRadius: '8px',
    marginBottom: '2rem',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
  },
  summaryTitle: {
    margin: '0 0 1.5rem 0',
    fontSize: '1.5rem',
    fontWeight: 'bold' as const,
    color: '#333',
    borderBottom: '2px solid #0066cc',
    paddingBottom: '0.5rem',
  },
  summaryGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
    gap: '1.5rem',
  },
  summaryCard: {
    padding: '1.5rem',
    backgroundColor: '#f9f9f9',
    borderRadius: '8px',
    border: '2px solid #e0e0e0',
    textAlign: 'center' as const,
  },
  criticalCard: {
    backgroundColor: '#fff5f5',
    borderColor: '#ff6b6b',
  },
  summaryNumber: {
    fontSize: '2.5rem',
    fontWeight: 'bold' as const,
    color: '#0066cc',
    marginBottom: '0.5rem',
  },
  summaryLabel: {
    fontSize: '1rem',
    color: '#666',
    fontWeight: 'bold' as const,
  },
  sectionContainer: {
    marginBottom: '2rem',
  },
  footer: {
    marginTop: '3rem',
    padding: '2rem',
    backgroundColor: '#fff',
    borderTop: '1px solid #e0e0e0',
    borderRadius: '8px',
    textAlign: 'center' as const,
    color: '#666',
  },
  footerMeta: {
    fontSize: '0.875rem',
    marginTop: '0.5rem',
  },
  watermark: {
    position: 'fixed' as const,
    top: '50%',
    left: '50%',
    transform: 'translate(-50%, -50%) rotate(-45deg)',
    fontSize: '6rem',
    fontWeight: 'bold' as const,
    color: 'rgba(0, 0, 0, 0.05)',
    pointerEvents: 'none' as const,
    userSelect: 'none' as const,
    zIndex: 1,
  },
};
