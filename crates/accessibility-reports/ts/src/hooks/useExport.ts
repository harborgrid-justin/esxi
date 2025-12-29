import { useState, useCallback } from 'react';
import { ReportData, ExportOptions, ExportFormat } from '../types';
import { PDFGenerator } from '../generators/PDFGenerator';
import { ExcelGenerator } from '../generators/ExcelGenerator';
import { HTMLGenerator } from '../generators/HTMLGenerator';
import { JSONGenerator } from '../generators/JSONGenerator';

interface UseExportReturn {
  isExporting: boolean;
  progress: number;
  error: string | null;
  exportReport: (reportData: ReportData, options: ExportOptions) => Promise<void>;
  downloadBlob: (blob: Blob, filename: string, format: ExportFormat) => void;
  cancelExport: () => void;
}

/**
 * Custom hook for exporting reports in various formats
 * Handles generation, progress tracking, and error handling
 */
export const useExport = (): UseExportReturn => {
  const [isExporting, setIsExporting] = useState(false);
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [abortController, setAbortController] = useState<AbortController | null>(null);

  /**
   * Export report in the specified format
   */
  const exportReport = useCallback(
    async (reportData: ReportData, options: ExportOptions): Promise<void> => {
      setIsExporting(true);
      setProgress(0);
      setError(null);

      const controller = new AbortController();
      setAbortController(controller);

      try {
        // Simulate progress updates
        const progressInterval = setInterval(() => {
          setProgress((prev) => {
            if (prev >= 90) {
              clearInterval(progressInterval);
              return 90;
            }
            return prev + 10;
          });
        }, 200);

        let blob: Blob;

        // Generate report based on format
        switch (options.format) {
          case 'pdf': {
            const generator = new PDFGenerator(options, reportData);
            blob = await generator.generate();
            break;
          }

          case 'excel': {
            const generator = new ExcelGenerator(options, reportData);
            blob = await generator.generate();
            break;
          }

          case 'html': {
            const generator = new HTMLGenerator(options, reportData);
            blob = await generator.generate();
            break;
          }

          case 'json': {
            const generator = new JSONGenerator(options, reportData);
            blob = await generator.generate();
            break;
          }

          default:
            throw new Error(`Unsupported export format: ${options.format}`);
        }

        clearInterval(progressInterval);
        setProgress(100);

        // Download the generated file
        if (!controller.signal.aborted) {
          downloadBlob(blob, options.filename, options.format);
        }

        setIsExporting(false);
      } catch (err) {
        clearInterval(progressInterval);
        const errorMessage = err instanceof Error ? err.message : 'Export failed';
        setError(errorMessage);
        setIsExporting(false);
        setProgress(0);
        throw err;
      } finally {
        setAbortController(null);
      }
    },
    []
  );

  /**
   * Download blob as file
   */
  const downloadBlob = useCallback(
    (blob: Blob, filename: string, format: ExportFormat): void => {
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;

      // Add appropriate file extension
      const extension = getFileExtension(format);
      link.download = filename.endsWith(extension)
        ? filename
        : `${filename}${extension}`;

      // Trigger download
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);

      // Cleanup
      setTimeout(() => {
        URL.revokeObjectURL(url);
      }, 100);
    },
    []
  );

  /**
   * Cancel ongoing export
   */
  const cancelExport = useCallback(() => {
    if (abortController) {
      abortController.abort();
      setAbortController(null);
    }
    setIsExporting(false);
    setProgress(0);
    setError('Export cancelled by user');
  }, [abortController]);

  return {
    isExporting,
    progress,
    error,
    exportReport,
    downloadBlob,
    cancelExport,
  };
};

/**
 * Get file extension for export format
 */
function getFileExtension(format: ExportFormat): string {
  const extensions: Record<ExportFormat, string> = {
    pdf: '.pdf',
    excel: '.xlsx',
    html: '.html',
    json: '.json',
  };
  return extensions[format];
}

/**
 * Estimate file size based on report data
 */
export function estimateFileSize(
  reportData: ReportData,
  format: ExportFormat,
  includeCharts: boolean,
  includeScreenshots: boolean
): number {
  let baseSize = 0;

  switch (format) {
    case 'pdf':
      baseSize = 500; // 500 KB base
      if (includeCharts) baseSize += 200;
      if (includeScreenshots) baseSize += reportData.issues.length * 50;
      break;

    case 'excel':
      baseSize = 300; // 300 KB base
      baseSize += reportData.issues.length * 2;
      break;

    case 'html':
      baseSize = 400; // 400 KB base
      if (includeCharts) baseSize += 150;
      if (includeScreenshots) baseSize += reportData.issues.length * 30;
      break;

    case 'json':
      baseSize = 100; // 100 KB base
      baseSize += reportData.issues.length * 5;
      break;
  }

  return baseSize; // Return size in KB
}

/**
 * Validate export options
 */
export function validateExportOptions(options: ExportOptions): string[] {
  const errors: string[] = [];

  if (!options.filename || options.filename.trim() === '') {
    errors.push('Filename is required');
  }

  if (!options.format) {
    errors.push('Export format is required');
  }

  if (options.format === 'pdf') {
    if (!options.orientation) {
      errors.push('PDF orientation is required');
    }
    if (!options.pageSize) {
      errors.push('PDF page size is required');
    }
  }

  return errors;
}
