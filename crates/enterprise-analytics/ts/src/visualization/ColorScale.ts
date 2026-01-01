/**
 * Color Scale and Palette Management
 * @module @harborgrid/enterprise-analytics/visualization
 */

import type { ColorScaleConfig } from '../types';

export interface ColorPalette {
  name: string;
  colors: string[];
  type: 'categorical' | 'sequential' | 'diverging';
  accessibility?: {
    colorBlindSafe: boolean;
    contrastRatio: number;
  };
}

export class ColorScale {
  private palettes: Map<string, ColorPalette>;
  private customColors: Map<string, string>;

  constructor() {
    this.palettes = new Map();
    this.customColors = new Map();
    this.registerDefaultPalettes();
  }

  // ============================================================================
  // Palette Registration
  // ============================================================================

  private registerDefaultPalettes(): void {
    // Categorical palettes
    this.registerPalette({
      name: 'default',
      colors: ['#1f77b4', '#ff7f0e', '#2ca02c', '#d62728', '#9467bd', '#8c564b', '#e377c2', '#7f7f7f', '#bcbd22', '#17becf'],
      type: 'categorical',
      accessibility: { colorBlindSafe: true, contrastRatio: 4.5 },
    });

    this.registerPalette({
      name: 'vibrant',
      colors: ['#e74c3c', '#3498db', '#2ecc71', '#f39c12', '#9b59b6', '#1abc9c', '#e67e22', '#34495e'],
      type: 'categorical',
      accessibility: { colorBlindSafe: false, contrastRatio: 4.0 },
    });

    this.registerPalette({
      name: 'pastel',
      colors: ['#a8e6cf', '#dcedc1', '#ffd3b6', '#ffaaa5', '#ff8b94', '#b4a7d6', '#d4a5a5', '#ffccbc'],
      type: 'categorical',
      accessibility: { colorBlindSafe: false, contrastRatio: 2.5 },
    });

    // Sequential palettes
    this.registerPalette({
      name: 'blues',
      colors: ['#f7fbff', '#deebf7', '#c6dbef', '#9ecae1', '#6baed6', '#4292c6', '#2171b5', '#08519c', '#08306b'],
      type: 'sequential',
      accessibility: { colorBlindSafe: true, contrastRatio: 3.0 },
    });

    this.registerPalette({
      name: 'greens',
      colors: ['#f7fcf5', '#e5f5e0', '#c7e9c0', '#a1d99b', '#74c476', '#41ab5d', '#238b45', '#006d2c', '#00441b'],
      type: 'sequential',
      accessibility: { colorBlindSafe: true, contrastRatio: 3.0 },
    });

    // Diverging palettes
    this.registerPalette({
      name: 'rdylbu',
      colors: ['#d73027', '#f46d43', '#fdae61', '#fee090', '#ffffbf', '#e0f3f8', '#abd9e9', '#74add1', '#4575b4'],
      type: 'diverging',
      accessibility: { colorBlindSafe: true, contrastRatio: 3.5 },
    });
  }

  registerPalette(palette: ColorPalette): void {
    this.palettes.set(palette.name, palette);
  }

  // ============================================================================
  // Color Generation
  // ============================================================================

  getColor(value: string | number, config: ColorScaleConfig): string {
    // Check custom colors first
    if (config.customColors && typeof value === 'string') {
      const customColor = config.customColors[value];
      if (customColor) {
        return customColor;
      }
    }

    // Get palette
    const palette = this.getPalette(config.scheme || 'default', config.type);

    if (config.type === 'categorical') {
      return this.getCategoricalColor(value, palette, config);
    } else if (config.type === 'sequential') {
      return this.getSequentialColor(value as number, palette, config);
    } else if (config.type === 'diverging') {
      return this.getDivergingColor(value as number, palette, config);
    }

    return palette.colors[0] || '#000000';
  }

  private getCategoricalColor(
    value: string | number,
    palette: ColorPalette,
    config: ColorScaleConfig
  ): string {
    if (config.domain && config.range) {
      const index = config.domain.indexOf(value);
      if (index !== -1 && index < config.range.length) {
        return config.range[index];
      }
    }

    // Hash the value to get a consistent color
    const hash = this.hashCode(String(value));
    const index = Math.abs(hash) % palette.colors.length;
    return palette.colors[index] || palette.colors[0]!;
  }

  private getSequentialColor(
    value: number,
    palette: ColorPalette,
    config: ColorScaleConfig
  ): string {
    const domain = config.domain as [number, number] || [0, 100];
    const normalized = (value - domain[0]) / (domain[1] - domain[0]);
    const clamped = Math.max(0, Math.min(1, normalized));
    const index = Math.floor(clamped * (palette.colors.length - 1));
    return palette.colors[index] || palette.colors[0]!;
  }

  private getDivergingColor(
    value: number,
    palette: ColorPalette,
    config: ColorScaleConfig
  ): string {
    const domain = config.domain as [number, number] || [-100, 100];
    const mid = (domain[0] + domain[1]) / 2;
    const normalized = (value - mid) / ((domain[1] - domain[0]) / 2);
    const clamped = Math.max(-1, Math.min(1, normalized));
    const index = Math.floor(((clamped + 1) / 2) * (palette.colors.length - 1));
    return palette.colors[index] || palette.colors[0]!;
  }

  // ============================================================================
  // Palette Management
  // ============================================================================

  getPalette(name: string, type?: 'categorical' | 'sequential' | 'diverging'): ColorPalette {
    const palette = this.palettes.get(name);

    if (palette) {
      if (type && palette.type !== type) {
        console.warn(`Palette ${name} is ${palette.type}, not ${type}`);
      }
      return palette;
    }

    // Return default palette if not found
    return this.palettes.get('default')!;
  }

  getAllPalettes(type?: 'categorical' | 'sequential' | 'diverging'): ColorPalette[] {
    const palettes = Array.from(this.palettes.values());

    if (type) {
      return palettes.filter((p) => p.type === type);
    }

    return palettes;
  }

  // ============================================================================
  // Color Utilities
  // ============================================================================

  interpolateColor(color1: string, color2: string, t: number): string {
    const rgb1 = this.hexToRgb(color1);
    const rgb2 = this.hexToRgb(color2);

    if (!rgb1 || !rgb2) {
      return color1;
    }

    const r = Math.round(rgb1.r + (rgb2.r - rgb1.r) * t);
    const g = Math.round(rgb1.g + (rgb2.g - rgb1.g) * t);
    const b = Math.round(rgb1.b + (rgb2.b - rgb1.b) * t);

    return this.rgbToHex(r, g, b);
  }

  generateGradient(colors: string[], steps: number): string[] {
    const gradient: string[] = [];
    const segmentSteps = Math.floor(steps / (colors.length - 1));

    for (let i = 0; i < colors.length - 1; i++) {
      for (let j = 0; j < segmentSteps; j++) {
        const t = j / segmentSteps;
        gradient.push(this.interpolateColor(colors[i]!, colors[i + 1]!, t));
      }
    }

    gradient.push(colors[colors.length - 1]!);
    return gradient;
  }

  lighten(color: string, amount: number): string {
    const rgb = this.hexToRgb(color);
    if (!rgb) return color;

    const r = Math.min(255, Math.round(rgb.r + (255 - rgb.r) * amount));
    const g = Math.min(255, Math.round(rgb.g + (255 - rgb.g) * amount));
    const b = Math.min(255, Math.round(rgb.b + (255 - rgb.b) * amount));

    return this.rgbToHex(r, g, b);
  }

  darken(color: string, amount: number): string {
    const rgb = this.hexToRgb(color);
    if (!rgb) return color;

    const r = Math.max(0, Math.round(rgb.r * (1 - amount)));
    const g = Math.max(0, Math.round(rgb.g * (1 - amount)));
    const b = Math.max(0, Math.round(rgb.b * (1 - amount)));

    return this.rgbToHex(r, g, b);
  }

  getContrastColor(backgroundColor: string): string {
    const rgb = this.hexToRgb(backgroundColor);
    if (!rgb) return '#000000';

    // Calculate relative luminance
    const luminance = (0.299 * rgb.r + 0.587 * rgb.g + 0.114 * rgb.b) / 255;

    return luminance > 0.5 ? '#000000' : '#ffffff';
  }

  // ============================================================================
  // Accessibility
  // ============================================================================

  getAccessiblePalette(type: 'categorical' | 'sequential' | 'diverging'): ColorPalette {
    const palettes = this.getAllPalettes(type);
    const accessible = palettes.filter(
      (p) => p.accessibility?.colorBlindSafe && (p.accessibility?.contrastRatio || 0) >= 4.5
    );

    return accessible[0] || palettes[0]!;
  }

  isColorBlindSafe(colors: string[]): boolean {
    // Simple check - in a real implementation would use a color blind simulator
    // For now, check if colors are sufficiently different in lightness
    const lightnesses = colors.map((c) => {
      const rgb = this.hexToRgb(c);
      if (!rgb) return 0;
      return (0.299 * rgb.r + 0.587 * rgb.g + 0.114 * rgb.b) / 255;
    });

    for (let i = 0; i < lightnesses.length; i++) {
      for (let j = i + 1; j < lightnesses.length; j++) {
        if (Math.abs((lightnesses[i] || 0) - (lightnesses[j] || 0)) < 0.15) {
          return false;
        }
      }
    }

    return true;
  }

  // ============================================================================
  // Color Conversion
  // ============================================================================

  private hexToRgb(hex: string): { r: number; g: number; b: number } | null {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result
      ? {
          r: parseInt(result[1]!, 16),
          g: parseInt(result[2]!, 16),
          b: parseInt(result[3]!, 16),
        }
      : null;
  }

  private rgbToHex(r: number, g: number, b: number): string {
    return '#' + [r, g, b].map((x) => x.toString(16).padStart(2, '0')).join('');
  }

  private hashCode(str: string): number {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash;
    }
    return hash;
  }
}

// Factory function
export function createColorScale(): ColorScale {
  return new ColorScale();
}
