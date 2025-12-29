import React, { useRef, useEffect, useState } from 'react';
import { ReportData } from '../../types';

interface PDFPreviewProps {
  reportData: ReportData;
  className?: string;
}

export const PDFPreview: React.FC<PDFPreviewProps> = ({
  reportData,
  className = '',
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages] = useState(5); // Simulated
  const [zoom, setZoom] = useState(100);

  useEffect(() => {
    renderPreview();
  }, [currentPage, zoom, reportData]);

  const renderPreview = () => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Set canvas size (A4 ratio)
    const scale = zoom / 100;
    canvas.width = 595 * scale;
    canvas.height = 842 * scale;

    // Clear canvas
    ctx.fillStyle = '#ffffff';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Draw page border
    ctx.strokeStyle = '#e0e0e0';
    ctx.lineWidth = 1;
    ctx.strokeRect(0, 0, canvas.width, canvas.height);

    // Draw header
    ctx.fillStyle = reportData.config.branding.primaryColor;
    ctx.fillRect(0, 0, canvas.width, 60 * scale);

    // Draw company name
    ctx.fillStyle = '#ffffff';
    ctx.font = `bold ${16 * scale}px Arial`;
    ctx.fillText(reportData.config.branding.companyName, 20 * scale, 35 * scale);

    // Draw title
    ctx.fillStyle = '#333333';
    ctx.font = `bold ${24 * scale}px Arial`;
    ctx.fillText(reportData.config.title, 20 * scale, 100 * scale);

    // Draw sample content
    ctx.fillStyle = '#666666';
    ctx.font = `${12 * scale}px Arial`;
    const lines = [
      `Generated: ${reportData.generatedAt.toLocaleDateString()}`,
      '',
      'This is a preview of your PDF report.',
      `Compliance Score: ${reportData.metrics.complianceScore}%`,
      `Total Issues: ${reportData.metrics.totalIssues}`,
      `Critical Issues: ${reportData.metrics.criticalIssues}`,
    ];

    lines.forEach((line, index) => {
      ctx.fillText(line, 20 * scale, (140 + index * 20) * scale);
    });

    // Draw page number
    ctx.fillStyle = '#999999';
    ctx.font = `${10 * scale}px Arial`;
    ctx.fillText(
      `Page ${currentPage} of ${totalPages}`,
      canvas.width - 100 * scale,
      canvas.height - 20 * scale
    );

    // Draw footer
    ctx.fillStyle = reportData.config.branding.primaryColor;
    ctx.fillRect(0, canvas.height - 40 * scale, canvas.width, 40 * scale);
    ctx.fillStyle = '#ffffff';
    ctx.font = `${10 * scale}px Arial`;
    ctx.fillText(
      reportData.config.branding.footerText || 'Accessibility Report',
      20 * scale,
      canvas.height - 18 * scale
    );
  };

  const handlePreviousPage = () => {
    if (currentPage > 1) {
      setCurrentPage(currentPage - 1);
    }
  };

  const handleNextPage = () => {
    if (currentPage < totalPages) {
      setCurrentPage(currentPage + 1);
    }
  };

  const handleZoomIn = () => {
    if (zoom < 200) {
      setZoom(zoom + 25);
    }
  };

  const handleZoomOut = () => {
    if (zoom > 50) {
      setZoom(zoom - 25);
    }
  };

  return (
    <div
      className={`pdf-preview ${className}`}
      style={styles.container}
      role="region"
      aria-label="PDF preview"
    >
      {/* Controls */}
      <div style={styles.controls} role="toolbar" aria-label="Preview controls">
        <div style={styles.controlGroup}>
          <button
            onClick={handlePreviousPage}
            disabled={currentPage === 1}
            style={{
              ...styles.button,
              ...(currentPage === 1 ? styles.buttonDisabled : {}),
            }}
            aria-label="Previous page"
          >
            ← Previous
          </button>
          <span style={styles.pageInfo} aria-live="polite">
            Page {currentPage} of {totalPages}
          </span>
          <button
            onClick={handleNextPage}
            disabled={currentPage === totalPages}
            style={{
              ...styles.button,
              ...(currentPage === totalPages ? styles.buttonDisabled : {}),
            }}
            aria-label="Next page"
          >
            Next →
          </button>
        </div>

        <div style={styles.controlGroup}>
          <button
            onClick={handleZoomOut}
            disabled={zoom === 50}
            style={{
              ...styles.button,
              ...(zoom === 50 ? styles.buttonDisabled : {}),
            }}
            aria-label="Zoom out"
          >
            −
          </button>
          <span style={styles.zoomInfo} aria-live="polite">
            {zoom}%
          </span>
          <button
            onClick={handleZoomIn}
            disabled={zoom === 200}
            style={{
              ...styles.button,
              ...(zoom === 200 ? styles.buttonDisabled : {}),
            }}
            aria-label="Zoom in"
          >
            +
          </button>
        </div>
      </div>

      {/* Preview Area */}
      <div style={styles.previewArea}>
        <div style={styles.canvasContainer}>
          <canvas
            ref={canvasRef}
            style={styles.canvas}
            role="img"
            aria-label={`PDF preview page ${currentPage}`}
          />
        </div>
      </div>

      {/* Info Panel */}
      <div style={styles.infoPanel}>
        <h3 style={styles.infoTitle}>Preview Information</h3>
        <dl style={styles.infoList}>
          <dt style={styles.infoLabel}>Document:</dt>
          <dd style={styles.infoValue}>{reportData.config.title}</dd>

          <dt style={styles.infoLabel}>Pages:</dt>
          <dd style={styles.infoValue}>{totalPages}</dd>

          <dt style={styles.infoLabel}>Orientation:</dt>
          <dd style={styles.infoValue}>Portrait</dd>

          <dt style={styles.infoLabel}>Size:</dt>
          <dd style={styles.infoValue}>A4 (210 × 297 mm)</dd>

          <dt style={styles.infoLabel}>Accessibility:</dt>
          <dd style={styles.infoValue}>PDF/UA Compliant</dd>
        </dl>

        <div style={styles.accessibility}>
          <h4 style={styles.accessibilityTitle}>Accessibility Features</h4>
          <ul style={styles.accessibilityList}>
            <li style={styles.accessibilityItem}>✓ Tagged PDF structure</li>
            <li style={styles.accessibilityItem}>✓ Alternative text for images</li>
            <li style={styles.accessibilityItem}>✓ Logical reading order</li>
            <li style={styles.accessibilityItem}>✓ Bookmarks for navigation</li>
            <li style={styles.accessibilityItem}>✓ High contrast colors</li>
          </ul>
        </div>
      </div>
    </div>
  );
};

const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column' as const,
    height: '100%',
    backgroundColor: '#f5f5f5',
  },
  controls: {
    display: 'flex',
    justifyContent: 'space-between',
    padding: '1rem',
    backgroundColor: '#fff',
    borderBottom: '1px solid #e0e0e0',
  },
  controlGroup: {
    display: 'flex',
    alignItems: 'center',
    gap: '1rem',
  },
  button: {
    padding: '0.5rem 1rem',
    fontSize: '0.875rem',
    border: '1px solid #ccc',
    borderRadius: '4px',
    backgroundColor: '#fff',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
  },
  buttonDisabled: {
    opacity: 0.5,
    cursor: 'not-allowed',
  },
  pageInfo: {
    fontSize: '0.875rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  zoomInfo: {
    fontSize: '0.875rem',
    fontWeight: 'bold' as const,
    color: '#333',
    minWidth: '50px',
    textAlign: 'center' as const,
  },
  previewArea: {
    flex: 1,
    padding: '2rem',
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'flex-start',
    overflowY: 'auto' as const,
  },
  canvasContainer: {
    backgroundColor: '#fff',
    boxShadow: '0 4px 12px rgba(0, 0, 0, 0.2)',
    borderRadius: '4px',
    overflow: 'hidden',
  },
  canvas: {
    display: 'block',
  },
  infoPanel: {
    padding: '1.5rem',
    backgroundColor: '#fff',
    borderTop: '1px solid #e0e0e0',
  },
  infoTitle: {
    margin: '0 0 1rem 0',
    fontSize: '1.125rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  infoList: {
    display: 'grid',
    gridTemplateColumns: '150px 1fr',
    gap: '0.75rem',
    margin: '0 0 1.5rem 0',
  },
  infoLabel: {
    fontWeight: 'bold' as const,
    color: '#666',
  },
  infoValue: {
    margin: 0,
    color: '#333',
  },
  accessibility: {
    padding: '1rem',
    backgroundColor: '#e3f2fd',
    borderRadius: '4px',
    borderLeft: '4px solid #0066cc',
  },
  accessibilityTitle: {
    margin: '0 0 0.75rem 0',
    fontSize: '1rem',
    fontWeight: 'bold' as const,
    color: '#0066cc',
  },
  accessibilityList: {
    listStyle: 'none',
    padding: 0,
    margin: 0,
  },
  accessibilityItem: {
    padding: '0.25rem 0',
    fontSize: '0.875rem',
    color: '#0066cc',
  },
};
