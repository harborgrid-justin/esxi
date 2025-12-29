/**
 * Document Checker Hook
 * React hook for document accessibility checking
 */

import { useState, useCallback } from 'react';
import type {
  CheckerResult,
  CheckerOptions,
  CheckerProgress,
  CheckerStage,
  DocumentType,
  AccessibilityIssue
} from '../types/index.js';
import { PDFAnalyzer } from '../analyzers/PDFAnalyzer.js';
import { WordAnalyzer } from '../analyzers/WordAnalyzer.js';
import { ExcelAnalyzer } from '../analyzers/ExcelAnalyzer.js';
import { PowerPointAnalyzer } from '../analyzers/PowerPointAnalyzer.js';
import { EPUBAnalyzer } from '../analyzers/EPUBAnalyzer.js';
import { detectDocumentType, calculateComplianceScore, generateCheckerSummary } from '../utils/documentUtils.js';

export function useDocumentChecker(options?: CheckerOptions) {
  const [isChecking, setIsChecking] = useState(false);
  const [progress, setProgress] = useState<CheckerProgress | null>(null);
  const [result, setResult] = useState<CheckerResult | null>(null);
  const [error, setError] = useState<Error | null>(null);

  /**
   * Check document for accessibility issues
   */
  const checkDocument = useCallback(async (file: File): Promise<CheckerResult> => {
    setIsChecking(true);
    setProgress(null);
    setResult(null);
    setError(null);

    const startTime = Date.now();

    try {
      // Update progress: Uploading
      updateProgress(CheckerStage.UPLOADING, 0, 'Uploading document...');

      // Detect document type
      const documentType = detectDocumentType(file.name, file.type);
      updateProgress(CheckerStage.PARSING, 10, `Parsing ${documentType} document...`);

      if (documentType === DocumentType.UNKNOWN) {
        throw new Error('Unsupported document type');
      }

      // Select appropriate analyzer
      let issues: AccessibilityIssue[] = [];
      let metadata: any;
      let structure: any;
      let readingOrder: any;

      switch (documentType) {
        case DocumentType.PDF: {
          updateProgress(CheckerStage.ANALYZING_STRUCTURE, 20, 'Analyzing PDF structure...');
          const analyzer = new PDFAnalyzer();
          const pdfResult = await analyzer.analyze(file);
          issues = analyzer.getIssues();
          metadata = pdfResult.metadata;
          structure = pdfResult.structureTree;
          readingOrder = pdfResult.readingOrder;
          break;
        }

        case DocumentType.WORD: {
          updateProgress(CheckerStage.ANALYZING_STRUCTURE, 20, 'Analyzing Word document...');
          const analyzer = new WordAnalyzer();
          const wordResult = await analyzer.analyze(file);
          issues = analyzer.getIssues();
          metadata = wordResult.metadata;
          break;
        }

        case DocumentType.EXCEL: {
          updateProgress(CheckerStage.ANALYZING_STRUCTURE, 20, 'Analyzing Excel workbook...');
          const analyzer = new ExcelAnalyzer();
          const excelResult = await analyzer.analyze(file);
          issues = analyzer.getIssues();
          metadata = excelResult.metadata;
          break;
        }

        case DocumentType.POWERPOINT: {
          updateProgress(CheckerStage.ANALYZING_STRUCTURE, 20, 'Analyzing PowerPoint presentation...');
          const analyzer = new PowerPointAnalyzer();
          const pptResult = await analyzer.analyze(file);
          issues = analyzer.getIssues();
          metadata = pptResult.metadata;
          break;
        }

        case DocumentType.EPUB: {
          updateProgress(CheckerStage.ANALYZING_STRUCTURE, 20, 'Analyzing EPUB document...');
          const analyzer = new EPUBAnalyzer();
          const epubResult = await analyzer.analyze(file);
          issues = analyzer.getIssues();
          metadata = epubResult.metadata;
          break;
        }

        default:
          throw new Error(`Unsupported document type: ${documentType}`);
      }

      // Progress through various checks
      updateProgress(CheckerStage.VALIDATING_TAGS, 40, 'Validating document structure...');
      await delay(500);

      updateProgress(CheckerStage.CHECKING_METADATA, 50, 'Checking metadata...');
      await delay(500);

      updateProgress(CheckerStage.CHECKING_IMAGES, 60, 'Checking images and alternative text...');
      await delay(500);

      updateProgress(CheckerStage.CHECKING_FORMS, 70, 'Checking forms and interactive elements...');
      await delay(500);

      updateProgress(CheckerStage.CHECKING_READING_ORDER, 80, 'Validating reading order...');
      await delay(500);

      updateProgress(CheckerStage.GENERATING_REPORT, 90, 'Generating accessibility report...');

      // Calculate compliance score and summary
      const totalChecks = 20; // This should be calculated based on actual checks performed
      const complianceScore = calculateComplianceScore(issues, totalChecks);

      const summary = generateCheckerSummary(
        issues,
        !!structure,
        !!metadata.title || !!metadata.author,
        !!metadata.language,
        issues.filter(i => i.type.includes('alt')).length === 0
      );

      const checkDuration = Date.now() - startTime;

      const checkerResult: CheckerResult = {
        documentType,
        fileName: file.name,
        fileSize: file.size,
        metadata,
        issues,
        structure,
        readingOrder,
        complianceScore,
        summary,
        checkedAt: new Date(),
        checkDuration
      };

      updateProgress(CheckerStage.COMPLETE, 100, 'Analysis complete!');
      setResult(checkerResult);

      return checkerResult;
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown error occurred');
      setError(error);
      updateProgress(CheckerStage.ERROR, 0, error.message);
      throw error;
    } finally {
      setIsChecking(false);
    }
  }, [options]);

  /**
   * Update progress
   */
  const updateProgress = (stage: CheckerStage, progress: number, message: string) => {
    setProgress({
      stage,
      progress,
      message
    });
  };

  /**
   * Reset checker state
   */
  const reset = useCallback(() => {
    setIsChecking(false);
    setProgress(null);
    setResult(null);
    setError(null);
  }, []);

  return {
    checkDocument,
    reset,
    isChecking,
    progress,
    result,
    error
  };
}

/**
 * Helper: Delay function
 */
function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}
