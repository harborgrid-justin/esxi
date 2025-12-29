# Accessibility Reports Generator

Enterprise-grade accessibility report generator with PDF/UA compliance support.

## Features

- **Multiple Report Templates**: Executive Summary, Technical Report, Compliance Audit, Remediation Guide
- **Multi-Format Export**: PDF (PDF/UA compliant), Excel, HTML, JSON
- **Interactive Report Builder**: Drag-and-drop section configuration
- **Custom Branding**: Full customization of colors, fonts, logos
- **Scheduled Exports**: Automated report generation and delivery
- **Accessibility First**: All generated reports are WCAG 2.1 compliant

## Installation

```bash
npm install @harborgrid/accessibility-reports
```

## Quick Start

```typescript
import { ReportBuilder, PDFGenerator } from '@harborgrid/accessibility-reports';

// Build report configuration
const reportData = {
  config: {...},
  issues: [...],
  metrics: {...},
  trends: [...],
};

// Generate PDF report
const options = {
  format: 'pdf',
  filename: 'accessibility-report',
  orientation: 'portrait',
  pageSize: 'A4',
  includeCharts: true,
  accessibility: {
    pdfUA: true,
    tagged: true,
    altText: true,
  },
};

const generator = new PDFGenerator(options, reportData);
const blob = await generator.generate();
```

## Components

### Report Builder
```tsx
import { ReportBuilder } from '@harborgrid/accessibility-reports';

<ReportBuilder
  onSave={(config) => console.log('Report saved', config)}
/>
```

### Report Viewer
```tsx
import { ReportViewer } from '@harborgrid/accessibility-reports';

<ReportViewer
  reportData={reportData}
  onExport={() => handleExport()}
/>
```

### Export Dialog
```tsx
import { ExportDialog } from '@harborgrid/accessibility-reports';

<ExportDialog
  reportData={reportData}
  isOpen={true}
  onExport={(options) => handleExport(options)}
  onCancel={() => setIsOpen(false)}
/>
```

## Hooks

### useReportBuilder
```typescript
const {
  config,
  template,
  sections,
  branding,
  isValid,
  setTemplate,
  updateSections,
  buildConfig,
} = useReportBuilder();
```

### useExport
```typescript
const {
  isExporting,
  progress,
  error,
  exportReport,
} = useExport();

await exportReport(reportData, options);
```

## Templates

- **Executive Summary**: High-level overview for stakeholders
- **Technical Report**: Detailed analysis for developers
- **Compliance Audit**: Regulatory compliance documentation
- **Remediation Guide**: Step-by-step fixing instructions

## License

MIT
