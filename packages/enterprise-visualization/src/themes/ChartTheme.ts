/**
 * Chart Theming System
 * Provides comprehensive theming for all visualization components
 */

import { ThemeConfig } from '../types';

/**
 * Base theme interface
 */
export interface ChartTheme extends ThemeConfig {
  name: string;
  palette: {
    primary: string[];
    secondary: string[];
    sequential: string[];
    diverging: string[];
    categorical: string[];
  };
  typography: {
    fontFamily: string;
    fontSize: {
      small: number;
      medium: number;
      large: number;
      xlarge: number;
    };
    fontWeight: {
      normal: number;
      medium: number;
      bold: number;
    };
  };
  spacing: {
    small: number;
    medium: number;
    large: number;
    xlarge: number;
  };
  borderRadius: {
    small: number;
    medium: number;
    large: number;
  };
  shadows: {
    small: string;
    medium: string;
    large: string;
  };
}

/**
 * Default Light Theme
 */
export const lightTheme: ChartTheme = {
  name: 'light',
  colorScheme: [
    '#3b82f6', // Blue
    '#8b5cf6', // Purple
    '#ec4899', // Pink
    '#f59e0b', // Amber
    '#10b981', // Emerald
    '#06b6d4', // Cyan
    '#f97316', // Orange
    '#84cc16', // Lime
    '#6366f1', // Indigo
    '#14b8a6', // Teal
  ],
  backgroundColor: '#ffffff',
  textColor: '#1f2937',
  gridColor: '#e5e7eb',
  fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
  fontSize: 12,
  palette: {
    primary: [
      '#eff6ff', '#dbeafe', '#bfdbfe', '#93c5fd', '#60a5fa',
      '#3b82f6', '#2563eb', '#1d4ed8', '#1e40af', '#1e3a8a',
    ],
    secondary: [
      '#f5f3ff', '#ede9fe', '#ddd6fe', '#c4b5fd', '#a78bfa',
      '#8b5cf6', '#7c3aed', '#6d28d9', '#5b21b6', '#4c1d95',
    ],
    sequential: [
      '#f0f9ff', '#e0f2fe', '#bae6fd', '#7dd3fc', '#38bdf8',
      '#0ea5e9', '#0284c7', '#0369a1', '#075985', '#0c4a6e',
    ],
    diverging: [
      '#7f1d1d', '#991b1b', '#dc2626', '#ef4444', '#f87171',
      '#f3f4f6', '#a5f3fc', '#67e8f9', '#22d3ee', '#06b6d4',
    ],
    categorical: [
      '#3b82f6', '#8b5cf6', '#ec4899', '#f59e0b', '#10b981',
      '#06b6d4', '#f97316', '#84cc16', '#6366f1', '#14b8a6',
    ],
  },
  typography: {
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
    fontSize: {
      small: 10,
      medium: 12,
      large: 14,
      xlarge: 16,
    },
    fontWeight: {
      normal: 400,
      medium: 500,
      bold: 700,
    },
  },
  spacing: {
    small: 4,
    medium: 8,
    large: 16,
    xlarge: 24,
  },
  borderRadius: {
    small: 2,
    medium: 4,
    large: 8,
  },
  shadows: {
    small: '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
    medium: '0 4px 6px -1px rgba(0, 0, 0, 0.1)',
    large: '0 10px 15px -3px rgba(0, 0, 0, 0.1)',
  },
};

/**
 * Dark Theme
 */
export const darkTheme: ChartTheme = {
  name: 'dark',
  colorScheme: [
    '#60a5fa', // Blue
    '#a78bfa', // Purple
    '#f472b6', // Pink
    '#fbbf24', // Amber
    '#34d399', // Emerald
    '#22d3ee', // Cyan
    '#fb923c', // Orange
    '#a3e635', // Lime
    '#818cf8', // Indigo
    '#2dd4bf', // Teal
  ],
  backgroundColor: '#1f2937',
  textColor: '#f9fafb',
  gridColor: '#374151',
  fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
  fontSize: 12,
  palette: {
    primary: [
      '#1e3a8a', '#1e40af', '#1d4ed8', '#2563eb', '#3b82f6',
      '#60a5fa', '#93c5fd', '#bfdbfe', '#dbeafe', '#eff6ff',
    ],
    secondary: [
      '#4c1d95', '#5b21b6', '#6d28d9', '#7c3aed', '#8b5cf6',
      '#a78bfa', '#c4b5fd', '#ddd6fe', '#ede9fe', '#f5f3ff',
    ],
    sequential: [
      '#0c4a6e', '#075985', '#0369a1', '#0284c7', '#0ea5e9',
      '#38bdf8', '#7dd3fc', '#bae6fd', '#e0f2fe', '#f0f9ff',
    ],
    diverging: [
      '#06b6d4', '#22d3ee', '#67e8f9', '#a5f3fc', '#f3f4f6',
      '#f87171', '#ef4444', '#dc2626', '#991b1b', '#7f1d1d',
    ],
    categorical: [
      '#60a5fa', '#a78bfa', '#f472b6', '#fbbf24', '#34d399',
      '#22d3ee', '#fb923c', '#a3e635', '#818cf8', '#2dd4bf',
    ],
  },
  typography: {
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
    fontSize: {
      small: 10,
      medium: 12,
      large: 14,
      xlarge: 16,
    },
    fontWeight: {
      normal: 400,
      medium: 500,
      bold: 700,
    },
  },
  spacing: {
    small: 4,
    medium: 8,
    large: 16,
    xlarge: 24,
  },
  borderRadius: {
    small: 2,
    medium: 4,
    large: 8,
  },
  shadows: {
    small: '0 1px 2px 0 rgba(0, 0, 0, 0.3)',
    medium: '0 4px 6px -1px rgba(0, 0, 0, 0.4)',
    large: '0 10px 15px -3px rgba(0, 0, 0, 0.5)',
  },
};

/**
 * High Contrast Theme (Accessibility)
 */
export const highContrastTheme: ChartTheme = {
  name: 'high-contrast',
  colorScheme: [
    '#000000', // Black
    '#ffff00', // Yellow
    '#00ffff', // Cyan
    '#ff00ff', // Magenta
    '#00ff00', // Green
    '#ff0000', // Red
    '#0000ff', // Blue
    '#ffffff', // White
  ],
  backgroundColor: '#ffffff',
  textColor: '#000000',
  gridColor: '#666666',
  fontFamily: 'Arial, sans-serif',
  fontSize: 14,
  palette: {
    primary: ['#000000', '#1a1a1a', '#333333', '#4d4d4d', '#666666', '#808080', '#999999', '#b3b3b3', '#cccccc', '#e6e6e6'],
    secondary: ['#ffff00', '#ffff33', '#ffff66', '#ffff99', '#ffffcc'],
    sequential: ['#000000', '#1a1a1a', '#333333', '#4d4d4d', '#666666', '#808080', '#999999', '#b3b3b3', '#cccccc', '#ffffff'],
    diverging: ['#ff0000', '#ff3333', '#ff6666', '#ff9999', '#ffffff', '#99ccff', '#6699ff', '#3366ff', '#0033ff'],
    categorical: ['#000000', '#ffff00', '#00ffff', '#ff00ff', '#00ff00', '#ff0000', '#0000ff', '#ffffff'],
  },
  typography: {
    fontFamily: 'Arial, sans-serif',
    fontSize: {
      small: 12,
      medium: 14,
      large: 16,
      xlarge: 18,
    },
    fontWeight: {
      normal: 400,
      medium: 600,
      bold: 700,
    },
  },
  spacing: {
    small: 6,
    medium: 12,
    large: 20,
    xlarge: 28,
  },
  borderRadius: {
    small: 0,
    medium: 0,
    large: 0,
  },
  shadows: {
    small: 'none',
    medium: 'none',
    large: 'none',
  },
};

/**
 * Pastel Theme
 */
export const pastelTheme: ChartTheme = {
  name: 'pastel',
  colorScheme: [
    '#a8d5e2', // Light Blue
    '#f9d5e5', // Light Pink
    '#eeeeaa', // Light Yellow
    '#c4e0c4', // Light Green
    '#e2c9f1', // Light Purple
    '#ffd9b3', // Light Orange
    '#b3e0f2', // Light Cyan
    '#f0b3c2', // Light Rose
  ],
  backgroundColor: '#fafafa',
  textColor: '#4a5568',
  gridColor: '#e2e8f0',
  fontFamily: 'Georgia, serif',
  fontSize: 12,
  palette: {
    primary: ['#e6f2ff', '#cce5ff', '#b3d9ff', '#99ccff', '#80bfff', '#66b3ff', '#4da6ff', '#3399ff', '#1a8cff', '#0080ff'],
    secondary: ['#ffe6f0', '#ffcce0', '#ffb3d1', '#ff99c2', '#ff80b3', '#ff66a3', '#ff4d94', '#ff3385', '#ff1a75', '#ff0066'],
    sequential: ['#f0f9ff', '#e0f2fe', '#bae6fd', '#7dd3fc', '#38bdf8', '#0ea5e9', '#0284c7', '#0369a1', '#075985', '#0c4a6e'],
    diverging: ['#fef3c7', '#fde68a', '#fcd34d', '#fbbf24', '#f59e0b', '#c4b5fd', '#a78bfa', '#8b5cf6', '#7c3aed', '#6d28d9'],
    categorical: ['#a8d5e2', '#f9d5e5', '#eeeeaa', '#c4e0c4', '#e2c9f1', '#ffd9b3', '#b3e0f2', '#f0b3c2'],
  },
  typography: {
    fontFamily: 'Georgia, serif',
    fontSize: {
      small: 10,
      medium: 12,
      large: 14,
      xlarge: 16,
    },
    fontWeight: {
      normal: 400,
      medium: 500,
      bold: 600,
    },
  },
  spacing: {
    small: 4,
    medium: 8,
    large: 16,
    xlarge: 24,
  },
  borderRadius: {
    small: 4,
    medium: 8,
    large: 12,
  },
  shadows: {
    small: '0 1px 3px 0 rgba(0, 0, 0, 0.1)',
    medium: '0 4px 6px -1px rgba(0, 0, 0, 0.1)',
    large: '0 10px 15px -3px rgba(0, 0, 0, 0.1)',
  },
};

/**
 * Corporate Theme
 */
export const corporateTheme: ChartTheme = {
  name: 'corporate',
  colorScheme: [
    '#1e40af', // Navy Blue
    '#059669', // Green
    '#dc2626', // Red
    '#d97706', // Orange
    '#7c3aed', // Purple
    '#0891b2', // Teal
    '#4b5563', // Gray
    '#ea580c', // Dark Orange
  ],
  backgroundColor: '#ffffff',
  textColor: '#111827',
  gridColor: '#d1d5db',
  fontFamily: '"Helvetica Neue", Helvetica, Arial, sans-serif',
  fontSize: 11,
  palette: {
    primary: ['#eff6ff', '#dbeafe', '#bfdbfe', '#93c5fd', '#60a5fa', '#3b82f6', '#2563eb', '#1d4ed8', '#1e40af', '#1e3a8a'],
    secondary: ['#f0fdf4', '#dcfce7', '#bbf7d0', '#86efac', '#4ade80', '#22c55e', '#16a34a', '#15803d', '#166534', '#14532d'],
    sequential: ['#f9fafb', '#f3f4f6', '#e5e7eb', '#d1d5db', '#9ca3af', '#6b7280', '#4b5563', '#374151', '#1f2937', '#111827'],
    diverging: ['#7f1d1d', '#991b1b', '#dc2626', '#ef4444', '#f87171', '#f3f4f6', '#34d399', '#10b981', '#059669', '#047857'],
    categorical: ['#1e40af', '#059669', '#dc2626', '#d97706', '#7c3aed', '#0891b2', '#4b5563', '#ea580c'],
  },
  typography: {
    fontFamily: '"Helvetica Neue", Helvetica, Arial, sans-serif',
    fontSize: {
      small: 9,
      medium: 11,
      large: 13,
      xlarge: 15,
    },
    fontWeight: {
      normal: 400,
      medium: 500,
      bold: 700,
    },
  },
  spacing: {
    small: 4,
    medium: 8,
    large: 12,
    xlarge: 20,
  },
  borderRadius: {
    small: 2,
    medium: 3,
    large: 4,
  },
  shadows: {
    small: '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
    medium: '0 2px 4px 0 rgba(0, 0, 0, 0.1)',
    large: '0 4px 8px 0 rgba(0, 0, 0, 0.15)',
  },
};

/**
 * Theme Manager Class
 */
export class ThemeManager {
  private currentTheme: ChartTheme = lightTheme;
  private customThemes: Map<string, ChartTheme> = new Map();

  constructor() {
    // Register default themes
    this.registerTheme(lightTheme);
    this.registerTheme(darkTheme);
    this.registerTheme(highContrastTheme);
    this.registerTheme(pastelTheme);
    this.registerTheme(corporateTheme);
  }

  /**
   * Register a custom theme
   */
  public registerTheme(theme: ChartTheme): void {
    this.customThemes.set(theme.name, theme);
  }

  /**
   * Get theme by name
   */
  public getTheme(name: string): ChartTheme | undefined {
    return this.customThemes.get(name);
  }

  /**
   * Set current theme
   */
  public setTheme(name: string): void {
    const theme = this.customThemes.get(name);
    if (theme) {
      this.currentTheme = theme;
    } else {
      console.warn(`Theme "${name}" not found. Using current theme.`);
    }
  }

  /**
   * Get current theme
   */
  public getCurrentTheme(): ChartTheme {
    return this.currentTheme;
  }

  /**
   * Create a custom theme based on an existing theme
   */
  public createCustomTheme(name: string, baseTheme: ChartTheme, overrides: Partial<ChartTheme>): ChartTheme {
    const customTheme: ChartTheme = {
      ...baseTheme,
      ...overrides,
      name,
    };
    this.registerTheme(customTheme);
    return customTheme;
  }

  /**
   * Get all available theme names
   */
  public getAvailableThemes(): string[] {
    return Array.from(this.customThemes.keys());
  }

  /**
   * Apply theme to DOM element
   */
  public applyTheme(element: HTMLElement, theme?: ChartTheme): void {
    const activeTheme = theme || this.currentTheme;

    element.style.setProperty('--chart-bg-color', activeTheme.backgroundColor || '#ffffff');
    element.style.setProperty('--chart-text-color', activeTheme.textColor || '#000000');
    element.style.setProperty('--chart-grid-color', activeTheme.gridColor || '#e0e0e0');
    element.style.setProperty('--chart-font-family', activeTheme.fontFamily || 'sans-serif');
    element.style.setProperty('--chart-font-size', `${activeTheme.fontSize || 12}px`);
  }
}

// Export singleton instance
export const themeManager = new ThemeManager();

// Export all themes
export const themes = {
  light: lightTheme,
  dark: darkTheme,
  highContrast: highContrastTheme,
  pastel: pastelTheme,
  corporate: corporateTheme,
};

export default themeManager;
