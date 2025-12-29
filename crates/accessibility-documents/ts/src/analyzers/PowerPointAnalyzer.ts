/**
 * PowerPoint Presentation Accessibility Analyzer
 * Analyzes Microsoft PowerPoint (.pptx) presentations for accessibility compliance
 */

import type {
  OfficeAnalysisResult,
  DocumentType,
  AccessibilityIssue,
  DocumentMetadata,
  ImageInfo,
  TableInfo,
  AccessibilitySeverity,
  WCAGLevel
} from '../types/index.js';
import { generateIssueId } from '../utils/documentUtils.js';

export class PowerPointAnalyzer {
  private issues: AccessibilityIssue[] = [];

  /**
   * Analyze PowerPoint presentation for accessibility
   */
  async analyze(file: File): Promise<OfficeAnalysisResult> {
    this.issues = [];

    try {
      const arrayBuffer = await file.arrayBuffer();
      const presentation = await this.parsePowerPointDocument(arrayBuffer);

      // Extract document components
      const metadata = await this.extractMetadata(presentation);
      const images = await this.analyzeImages(presentation);
      const hasAltText = images.every(img => img.hasAltText);
      const tables = await this.analyzeTables(presentation);

      // Validate accessibility requirements
      await this.validateAccessibility(presentation);

      return {
        documentType: DocumentType.POWERPOINT,
        hasStyles: true,
        hasHeadings: true, // Slides have title placeholders
        hasTableOfContents: false,
        hasAltText,
        language: metadata.language,
        metadata,
        headings: [],
        images,
        tables,
        lists: [],
        links: []
      };
    } catch (error) {
      throw new Error(`PowerPoint analysis failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Get all accessibility issues found
   */
  getIssues(): AccessibilityIssue[] {
    return this.issues;
  }

  /**
   * Parse PowerPoint document (PPTX)
   */
  private async parsePowerPointDocument(arrayBuffer: ArrayBuffer): Promise<any> {
    // In production, use pptxgenjs or similar library to parse PPTX
    return {
      slides: [],
      metadata: {},
      layouts: [],
      masters: []
    };
  }

  /**
   * Extract document metadata
   */
  private async extractMetadata(presentation: any): Promise<DocumentMetadata> {
    const metadata: DocumentMetadata = {
      title: presentation.metadata?.title,
      author: presentation.metadata?.author,
      subject: presentation.metadata?.subject,
      keywords: presentation.metadata?.keywords?.split(',').map((k: string) => k.trim()),
      language: presentation.metadata?.language,
      creator: presentation.metadata?.creator,
      creationDate: presentation.metadata?.created ? new Date(presentation.metadata.created) : undefined,
      modificationDate: presentation.metadata?.modified ? new Date(presentation.metadata.modified) : undefined
    };

    // Validate metadata
    if (!metadata.title) {
      this.addIssue({
        type: 'ppt_missing_title',
        title: 'Missing presentation title',
        description: 'PowerPoint presentation lacks a title.',
        severity: AccessibilitySeverity.WARNING,
        wcagLevel: WCAGLevel.A,
        wcagCriteria: ['2.4.2'],
        remediation: {
          action: 'Add presentation title',
          description: 'Set the presentation title in File > Info > Properties.',
          steps: [
            'Go to File > Info',
            'Click Properties > Advanced Properties',
            'Enter a descriptive title'
          ],
          automated: false,
          estimatedEffort: 'low',
          priority: 2
        }
      });
    }

    if (!metadata.language) {
      this.addIssue({
        type: 'ppt_missing_language',
        title: 'Missing presentation language',
        description: 'Presentation language not specified.',
        severity: AccessibilitySeverity.ERROR,
        wcagLevel: WCAGLevel.A,
        wcagCriteria: ['3.1.1'],
        remediation: {
          action: 'Set presentation language',
          description: 'Specify the language in File > Options > Language.',
          steps: [
            'Go to File > Options > Language',
            'Set Office display and authoring languages',
            'Select appropriate language'
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
   * Analyze images and graphics
   */
  private async analyzeImages(presentation: any): Promise<ImageInfo[]> {
    const images: ImageInfo[] = [];

    // In production, extract images from all slides
    presentation.slides?.forEach((slide: any, slideIndex: number) => {
      // Extract shapes, images, charts from slide
      // Example:
      slide.shapes?.forEach((shape: any) => {
        if (shape.type === 'image' || shape.type === 'picture') {
          const image: ImageInfo = {
            id: shape.id || `img-${slideIndex}-${images.length}`,
            page: slideIndex + 1,
            hasAltText: !!shape.altText,
            altText: shape.altText,
            width: shape.width || 0,
            height: shape.height || 0,
            format: shape.format,
            isDecorative: shape.isDecorative
          };

          if (!image.hasAltText && !image.isDecorative) {
            this.addIssue({
              type: 'ppt_image_no_alt',
              title: `Slide ${slideIndex + 1}: Image missing alternative text`,
              description: 'Image lacks alternative text.',
              severity: AccessibilitySeverity.CRITICAL,
              wcagLevel: WCAGLevel.A,
              wcagCriteria: ['1.1.1'],
              pageNumber: slideIndex + 1,
              remediation: {
                action: 'Add alternative text',
                description: 'Provide descriptive alternative text for images.',
                steps: [
                  'Right-click image',
                  'Select "Edit Alt Text"',
                  'Enter descriptive text',
                  'For decorative images, check "Mark as decorative"'
                ],
                automated: false,
                estimatedEffort: 'low',
                priority: 1
              }
            });
          }

          images.push(image);
        }
      });
    });

    return images;
  }

  /**
   * Analyze tables
   */
  private async analyzeTables(presentation: any): Promise<TableInfo[]> {
    const tables: TableInfo[] = [];

    presentation.slides?.forEach((slide: any, slideIndex: number) => {
      slide.tables?.forEach((table: any, tableIndex: number) => {
        const tableInfo: TableInfo = {
          id: `table-${slideIndex}-${tableIndex}`,
          rowCount: table.rows?.length || 0,
          columnCount: table.columns?.length || 0,
          hasHeaders: table.hasHeaderRow || false,
          hasCaption: false,
          page: slideIndex + 1,
          headerCells: [],
          isAccessible: table.hasHeaderRow || false
        };

        if (!tableInfo.hasHeaders) {
          this.addIssue({
            type: 'ppt_table_no_headers',
            title: `Slide ${slideIndex + 1}: Table missing header row`,
            description: 'Table lacks designated header row.',
            severity: AccessibilitySeverity.ERROR,
            wcagLevel: WCAGLevel.A,
            wcagCriteria: ['1.3.1'],
            pageNumber: slideIndex + 1,
            remediation: {
              action: 'Add table headers',
              description: 'Designate the first row as header row.',
              steps: [
                'Select table',
                'Go to Table Design tab',
                'Check "Header Row" option'
              ],
              automated: false,
              estimatedEffort: 'low',
              priority: 2
            }
          });
        }

        tables.push(tableInfo);
      });
    });

    return tables;
  }

  /**
   * Validate overall accessibility
   */
  private async validateAccessibility(presentation: any): Promise<void> {
    // Check slide reading order
    presentation.slides?.forEach((slide: any, index: number) => {
      this.validateSlideStructure(slide, index + 1);
    });

    // Check for sufficient color contrast
    this.checkColorContrast(presentation);
  }

  /**
   * Validate individual slide structure
   */
  private validateSlideStructure(slide: any, slideNumber: number): void {
    // Check if slide has a title
    const hasTitle = slide.title && slide.title.trim().length > 0;

    if (!hasTitle) {
      this.addIssue({
        type: 'ppt_slide_no_title',
        title: `Slide ${slideNumber} missing title`,
        description: 'Every slide should have a unique, descriptive title.',
        severity: AccessibilitySeverity.ERROR,
        wcagLevel: WCAGLevel.A,
        wcagCriteria: ['2.4.2', '2.4.6'],
        pageNumber: slideNumber,
        remediation: {
          action: 'Add slide title',
          description: 'Ensure each slide has a descriptive title.',
          steps: [
            'Click in the title placeholder',
            'Enter a descriptive title',
            'For title slides without visible title, add title off-screen'
          ],
          automated: false,
          estimatedEffort: 'low',
          priority: 1
        }
      });
    }

    // Check reading order
    const hasCustomReadingOrder = slide.readingOrder && slide.readingOrder.length > 0;
    if (!hasCustomReadingOrder && slide.shapes?.length > 3) {
      this.addIssue({
        type: 'ppt_reading_order',
        title: `Slide ${slideNumber} may have incorrect reading order`,
        description: 'Complex slide may need custom reading order.',
        severity: AccessibilitySeverity.WARNING,
        wcagLevel: WCAGLevel.A,
        wcagCriteria: ['1.3.2'],
        pageNumber: slideNumber,
        remediation: {
          action: 'Check reading order',
          description: 'Verify and set correct reading order for slide objects.',
          steps: [
            'Go to Home > Arrange > Selection Pane',
            'Verify objects are in correct reading order (bottom to top)',
            'Drag to reorder if needed'
          ],
          automated: false,
          estimatedEffort: 'low',
          priority: 3
        }
      });
    }

    // Check for text in images
    if (slide.textImages && slide.textImages.length > 0) {
      this.addIssue({
        type: 'ppt_text_in_image',
        title: `Slide ${slideNumber} contains text in images`,
        description: 'Text embedded in images is not accessible.',
        severity: AccessibilitySeverity.ERROR,
        wcagLevel: WCAGLevel.A,
        wcagCriteria: ['1.4.5'],
        pageNumber: slideNumber,
        remediation: {
          action: 'Use real text instead of images',
          description: 'Replace images of text with actual text.',
          steps: [
            'Identify images containing text',
            'Replace with text boxes',
            'If image must be used, provide comprehensive alt text'
          ],
          automated: false,
          estimatedEffort: 'medium',
          priority: 2
        }
      });
    }

    // Check for animations that auto-play
    if (slide.animations?.some((a: any) => a.autoStart)) {
      this.addIssue({
        type: 'ppt_auto_animation',
        title: `Slide ${slideNumber} has auto-playing animations`,
        description: 'Auto-playing animations can be disorienting.',
        severity: AccessibilitySeverity.WARNING,
        wcagLevel: WCAGLevel.AA,
        wcagCriteria: ['2.2.2'],
        pageNumber: slideNumber,
        remediation: {
          action: 'Remove auto-start animations',
          description: 'Set animations to start on click.',
          steps: [
            'Go to Animations tab',
            'Select animation',
            'Change Start setting to "On Click"'
          ],
          automated: false,
          estimatedEffort: 'low',
          priority: 3
        }
      });
    }
  }

  /**
   * Check color contrast
   */
  private checkColorContrast(presentation: any): void {
    // In production, analyze actual colors used
    // This is a complex check requiring color analysis

    this.addIssue({
      type: 'ppt_color_contrast',
      title: 'Review color contrast',
      description: 'Ensure sufficient contrast between text and background.',
      severity: AccessibilitySeverity.INFO,
      wcagLevel: WCAGLevel.AA,
      wcagCriteria: ['1.4.3', '1.4.6'],
      remediation: {
        action: 'Check color contrast',
        description: 'Verify all text meets WCAG contrast requirements.',
        steps: [
          'Use PowerPoint Accessibility Checker',
          'Test contrast ratios (4.5:1 for normal text, 3:1 for large)',
          'Adjust colors as needed',
          'Avoid relying on color alone to convey information'
        ],
        automated: true,
        estimatedEffort: 'medium',
        priority: 3
      }
    });
  }

  /**
   * Add accessibility issue
   */
  private addIssue(issue: Omit<AccessibilityIssue, 'id' | 'timestamp'>): void {
    this.issues.push({
      ...issue,
      id: generateIssueId(issue.type, issue.pageNumber?.toString()),
      timestamp: new Date()
    });
  }
}
