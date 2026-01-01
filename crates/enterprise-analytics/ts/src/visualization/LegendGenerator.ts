/**
 * Automatic Legend Generation
 * @module @harborgrid/enterprise-analytics/visualization
 */

import type { LegendConfig } from '../types';

export interface LegendItem {
  label: string;
  color: string;
  shape?: 'circle' | 'square' | 'line' | 'triangle';
  value?: number;
  visible?: boolean;
}

export interface LegendOptions extends LegendConfig {
  maxItems?: number;
  itemSpacing?: number;
  fontSize?: number;
  onItemClick?: (item: LegendItem) => void;
  onItemHover?: (item: LegendItem) => void;
}

export class LegendGenerator {
  private container: HTMLElement | null = null;
  private items: LegendItem[] = [];
  private options: LegendOptions;

  constructor(options: LegendOptions) {
    this.options = {
      show: true,
      position: 'right',
      orientation: 'vertical',
      interactive: true,
      maxItems: 20,
      itemSpacing: 8,
      fontSize: 12,
      ...options,
    };
  }

  // ============================================================================
  // Legend Creation
  // ============================================================================

  create(container: HTMLElement, items: LegendItem[]): void {
    if (!this.options.show) return;

    this.container = container;
    this.items = items.slice(0, this.options.maxItems);

    const legendEl = this.createLegendElement();
    container.appendChild(legendEl);
  }

  private createLegendElement(): HTMLElement {
    const legend = document.createElement('div');
    legend.className = 'analytics-legend';
    legend.style.cssText = this.getLegendStyles();

    // Add title if provided
    if (this.options.title) {
      const title = document.createElement('div');
      title.className = 'analytics-legend-title';
      title.textContent = this.options.title;
      title.style.cssText = `
        font-weight: bold;
        margin-bottom: 8px;
        font-size: ${(this.options.fontSize || 12) + 2}px;
      `;
      legend.appendChild(title);
    }

    // Add legend items
    const itemsContainer = document.createElement('div');
    itemsContainer.className = 'analytics-legend-items';
    itemsContainer.style.cssText = this.getItemsContainerStyles();

    for (const item of this.items) {
      const itemEl = this.createLegendItem(item);
      itemsContainer.appendChild(itemEl);
    }

    legend.appendChild(itemsContainer);

    return legend;
  }

  private createLegendItem(item: LegendItem): HTMLElement {
    const itemEl = document.createElement('div');
    itemEl.className = 'analytics-legend-item';
    itemEl.style.cssText = this.getLegendItemStyles();
    itemEl.dataset.label = item.label;

    // Add marker
    const marker = this.createMarker(item);
    itemEl.appendChild(marker);

    // Add label
    const label = document.createElement('span');
    label.className = 'analytics-legend-label';
    label.textContent = item.label;
    label.style.cssText = `
      font-size: ${this.options.fontSize}px;
      margin-left: 8px;
      color: ${item.visible === false ? '#999' : '#333'};
    `;

    if (item.value !== undefined) {
      label.textContent += ` (${item.value})`;
    }

    itemEl.appendChild(label);

    // Add interactivity
    if (this.options.interactive) {
      this.addInteractivity(itemEl, item);
    }

    return itemEl;
  }

  private createMarker(item: LegendItem): HTMLElement {
    const marker = document.createElement('div');
    marker.className = 'analytics-legend-marker';

    const shape = item.shape || 'square';
    const size = this.options.fontSize || 12;

    switch (shape) {
      case 'circle':
        marker.style.cssText = `
          width: ${size}px;
          height: ${size}px;
          border-radius: 50%;
          background-color: ${item.color};
          display: inline-block;
          vertical-align: middle;
        `;
        break;

      case 'square':
        marker.style.cssText = `
          width: ${size}px;
          height: ${size}px;
          background-color: ${item.color};
          display: inline-block;
          vertical-align: middle;
        `;
        break;

      case 'line':
        marker.style.cssText = `
          width: ${size * 1.5}px;
          height: 2px;
          background-color: ${item.color};
          display: inline-block;
          vertical-align: middle;
          margin-top: ${size / 2}px;
        `;
        break;

      case 'triangle':
        marker.style.cssText = `
          width: 0;
          height: 0;
          border-left: ${size / 2}px solid transparent;
          border-right: ${size / 2}px solid transparent;
          border-bottom: ${size}px solid ${item.color};
          display: inline-block;
          vertical-align: middle;
        `;
        break;
    }

    return marker;
  }

  // ============================================================================
  // Interactivity
  // ============================================================================

  private addInteractivity(itemEl: HTMLElement, item: LegendItem): void {
    itemEl.style.cursor = 'pointer';

    itemEl.addEventListener('click', () => {
      if (this.options.onItemClick) {
        this.options.onItemClick(item);
      }

      // Toggle visibility
      item.visible = !(item.visible ?? true);
      this.update();
    });

    itemEl.addEventListener('mouseenter', () => {
      itemEl.style.backgroundColor = '#f5f5f5';
      if (this.options.onItemHover) {
        this.options.onItemHover(item);
      }
    });

    itemEl.addEventListener('mouseleave', () => {
      itemEl.style.backgroundColor = 'transparent';
    });
  }

  // ============================================================================
  // Legend Updates
  // ============================================================================

  update(items?: LegendItem[]): void {
    if (items) {
      this.items = items.slice(0, this.options.maxItems);
    }

    if (this.container) {
      this.container.innerHTML = '';
      this.create(this.container, this.items);
    }
  }

  updateItem(label: string, updates: Partial<LegendItem>): void {
    const item = this.items.find((i) => i.label === label);
    if (item) {
      Object.assign(item, updates);
      this.update();
    }
  }

  toggleItem(label: string): void {
    const item = this.items.find((i) => i.label === label);
    if (item) {
      item.visible = !(item.visible ?? true);
      this.update();
    }
  }

  // ============================================================================
  // Styles
  // ============================================================================

  private getLegendStyles(): string {
    const { position, orientation } = this.options;

    let styles = `
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      padding: 12px;
      background-color: white;
      border: 1px solid #e0e0e0;
      border-radius: 4px;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    `;

    if (position === 'top' || position === 'bottom') {
      styles += `
        display: flex;
        justify-content: center;
      `;
    }

    return styles;
  }

  private getItemsContainerStyles(): string {
    const { orientation, itemSpacing } = this.options;

    if (orientation === 'horizontal') {
      return `
        display: flex;
        flex-direction: row;
        gap: ${itemSpacing}px;
        flex-wrap: wrap;
      `;
    }

    return `
      display: flex;
      flex-direction: column;
      gap: ${itemSpacing}px;
    `;
  }

  private getLegendItemStyles(): string {
    return `
      display: flex;
      align-items: center;
      padding: 4px 8px;
      border-radius: 3px;
      transition: background-color 0.2s;
    `;
  }

  // ============================================================================
  // Utilities
  // ============================================================================

  getVisibleItems(): LegendItem[] {
    return this.items.filter((item) => item.visible !== false);
  }

  getHiddenItems(): LegendItem[] {
    return this.items.filter((item) => item.visible === false);
  }

  showAll(): void {
    this.items.forEach((item) => {
      item.visible = true;
    });
    this.update();
  }

  hideAll(): void {
    this.items.forEach((item) => {
      item.visible = false;
    });
    this.update();
  }

  destroy(): void {
    if (this.container) {
      this.container.innerHTML = '';
      this.container = null;
    }
    this.items = [];
  }

  // ============================================================================
  // Export
  // ============================================================================

  toHTML(): string {
    const temp = document.createElement('div');
    this.create(temp, this.items);
    return temp.innerHTML;
  }

  toJSON(): LegendItem[] {
    return this.items.map((item) => ({ ...item }));
  }
}

// Factory function
export function createLegendGenerator(options: LegendOptions): LegendGenerator {
  return new LegendGenerator(options);
}
