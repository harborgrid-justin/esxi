/**
 * WCAG Rule Engine - TypeScript Implementation
 */

import { Rule, WCAGLevel, Principle, Issue, Severity, IssueContext, Position } from '../types';
import { v4 as uuidv4 } from 'uuid';

/**
 * WCAG Rule Engine for executing accessibility rules
 */
export class RuleEngine {
  private rules: Map<string, Rule>;

  constructor() {
    this.rules = new Map();
    this.initializeRules();
  }

  /**
   * Initialize all WCAG rules
   */
  private initializeRules(): void {
    const allRules = [
      ...this.getLevelARules(),
      ...this.getLevelAARules(),
      ...this.getLevelAAARules(),
    ];

    allRules.forEach(rule => {
      this.rules.set(rule.id, rule);
    });
  }

  /**
   * Get all rules
   */
  getAllRules(): Rule[] {
    return Array.from(this.rules.values());
  }

  /**
   * Get rules by WCAG levels
   */
  getRulesByLevels(levels: WCAGLevel[]): Rule[] {
    return this.getAllRules().filter(rule => levels.includes(rule.level));
  }

  /**
   * Get a rule by ID
   */
  getRuleById(id: string): Rule | undefined {
    return this.rules.get(id);
  }

  /**
   * Get rules by principle
   */
  getRulesByPrinciple(principle: Principle): Rule[] {
    return this.getAllRules().filter(rule => rule.principle === principle);
  }

  /**
   * Get rules by tag
   */
  getRulesByTag(tag: string): Rule[] {
    return this.getAllRules().filter(rule => rule.tags.includes(tag));
  }

  /**
   * WCAG 2.1 Level A Rules
   */
  private getLevelARules(): Rule[] {
    return [
      {
        id: 'image-alt',
        name: 'Images must have alternative text',
        description: 'All img elements must have an alt attribute. Images convey information that must be available to screen reader users.',
        level: WCAGLevel.A,
        principle: Principle.Perceivable,
        guideline: '1.1',
        successCriterion: 'non-text-content',
        tags: ['images', 'alt-text'],
      },
      {
        id: 'input-image-alt',
        name: 'Image buttons must have alternative text',
        description: "Input elements with type='image' must have an alt attribute.",
        level: WCAGLevel.A,
        principle: Principle.Perceivable,
        guideline: '1.1',
        successCriterion: 'non-text-content',
        tags: ['forms', 'buttons'],
      },
      {
        id: 'area-alt',
        name: 'Image map areas must have alternative text',
        description: 'Area elements must have an alt attribute.',
        level: WCAGLevel.A,
        principle: Principle.Perceivable,
        guideline: '1.1',
        successCriterion: 'non-text-content',
        tags: ['images', 'image-maps'],
      },
      {
        id: 'label',
        name: 'Form inputs must have labels',
        description: 'Every form input must have a properly associated label element.',
        level: WCAGLevel.A,
        principle: Principle.Perceivable,
        guideline: '1.3',
        successCriterion: 'info-and-relationships',
        tags: ['forms', 'labels'],
      },
      {
        id: 'document-title',
        name: 'Pages must have a title',
        description: 'Every HTML document must have a descriptive title element.',
        level: WCAGLevel.A,
        principle: Principle.Operable,
        guideline: '2.4',
        successCriterion: 'page-titled',
        tags: ['page-structure', 'titles'],
      },
      {
        id: 'html-has-lang',
        name: 'HTML element must have lang attribute',
        description: 'The html element must have a lang attribute to identify the page language.',
        level: WCAGLevel.A,
        principle: Principle.Understandable,
        guideline: '3.1',
        successCriterion: 'language-of-page',
        tags: ['language'],
      },
      {
        id: 'html-lang-valid',
        name: 'HTML lang attribute must be valid',
        description: 'The lang attribute must contain a valid language code.',
        level: WCAGLevel.A,
        principle: Principle.Understandable,
        guideline: '3.1',
        successCriterion: 'language-of-page',
        tags: ['language'],
      },
      {
        id: 'link-name',
        name: 'Links must have discernible text',
        description: 'Every link must have accessible text that describes its purpose.',
        level: WCAGLevel.A,
        principle: Principle.Operable,
        guideline: '2.4',
        successCriterion: 'link-purpose-in-context',
        tags: ['links', 'link-text'],
      },
      {
        id: 'button-name',
        name: 'Buttons must have accessible text',
        description: 'Button elements must have visible text or an aria-label.',
        level: WCAGLevel.A,
        principle: Principle.Robust,
        guideline: '4.1',
        successCriterion: 'name-role-value',
        tags: ['buttons', 'aria'],
      },
      {
        id: 'duplicate-id',
        name: 'IDs must be unique',
        description: 'Each id attribute value must be unique within the page.',
        level: WCAGLevel.A,
        principle: Principle.Robust,
        guideline: '4.1',
        successCriterion: 'parsing',
        tags: ['parsing', 'html'],
      },
      {
        id: 'table-headers',
        name: 'Data tables must have headers',
        description: 'Tables used for data must have proper th elements.',
        level: WCAGLevel.A,
        principle: Principle.Perceivable,
        guideline: '1.3',
        successCriterion: 'info-and-relationships',
        tags: ['tables'],
      },
      {
        id: 'list',
        name: 'Lists must be properly marked up',
        description: 'List content must use proper list elements (ul, ol, dl).',
        level: WCAGLevel.A,
        principle: Principle.Perceivable,
        guideline: '1.3',
        successCriterion: 'info-and-relationships',
        tags: ['structure', 'lists'],
      },
      {
        id: 'meta-refresh',
        name: 'Meta refresh must not be used',
        description: 'Pages must not use meta refresh to automatically redirect or refresh.',
        level: WCAGLevel.A,
        principle: Principle.Operable,
        guideline: '2.2',
        successCriterion: 'timing-adjustable',
        tags: ['timing', 'redirects'],
      },
      {
        id: 'aria-valid-attr',
        name: 'ARIA attributes must be valid',
        description: 'Elements must only use valid ARIA attributes.',
        level: WCAGLevel.A,
        principle: Principle.Robust,
        guideline: '4.1',
        successCriterion: 'name-role-value',
        tags: ['aria'],
      },
      {
        id: 'aria-valid-attr-value',
        name: 'ARIA attributes must have valid values',
        description: 'ARIA attribute values must conform to allowed value specifications.',
        level: WCAGLevel.A,
        principle: Principle.Robust,
        guideline: '4.1',
        successCriterion: 'name-role-value',
        tags: ['aria'],
      },
    ];
  }

  /**
   * WCAG 2.1 Level AA Rules
   */
  private getLevelAARules(): Rule[] {
    return [
      {
        id: 'color-contrast',
        name: 'Text must have sufficient color contrast',
        description: 'Text and images of text must have a contrast ratio of at least 4.5:1.',
        level: WCAGLevel.AA,
        principle: Principle.Perceivable,
        guideline: '1.4',
        successCriterion: 'contrast-minimum',
        tags: ['color', 'contrast'],
      },
      {
        id: 'heading-order',
        name: 'Headings must be in correct order',
        description: 'Heading levels should not be skipped (e.g., h1 to h3 without h2).',
        level: WCAGLevel.AA,
        principle: Principle.Operable,
        guideline: '2.4',
        successCriterion: 'headings-and-labels',
        tags: ['headings', 'structure'],
      },
      {
        id: 'empty-heading',
        name: 'Headings must not be empty',
        description: 'Heading elements must contain text content.',
        level: WCAGLevel.AA,
        principle: Principle.Operable,
        guideline: '2.4',
        successCriterion: 'headings-and-labels',
        tags: ['headings'],
      },
      {
        id: 'focus-visible',
        name: 'Keyboard focus must be visible',
        description: 'Any keyboard operable interface must have a visible focus indicator.',
        level: WCAGLevel.AA,
        principle: Principle.Operable,
        guideline: '2.4',
        successCriterion: 'focus-visible',
        tags: ['keyboard', 'focus'],
      },
      {
        id: 'autocomplete',
        name: 'Input fields must have autocomplete attribute',
        description: 'Form inputs for personal information should have appropriate autocomplete values.',
        level: WCAGLevel.AA,
        principle: Principle.Perceivable,
        guideline: '1.3',
        successCriterion: 'identify-input-purpose',
        tags: ['forms', 'autocomplete'],
      },
    ];
  }

  /**
   * WCAG 2.1 Level AAA Rules
   */
  private getLevelAAARules(): Rule[] {
    return [
      {
        id: 'color-contrast-enhanced',
        name: 'Text must have enhanced color contrast',
        description: 'Text and images of text must have a contrast ratio of at least 7:1.',
        level: WCAGLevel.AAA,
        principle: Principle.Perceivable,
        guideline: '1.4',
        successCriterion: 'contrast-enhanced',
        tags: ['color', 'contrast'],
      },
      {
        id: 'target-size',
        name: 'Touch targets must be at least 44x44 pixels',
        description: 'The size of the target for pointer inputs is at least 44 by 44 CSS pixels.',
        level: WCAGLevel.AAA,
        principle: Principle.Operable,
        guideline: '2.5',
        successCriterion: 'target-size',
        tags: ['touch', 'mobile'],
      },
    ];
  }

  /**
   * Create an issue instance
   */
  createIssue(
    rule: Rule,
    severity: Severity,
    message: string,
    context: IssueContext,
    fixSuggestions: string[] = [],
    impactDescription?: string
  ): Issue {
    return {
      id: uuidv4(),
      ruleId: rule.id,
      ruleName: rule.name,
      severity,
      level: rule.level,
      principle: rule.principle,
      message,
      help: `See WCAG 2.1 ${rule.level} - ${rule.successCriterion}`,
      helpUrl: `https://www.w3.org/WAI/WCAG21/Understanding/${rule.successCriterion}`,
      context,
      fixSuggestions,
      impactDescription: impactDescription || `This violates WCAG ${rule.level} principle: ${rule.principle}`,
      wcagReference: `WCAG 2.1 ${rule.level} - ${rule.successCriterion}`,
    };
  }

  /**
   * Create issue context from element data
   */
  createContext(
    html: string,
    outerHtml: string,
    selector: string,
    attributes: Record<string, string> = {},
    computedStyles?: Record<string, string>
  ): IssueContext {
    return {
      html,
      outerHtml,
      position: {
        line: 0,
        column: 0,
        xpath: `//${selector.split(/[#.]/)[0]}`,
        selector,
      },
      attributes,
      computedStyles,
    };
  }
}

/**
 * Singleton instance of the rule engine
 */
let ruleEngineInstance: RuleEngine | null = null;

/**
 * Get the singleton rule engine instance
 */
export function getRuleEngine(): RuleEngine {
  if (!ruleEngineInstance) {
    ruleEngineInstance = new RuleEngine();
  }
  return ruleEngineInstance;
}
