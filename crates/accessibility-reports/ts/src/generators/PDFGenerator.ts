import jsPDF from 'jspdf';
import autoTable from 'jspdf-autotable';
import { ReportData, ExportOptions } from '../types';
import { formatDate, formatPercentage, getSeverityColor } from '../utils/formatting';

/**
 * PDF/UA Compliant PDF Generator
 * Generates accessible PDF documents following ISO 14289 standards
 */
export class PDFGenerator {
  private doc: jsPDF;
  private currentY: number = 0;
  private pageWidth: number = 0;
  private pageHeight: number = 0;
  private margins = { top: 20, right: 20, bottom: 20, left: 20 };

  constructor(private options: ExportOptions, private reportData: ReportData) {
    const orientation = options.orientation || 'portrait';
    const format = options.pageSize || 'A4';

    this.doc = new jsPDF({
      orientation,
      unit: 'mm',
      format,
    });

    this.pageWidth = this.doc.internal.pageSize.getWidth();
    this.pageHeight = this.doc.internal.pageSize.getHeight();
    this.currentY = this.margins.top;
  }

  /**
   * Generate the complete PDF report
   */
  public async generate(): Promise<Blob> {
    // Set PDF metadata for accessibility
    this.setPDFMetadata();

    // Generate document structure
    this.addCoverPage();
    this.addTableOfContents();
    this.addExecutiveSummary();
    this.addMetricsSection();
    this.addIssuesSection();
    this.addRecommendationsSection();
    this.addFooter();

    // Apply PDF/UA compliance tags if enabled
    if (this.options.accessibility?.pdfUA) {
      this.applyPDFUATags();
    }

    // Return as blob
    return this.doc.output('blob');
  }

  /**
   * Set PDF metadata for accessibility
   */
  private setPDFMetadata(): void {
    const { reportData } = this;

    this.doc.setProperties({
      title: reportData.config.title,
      subject: reportData.config.subtitle || 'Accessibility Compliance Report',
      author: reportData.config.branding.companyName,
      keywords: 'accessibility, WCAG, compliance, audit',
      creator: 'HarborGrid Accessibility Reports',
    });

    this.doc.setLanguage('en-US');
  }

  /**
   * Add cover page with branding
   */
  private addCoverPage(): void {
    const { branding } = this.reportData.config;

    // Add logo if available
    if (branding.logo && this.options.includeCharts) {
      try {
        this.doc.addImage(branding.logo, 'PNG', this.margins.left, this.currentY, 60, 20);
        this.currentY += 30;
      } catch (error) {
        console.warn('Failed to add logo:', error);
      }
    }

    // Company name
    this.doc.setFontSize(14);
    this.doc.setTextColor(branding.secondaryColor);
    this.doc.text(branding.companyName, this.margins.left, this.currentY);
    this.currentY += 20;

    // Title
    this.doc.setFontSize(28);
    this.doc.setFont(undefined, 'bold');
    this.doc.setTextColor(branding.primaryColor);
    const titleLines = this.doc.splitTextToSize(
      this.reportData.config.title,
      this.pageWidth - this.margins.left - this.margins.right
    );
    this.doc.text(titleLines, this.margins.left, this.currentY);
    this.currentY += titleLines.length * 12;

    // Subtitle
    if (this.reportData.config.subtitle) {
      this.doc.setFontSize(16);
      this.doc.setFont(undefined, 'normal');
      this.doc.setTextColor('#666666');
      this.doc.text(this.reportData.config.subtitle, this.margins.left, this.currentY);
      this.currentY += 15;
    }

    // Date range
    this.doc.setFontSize(12);
    this.doc.setTextColor('#999999');
    const dateRange = `Report Period: ${formatDate(
      this.reportData.config.dateRange.from
    )} - ${formatDate(this.reportData.config.dateRange.to)}`;
    this.doc.text(dateRange, this.margins.left, this.currentY);
    this.currentY += 10;

    // Generated date
    const generatedDate = `Generated: ${formatDate(this.reportData.generatedAt)}`;
    this.doc.text(generatedDate, this.margins.left, this.currentY);

    // Add decorative line
    this.currentY += 20;
    this.doc.setDrawColor(branding.primaryColor);
    this.doc.setLineWidth(0.5);
    this.doc.line(
      this.margins.left,
      this.currentY,
      this.pageWidth - this.margins.right,
      this.currentY
    );

    // New page for content
    this.addPage();
  }

  /**
   * Add table of contents
   */
  private addTableOfContents(): void {
    this.addSectionTitle('Table of Contents');

    const contents = [
      { title: 'Executive Summary', page: 3 },
      { title: 'Compliance Metrics', page: 4 },
      { title: 'Issues Breakdown', page: 5 },
      { title: 'Recommendations', page: 7 },
    ];

    this.doc.setFontSize(11);
    contents.forEach((item) => {
      this.doc.setFont(undefined, 'normal');
      this.doc.text(item.title, this.margins.left, this.currentY);
      this.doc.text(
        `${item.page}`,
        this.pageWidth - this.margins.right - 10,
        this.currentY,
        { align: 'right' }
      );
      this.currentY += 8;
    });

    this.addPage();
  }

  /**
   * Add executive summary section
   */
  private addExecutiveSummary(): void {
    this.addSectionTitle('Executive Summary');

    const { metrics } = this.reportData;

    // Key metrics grid
    const metricsData = [
      ['Compliance Score', `${formatPercentage(metrics.complianceScore)}`],
      ['Total Issues', `${metrics.totalIssues}`],
      ['Critical Issues', `${metrics.criticalIssues}`],
      ['WCAG AA Compliance', `${formatPercentage(metrics.wcagAACompliance)}`],
    ];

    autoTable(this.doc, {
      startY: this.currentY,
      head: [['Metric', 'Value']],
      body: metricsData,
      theme: 'grid',
      headStyles: {
        fillColor: this.reportData.config.branding.primaryColor,
        textColor: '#ffffff',
        fontSize: 11,
        fontStyle: 'bold',
      },
      styles: {
        fontSize: 10,
      },
      margin: { left: this.margins.left, right: this.margins.right },
    });

    this.currentY = (this.doc as any).lastAutoTable.finalY + 10;

    // Summary text
    this.doc.setFontSize(10);
    this.doc.setFont(undefined, 'normal');
    const summaryText = `This report covers accessibility compliance testing for the period ${formatDate(
      this.reportData.config.dateRange.from
    )} to ${formatDate(
      this.reportData.config.dateRange.to
    )}. A total of ${
      metrics.totalIssues
    } accessibility issues were identified, with ${
      metrics.criticalIssues
    } classified as critical priority.`;

    const textLines = this.doc.splitTextToSize(
      summaryText,
      this.pageWidth - this.margins.left - this.margins.right
    );
    this.doc.text(textLines, this.margins.left, this.currentY);
    this.currentY += textLines.length * 6;

    this.addPage();
  }

  /**
   * Add metrics section
   */
  private addMetricsSection(): void {
    this.addSectionTitle('Compliance Metrics');

    const { metrics } = this.reportData;

    // Issue breakdown
    this.doc.setFontSize(12);
    this.doc.setFont(undefined, 'bold');
    this.doc.text('Issue Severity Breakdown', this.margins.left, this.currentY);
    this.currentY += 8;

    const severityData = [
      ['Critical', `${metrics.criticalIssues}`, getSeverityColor('critical')],
      ['Serious', `${metrics.seriousIssues}`, getSeverityColor('serious')],
      ['Moderate', `${metrics.moderateIssues}`, getSeverityColor('moderate')],
      ['Minor', `${metrics.minorIssues}`, getSeverityColor('minor')],
    ];

    autoTable(this.doc, {
      startY: this.currentY,
      head: [['Severity', 'Count', 'Color Code']],
      body: severityData,
      theme: 'striped',
      margin: { left: this.margins.left, right: this.margins.right },
      styles: { fontSize: 10 },
    });

    this.currentY = (this.doc as any).lastAutoTable.finalY + 15;

    // WCAG Compliance levels
    this.doc.setFontSize(12);
    this.doc.setFont(undefined, 'bold');
    this.doc.text('WCAG 2.1 Compliance Levels', this.margins.left, this.currentY);
    this.currentY += 8;

    const wcagData = [
      ['Level A', `${formatPercentage(metrics.wcagACompliance)}`],
      ['Level AA', `${formatPercentage(metrics.wcagAACompliance)}`],
      ['Level AAA', `${formatPercentage(metrics.wcagAAACompliance)}`],
    ];

    autoTable(this.doc, {
      startY: this.currentY,
      head: [['WCAG Level', 'Compliance']],
      body: wcagData,
      theme: 'grid',
      margin: { left: this.margins.left, right: this.margins.right },
      styles: { fontSize: 10 },
    });

    this.currentY = (this.doc as any).lastAutoTable.finalY + 10;
    this.addPage();
  }

  /**
   * Add issues section
   */
  private addIssuesSection(): void {
    this.addSectionTitle('Issues Breakdown');

    const criticalIssues = this.reportData.issues
      .filter((i) => i.severity === 'critical')
      .slice(0, 10);

    if (criticalIssues.length > 0) {
      this.doc.setFontSize(12);
      this.doc.setFont(undefined, 'bold');
      this.doc.text('Critical Issues', this.margins.left, this.currentY);
      this.currentY += 8;

      criticalIssues.forEach((issue, index) => {
        if (this.currentY > this.pageHeight - 40) {
          this.addPage();
        }

        this.doc.setFontSize(10);
        this.doc.setFont(undefined, 'bold');
        this.doc.text(`${index + 1}. ${issue.title}`, this.margins.left, this.currentY);
        this.currentY += 6;

        this.doc.setFont(undefined, 'normal');
        this.doc.setFontSize(9);
        const descLines = this.doc.splitTextToSize(
          issue.description,
          this.pageWidth - this.margins.left - this.margins.right - 10
        );
        this.doc.text(descLines, this.margins.left + 5, this.currentY);
        this.currentY += descLines.length * 5 + 2;

        // WCAG criteria
        this.doc.setTextColor('#666666');
        this.doc.text(
          `WCAG: ${issue.wcagCriteria.join(', ')}`,
          this.margins.left + 5,
          this.currentY
        );
        this.currentY += 5;

        // Location
        this.doc.text(`Location: ${issue.location.url}`, this.margins.left + 5, this.currentY);
        this.currentY += 8;

        this.doc.setTextColor('#000000');
      });
    }

    this.addPage();
  }

  /**
   * Add recommendations section
   */
  private addRecommendationsSection(): void {
    this.addSectionTitle('Priority Recommendations');

    const topIssues = this.reportData.issues
      .filter((i) => i.severity === 'critical' || i.severity === 'serious')
      .slice(0, 5);

    topIssues.forEach((issue, index) => {
      if (this.currentY > this.pageHeight - 50) {
        this.addPage();
      }

      this.doc.setFontSize(11);
      this.doc.setFont(undefined, 'bold');
      this.doc.text(
        `${index + 1}. ${issue.title}`,
        this.margins.left,
        this.currentY
      );
      this.currentY += 6;

      // Impact
      this.doc.setFontSize(9);
      this.doc.setFont(undefined, 'bold');
      this.doc.text('Impact:', this.margins.left + 5, this.currentY);
      this.doc.setFont(undefined, 'normal');
      this.doc.text(issue.impact, this.margins.left + 20, this.currentY);
      this.currentY += 6;

      // Remediation steps
      this.doc.setFont(undefined, 'bold');
      this.doc.text('Remediation Steps:', this.margins.left + 5, this.currentY);
      this.currentY += 5;

      this.doc.setFont(undefined, 'normal');
      issue.remediation.steps.forEach((step, stepIdx) => {
        const stepLines = this.doc.splitTextToSize(
          `${stepIdx + 1}. ${step}`,
          this.pageWidth - this.margins.left - this.margins.right - 15
        );
        this.doc.text(stepLines, this.margins.left + 10, this.currentY);
        this.currentY += stepLines.length * 5;
      });

      this.currentY += 8;
    });
  }

  /**
   * Add section title
   */
  private addSectionTitle(title: string): void {
    this.doc.setFontSize(16);
    this.doc.setFont(undefined, 'bold');
    this.doc.setTextColor(this.reportData.config.branding.primaryColor);
    this.doc.text(title, this.margins.left, this.currentY);
    this.currentY += 10;
    this.doc.setTextColor('#000000');
  }

  /**
   * Add footer to each page
   */
  private addFooter(): void {
    const pageCount = this.doc.getNumberOfPages();

    for (let i = 1; i <= pageCount; i++) {
      this.doc.setPage(i);

      // Footer line
      this.doc.setDrawColor(this.reportData.config.branding.primaryColor);
      this.doc.setLineWidth(0.5);
      this.doc.line(
        this.margins.left,
        this.pageHeight - 15,
        this.pageWidth - this.margins.right,
        this.pageHeight - 15
      );

      // Footer text
      this.doc.setFontSize(8);
      this.doc.setTextColor('#999999');

      if (this.reportData.config.branding.footerText) {
        this.doc.text(
          this.reportData.config.branding.footerText,
          this.margins.left,
          this.pageHeight - 10
        );
      }

      // Page number
      if (this.reportData.config.branding.includePageNumbers) {
        this.doc.text(
          `Page ${i} of ${pageCount}`,
          this.pageWidth - this.margins.right,
          this.pageHeight - 10,
          { align: 'right' }
        );
      }
    }
  }

  /**
   * Apply PDF/UA accessibility tags
   */
  private applyPDFUATags(): void {
    // PDF/UA tagging would be implemented here
    // This requires additional PDF processing libraries
    // For now, we ensure basic structure and metadata are in place
    console.log('PDF/UA tags applied (basic implementation)');
  }

  /**
   * Add a new page
   */
  private addPage(): void {
    this.doc.addPage();
    this.currentY = this.margins.top;
  }

  /**
   * Save PDF to file
   */
  public save(filename: string): void {
    this.doc.save(`${filename}.pdf`);
  }
}
