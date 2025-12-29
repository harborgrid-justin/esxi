/**
 * ExportDialog - Export configuration dialog
 */

import React, { useState } from 'react';
import { ExportOptions, ReportFormat } from '../../types';

export interface ExportDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onExport: (options: ExportOptions) => void;
}

export const ExportDialog: React.FC<ExportDialogProps> = ({
  isOpen,
  onClose,
  onExport,
}) => {
  const [format, setFormat] = useState<ReportFormat>('pdf');
  const [orientation, setOrientation] = useState<'portrait' | 'landscape'>(
    'portrait'
  );
  const [paperSize, setPaperSize] = useState<'A4' | 'Letter' | 'Legal'>('A4');
  const [includeFilters, setIncludeFilters] = useState(true);
  const [includeData, setIncludeData] = useState(true);
  const [includeCharts, setIncludeCharts] = useState(true);

  const handleExport = () => {
    const options: ExportOptions = {
      format,
      orientation,
      paper_size: paperSize,
      include_filters: includeFilters,
      include_data: includeData,
      include_charts: includeCharts,
    };

    onExport(options);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="export-dialog-overlay" onClick={onClose}>
      <div className="export-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>Export Dashboard</h2>
          <button className="close-button" onClick={onClose}>
            ✕
          </button>
        </div>

        <div className="dialog-content">
          <div className="form-group">
            <label htmlFor="format">Export Format</label>
            <select
              id="format"
              value={format}
              onChange={(e) => setFormat(e.target.value as ReportFormat)}
            >
              <option value="pdf">PDF Document</option>
              <option value="excel">Excel Spreadsheet</option>
              <option value="csv">CSV File</option>
              <option value="json">JSON Data</option>
            </select>
          </div>

          {format === 'pdf' && (
            <>
              <div className="form-group">
                <label htmlFor="orientation">Page Orientation</label>
                <select
                  id="orientation"
                  value={orientation}
                  onChange={(e) =>
                    setOrientation(e.target.value as 'portrait' | 'landscape')
                  }
                >
                  <option value="portrait">Portrait</option>
                  <option value="landscape">Landscape</option>
                </select>
              </div>

              <div className="form-group">
                <label htmlFor="paperSize">Paper Size</label>
                <select
                  id="paperSize"
                  value={paperSize}
                  onChange={(e) =>
                    setPaperSize(e.target.value as 'A4' | 'Letter' | 'Legal')
                  }
                >
                  <option value="A4">A4</option>
                  <option value="Letter">Letter</option>
                  <option value="Legal">Legal</option>
                </select>
              </div>
            </>
          )}

          <div className="form-group">
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={includeFilters}
                onChange={(e) => setIncludeFilters(e.target.checked)}
              />
              <span>Include active filters</span>
            </label>
          </div>

          {(format === 'excel' || format === 'csv') && (
            <div className="form-group">
              <label className="checkbox-label">
                <input
                  type="checkbox"
                  checked={includeData}
                  onChange={(e) => setIncludeData(e.target.checked)}
                />
                <span>Include widget data</span>
              </label>
            </div>
          )}

          {format === 'excel' && (
            <div className="form-group">
              <label className="checkbox-label">
                <input
                  type="checkbox"
                  checked={includeCharts}
                  onChange={(e) => setIncludeCharts(e.target.checked)}
                />
                <span>Include charts</span>
              </label>
            </div>
          )}

          <div className="export-info">
            <p>
              The dashboard will be exported in <strong>{format.toUpperCase()}</strong> format
              and downloaded to your device.
            </p>
          </div>
        </div>

        <div className="dialog-actions">
          <button className="btn-cancel" onClick={onClose}>
            Cancel
          </button>
          <button className="btn-export" onClick={handleExport}>
            Export
          </button>
        </div>

        <style jsx>{`
          .export-dialog-overlay {
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.5);
            display: flex;
            align-items: center;
            justify-content: center;
            z-index: 1000;
          }

          .export-dialog {
            background: white;
            border-radius: 8px;
            width: 90%;
            max-width: 500px;
            max-height: 90vh;
            overflow: auto;
            box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
          }

          .dialog-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 20px 24px;
            border-bottom: 1px solid #e0e0e0;
          }

          .dialog-header h2 {
            margin: 0;
            font-size: 20px;
            color: #333;
          }

          .close-button {
            background: none;
            border: none;
            font-size: 24px;
            color: #999;
            cursor: pointer;
            padding: 0;
            width: 32px;
            height: 32px;
            display: flex;
            align-items: center;
            justify-content: center;
          }

          .close-button:hover {
            color: #333;
          }

          .dialog-content {
            padding: 24px;
          }

          .form-group {
            margin-bottom: 20px;
          }

          .form-group label {
            display: block;
            margin-bottom: 8px;
            font-weight: 600;
            color: #333;
            font-size: 14px;
          }

          .form-group select {
            width: 100%;
            padding: 10px 12px;
            border: 1px solid #ddd;
            border-radius: 4px;
            font-size: 14px;
            background: white;
          }

          .checkbox-label {
            display: flex;
            align-items: center;
            gap: 8px;
            cursor: pointer;
          }

          .checkbox-label input[type='checkbox'] {
            cursor: pointer;
            width: 18px;
            height: 18px;
          }

          .checkbox-label span {
            font-weight: normal;
          }

          .export-info {
            margin-top: 20px;
            padding: 12px;
            background: #f5f5f5;
            border-radius: 4px;
          }

          .export-info p {
            margin: 0;
            font-size: 13px;
            color: #666;
          }

          .dialog-actions {
            display: flex;
            gap: 12px;
            justify-content: flex-end;
            padding: 16px 24px;
            border-top: 1px solid #e0e0e0;
          }

          .btn-cancel,
          .btn-export {
            padding: 10px 20px;
            border-radius: 4px;
            font-size: 14px;
            cursor: pointer;
            border: none;
          }

          .btn-cancel {
            background: white;
            color: #333;
            border: 1px solid #ddd;
          }

          .btn-cancel:hover {
            background: #f5f5f5;
          }

          .btn-export {
            background: #1976d2;
            color: white;
          }

          .btn-export:hover {
            background: #1565c0;
          }
        `}</style>
      </div>
    </div>
  );
};
