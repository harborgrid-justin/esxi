/**
 * Alternative Text Validator
 * Validates quality and appropriateness of alternative text
 */

import type { ValidationResult, ValidationError, ValidationWarning } from '../types/index.js';

export class AltTextValidator {
  private errors: ValidationError[] = [];
  private warnings: ValidationWarning[] = [];

  /**
   * Validate alternative text
   */
  validate(altText: string, context?: AltTextContext): ValidationResult {
    this.errors = [];
    this.warnings = [];

    // Check if alt text exists
    if (!altText || altText.trim().length === 0) {
      this.addError('ALT_TEXT_EMPTY', 'Alternative text is empty');
      return {
        valid: false,
        errors: this.errors,
        warnings: this.warnings
      };
    }

    const trimmed = altText.trim();

    // Check length
    this.validateLength(trimmed);

    // Check for bad patterns
    this.validatePatterns(trimmed);

    // Check for file names
    this.validateNotFileName(trimmed);

    // Check for meaningful content
    this.validateMeaningful(trimmed);

    // Context-specific validation
    if (context) {
      this.validateContext(trimmed, context);
    }

    return {
      valid: this.errors.length === 0,
      errors: this.errors,
      warnings: this.warnings
    };
  }

  /**
   * Validate alt text length
   */
  private validateLength(altText: string): void {
    if (altText.length < 5) {
      this.addWarning('ALT_TEXT_TOO_SHORT', 'Alternative text is very short (less than 5 characters)');
    }

    if (altText.length > 150) {
      this.addWarning('ALT_TEXT_TOO_LONG', 'Alternative text is long (more than 150 characters). Consider using a caption or long description.');
    }

    if (altText.length > 300) {
      this.addError('ALT_TEXT_EXCESSIVE', 'Alternative text is excessively long (more than 300 characters). Use longdesc or adjacent text instead.');
    }
  }

  /**
   * Validate for bad patterns
   */
  private validatePatterns(altText: string): void {
    const lower = altText.toLowerCase();

    // Check for redundant phrases
    const redundantPhrases = [
      'image of',
      'picture of',
      'photo of',
      'graphic of',
      'icon of',
      'screenshot of',
      'illustration of',
      'diagram of'
    ];

    for (const phrase of redundantPhrases) {
      if (lower.startsWith(phrase)) {
        this.addWarning('ALT_REDUNDANT_PHRASE', `Avoid starting with "${phrase}". Screen readers already announce it's an image.`);
        break;
      }
    }

    // Check for placeholder text
    const placeholderPatterns = [
      /^image\d*$/i,
      /^img\d*$/i,
      /^graphic\d*$/i,
      /^picture\d*$/i,
      /^photo\d*$/i,
      /^untitled/i,
      /^spacer/i,
      /^placeholder/i
    ];

    for (const pattern of placeholderPatterns) {
      if (pattern.test(altText)) {
        this.addError('ALT_PLACEHOLDER', `"${altText}" appears to be placeholder text, not a real description.`);
        break;
      }
    }

    // Check for all caps
    if (altText === altText.toUpperCase() && altText.length > 10) {
      this.addWarning('ALT_ALL_CAPS', 'Avoid all-caps text. It can be harder to read.');
    }

    // Check for excessive punctuation
    const punctuationCount = (altText.match(/[!?]{2,}/g) || []).length;
    if (punctuationCount > 0) {
      this.addWarning('ALT_EXCESSIVE_PUNCTUATION', 'Avoid excessive punctuation (e.g., "!!!" or "???").');
    }
  }

  /**
   * Validate it's not a file name
   */
  private validateNotFileName(altText: string): void {
    // Check for common image file extensions
    const fileExtensions = /\.(jpg|jpeg|png|gif|svg|webp|bmp|tiff|ico)$/i;

    if (fileExtensions.test(altText)) {
      this.addError('ALT_IS_FILENAME', `"${altText}" appears to be a file name, not a description.`);
    }

    // Check for file name patterns
    if (/^[A-Z0-9_-]+\.(jpg|png|gif)/i.test(altText)) {
      this.addError('ALT_IS_FILENAME', 'Alternative text looks like a file name.');
    }

    // Check for common file name patterns
    if (/^(img|image|pic|photo|graphic)[-_]\d+/i.test(altText)) {
      this.addWarning('ALT_FILENAME_PATTERN', 'Alternative text may be a file name pattern (e.g., "img_001").');
    }
  }

  /**
   * Validate text is meaningful
   */
  private validateMeaningful(altText: string): void {
    // Check for single character or number
    if (altText.length === 1) {
      this.addWarning('ALT_SINGLE_CHAR', 'Single character alt text may not be descriptive enough.');
    }

    // Check for just numbers
    if (/^\d+$/.test(altText)) {
      this.addWarning('ALT_JUST_NUMBER', 'Alt text is just a number. Consider adding context.');
    }

    // Check for repeated characters
    if (/^(.)\1{4,}$/.test(altText)) {
      this.addError('ALT_REPEATED_CHARS', 'Alt text contains only repeated characters.');
    }

    // Check for non-descriptive single words
    const nonDescriptiveWords = ['image', 'photo', 'picture', 'graphic', 'icon', 'logo', 'banner'];
    if (nonDescriptiveWords.includes(altText.toLowerCase())) {
      this.addError('ALT_NON_DESCRIPTIVE', `"${altText}" is not descriptive. Describe what the image shows.`);
    }
  }

  /**
   * Validate based on context
   */
  private validateContext(altText: string, context: AltTextContext): void {
    // For decorative images
    if (context.isDecorative) {
      if (altText.length > 0) {
        this.addWarning('ALT_DECORATIVE_NOT_EMPTY', 'Decorative images should have empty alt text (alt="").');
      }
    }

    // For complex images (charts, diagrams)
    if (context.isComplex) {
      if (altText.length < 20) {
        this.addWarning('ALT_COMPLEX_TOO_SHORT', 'Complex images (charts, diagrams) need longer descriptions. Consider adding a long description.');
      }
    }

    // For logos
    if (context.isLogo) {
      const lower = altText.toLowerCase();
      if (!lower.includes('logo') && context.companyName && !lower.includes(context.companyName.toLowerCase())) {
        this.addWarning('ALT_LOGO_MISSING_NAME', 'Logo alt text should include company/product name.');
      }
    }

    // For buttons/interactive elements
    if (context.isInteractive) {
      const lower = altText.toLowerCase();
      if (!this.containsActionWord(lower)) {
        this.addWarning('ALT_INTERACTIVE_NO_ACTION', 'Interactive images should describe the action (e.g., "Search", "Submit", "Close").');
      }
    }

    // For informational graphics
    if (context.isInformational) {
      if (altText.length < 30) {
        this.addWarning('ALT_INFO_TOO_SHORT', 'Informational graphics need detailed descriptions of key information.');
      }
    }
  }

  /**
   * Check if text contains action words
   */
  private containsActionWord(text: string): boolean {
    const actionWords = [
      'search', 'submit', 'send', 'close', 'open', 'play', 'pause', 'stop',
      'next', 'previous', 'delete', 'edit', 'save', 'cancel', 'download',
      'upload', 'share', 'print', 'refresh', 'reload', 'go', 'navigate'
    ];

    return actionWords.some(word => text.includes(word));
  }

  /**
   * Add validation error
   */
  private addError(code: string, message: string): void {
    this.errors.push({ code, message });
  }

  /**
   * Add validation warning
   */
  private addWarning(code: string, message: string): void {
    this.warnings.push({ code, message });
  }

  /**
   * Generate alt text suggestions
   */
  static generateSuggestions(imageContext: Partial<AltTextContext>): string[] {
    const suggestions: string[] = [];

    if (imageContext.isDecorative) {
      suggestions.push('Use empty alt text: alt=""');
    }

    if (imageContext.isLogo && imageContext.companyName) {
      suggestions.push(`${imageContext.companyName} logo`);
    }

    if (imageContext.isComplex) {
      suggestions.push('Provide a brief description in alt text');
      suggestions.push('Add a detailed long description nearby or use longdesc attribute');
    }

    if (imageContext.isInteractive) {
      suggestions.push('Describe the action the button performs');
      suggestions.push('Example: "Search", "Submit form", "Close dialog"');
    }

    return suggestions;
  }
}

/**
 * Context information for alt text validation
 */
export interface AltTextContext {
  isDecorative?: boolean;
  isComplex?: boolean;
  isLogo?: boolean;
  isInteractive?: boolean;
  isInformational?: boolean;
  companyName?: string;
  imageType?: string;
}
