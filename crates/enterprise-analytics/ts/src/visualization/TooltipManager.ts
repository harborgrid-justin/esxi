/**
 * Interactive Tooltip Management
 * @module @harborgrid/enterprise-analytics/visualization
 */

import type { TooltipConfig } from '../types';

export interface TooltipData {
  title?: string;
  fields: Array<{
    label: string;
    value: unknown;
    format?: string;
  }>;
  x?: number;
  y?: number;
}

export interface TooltipOptions extends TooltipConfig {
  followCursor?: boolean;
  offset?: { x: number; y: number };
  maxWidth?: number;
  delay?: number;
  theme?: 'light' | 'dark';
}

export class TooltipManager {
  private container: HTMLElement | null = null;
  private tooltip: HTMLElement | null = null;
  private options: TooltipOptions;
  private showTimeout: NodeJS.Timeout | null = null;

  constructor(options: TooltipOptions = { show: true }) {
    this.options = {
      show: true,
      followCursor: true,
      offset: { x: 10, y: 10 },
      maxWidth: 300,
      delay: 0,
      theme: 'light',
      ...options,
    };
  }

  // ============================================================================
  // Initialization
  // ============================================================================

  initialize(container: HTMLElement): void {
    this.container = container;

    if (!this.options.show) return;

    this.createTooltipElement();
  }

  private createTooltipElement(): void {
    if (this.tooltip) return;

    this.tooltip = document.createElement('div');
    this.tooltip.className = 'analytics-tooltip';
    this.tooltip.style.cssText = this.getTooltipStyles();
    this.tooltip.style.display = 'none';

    document.body.appendChild(this.tooltip);
  }

  // ============================================================================
  // Show/Hide
  // ============================================================================

  show(data: TooltipData, event?: MouseEvent): void {
    if (!this.options.show || !this.tooltip) return;

    const showTooltip = () => {
      if (!this.tooltip) return;

      this.tooltip.innerHTML = this.renderContent(data);
      this.tooltip.style.display = 'block';

      if (event) {
        this.positionTooltip(event);
      } else if (data.x !== undefined && data.y !== undefined) {
        this.tooltip.style.left = `${data.x}px`;
        this.tooltip.style.top = `${data.y}px`;
      }
    };

    if (this.options.delay && this.options.delay > 0) {
      this.showTimeout = setTimeout(showTooltip, this.options.delay);
    } else {
      showTooltip();
    }
  }

  hide(): void {
    if (this.showTimeout) {
      clearTimeout(this.showTimeout);
      this.showTimeout = null;
    }

    if (this.tooltip) {
      this.tooltip.style.display = 'none';
    }
  }

  // ============================================================================
  // Positioning
  // ============================================================================

  private positionTooltip(event: MouseEvent): void {
    if (!this.tooltip) return;

    const offset = this.options.offset || { x: 10, y: 10 };
    let x = event.clientX + offset.x;
    let y = event.clientY + offset.y;

    // Get tooltip dimensions
    const tooltipRect = this.tooltip.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    // Adjust if tooltip would go off screen
    if (x + tooltipRect.width > viewportWidth) {
      x = event.clientX - tooltipRect.width - offset.x;
    }

    if (y + tooltipRect.height > viewportHeight) {
      y = event.clientY - tooltipRect.height - offset.y;
    }

    this.tooltip.style.left = `${x}px`;
    this.tooltip.style.top = `${y}px`;
  }

  // ============================================================================
  // Content Rendering
  // ============================================================================

  private renderContent(data: TooltipData): string {
    let html = '';

    // Title
    if (data.title) {
      html += `<div class="tooltip-title" style="${this.getTitleStyles()}">${data.title}</div>`;
    }

    // Fields
    if (data.fields.length > 0) {
      html += '<div class="tooltip-fields" style="margin-top: 8px;">';

      for (const field of data.fields) {
        const formattedValue = this.formatValue(field.value, field.format);
        html += `
          <div class="tooltip-field" style="${this.getFieldStyles()}">
            <span class="tooltip-label" style="color: #666;">${field.label}:</span>
            <span class="tooltip-value" style="font-weight: 600; margin-left: 8px;">${formattedValue}</span>
          </div>
        `;
      }

      html += '</div>';
    }

    // Custom template if provided
    if (this.options.customTemplate) {
      html = this.applyTemplate(this.options.customTemplate, data);
    }

    return html;
  }

  private formatValue(value: unknown, format?: string): string {
    if (value === null || value === undefined) {
      return 'N/A';
    }

    if (format) {
      if (format.startsWith(',.')) {
        // Number format with thousand separators
        const decimals = parseInt(format.substring(2), 10) || 0;
        if (typeof value === 'number') {
          return value.toLocaleString(undefined, {
            minimumFractionDigits: decimals,
            maximumFractionDigits: decimals,
          });
        }
      } else if (format === '%') {
        // Percentage
        if (typeof value === 'number') {
          return `${(value * 100).toFixed(1)}%`;
        }
      } else if (format.includes('date')) {
        // Date format
        if (value instanceof Date) {
          return value.toLocaleDateString();
        }
      }
    }

    return String(value);
  }

  private applyTemplate(template: string, data: TooltipData): string {
    let html = template;

    // Replace title
    html = html.replace(/\{title\}/g, data.title || '');

    // Replace fields
    for (const field of data.fields) {
      const placeholder = `{${field.label}}`;
      const formattedValue = this.formatValue(field.value, field.format);
      html = html.replace(new RegExp(placeholder, 'g'), formattedValue);
    }

    return html;
  }

  // ============================================================================
  // Event Handlers
  // ============================================================================

  attachToElement(
    element: HTMLElement,
    getTooltipData: (event: MouseEvent) => TooltipData | null
  ): void {
    element.addEventListener('mouseenter', (event) => {
      const data = getTooltipData(event as MouseEvent);
      if (data) {
        this.show(data, event as MouseEvent);
      }
    });

    element.addEventListener('mousemove', (event) => {
      if (this.options.followCursor && this.tooltip && this.tooltip.style.display !== 'none') {
        this.positionTooltip(event as MouseEvent);
      }
    });

    element.addEventListener('mouseleave', () => {
      this.hide();
    });
  }

  attachToElements(
    elements: HTMLElement[],
    getTooltipData: (element: HTMLElement, event: MouseEvent) => TooltipData | null
  ): void {
    for (const element of elements) {
      this.attachToElement(element, (event) => getTooltipData(element, event));
    }
  }

  // ============================================================================
  // Styles
  // ============================================================================

  private getTooltipStyles(): string {
    const isDark = this.options.theme === 'dark';

    return `
      position: fixed;
      z-index: 10000;
      max-width: ${this.options.maxWidth}px;
      padding: 12px 16px;
      background-color: ${isDark ? '#2c3e50' : 'white'};
      color: ${isDark ? 'white' : '#333'};
      border: 1px solid ${isDark ? '#34495e' : '#e0e0e0'};
      border-radius: 6px;
      box-shadow: 0 4px 12px rgba(0, 0, 0, ${isDark ? '0.3' : '0.15'});
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      font-size: 13px;
      line-height: 1.5;
      pointer-events: none;
      transition: opacity 0.2s ease-in-out;
      white-space: nowrap;
    `;
  }

  private getTitleStyles(): string {
    return `
      font-weight: bold;
      font-size: 14px;
      margin-bottom: 8px;
      padding-bottom: 8px;
      border-bottom: 1px solid ${this.options.theme === 'dark' ? '#34495e' : '#e0e0e0'};
    `;
  }

  private getFieldStyles(): string {
    return `
      display: flex;
      justify-content: space-between;
      margin: 4px 0;
      gap: 16px;
    `;
  }

  // ============================================================================
  // Configuration
  // ============================================================================

  updateOptions(options: Partial<TooltipOptions>): void {
    this.options = { ...this.options, ...options };

    if (this.tooltip) {
      this.tooltip.style.cssText = this.getTooltipStyles();
    }
  }

  setTheme(theme: 'light' | 'dark'): void {
    this.updateOptions({ theme });
  }

  // ============================================================================
  // Cleanup
  // ============================================================================

  destroy(): void {
    this.hide();

    if (this.tooltip && this.tooltip.parentNode) {
      this.tooltip.parentNode.removeChild(this.tooltip);
    }

    this.tooltip = null;
    this.container = null;

    if (this.showTimeout) {
      clearTimeout(this.showTimeout);
      this.showTimeout = null;
    }
  }
}

// Factory function
export function createTooltipManager(options?: TooltipOptions): TooltipManager {
  return new TooltipManager(options);
}
