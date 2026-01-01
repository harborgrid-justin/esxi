/**
 * TemplateEngine - Template rendering and compilation
 * Supports multiple template engines and localization
 */

import Handlebars from 'handlebars';
import Mustache from 'mustache';
import MarkdownIt from 'markdown-it';
import juice from 'juice';
import { NotificationTemplate, TemplateChannel, NotificationChannelType } from '../types';

export interface TemplateEngineConfig {
  defaultEngine: 'handlebars' | 'mustache';
  enableMarkdown: boolean;
  inlineCss: boolean;
  customHelpers?: Record<string, (...args: unknown[]) => unknown>;
  partials?: Record<string, string>;
  layouts?: Record<string, string>;
}

export interface RenderContext {
  data: Record<string, unknown>;
  locale?: string;
  timezone?: string;
  user?: {
    id: string;
    name: string;
    email?: string;
  };
  tenant?: {
    id: string;
    name: string;
  };
}

export interface RenderedTemplate {
  subject?: string;
  body: string;
  html?: string;
  plainText?: string;
}

export class TemplateEngine {
  private config: TemplateEngineConfig;
  private handlebars: typeof Handlebars;
  private markdown: MarkdownIt;
  private compiledTemplates: Map<string, HandlebarsTemplateDelegate>;

  constructor(config: Partial<TemplateEngineConfig> = {}) {
    this.config = {
      defaultEngine: config.defaultEngine ?? 'handlebars',
      enableMarkdown: config.enableMarkdown ?? true,
      inlineCss: config.inlineCss ?? true,
      customHelpers: config.customHelpers ?? {},
      partials: config.partials ?? {},
      layouts: config.layouts ?? {},
    };

    this.handlebars = Handlebars.create();
    this.markdown = new MarkdownIt({
      html: true,
      linkify: true,
      typographer: true,
    });
    this.compiledTemplates = new Map();

    this.initializeHelpers();
    this.registerPartials();
  }

  /**
   * Render a template
   */
  render(
    template: NotificationTemplate,
    channel: NotificationChannelType,
    context: RenderContext
  ): RenderedTemplate {
    const templateChannel = template.channels.find(c => c.type === channel);
    if (!templateChannel) {
      throw new Error(`Template does not support channel: ${channel}`);
    }

    // Prepare context with default values
    const enrichedContext = this.enrichContext(context, template);

    // Render based on template type
    const rendered = this.renderChannel(templateChannel, enrichedContext);

    return rendered;
  }

  /**
   * Render a simple string template
   */
  renderString(template: string, context: Record<string, unknown>): string {
    if (this.config.defaultEngine === 'handlebars') {
      return this.renderHandlebars(template, context);
    } else {
      return this.renderMustache(template, context);
    }
  }

  /**
   * Compile and cache a template
   */
  compile(templateId: string, template: string): void {
    const compiled = this.handlebars.compile(template);
    this.compiledTemplates.set(templateId, compiled);
  }

  /**
   * Clear compiled template cache
   */
  clearCache(templateId?: string): void {
    if (templateId) {
      this.compiledTemplates.delete(templateId);
    } else {
      this.compiledTemplates.clear();
    }
  }

  /**
   * Register a custom helper
   */
  registerHelper(name: string, helper: Handlebars.HelperDelegate): void {
    this.handlebars.registerHelper(name, helper);
  }

  /**
   * Register a partial template
   */
  registerPartial(name: string, template: string): void {
    this.handlebars.registerPartial(name, template);
  }

  /**
   * Render channel-specific template
   */
  private renderChannel(
    templateChannel: TemplateChannel,
    context: Record<string, unknown>
  ): RenderedTemplate {
    const result: RenderedTemplate = {
      body: '',
    };

    // Render subject if present
    if (templateChannel.subject) {
      result.subject = this.renderString(templateChannel.subject, context);
    }

    // Render body
    result.body = this.renderString(templateChannel.body, context);

    // Render HTML if present
    if (templateChannel.html) {
      let html = this.renderString(templateChannel.html, context);

      // Apply layout if specified
      if (templateChannel.layout) {
        const layout = this.config.layouts?.[templateChannel.layout];
        if (layout) {
          html = this.renderString(layout, { ...context, content: html });
        }
      }

      // Inline CSS if enabled
      if (this.config.inlineCss) {
        html = juice(html);
      }

      result.html = html;
    }

    // Convert markdown to HTML if enabled
    if (this.config.enableMarkdown && !result.html) {
      result.html = this.markdown.render(result.body);
      if (this.config.inlineCss) {
        result.html = juice(result.html);
      }
    }

    // Generate plain text from HTML
    if (result.html && !result.plainText) {
      result.plainText = this.htmlToPlainText(result.html);
    }

    return result;
  }

  /**
   * Enrich context with default values
   */
  private enrichContext(
    context: RenderContext,
    template: NotificationTemplate
  ): Record<string, unknown> {
    const now = new Date();

    return {
      ...context.data,
      user: context.user,
      tenant: context.tenant,
      locale: context.locale ?? 'en',
      timezone: context.timezone ?? 'UTC',
      template: {
        id: template.id,
        name: template.name,
        type: template.type,
      },
      timestamp: now,
      year: now.getFullYear(),
      date: now.toLocaleDateString(),
      time: now.toLocaleTimeString(),
    };
  }

  /**
   * Render using Handlebars
   */
  private renderHandlebars(template: string, context: Record<string, unknown>): string {
    try {
      const compiled = this.handlebars.compile(template);
      return compiled(context);
    } catch (error) {
      throw new Error(`Handlebars rendering error: ${error}`);
    }
  }

  /**
   * Render using Mustache
   */
  private renderMustache(template: string, context: Record<string, unknown>): string {
    try {
      return Mustache.render(template, context);
    } catch (error) {
      throw new Error(`Mustache rendering error: ${error}`);
    }
  }

  /**
   * Convert HTML to plain text
   */
  private htmlToPlainText(html: string): string {
    return html
      .replace(/<style[^>]*>.*?<\/style>/gi, '')
      .replace(/<script[^>]*>.*?<\/script>/gi, '')
      .replace(/<\/div>/gi, '\n')
      .replace(/<\/li>/gi, '\n')
      .replace(/<li>/gi, '  *  ')
      .replace(/<\/ul>/gi, '\n')
      .replace(/<\/p>/gi, '\n')
      .replace(/<br\s*\/?>/gi, '\n')
      .replace(/<[^>]+>/g, '')
      .replace(/&nbsp;/gi, ' ')
      .replace(/&amp;/gi, '&')
      .replace(/&lt;/gi, '<')
      .replace(/&gt;/gi, '>')
      .replace(/&quot;/gi, '"')
      .replace(/&#39;/gi, "'")
      .replace(/\n\s*\n\s*\n/g, '\n\n')
      .trim();
  }

  /**
   * Initialize built-in helpers
   */
  private initializeHelpers(): void {
    // Date formatting
    this.handlebars.registerHelper('formatDate', (date: Date, format: string) => {
      if (!date) return '';
      const d = new Date(date);
      // Simple format implementation
      return d.toLocaleDateString();
    });

    // Number formatting
    this.handlebars.registerHelper('formatNumber', (num: number, decimals: number = 0) => {
      if (typeof num !== 'number') return '';
      return num.toFixed(decimals);
    });

    // Currency formatting
    this.handlebars.registerHelper(
      'formatCurrency',
      (amount: number, currency: string = 'USD') => {
        if (typeof amount !== 'number') return '';
        return new Intl.NumberFormat('en-US', {
          style: 'currency',
          currency,
        }).format(amount);
      }
    );

    // String operations
    this.handlebars.registerHelper('uppercase', (str: string) => {
      return typeof str === 'string' ? str.toUpperCase() : '';
    });

    this.handlebars.registerHelper('lowercase', (str: string) => {
      return typeof str === 'string' ? str.toLowerCase() : '';
    });

    this.handlebars.registerHelper('capitalize', (str: string) => {
      if (typeof str !== 'string') return '';
      return str.charAt(0).toUpperCase() + str.slice(1).toLowerCase();
    });

    // Conditionals
    this.handlebars.registerHelper('eq', (a: unknown, b: unknown) => {
      return a === b;
    });

    this.handlebars.registerHelper('ne', (a: unknown, b: unknown) => {
      return a !== b;
    });

    this.handlebars.registerHelper('gt', (a: number, b: number) => {
      return a > b;
    });

    this.handlebars.registerHelper('gte', (a: number, b: number) => {
      return a >= b;
    });

    this.handlebars.registerHelper('lt', (a: number, b: number) => {
      return a < b;
    });

    this.handlebars.registerHelper('lte', (a: number, b: number) => {
      return a <= b;
    });

    // Array operations
    this.handlebars.registerHelper('join', (arr: unknown[], separator: string = ', ') => {
      if (!Array.isArray(arr)) return '';
      return arr.join(separator);
    });

    this.handlebars.registerHelper('length', (arr: unknown[]) => {
      if (Array.isArray(arr)) return arr.length;
      if (typeof arr === 'string') return arr.length;
      if (typeof arr === 'object' && arr !== null) return Object.keys(arr).length;
      return 0;
    });

    // Default value
    this.handlebars.registerHelper('default', (value: unknown, defaultValue: unknown) => {
      return value ?? defaultValue;
    });

    // JSON stringify
    this.handlebars.registerHelper('json', (obj: unknown) => {
      return JSON.stringify(obj, null, 2);
    });

    // Truncate
    this.handlebars.registerHelper('truncate', (str: string, length: number = 50) => {
      if (typeof str !== 'string') return '';
      if (str.length <= length) return str;
      return str.substring(0, length) + '...';
    });

    // Register custom helpers
    if (this.config.customHelpers) {
      Object.entries(this.config.customHelpers).forEach(([name, helper]) => {
        this.handlebars.registerHelper(name, helper as Handlebars.HelperDelegate);
      });
    }
  }

  /**
   * Register partials
   */
  private registerPartials(): void {
    if (this.config.partials) {
      Object.entries(this.config.partials).forEach(([name, template]) => {
        this.handlebars.registerPartial(name, template);
      });
    }
  }
}

export default TemplateEngine;
