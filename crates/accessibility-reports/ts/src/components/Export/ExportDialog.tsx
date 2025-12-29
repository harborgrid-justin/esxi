import React, { useState } from 'react';
import { ExportOptions, ExportFormat, ReportData } from '../../types';

interface ExportDialogProps {
  reportData: ReportData;
  onExport: (options: ExportOptions) => void;
  onCancel: () => void;
  isOpen: boolean;
  className?: string;
}

export const ExportDialog: React.FC<ExportDialogProps> = ({
  reportData,
  onExport,
  onCancel,
  isOpen,
  className = '',
}) => {
  const [format, setFormat] = useState<ExportFormat>('pdf');
  const [filename, setFilename] = useState(
    `accessibility-report-${new Date().toISOString().split('T')[0]}`
  );
  const [orientation, setOrientation] = useState<'portrait' | 'landscape'>('portrait');
  const [pageSize, setPageSize] = useState<'A4' | 'Letter' | 'Legal'>('A4');
  const [includeCharts, setIncludeCharts] = useState(true);
  const [includeScreenshots, setIncludeScreenshots] = useState(true);
  const [pdfUA, setPdfUA] = useState(true);
  const [compression, setCompression] = useState(false);

  const handleExport = () => {
    const options: ExportOptions = {
      format,
      filename,
      orientation,
      pageSize,
      includeCharts,
      includeScreenshots,
      compression,
      accessibility: {
        pdfUA,
        tagged: true,
        altText: true,
        structure: true,
      },
    };
    onExport(options);
  };

  if (!isOpen) return null;

  const getFormatIcon = (fmt: ExportFormat): string => {
    const icons = {
      pdf: '📄',
      excel: '📊',
      html: '🌐',
      json: '{ }',
    };
    return icons[fmt] || '📄';
  };

  const getEstimatedSize = (): string => {
    let baseSize = 0.5; // MB
    if (includeCharts) baseSize += 0.2;
    if (includeScreenshots) baseSize += reportData.issues.length * 0.05;
    if (compression) baseSize *= 0.6;
    return `~${baseSize.toFixed(1)} MB`;
  };

  return (
    <div
      className={`export-dialog ${className}`}
      style={styles.overlay}
      role="dialog"
      aria-labelledby="export-dialog-title"
      aria-modal="true"
    >
      <div style={styles.dialog}>
        {/* Header */}
        <header style={styles.header}>
          <h2 id="export-dialog-title" style={styles.title}>
            Export Report
          </h2>
          <button
            onClick={onCancel}
            style={styles.closeButton}
            aria-label="Close export dialog"
          >
            ×
          </button>
        </header>

        {/* Content */}
        <div style={styles.content}>
          {/* Format Selection */}
          <section style={styles.section}>
            <h3 style={styles.sectionTitle}>Export Format</h3>
            <div style={styles.formatGrid}>
              {(['pdf', 'excel', 'html', 'json'] as ExportFormat[]).map((fmt) => (
                <label
                  key={fmt}
                  style={{
                    ...styles.formatCard,
                    ...(format === fmt ? styles.formatCardSelected : {}),
                  }}
                >
                  <input
                    type="radio"
                    name="format"
                    value={fmt}
                    checked={format === fmt}
                    onChange={(e) => setFormat(e.target.value as ExportFormat)}
                    style={styles.radioInput}
                  />
                  <span style={styles.formatIcon} aria-hidden="true">
                    {getFormatIcon(fmt)}
                  </span>
                  <span style={styles.formatLabel}>{fmt.toUpperCase()}</span>
                </label>
              ))}
            </div>
          </section>

          {/* Filename */}
          <section style={styles.section}>
            <label htmlFor="filename" style={styles.label}>
              Filename
            </label>
            <input
              id="filename"
              type="text"
              value={filename}
              onChange={(e) => setFilename(e.target.value)}
              style={styles.input}
              required
              aria-required="true"
            />
            <p style={styles.helpText}>
              Extension will be added automatically
            </p>
          </section>

          {/* PDF-specific options */}
          {format === 'pdf' && (
            <>
              <section style={styles.section}>
                <h3 style={styles.sectionTitle}>PDF Options</h3>
                <div style={styles.optionsGrid}>
                  <div style={styles.optionGroup}>
                    <label htmlFor="orientation" style={styles.label}>
                      Orientation
                    </label>
                    <select
                      id="orientation"
                      value={orientation}
                      onChange={(e) =>
                        setOrientation(e.target.value as 'portrait' | 'landscape')
                      }
                      style={styles.select}
                    >
                      <option value="portrait">Portrait</option>
                      <option value="landscape">Landscape</option>
                    </select>
                  </div>

                  <div style={styles.optionGroup}>
                    <label htmlFor="page-size" style={styles.label}>
                      Page Size
                    </label>
                    <select
                      id="page-size"
                      value={pageSize}
                      onChange={(e) =>
                        setPageSize(e.target.value as 'A4' | 'Letter' | 'Legal')
                      }
                      style={styles.select}
                    >
                      <option value="A4">A4 (210 × 297 mm)</option>
                      <option value="Letter">Letter (8.5 × 11 in)</option>
                      <option value="Legal">Legal (8.5 × 14 in)</option>
                    </select>
                  </div>
                </div>
              </section>

              <section style={styles.section}>
                <h3 style={styles.sectionTitle}>Accessibility Features</h3>
                <div style={styles.checkboxGroup}>
                  <label style={styles.checkboxLabel}>
                    <input
                      type="checkbox"
                      checked={pdfUA}
                      onChange={(e) => setPdfUA(e.target.checked)}
                      style={styles.checkbox}
                    />
                    <span>
                      <strong>PDF/UA Compliance</strong> - Generate accessible PDF
                    </span>
                  </label>
                  <p style={styles.infoText}>
                    Ensures PDF meets Universal Accessibility standards (ISO 14289)
                  </p>
                </div>
              </section>
            </>
          )}

          {/* Content Options */}
          <section style={styles.section}>
            <h3 style={styles.sectionTitle}>Content Options</h3>
            <div style={styles.checkboxGroup}>
              <label style={styles.checkboxLabel}>
                <input
                  type="checkbox"
                  checked={includeCharts}
                  onChange={(e) => setIncludeCharts(e.target.checked)}
                  style={styles.checkbox}
                />
                <span>Include charts and graphs</span>
              </label>

              <label style={styles.checkboxLabel}>
                <input
                  type="checkbox"
                  checked={includeScreenshots}
                  onChange={(e) => setIncludeScreenshots(e.target.checked)}
                  style={styles.checkbox}
                />
                <span>Include screenshots</span>
              </label>

              <label style={styles.checkboxLabel}>
                <input
                  type="checkbox"
                  checked={compression}
                  onChange={(e) => setCompression(e.target.checked)}
                  style={styles.checkbox}
                />
                <span>Enable compression (smaller file size)</span>
              </label>
            </div>
          </section>

          {/* Estimated Size */}
          <div style={styles.estimateBox}>
            <span style={styles.estimateLabel}>Estimated file size:</span>
            <span style={styles.estimateValue}>{getEstimatedSize()}</span>
          </div>
        </div>

        {/* Footer */}
        <footer style={styles.footer}>
          <button
            onClick={onCancel}
            style={styles.cancelButton}
            aria-label="Cancel export"
          >
            Cancel
          </button>
          <button
            onClick={handleExport}
            style={styles.exportButton}
            disabled={!filename}
            aria-label="Export report"
          >
            Export {format.toUpperCase()}
          </button>
        </footer>
      </div>
    </div>
  );
};

const styles = {
  overlay: {
    position: 'fixed' as const,
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1000,
    padding: '1rem',
  },
  dialog: {
    backgroundColor: '#fff',
    borderRadius: '8px',
    maxWidth: '700px',
    width: '100%',
    maxHeight: '90vh',
    display: 'flex',
    flexDirection: 'column' as const,
    boxShadow: '0 4px 20px rgba(0, 0, 0, 0.3)',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '1.5rem',
    borderBottom: '1px solid #e0e0e0',
  },
  title: {
    margin: 0,
    fontSize: '1.5rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  closeButton: {
    border: 'none',
    backgroundColor: 'transparent',
    fontSize: '2rem',
    cursor: 'pointer',
    color: '#999',
    width: '40px',
    height: '40px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    borderRadius: '4px',
    transition: 'all 0.2s ease',
  },
  content: {
    padding: '1.5rem',
    overflowY: 'auto' as const,
    flex: 1,
  },
  section: {
    marginBottom: '1.5rem',
  },
  sectionTitle: {
    margin: '0 0 1rem 0',
    fontSize: '1.125rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  formatGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(4, 1fr)',
    gap: '1rem',
  },
  formatCard: {
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'center',
    padding: '1rem',
    border: '2px solid #e0e0e0',
    borderRadius: '6px',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
  },
  formatCardSelected: {
    borderColor: '#0066cc',
    backgroundColor: '#e3f2fd',
  },
  radioInput: {
    position: 'absolute' as const,
    opacity: 0,
  },
  formatIcon: {
    fontSize: '2rem',
    marginBottom: '0.5rem',
  },
  formatLabel: {
    fontWeight: 'bold' as const,
    fontSize: '0.875rem',
  },
  label: {
    display: 'block',
    marginBottom: '0.5rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  input: {
    width: '100%',
    padding: '0.75rem',
    fontSize: '1rem',
    border: '1px solid #ccc',
    borderRadius: '4px',
    boxSizing: 'border-box' as const,
  },
  select: {
    width: '100%',
    padding: '0.75rem',
    fontSize: '1rem',
    border: '1px solid #ccc',
    borderRadius: '4px',
    boxSizing: 'border-box' as const,
  },
  helpText: {
    margin: '0.5rem 0 0 0',
    fontSize: '0.875rem',
    color: '#666',
  },
  infoText: {
    margin: '0.25rem 0 0 1.75rem',
    fontSize: '0.875rem',
    color: '#666',
    fontStyle: 'italic' as const,
  },
  optionsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(2, 1fr)',
    gap: '1rem',
  },
  optionGroup: {
    display: 'flex',
    flexDirection: 'column' as const,
  },
  checkboxGroup: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '0.75rem',
  },
  checkboxLabel: {
    display: 'flex',
    alignItems: 'flex-start',
    gap: '0.75rem',
    cursor: 'pointer',
  },
  checkbox: {
    width: '20px',
    height: '20px',
    cursor: 'pointer',
    marginTop: '0.125rem',
    flexShrink: 0,
  },
  estimateBox: {
    display: 'flex',
    justifyContent: 'space-between',
    padding: '1rem',
    backgroundColor: '#f0f7ff',
    borderRadius: '4px',
    border: '1px solid #b3d9ff',
    marginTop: '1rem',
  },
  estimateLabel: {
    fontWeight: 'bold' as const,
    color: '#333',
  },
  estimateValue: {
    fontWeight: 'bold' as const,
    color: '#0066cc',
    fontSize: '1.125rem',
  },
  footer: {
    display: 'flex',
    justifyContent: 'flex-end',
    gap: '1rem',
    padding: '1.5rem',
    borderTop: '1px solid #e0e0e0',
  },
  cancelButton: {
    padding: '0.75rem 1.5rem',
    fontSize: '1rem',
    border: '1px solid #ccc',
    borderRadius: '4px',
    backgroundColor: '#fff',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
  },
  exportButton: {
    padding: '0.75rem 1.5rem',
    fontSize: '1rem',
    border: 'none',
    borderRadius: '4px',
    backgroundColor: '#0066cc',
    color: '#fff',
    cursor: 'pointer',
    fontWeight: 'bold' as const,
    transition: 'all 0.2s ease',
  },
};
