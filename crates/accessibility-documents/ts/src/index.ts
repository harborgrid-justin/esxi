/**
 * @harborgrid/accessibility-documents
 * Enterprise Document Accessibility Checker
 *
 * Complete accessibility analysis for PDF/UA, EPUB, and Office documents
 * with WCAG 2.1/2.2 compliance validation
 */

// Types
export * from './types/index.js';

// Components
export { DocumentChecker } from './components/Checker/DocumentChecker.js';
export { FileUpload } from './components/Checker/FileUpload.js';
export { AnalysisProgress } from './components/Checker/AnalysisProgress.js';
export { CheckerResults } from './components/Results/CheckerResults.js';
export { IssueList } from './components/Results/IssueList.js';
export { StructureViewer } from './components/Results/StructureViewer.js';
export { PDFAnalyzer as PDFAnalyzerComponent } from './components/PDF/PDFAnalyzer.js';
export { TagTreeViewer } from './components/PDF/TagTreeViewer.js';
export { ReadingOrderViewer } from './components/PDF/ReadingOrderViewer.js';
export { WordAnalyzer as WordAnalyzerComponent } from './components/Office/WordAnalyzer.js';
export { ExcelAnalyzer as ExcelAnalyzerComponent } from './components/Office/ExcelAnalyzer.js';
export { PowerPointAnalyzer as PowerPointAnalyzerComponent } from './components/Office/PowerPointAnalyzer.js';

// Analyzers
export { PDFAnalyzer } from './analyzers/PDFAnalyzer.js';
export { WordAnalyzer } from './analyzers/WordAnalyzer.js';
export { ExcelAnalyzer } from './analyzers/ExcelAnalyzer.js';
export { PowerPointAnalyzer } from './analyzers/PowerPointAnalyzer.js';
export { EPUBAnalyzer } from './analyzers/EPUBAnalyzer.js';

// Validators
export { PDFTagValidator } from './validators/PDFTagValidator.js';
export { AltTextValidator } from './validators/AltTextValidator.js';
export { HeadingValidator } from './validators/HeadingValidator.js';
export { TableValidator } from './validators/TableValidator.js';
export { ListValidator } from './validators/ListValidator.js';

// Remediators
export { PDFRemediator } from './remediators/PDFRemediator.js';
export { OfficeRemediator } from './remediators/OfficeRemediator.js';

// Hooks
export { useDocumentChecker } from './hooks/useDocumentChecker.js';

// Utils
export * from './utils/documentUtils.js';
