/**
 * Enterprise Accessibility Report Generator
 * Main entry point for the accessibility reports package
 */

// Export all types
export * from './types';

// Export components
export { ReportBuilder } from './components/ReportBuilder/ReportBuilder';
export { SectionSelector } from './components/ReportBuilder/SectionSelector';
export { TemplateLibrary } from './components/ReportBuilder/TemplateLibrary';
export { BrandingConfig as BrandingConfigComponent } from './components/ReportBuilder/BrandingConfig';
export { ReportViewer } from './components/ReportViewer/ReportViewer';
export { ReportSection } from './components/ReportViewer/ReportSection';
export { TableOfContents } from './components/ReportViewer/TableOfContents';
export { ExportDialog } from './components/Export/ExportDialog';
export { PDFPreview } from './components/Export/PDFPreview';
export { ScheduledExports } from './components/Export/ScheduledExports';

// Export generators
export { PDFGenerator } from './generators/PDFGenerator';
export { ExcelGenerator } from './generators/ExcelGenerator';
export { HTMLGenerator } from './generators/HTMLGenerator';
export { JSONGenerator } from './generators/JSONGenerator';

// Export templates
export { ExecutiveSummaryTemplate } from './templates/ExecutiveSummary';
export { TechnicalReportTemplate } from './templates/TechnicalReport';
export { ComplianceAuditTemplate } from './templates/ComplianceAudit';
export { RemediationGuideTemplate } from './templates/RemediationGuide';

// Export hooks
export { useReportBuilder } from './hooks/useReportBuilder';
export { useExport } from './hooks/useExport';

// Export utilities
export * from './utils/formatting';

// Version
export const VERSION = '1.0.0';
