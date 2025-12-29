/**
 * Word Document Accessibility Analyzer
 * Analyzes Microsoft Word (.docx) documents for accessibility compliance
 */

import type {
  OfficeAnalysisResult,
  DocumentType,
  AccessibilityIssue,
  DocumentMetadata,
  HeadingInfo,
  ImageInfo,
  TableInfo,
  ListInfo,
  LinkInfo,
  AccessibilitySeverity,
  WCAGLevel
} from '../types/index.js';
import { generateIssueId } from '../utils/documentUtils.js';

export class WordAnalyzer {
  private issues: AccessibilityIssue[] = [];

  /**
   * Analyze Word document for accessibility
   */
  async analyze(file: File): Promise<OfficeAnalysisResult> {
    this.issues = [];

    try {
      const arrayBuffer = await file.arrayBuffer();
      const content = await this.parseWordDocument(arrayBuffer);

      // Extract document components
      const metadata = await this.extractMetadata(content);
      const hasStyles = await this.checkStyles(content);
      const headings = await this.analyzeHeadings(content);
      const hasHeadings = headings.length > 0;
      const hasTableOfContents = await this.checkTableOfContents(content);
      const images = await this.analyzeImages(content);
      const hasAltText = images.every(img => img.hasAltText);
      const tables = await this.analyzeTables(content);
      const lists = await this.analyzeLists(content);
      const links = await this.analyzeLinks(content);
      const language = metadata.language;

      // Validate accessibility requirements
      await this.validateAccessibility(content, metadata);

      return {
        documentType: DocumentType.WORD,
        hasStyles,
        hasHeadings,
        hasTableOfContents,
        hasAltText,
        language,
        metadata,
        headings,
        images,
        tables,
        lists,
        links
      };
    } catch (error) {
      throw new Error(`Word analysis failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Get all accessibility issues found
   */
  getIssues(): AccessibilityIssue[] {
    return this.issues;
  }

  /**
   * Parse Word document (DOCX)
   */
  private async parseWordDocument(arrayBuffer: ArrayBuffer): Promise<any> {
    // In production, use mammoth.js or docx library
    // This returns a parsed representation of the document
    return {
      document: {},
      styles: [],
      numbering: [],
      relationships: []
    };
  }

  /**
   * Extract document metadata
   */
  private async extractMetadata(content: any): Promise<DocumentMetadata> {
    const metadata: DocumentMetadata = {
      title: content.document?.title,
      author: content.document?.author,
      subject: content.document?.subject,
      keywords: content.document?.keywords?.split(',').map((k: string) => k.trim()),
      language: content.document?.language,
      creator: content.document?.creator,
      creationDate: content.document?.created ? new Date(content.document.created) : undefined,
      modificationDate: content.document?.modified ? new Date(content.document.modified) : undefined
    };

    // Validate metadata
    if (!metadata.title) {
      this.addIssue({
        type: 'word_missing_title',
        title: 'Missing document title',
        description: 'Word document lacks a title in document properties.',
        severity: AccessibilitySeverity.WARNING,
        wcagLevel: WCAGLevel.A,
        wcagCriteria: ['2.4.2'],
        remediation: {
          action: 'Add document title',
          description: 'Set the document title in File > Info > Properties.',
          steps: [
            'Go to File > Info',
            'Click Properties > Advanced Properties',
            'Enter a descriptive title in the Summary tab'
          ],
          automated: false,
          estimatedEffort: 'low',
          priority: 2
        }
      });
    }

    if (!metadata.language) {
      this.addIssue({
        type: 'word_missing_language',
        title: 'Missing document language',
        description: 'Document language not specified.',
        severity: AccessibilitySeverity.ERROR,
        wcagLevel: WCAGLevel.A,
        wcagCriteria: ['3.1.1'],
        remediation: {
          action: 'Set document language',
          description: 'Specify the document language for proper text-to-speech.',
          steps: [
            'Go to Review > Language > Set Proofing Language',
            'Select the appropriate language',
            'Check "Set as Default" if needed'
          ],
          automated: false,
          estimatedEffort: 'low',
          priority: 2
        }
      });
    }

    return metadata;
  }

  /**
   * Check for proper use of styles
   */
  private async checkStyles(content: any): Promise<boolean> {
    const hasStyles = content.styles && content.styles.length > 0;

    if (!hasStyles) {
      this.addIssue({
        type: 'word_no_styles',
        title: 'No styles used',
        description: 'Document does not use built-in styles, making it harder to navigate.',
        severity: AccessibilitySeverity.WARNING,
        wcagLevel: WCAGLevel.AA,
        wcagCriteria: ['1.3.1'],
        remediation: {
          action: 'Use built-in styles',
          description: 'Apply Heading and other built-in styles to structure content.',
          steps: [
            'Select text to format',
            'Choose appropriate style from Home > Styles',
            'Use Heading 1-6 for headings, Normal for body text'
          ],
          automated: false,
          estimatedEffort: 'medium',
          priority: 3
        }
      });
    }

    return hasStyles;
  }

  /**
   * Analyze heading structure
   */
  private async analyzeHeadings(content: any): Promise<HeadingInfo[]> {
    const headings: HeadingInfo[] = [];

    // In production, extract actual headings from document
    // Example:
    // Parse paragraphs with heading styles
    // Extract text, level, and position

    if (headings.length === 0) {
      this.addIssue({
        type: 'word_no_headings',
        title: 'No headings found',
        description: 'Document has no headings, making navigation difficult.',
        severity: AccessibilitySeverity.ERROR,
        wcagLevel: WCAGLevel.A,
        wcagCriteria: ['1.3.1', '2.4.6'],
        remediation: {
          action: 'Add headings',
          description: 'Use Heading styles to create document structure.',
          steps: [
            'Identify major sections and subsections',
            'Apply Heading 1 for main sections',
            'Apply Heading 2-6 for subsections',
            'Maintain proper hierarchy (don\'t skip levels)'
          ],
          automated: false,
          estimatedEffort: 'medium',
          priority: 2
        }
      });
    } else {
      // Validate heading hierarchy
      this.validateHeadingHierarchy(headings);
    }

    return headings;
  }

  /**
   * Validate heading hierarchy
   */
  private validateHeadingHierarchy(headings: HeadingInfo[]): void {
    if (headings.length === 0) return;

    // Check if first heading is H1
    if (headings[0].level !== 1) {
      this.addIssue({
        type: 'word_heading_hierarchy',
        title: 'Improper heading hierarchy',
        description: `Document should start with Heading 1, found Heading ${headings[0].level}.`,
        severity: AccessibilitySeverity.WARNING,
        wcagLevel: WCAGLevel.AA,
        wcagCriteria: ['1.3.1'],
        remediation: {
          action: 'Fix heading hierarchy',
          description: 'Start document with Heading 1 and maintain logical hierarchy.',
          steps: [
            'Change first heading to Heading 1',
            'Ensure headings follow logical order',
            'Don\'t skip heading levels'
          ],
          automated: false,
          estimatedEffort: 'low',
          priority: 3
        }
      });
    }

    // Check for skipped levels
    for (let i = 1; i < headings.length; i++) {
      if (headings[i].level > headings[i - 1].level + 1) {
        this.addIssue({
          type: 'word_heading_skip',
          title: 'Heading level skipped',
          description: `Heading level skipped from ${headings[i - 1].level} to ${headings[i].level} at "${headings[i].text}".`,
          severity: AccessibilitySeverity.WARNING,
          wcagLevel: WCAGLevel.AA,
          wcagCriteria: ['1.3.1'],
          remediation: {
            action: 'Fix heading level',
            description: 'Don\'t skip heading levels in hierarchy.',
            steps: [
              'Identify skipped levels',
              'Adjust heading levels to maintain logical order'
            ],
            automated: false,
            estimatedEffort: 'low',
            priority: 3
          }
        });
      }
    }
  }

  /**
   * Check for table of contents
   */
  private async checkTableOfContents(content: any): Promise<boolean> {
    // In production, check for actual TOC field
    return false;
  }

  /**
   * Analyze images
   */
  private async analyzeImages(content: any): Promise<ImageInfo[]> {
    const images: ImageInfo[] = [];

    // In production, extract images from document
    // Check for alt text in image properties

    images.forEach((image, index) => {
      if (!image.hasAltText && !image.isDecorative) {
        this.addIssue({
          type: 'word_image_no_alt',
          title: `Image ${index + 1} missing alternative text`,
          description: 'Image lacks alternative text for screen readers.',
          severity: AccessibilitySeverity.CRITICAL,
          wcagLevel: WCAGLevel.A,
          wcagCriteria: ['1.1.1'],
          remediation: {
            action: 'Add alternative text',
            description: 'Provide descriptive alternative text for images.',
            steps: [
              'Right-click image and select "Edit Alt Text"',
              'Enter descriptive text in the Alt Text pane',
              'For decorative images, check "Mark as decorative"'
            ],
            automated: false,
            estimatedEffort: 'low',
            priority: 1
          }
        });
      }
    });

    return images;
  }

  /**
   * Analyze tables
   */
  private async analyzeTables(content: any): Promise<TableInfo[]> {
    const tables: TableInfo[] = [];

    // In production, extract tables and check structure

    tables.forEach((table, index) => {
      if (!table.hasHeaders) {
        this.addIssue({
          type: 'word_table_no_headers',
          title: `Table ${index + 1} missing header row`,
          description: 'Table lacks designated header row.',
          severity: AccessibilitySeverity.ERROR,
          wcagLevel: WCAGLevel.A,
          wcagCriteria: ['1.3.1'],
          remediation: {
            action: 'Add table headers',
            description: 'Designate the first row as header row.',
            steps: [
              'Click in table',
              'Go to Table Design tab',
              'Check "Header Row" in Table Style Options'
            ],
            automated: false,
            estimatedEffort: 'low',
            priority: 2
          }
        });
      }

      if (!table.hasCaption) {
        this.addIssue({
          type: 'word_table_no_caption',
          title: `Table ${index + 1} missing caption`,
          description: 'Table should have a descriptive caption.',
          severity: AccessibilitySeverity.WARNING,
          wcagLevel: WCAGLevel.AA,
          wcagCriteria: ['1.3.1'],
          remediation: {
            action: 'Add table caption',
            description: 'Insert a caption above the table.',
            steps: [
              'Right-click table and select "Insert Caption"',
              'Enter descriptive caption text',
              'Position above table'
            ],
            automated: false,
            estimatedEffort: 'low',
            priority: 3
          }
        });
      }
    });

    return tables;
  }

  /**
   * Analyze lists
   */
  private async analyzeLists(content: any): Promise<ListInfo[]> {
    const lists: ListInfo[] = [];
    // In production, extract lists from document
    return lists;
  }

  /**
   * Analyze links
   */
  private async analyzeLinks(content: any): Promise<LinkInfo[]> {
    const links: LinkInfo[] = [];

    // In production, extract hyperlinks
    links.forEach((link, index) => {
      if (!link.text || link.text.trim().length === 0) {
        this.addIssue({
          type: 'word_link_no_text',
          title: `Link ${index + 1} missing descriptive text`,
          description: 'Hyperlink lacks meaningful text.',
          severity: AccessibilitySeverity.ERROR,
          wcagLevel: WCAGLevel.A,
          wcagCriteria: ['2.4.4'],
          remediation: {
            action: 'Add link text',
            description: 'Use descriptive text for hyperlinks.',
            steps: [
              'Select the link',
              'Replace URL with descriptive text',
              'Avoid generic text like "click here"'
            ],
            automated: false,
            estimatedEffort: 'low',
            priority: 2
          }
        });
      }
    });

    return links;
  }

  /**
   * Validate overall accessibility
   */
  private async validateAccessibility(content: any, metadata: DocumentMetadata): Promise<void> {
    // Run Word's built-in accessibility checker if available
    // Validate color contrast, reading order, etc.
  }

  /**
   * Add accessibility issue
   */
  private addIssue(issue: Omit<AccessibilityIssue, 'id' | 'timestamp'>): void {
    this.issues.push({
      ...issue,
      id: generateIssueId(issue.type),
      timestamp: new Date()
    });
  }
}
