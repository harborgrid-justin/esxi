/**
 * Semantic HTML Analyzer
 * Analyzes semantic HTML structure and ARIA usage
 */

import { ARIARole, SemanticAnalysis, ValidationResult, ValidationError, ValidationWarning } from '../types';
import { ImplicitRoleAnalyzer } from './ImplicitRoleAnalyzer';

export class SemanticAnalyzer {
  private implicitRoleAnalyzer = new ImplicitRoleAnalyzer();

  analyze(element: HTMLElement): SemanticAnalysis {
    const tagName = element.tagName.toLowerCase();
    const explicitRole = element.getAttribute('role') as ARIARole | undefined;
    const implicitRole = this.implicitRoleAnalyzer.getImplicitRole(element);

    const roleConflict = !!(explicitRole && implicitRole && explicitRole !== implicitRole);
    const semanticIssues: string[] = [];
    const recommendations: string[] = [];

    // Check for role conflicts
    if (roleConflict) {
      semanticIssues.push(`Explicit role="${explicitRole}" conflicts with implicit role="${implicitRole}"`);
      recommendations.push('Consider using native HTML semantics instead of ARIA roles');
    }

    // Check for redundant roles
    if (explicitRole && implicitRole && explicitRole === implicitRole) {
      semanticIssues.push(`Redundant role="${explicitRole}" on <${tagName}> element`);
      recommendations.push('Remove redundant role attribute and rely on native semantics');
    }

    // Check for semantic misuse
    this.checkSemanticMisuse(element, tagName, explicitRole, semanticIssues, recommendations);

    // Check for missing semantic elements
    this.checkMissingSemantics(element, tagName, explicitRole, semanticIssues, recommendations);

    return {
      element: tagName,
      implicitRole,
      explicitRole,
      roleConflict,
      semanticIssues,
      recommendations,
    };
  }

  private checkSemanticMisuse(
    element: HTMLElement,
    tagName: string,
    role: ARIARole | undefined,
    issues: string[],
    recommendations: string[]
  ): void {
    // Check for div/span with roles that have native equivalents
    if (tagName === 'div' || tagName === 'span') {
      const nativeEquivalents: Record<string, string> = {
        'button': '<button>',
        'link': '<a href="...">',
        'heading': '<h1>-<h6>',
        'list': '<ul> or <ol>',
        'listitem': '<li>',
        'table': '<table>',
        'row': '<tr>',
        'cell': '<td>',
        'navigation': '<nav>',
        'main': '<main>',
        'article': '<article>',
        'section': '<section>',
        'aside': '<aside>',
        'header': '<header>',
        'footer': '<footer>',
      };

      if (role && nativeEquivalents[role]) {
        issues.push(`Using ${tagName} with role="${role}" instead of native ${nativeEquivalents[role]}`);
        recommendations.push(`Replace with ${nativeEquivalents[role]} element`);
      }
    }

    // Check for semantic elements with presentation role
    const semanticElements = ['nav', 'main', 'header', 'footer', 'aside', 'article', 'section'];
    if (semanticElements.includes(tagName) && (role === 'presentation' || role === 'none')) {
      issues.push(`Semantic <${tagName}> element has role="${role}"`);
      recommendations.push('Remove role to preserve semantic meaning, or use <div> instead');
    }

    // Check for links without href
    if (tagName === 'a' && !element.hasAttribute('href')) {
      if (role !== 'button') {
        issues.push('Anchor without href should have role="button" or add href attribute');
        recommendations.push('Add href attribute or change to <button> element');
      }
    }

    // Check for buttons with link role
    if (tagName === 'button' && role === 'link') {
      issues.push('Button element has role="link"');
      recommendations.push('Use <a href="..."> for links');
    }

    // Check for img without alt
    if (tagName === 'img' && !element.hasAttribute('alt') && !element.hasAttribute('aria-label') && !element.hasAttribute('aria-labelledby')) {
      issues.push('Image missing alt text and accessible name');
      recommendations.push('Add alt attribute to provide text alternative');
    }

    // Check for form inputs without labels
    if (['input', 'select', 'textarea'].includes(tagName)) {
      const hasLabel = this.hasAssociatedLabel(element);
      const hasAriaLabel = element.hasAttribute('aria-label') || element.hasAttribute('aria-labelledby');

      if (!hasLabel && !hasAriaLabel) {
        issues.push('Form control missing label');
        recommendations.push('Add <label> element or aria-label attribute');
      }
    }
  }

  private checkMissingSemantics(
    element: HTMLElement,
    tagName: string,
    role: ARIARole | undefined,
    issues: string[],
    recommendations: string[]
  ): void {
    // Check for interactive elements without role
    const interactiveClasses = ['btn', 'button', 'link', 'tab', 'menu', 'dropdown'];
    const classList = Array.from(element.classList);

    const hasInteractiveClass = interactiveClasses.some(cls =>
      classList.some(c => c.toLowerCase().includes(cls))
    );

    if (hasInteractiveClass && !role && tagName === 'div') {
      issues.push('Interactive element (based on class name) missing ARIA role');
      recommendations.push('Add appropriate role or use semantic HTML element');
    }

    // Check for landmark regions
    if (tagName === 'div' && !role) {
      const hasLandmarkClass = classList.some(c =>
        ['header', 'footer', 'nav', 'sidebar', 'main', 'content'].includes(c.toLowerCase())
      );

      if (hasLandmarkClass) {
        recommendations.push('Consider using semantic landmark elements or ARIA landmark roles');
      }
    }
  }

  private hasAssociatedLabel(element: HTMLElement): boolean {
    const id = element.id;
    if (!id) return false;

    const label = element.ownerDocument?.querySelector(`label[for="${id}"]`);
    return !!label || !!element.closest('label');
  }

  validateSemanticStructure(root: HTMLElement | Document): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const info: any[] = [];

    const document = root instanceof Document ? root : root.ownerDocument;
    if (!document) {
      return { valid: true, errors, warnings, info };
    }

    // Check for multiple main landmarks
    const mains = document.querySelectorAll('main, [role="main"]');
    if (mains.length > 1) {
      errors.push({
        type: 'semantic',
        severity: 'error',
        message: `Document contains ${mains.length} main landmarks (should have only one)`,
        wcagCriterion: '1.3.1',
      });
    }

    // Check for multiple banner landmarks
    const banners = document.querySelectorAll('body > header, [role="banner"]');
    if (banners.length > 1) {
      warnings.push({
        type: 'semantic',
        severity: 'warning',
        message: `Document contains ${banners.length} banner landmarks (should have only one)`,
        suggestion: 'Use only one banner landmark per page',
      });
    }

    // Check for multiple contentinfo landmarks
    const contentinfos = document.querySelectorAll('body > footer, [role="contentinfo"]');
    if (contentinfos.length > 1) {
      warnings.push({
        type: 'semantic',
        severity: 'warning',
        message: `Document contains ${contentinfos.length} contentinfo landmarks (should have only one)`,
        suggestion: 'Use only one contentinfo landmark per page',
      });
    }

    // Check heading hierarchy
    const headings = Array.from(document.querySelectorAll('h1, h2, h3, h4, h5, h6, [role="heading"]'));
    this.validateHeadingHierarchy(headings, warnings);

    // Check for skip links
    const skipLinks = document.querySelectorAll('a[href^="#"]');
    if (skipLinks.length === 0) {
      warnings.push({
        type: 'semantic',
        severity: 'warning',
        message: 'No skip links found',
        suggestion: 'Add skip link to main content for keyboard users',
      });
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      info,
    };
  }

  private validateHeadingHierarchy(headings: Element[], warnings: ValidationWarning[]): void {
    let previousLevel = 0;

    for (const heading of headings) {
      let level: number;

      if (heading.hasAttribute('role')) {
        const ariaLevel = heading.getAttribute('aria-level');
        level = ariaLevel ? parseInt(ariaLevel, 10) : 2;
      } else {
        level = parseInt(heading.tagName.substring(1), 10);
      }

      if (previousLevel > 0 && level > previousLevel + 1) {
        warnings.push({
          type: 'semantic',
          severity: 'warning',
          message: `Heading level skipped from h${previousLevel} to h${level}`,
          element: heading.tagName.toLowerCase(),
          suggestion: 'Use sequential heading levels for proper document structure',
        });
      }

      previousLevel = level;
    }
  }

  checkAccessibleName(element: HTMLElement): {
    hasName: boolean;
    source: 'aria-label' | 'aria-labelledby' | 'content' | 'alt' | 'title' | 'none';
    name: string;
  } {
    // Check aria-labelledby first (highest precedence)
    if (element.hasAttribute('aria-labelledby')) {
      const ids = element.getAttribute('aria-labelledby')!.split(/\s+/);
      const texts = ids
        .map(id => element.ownerDocument?.getElementById(id)?.textContent?.trim())
        .filter(Boolean);

      if (texts.length > 0) {
        return {
          hasName: true,
          source: 'aria-labelledby',
          name: texts.join(' '),
        };
      }
    }

    // Check aria-label
    if (element.hasAttribute('aria-label')) {
      const label = element.getAttribute('aria-label')!.trim();
      if (label) {
        return {
          hasName: true,
          source: 'aria-label',
          name: label,
        };
      }
    }

    // Check alt attribute (for images)
    if (element.hasAttribute('alt')) {
      const alt = element.getAttribute('alt')!.trim();
      return {
        hasName: true,
        source: 'alt',
        name: alt,
      };
    }

    // Check content
    const content = element.textContent?.trim() || '';
    if (content) {
      return {
        hasName: true,
        source: 'content',
        name: content,
      };
    }

    // Check title attribute (lowest precedence)
    if (element.hasAttribute('title')) {
      const title = element.getAttribute('title')!.trim();
      if (title) {
        return {
          hasName: true,
          source: 'title',
          name: title,
        };
      }
    }

    return {
      hasName: false,
      source: 'none',
      name: '',
    };
  }
}

export const semanticAnalyzer = new SemanticAnalyzer();
