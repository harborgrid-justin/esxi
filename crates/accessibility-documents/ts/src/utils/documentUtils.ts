/**
 * Document Utilities
 * Helper functions for document processing and accessibility checking
 */

import type {
  DocumentType,
  AccessibilitySeverity,
  AccessibilityIssue,
  WCAGLevel,
  ComplianceScore,
  CheckerSummary
} from '../types/index.js';

/**
 * Detect document type from file extension and MIME type
 */
export function detectDocumentType(fileName: string, mimeType?: string): DocumentType {
  const extension = fileName.toLowerCase().split('.').pop();

  if (extension === 'pdf' || mimeType === 'application/pdf') {
    return DocumentType.PDF;
  }
  if (extension === 'docx' || mimeType === 'application/vnd.openxmlformats-officedocument.wordprocessingml.document') {
    return DocumentType.WORD;
  }
  if (extension === 'xlsx' || mimeType === 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet') {
    return DocumentType.EXCEL;
  }
  if (extension === 'pptx' || mimeType === 'application/vnd.openxmlformats-officedocument.presentationml.presentation') {
    return DocumentType.POWERPOINT;
  }
  if (extension === 'epub' || mimeType === 'application/epub+zip') {
    return DocumentType.EPUB;
  }

  return DocumentType.UNKNOWN;
}

/**
 * Generate unique issue ID
 */
export function generateIssueId(type: string, location?: string): string {
  const timestamp = Date.now();
  const random = Math.random().toString(36).substring(2, 9);
  const locationHash = location ? `-${hashString(location)}` : '';
  return `${type}-${timestamp}-${random}${locationHash}`;
}

/**
 * Simple string hash function
 */
function hashString(str: string): string {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // Convert to 32bit integer
  }
  return Math.abs(hash).toString(36);
}

/**
 * Calculate compliance score from issues
 */
export function calculateComplianceScore(
  issues: AccessibilityIssue[],
  totalChecks: number
): ComplianceScore {
  const criticalCount = issues.filter(i => i.severity === AccessibilitySeverity.CRITICAL).length;
  const errorCount = issues.filter(i => i.severity === AccessibilitySeverity.ERROR).length;
  const warningCount = issues.filter(i => i.severity === AccessibilitySeverity.WARNING).length;
  const infoCount = issues.filter(i => i.severity === AccessibilitySeverity.INFO).length;

  const failedChecks = criticalCount + errorCount;
  const passedChecks = totalChecks - failedChecks;

  // Calculate overall score (0-100)
  let overall = 100;
  overall -= criticalCount * 10; // Critical issues: -10 points each
  overall -= errorCount * 5;      // Errors: -5 points each
  overall -= warningCount * 2;    // Warnings: -2 points each
  overall -= infoCount * 0.5;     // Info: -0.5 points each
  overall = Math.max(0, Math.min(100, overall));

  // Calculate WCAG level scores
  const wcagAIssues = issues.filter(i => i.wcagLevel === WCAGLevel.A && i.severity !== AccessibilitySeverity.INFO);
  const wcagAAIssues = issues.filter(i => i.wcagLevel === WCAGLevel.AA && i.severity !== AccessibilitySeverity.INFO);
  const wcagAAAIssues = issues.filter(i => i.wcagLevel === WCAGLevel.AAA && i.severity !== AccessibilitySeverity.INFO);

  const wcagA = wcagAIssues.length === 0 ? 100 : Math.max(0, 100 - (wcagAIssues.length * 10));
  const wcagAA = wcagAAIssues.length === 0 ? 100 : Math.max(0, 100 - (wcagAAIssues.length * 10));
  const wcagAAA = wcagAAAIssues.length === 0 ? 100 : Math.max(0, 100 - (wcagAAAIssues.length * 10));

  // PDF/UA score (if applicable)
  const pdfuaIssues = issues.filter(i => i.pdfuaRequirement && i.severity !== AccessibilitySeverity.INFO);
  const pdfua = pdfuaIssues.length === 0 ? 100 : Math.max(0, 100 - (pdfuaIssues.length * 8));

  return {
    overall,
    pdfua,
    wcagA,
    wcagAA,
    wcagAAA,
    passedChecks,
    failedChecks,
    warningChecks: warningCount,
    totalChecks
  };
}

/**
 * Generate checker summary from issues and analysis
 */
export function generateCheckerSummary(
  issues: AccessibilityIssue[],
  hasTaggedStructure: boolean,
  hasMetadata: boolean,
  hasLanguage: boolean,
  hasAlternativeText: boolean
): CheckerSummary {
  const criticalIssues = issues.filter(i => i.severity === AccessibilitySeverity.CRITICAL).length;
  const errorIssues = issues.filter(i => i.severity === AccessibilitySeverity.ERROR).length;
  const warningIssues = issues.filter(i => i.severity === AccessibilitySeverity.WARNING).length;
  const infoIssues = issues.filter(i => i.severity === AccessibilitySeverity.INFO).length;

  const isPDFUA = criticalIssues === 0 && errorIssues === 0 && hasTaggedStructure && hasMetadata;
  const isWCAGA = issues.filter(i => i.wcagLevel === WCAGLevel.A && i.severity !== AccessibilitySeverity.INFO).length === 0;
  const isWCAGAA = isWCAGA && issues.filter(i => i.wcagLevel === WCAGLevel.AA && i.severity !== AccessibilitySeverity.INFO).length === 0;
  const isWCAGAAA = isWCAGAA && issues.filter(i => i.wcagLevel === WCAGLevel.AAA && i.severity !== AccessibilitySeverity.INFO).length === 0;

  const recommendedActions = generateRecommendedActions(issues, {
    hasTaggedStructure,
    hasMetadata,
    hasLanguage,
    hasAlternativeText
  });

  return {
    totalIssues: issues.length,
    criticalIssues,
    errorIssues,
    warningIssues,
    infoIssues,
    isPDFUA,
    isWCAGA,
    isWCAGAA,
    isWCAGAAA,
    hasTaggedStructure,
    hasMetadata,
    hasLanguage,
    hasAlternativeText,
    recommendedActions
  };
}

/**
 * Generate recommended actions based on issues
 */
function generateRecommendedActions(
  issues: AccessibilityIssue[],
  flags: {
    hasTaggedStructure: boolean;
    hasMetadata: boolean;
    hasLanguage: boolean;
    hasAlternativeText: boolean;
  }
): string[] {
  const actions: string[] = [];

  if (!flags.hasTaggedStructure) {
    actions.push('Add tagged structure to enable screen reader access');
  }

  if (!flags.hasMetadata) {
    actions.push('Add document metadata (title, language, author)');
  }

  if (!flags.hasLanguage) {
    actions.push('Specify document language for proper text-to-speech');
  }

  if (!flags.hasAlternativeText) {
    actions.push('Add alternative text to all images and figures');
  }

  const criticalIssues = issues.filter(i => i.severity === AccessibilitySeverity.CRITICAL);
  if (criticalIssues.length > 0) {
    actions.push(`Fix ${criticalIssues.length} critical accessibility issue(s)`);
  }

  const headingIssues = issues.filter(i => i.type.includes('heading'));
  if (headingIssues.length > 0) {
    actions.push('Improve heading structure and hierarchy');
  }

  const tableIssues = issues.filter(i => i.type.includes('table'));
  if (tableIssues.length > 0) {
    actions.push('Add table headers and proper structure');
  }

  const formIssues = issues.filter(i => i.type.includes('form'));
  if (formIssues.length > 0) {
    actions.push('Add labels and descriptions to form fields');
  }

  const readingOrderIssues = issues.filter(i => i.type.includes('reading_order'));
  if (readingOrderIssues.length > 0) {
    actions.push('Fix reading order for logical content flow');
  }

  return actions.slice(0, 5); // Return top 5 actions
}

/**
 * Format file size for display
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
}

/**
 * Format duration in milliseconds to readable string
 */
export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`;
}

/**
 * Validate alt text quality
 */
export function validateAltTextQuality(altText: string): {
  valid: boolean;
  issues: string[];
} {
  const issues: string[] = [];

  if (!altText || altText.trim().length === 0) {
    return { valid: false, issues: ['Alt text is empty'] };
  }

  // Check for common bad practices
  if (altText.toLowerCase().includes('image of') || altText.toLowerCase().includes('picture of')) {
    issues.push('Avoid redundant phrases like "image of" or "picture of"');
  }

  if (altText.length < 5) {
    issues.push('Alt text is too short (minimum 5 characters recommended)');
  }

  if (altText.length > 150) {
    issues.push('Alt text is too long (maximum 150 characters recommended)');
  }

  if (altText === altText.toUpperCase() && altText.length > 10) {
    issues.push('Avoid all-caps text');
  }

  // Check for file names
  if (/\.(jpg|jpeg|png|gif|svg|webp)$/i.test(altText)) {
    issues.push('Alt text appears to be a file name');
  }

  return {
    valid: issues.length === 0,
    issues
  };
}

/**
 * Extract text content from HTML
 */
export function extractTextContent(html: string): string {
  // Remove script and style tags
  let text = html.replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '');
  text = text.replace(/<style\b[^<]*(?:(?!<\/style>)<[^<]*)*<\/style>/gi, '');

  // Remove HTML tags
  text = text.replace(/<[^>]+>/g, ' ');

  // Decode HTML entities
  text = text
    .replace(/&nbsp;/g, ' ')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&amp;/g, '&')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'");

  // Clean up whitespace
  text = text.replace(/\s+/g, ' ').trim();

  return text;
}

/**
 * Check if color contrast meets WCAG requirements
 */
export function checkColorContrast(
  foreground: string,
  background: string,
  level: WCAGLevel = WCAGLevel.AA,
  isLargeText: boolean = false
): { pass: boolean; ratio: number; required: number } {
  const fgLuminance = getRelativeLuminance(foreground);
  const bgLuminance = getRelativeLuminance(background);

  const ratio = (Math.max(fgLuminance, bgLuminance) + 0.05) /
                (Math.min(fgLuminance, bgLuminance) + 0.05);

  let required = 4.5; // WCAG AA normal text
  if (level === WCAGLevel.AAA) {
    required = isLargeText ? 4.5 : 7;
  } else if (level === WCAGLevel.AA) {
    required = isLargeText ? 3 : 4.5;
  }

  return {
    pass: ratio >= required,
    ratio: Math.round(ratio * 100) / 100,
    required
  };
}

/**
 * Calculate relative luminance for color contrast
 */
function getRelativeLuminance(color: string): number {
  const rgb = parseColor(color);
  const [r, g, b] = rgb.map(val => {
    const sRGB = val / 255;
    return sRGB <= 0.03928
      ? sRGB / 12.92
      : Math.pow((sRGB + 0.055) / 1.055, 2.4);
  });

  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/**
 * Parse color string to RGB values
 */
function parseColor(color: string): [number, number, number] {
  // Handle hex colors
  if (color.startsWith('#')) {
    const hex = color.slice(1);
    if (hex.length === 3) {
      return [
        parseInt(hex[0] + hex[0], 16),
        parseInt(hex[1] + hex[1], 16),
        parseInt(hex[2] + hex[2], 16)
      ];
    }
    return [
      parseInt(hex.slice(0, 2), 16),
      parseInt(hex.slice(2, 4), 16),
      parseInt(hex.slice(4, 6), 16)
    ];
  }

  // Handle rgb/rgba colors
  const match = color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
  if (match) {
    return [parseInt(match[1]), parseInt(match[2]), parseInt(match[3])];
  }

  // Default to black
  return [0, 0, 0];
}

/**
 * Validate heading hierarchy
 */
export function validateHeadingHierarchy(headings: { level: number; text: string }[]): {
  valid: boolean;
  issues: string[];
} {
  const issues: string[] = [];

  if (headings.length === 0) {
    return { valid: false, issues: ['No headings found in document'] };
  }

  // Check if first heading is H1
  if (headings[0].level !== 1) {
    issues.push(`Document should start with H1, found H${headings[0].level}`);
  }

  // Check for skipped levels
  for (let i = 1; i < headings.length; i++) {
    const prevLevel = headings[i - 1].level;
    const currLevel = headings[i].level;

    if (currLevel > prevLevel + 1) {
      issues.push(
        `Heading level skipped from H${prevLevel} to H${currLevel} at "${headings[i].text}"`
      );
    }
  }

  // Check for multiple H1s
  const h1Count = headings.filter(h => h.level === 1).length;
  if (h1Count > 1) {
    issues.push(`Document has ${h1Count} H1 headings, should have only one`);
  }

  return {
    valid: issues.length === 0,
    issues
  };
}

/**
 * Sanitize file name for safe storage
 */
export function sanitizeFileName(fileName: string): string {
  return fileName
    .replace(/[^a-z0-9._-]/gi, '_')
    .replace(/_{2,}/g, '_')
    .toLowerCase();
}

/**
 * Check if language code is valid (ISO 639-1)
 */
export function isValidLanguageCode(code: string): boolean {
  const validCodes = [
    'aa', 'ab', 'ae', 'af', 'ak', 'am', 'an', 'ar', 'as', 'av', 'ay', 'az',
    'ba', 'be', 'bg', 'bh', 'bi', 'bm', 'bn', 'bo', 'br', 'bs',
    'ca', 'ce', 'ch', 'co', 'cr', 'cs', 'cu', 'cv', 'cy',
    'da', 'de', 'dv', 'dz',
    'ee', 'el', 'en', 'eo', 'es', 'et', 'eu',
    'fa', 'ff', 'fi', 'fj', 'fo', 'fr', 'fy',
    'ga', 'gd', 'gl', 'gn', 'gu', 'gv',
    'ha', 'he', 'hi', 'ho', 'hr', 'ht', 'hu', 'hy', 'hz',
    'ia', 'id', 'ie', 'ig', 'ii', 'ik', 'io', 'is', 'it', 'iu',
    'ja', 'jv',
    'ka', 'kg', 'ki', 'kj', 'kk', 'kl', 'km', 'kn', 'ko', 'kr', 'ks', 'ku', 'kv', 'kw', 'ky',
    'la', 'lb', 'lg', 'li', 'ln', 'lo', 'lt', 'lu', 'lv',
    'mg', 'mh', 'mi', 'mk', 'ml', 'mn', 'mr', 'ms', 'mt', 'my',
    'na', 'nb', 'nd', 'ne', 'ng', 'nl', 'nn', 'no', 'nr', 'nv', 'ny',
    'oc', 'oj', 'om', 'or', 'os',
    'pa', 'pi', 'pl', 'ps', 'pt',
    'qu',
    'rm', 'rn', 'ro', 'ru', 'rw',
    'sa', 'sc', 'sd', 'se', 'sg', 'si', 'sk', 'sl', 'sm', 'sn', 'so', 'sq', 'sr', 'ss', 'st', 'su', 'sv', 'sw',
    'ta', 'te', 'tg', 'th', 'ti', 'tk', 'tl', 'tn', 'to', 'tr', 'ts', 'tt', 'tw', 'ty',
    'ug', 'uk', 'ur', 'uz',
    've', 'vi', 'vo',
    'wa', 'wo',
    'xh',
    'yi', 'yo',
    'za', 'zh', 'zu'
  ];

  return validCodes.includes(code.toLowerCase().substring(0, 2));
}
