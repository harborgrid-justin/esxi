/**
 * EPUB Accessibility Analyzer
 * Analyzes EPUB documents for accessibility compliance according to EPUB Accessibility 1.1
 */

import type {
  EPUBAnalysisResult,
  AccessibilityIssue,
  DocumentMetadata,
  ContentDocumentInfo,
  NavigationInfo,
  MediaOverlayInfo,
  AccessibilitySeverity,
  WCAGLevel
} from '../types/index.js';
import { generateIssueId } from '../utils/documentUtils.js';

export class EPUBAnalyzer {
  private issues: AccessibilityIssue[] = [];

  /**
   * Analyze EPUB document for accessibility
   */
  async analyze(file: File): Promise<EPUBAnalysisResult> {
    this.issues = [];

    try {
      const arrayBuffer = await file.arrayBuffer();
      const epub = await this.parseEPUBDocument(arrayBuffer);

      // Extract EPUB components
      const version = await this.getEPUBVersion(epub);
      const metadata = await this.extractMetadata(epub);
      const hasNavigation = await this.checkNavigation(epub);
      const hasSemantics = await this.checkSemantics(epub);
      const hasAccessibilityMetadata = await this.checkAccessibilityMetadata(epub);
      const contentDocuments = await this.analyzeContentDocuments(epub);
      const navigation = await this.extractNavigation(epub);
      const mediaOverlays = await this.extractMediaOverlays(epub);

      // Validate EPUB accessibility
      await this.validateAccessibility(epub, metadata);

      return {
        version,
        hasNavigation,
        hasSemantics,
        hasAccessibilityMetadata,
        metadata,
        contentDocuments,
        navigation,
        mediaOverlays
      };
    } catch (error) {
      throw new Error(`EPUB analysis failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Get all accessibility issues found
   */
  getIssues(): AccessibilityIssue[] {
    return this.issues;
  }

  /**
   * Parse EPUB document
   */
  private async parseEPUBDocument(arrayBuffer: ArrayBuffer): Promise<any> {
    // In production, use JSZip to extract EPUB (which is a ZIP file)
    // Parse OPF file, content documents, navigation, etc.
    return {
      opf: {},
      contentDocuments: [],
      navigation: null,
      toc: null,
      metadata: {}
    };
  }

  /**
   * Get EPUB version
   */
  private async getEPUBVersion(epub: any): Promise<string> {
    return epub.opf?.version || '3.0';
  }

  /**
   * Extract metadata
   */
  private async extractMetadata(epub: any): Promise<DocumentMetadata> {
    const opfMetadata = epub.opf?.metadata || {};

    const metadata: DocumentMetadata = {
      title: opfMetadata['dc:title'],
      author: opfMetadata['dc:creator'],
      subject: opfMetadata['dc:subject'],
      language: opfMetadata['dc:language'],
      keywords: opfMetadata['dc:subject']?.split(',').map((k: string) => k.trim()),
      creator: opfMetadata['dc:publisher'],
      creationDate: opfMetadata['dc:date'] ? new Date(opfMetadata['dc:date']) : undefined
    };

    // Validate required metadata
    if (!metadata.title) {
      this.addIssue({
        type: 'epub_missing_title',
        title: 'Missing EPUB title',
        description: 'EPUB lacks a title in metadata.',
        severity: AccessibilitySeverity.ERROR,
        wcagLevel: WCAGLevel.A,
        wcagCriteria: ['2.4.2'],
        remediation: {
          action: 'Add title metadata',
          description: 'Add dc:title to OPF metadata.',
          steps: [
            'Open OPF file',
            'Add <dc:title> element in metadata section',
            'Provide descriptive title'
          ],
          automated: false,
          estimatedEffort: 'low',
          priority: 1
        }
      });
    }

    if (!metadata.language) {
      this.addIssue({
        type: 'epub_missing_language',
        title: 'Missing language declaration',
        description: 'EPUB does not declare primary language.',
        severity: AccessibilitySeverity.CRITICAL,
        wcagLevel: WCAGLevel.A,
        wcagCriteria: ['3.1.1'],
        remediation: {
          action: 'Add language metadata',
          description: 'Declare the primary language in OPF metadata.',
          steps: [
            'Open OPF file',
            'Add <dc:language> element (e.g., "en", "es", "fr")',
            'Ensure content documents also declare language'
          ],
          automated: false,
          estimatedEffort: 'low',
          priority: 1
        }
      });
    }

    return metadata;
  }

  /**
   * Check for navigation document
   */
  private async checkNavigation(epub: any): Promise<boolean> {
    const hasNav = !!epub.navigation;

    if (!hasNav) {
      this.addIssue({
        type: 'epub_missing_navigation',
        title: 'Missing navigation document',
        description: 'EPUB 3 requires a navigation document.',
        severity: AccessibilitySeverity.CRITICAL,
        wcagLevel: WCAGLevel.A,
        wcagCriteria: ['2.4.5'],
        remediation: {
          action: 'Add navigation document',
          description: 'Create a navigation document (nav.xhtml) with table of contents.',
          steps: [
            'Create nav.xhtml file',
            'Add <nav epub:type="toc"> with nested list',
            'Reference in OPF with properties="nav"',
            'Include page-list and landmarks if applicable'
          ],
          automated: false,
          estimatedEffort: 'medium',
          priority: 1
        }
      });
    }

    return hasNav;
  }

  /**
   * Check for semantic markup
   */
  private async checkSemantics(epub: any): Promise<boolean> {
    let hasSemantics = false;

    // Check if content documents use semantic HTML and ARIA
    epub.contentDocuments?.forEach((doc: any) => {
      if (doc.semanticElements && doc.semanticElements.length > 0) {
        hasSemantics = true;
      }
    });

    if (!hasSemantics) {
      this.addIssue({
        type: 'epub_no_semantics',
        title: 'Lack of semantic markup',
        description: 'Content documents should use semantic HTML5 elements.',
        severity: AccessibilitySeverity.WARNING,
        wcagLevel: WCAGLevel.AA,
        wcagCriteria: ['1.3.1'],
        remediation: {
          action: 'Add semantic markup',
          description: 'Use semantic HTML5 elements and epub:type attributes.',
          steps: [
            'Use <section>, <article>, <nav>, <aside> elements',
            'Add epub:type attributes (e.g., epub:type="chapter")',
            'Use proper heading hierarchy',
            'Mark up lists, tables, figures appropriately'
          ],
          automated: false,
          estimatedEffort: 'high',
          priority: 3
        }
      });
    }

    return hasSemantics;
  }

  /**
   * Check for accessibility metadata
   */
  private async checkAccessibilityMetadata(epub: any): Promise<boolean> {
    const a11yMetadata = epub.opf?.metadata?.accessibility || {};

    const hasFeatures = !!a11yMetadata['schema:accessibilityFeature'];
    const hasHazards = a11yMetadata['schema:accessibilityHazard'] !== undefined;
    const hasSummary = !!a11yMetadata['schema:accessibilitySummary'];
    const hasMode = !!a11yMetadata['schema:accessMode'];

    if (!hasFeatures || !hasHazards || !hasSummary || !hasMode) {
      this.addIssue({
        type: 'epub_missing_a11y_metadata',
        title: 'Missing accessibility metadata',
        description: 'EPUB should include schema.org accessibility metadata.',
        severity: AccessibilitySeverity.WARNING,
        wcagLevel: WCAGLevel.AA,
        remediation: {
          action: 'Add accessibility metadata',
          description: 'Add schema.org accessibility properties to OPF metadata.',
          steps: [
            'Add schema:accessibilityFeature (e.g., "alternativeText", "structuralNavigation")',
            'Add schema:accessibilityHazard (e.g., "none" or specific hazards)',
            'Add schema:accessibilitySummary with description',
            'Add schema:accessMode and schema:accessModeSufficient'
          ],
          automated: false,
          estimatedEffort: 'medium',
          priority: 3,
          codeExample: `<meta property="schema:accessibilityFeature">alternativeText</meta>
<meta property="schema:accessibilityHazard">none</meta>
<meta property="schema:accessibilitySummary">This publication includes alternative text for images and proper heading structure.</meta>`
        }
      });
    }

    return hasFeatures && hasHazards && hasSummary && hasMode;
  }

  /**
   * Analyze content documents
   */
  private async analyzeContentDocuments(epub: any): Promise<ContentDocumentInfo[]> {
    const documents: ContentDocumentInfo[] = [];

    epub.contentDocuments?.forEach((doc: any, index: number) => {
      const docInfo: ContentDocumentInfo = {
        href: doc.href,
        title: doc.title,
        hasHeadings: doc.headings && doc.headings.length > 0,
        hasLandmarks: doc.landmarks && doc.landmarks.length > 0,
        hasAltText: doc.images ? doc.images.every((img: any) => img.hasAlt) : true,
        language: doc.language
      };

      // Check for headings
      if (!docInfo.hasHeadings) {
        this.addIssue({
          type: 'epub_doc_no_headings',
          title: `Document "${doc.href}" lacks headings`,
          description: 'Content document should have proper heading structure.',
          severity: AccessibilitySeverity.WARNING,
          wcagLevel: WCAGLevel.AA,
          wcagCriteria: ['1.3.1', '2.4.6'],
          remediation: {
            action: 'Add headings',
            description: 'Use h1-h6 elements to structure content.',
            steps: [
              'Identify major sections',
              'Add heading elements in logical hierarchy',
              'Ensure each document starts with h1'
            ],
            automated: false,
            estimatedEffort: 'medium',
            priority: 2
          }
        });
      }

      // Check for alt text on images
      if (!docInfo.hasAltText) {
        this.addIssue({
          type: 'epub_doc_missing_alt',
          title: `Document "${doc.href}" has images without alt text`,
          description: 'All images must have alternative text.',
          severity: AccessibilitySeverity.CRITICAL,
          wcagLevel: WCAGLevel.A,
          wcagCriteria: ['1.1.1'],
          remediation: {
            action: 'Add alt text to images',
            description: 'Provide alt attributes for all img elements.',
            steps: [
              'Find all <img> elements',
              'Add alt="" for decorative images',
              'Add descriptive alt="..." for meaningful images'
            ],
            automated: false,
            estimatedEffort: 'medium',
            priority: 1
          }
        });
      }

      // Check for language declaration
      if (!docInfo.language && !epub.opf?.metadata?.['dc:language']) {
        this.addIssue({
          type: 'epub_doc_no_language',
          title: `Document "${doc.href}" missing language`,
          description: 'Content document should declare language.',
          severity: AccessibilitySeverity.ERROR,
          wcagLevel: WCAGLevel.A,
          wcagCriteria: ['3.1.1'],
          remediation: {
            action: 'Add language attribute',
            description: 'Add xml:lang and lang attributes to html element.',
            steps: [
              'Add xml:lang="en" to <html> element (replace "en" with appropriate code)',
              'Add lang="en" for HTML5 compatibility'
            ],
            automated: false,
            estimatedEffort: 'low',
            priority: 2
          }
        });
      }

      documents.push(docInfo);
    });

    return documents;
  }

  /**
   * Extract navigation structures
   */
  private async extractNavigation(epub: any): Promise<NavigationInfo[]> {
    const navigation: NavigationInfo[] = [];

    // Extract table of contents
    if (epub.navigation?.toc) {
      navigation.push({
        type: 'toc',
        items: epub.navigation.toc
      });
    }

    // Extract page list if present
    if (epub.navigation?.pageList) {
      navigation.push({
        type: 'page-list',
        items: epub.navigation.pageList
      });
    }

    // Extract landmarks
    if (epub.navigation?.landmarks) {
      navigation.push({
        type: 'landmarks',
        items: epub.navigation.landmarks
      });
    } else {
      this.addIssue({
        type: 'epub_no_landmarks',
        title: 'Missing landmarks navigation',
        description: 'EPUB should include landmarks for quick navigation.',
        severity: AccessibilitySeverity.INFO,
        wcagLevel: WCAGLevel.AAA,
        remediation: {
          action: 'Add landmarks navigation',
          description: 'Create a landmarks nav in navigation document.',
          steps: [
            'Add <nav epub:type="landmarks"> to nav.xhtml',
            'Include links to major sections (cover, toc, bodymatter, etc.)',
            'Use appropriate epub:type values'
          ],
          automated: false,
          estimatedEffort: 'low',
          priority: 4
        }
      });
    }

    return navigation;
  }

  /**
   * Extract media overlays (for synchronized audio/text)
   */
  private async extractMediaOverlays(epub: any): Promise<MediaOverlayInfo[] | undefined> {
    if (!epub.mediaOverlays || epub.mediaOverlays.length === 0) {
      return undefined;
    }

    return epub.mediaOverlays;
  }

  /**
   * Validate overall accessibility
   */
  private async validateAccessibility(epub: any, metadata: DocumentMetadata): Promise<void> {
    // Check EPUB version
    const version = await this.getEPUBVersion(epub);
    if (version.startsWith('2.')) {
      this.addIssue({
        type: 'epub_old_version',
        title: 'Using EPUB 2.0',
        description: 'EPUB 3.0+ has better accessibility support.',
        severity: AccessibilitySeverity.INFO,
        wcagLevel: WCAGLevel.AA,
        remediation: {
          action: 'Consider upgrading to EPUB 3',
          description: 'EPUB 3 provides better accessibility features.',
          steps: [
            'Review EPUB 3 migration guide',
            'Update OPF to version 3.0',
            'Convert NCX to navigation document',
            'Add accessibility metadata'
          ],
          automated: false,
          estimatedEffort: 'high',
          priority: 4
        }
      });
    }

    // Check for MathML if mathematical content
    // Check for proper table markup
    // Validate CSS doesn't interfere with accessibility
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
