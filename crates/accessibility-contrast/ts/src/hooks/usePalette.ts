/**
 * React hook for color palette management and analysis
 */

import { useState, useMemo, useCallback } from 'react';
import { RGB, ColorPalette, PaletteColor, PaletteContrastPair } from '../types';
import { calculateContrast } from '../algorithms/ContrastCalculator';
import { optimizePalette } from '../algorithms/ColorOptimizer';
import { hexToRgb, rgbToHex, isValidHex } from '../utils/colorMath';

export interface UsePaletteOptions {
  /** Initial palette name */
  name?: string;
  /** Initial colors */
  initialColors?: PaletteColor[];
  /** Background color for contrast checking */
  background?: string | RGB;
  /** Minimum contrast ratio for compliance */
  minContrastRatio?: number;
}

export interface UsePaletteResult {
  /** Palette data */
  palette: ColorPalette;
  /** Is palette WCAG compliant */
  isCompliant: boolean;
  /** Add a color to palette */
  addColor: (color: PaletteColor) => void;
  /** Remove a color from palette */
  removeColor: (id: string) => void;
  /** Update a color in palette */
  updateColor: (id: string, color: Partial<PaletteColor>) => void;
  /** Clear all colors */
  clearColors: () => void;
  /** Set background color */
  setBackground: (background: string | RGB) => void;
  /** Get contrast for specific pair */
  getContrast: (fgId: string, bgId: string) => PaletteContrastPair | null;
  /** Optimize palette for accessibility */
  optimize: () => void;
  /** Export palette as JSON */
  exportJSON: () => string;
  /** Import palette from JSON */
  importJSON: (json: string) => void;
}

/**
 * Hook for managing and analyzing color palettes
 */
export function usePalette(options: UsePaletteOptions = {}): UsePaletteResult {
  const [name, setName] = useState(options.name || 'Untitled Palette');
  const [colors, setColors] = useState<PaletteColor[]>(options.initialColors || []);
  const [background, setBackgroundState] = useState<RGB>(() => {
    if (!options.background) return { r: 255, g: 255, b: 255 };
    return typeof options.background === 'string'
      ? hexToRgb(options.background)
      : options.background;
  });

  const minContrastRatio = options.minContrastRatio || 4.5;

  // Calculate contrast matrix
  const contrastMatrix = useMemo<PaletteContrastPair[]>(() => {
    const matrix: PaletteContrastPair[] = [];

    for (const fg of colors) {
      for (const bg of colors) {
        if (fg.id === bg.id) continue;

        const contrast = calculateContrast(fg.color, bg.color);
        matrix.push({
          foreground: fg,
          background: bg,
          contrast,
        });
      }

      // Also check against background
      const bgContrast = calculateContrast(fg.color, background);
      matrix.push({
        foreground: fg,
        background: {
          id: 'background',
          name: 'Background',
          color: background,
          hex: rgbToHex(background),
          role: 'background',
        },
        contrast: bgContrast,
      });
    }

    return matrix;
  }, [colors, background]);

  // Check if palette is WCAG compliant
  const isCompliant = useMemo(() => {
    // All text colors should meet minimum contrast with background
    return colors.every((color) => {
      const contrast = calculateContrast(color.color, background);
      return contrast.ratio >= minContrastRatio;
    });
  }, [colors, background, minContrastRatio]);

  // Create palette object
  const palette = useMemo<ColorPalette>(() => {
    return {
      id: `palette-${Date.now()}`,
      name,
      colors,
      contrastMatrix,
      wcagCompliant: isCompliant,
    };
  }, [name, colors, contrastMatrix, isCompliant]);

  // Callbacks
  const addColor = useCallback((color: PaletteColor) => {
    setColors((prev) => [...prev, color]);
  }, []);

  const removeColor = useCallback((id: string) => {
    setColors((prev) => prev.filter((c) => c.id !== id));
  }, []);

  const updateColor = useCallback((id: string, updates: Partial<PaletteColor>) => {
    setColors((prev) =>
      prev.map((c) => (c.id === id ? { ...c, ...updates } : c))
    );
  }, []);

  const clearColors = useCallback(() => {
    setColors([]);
  }, []);

  const setBackground = useCallback((bg: string | RGB) => {
    const rgb = typeof bg === 'string' ? hexToRgb(bg) : bg;
    setBackgroundState(rgb);
  }, []);

  const getContrast = useCallback(
    (fgId: string, bgId: string): PaletteContrastPair | null => {
      return (
        contrastMatrix.find(
          (pair) => pair.foreground.id === fgId && pair.background.id === bgId
        ) || null
      );
    },
    [contrastMatrix]
  );

  const optimize = useCallback(() => {
    const rgbColors = colors.map((c) => c.color);
    const optimizedMap = optimizePalette(rgbColors, background, minContrastRatio);

    setColors((prev) =>
      prev.map((color) => {
        const optimized = optimizedMap.get(color.color);
        if (optimized && optimized !== color.color) {
          return {
            ...color,
            color: optimized,
            hex: rgbToHex(optimized),
          };
        }
        return color;
      })
    );
  }, [colors, background, minContrastRatio]);

  const exportJSON = useCallback((): string => {
    return JSON.stringify(palette, null, 2);
  }, [palette]);

  const importJSON = useCallback((json: string) => {
    try {
      const imported = JSON.parse(json) as ColorPalette;
      setName(imported.name);
      setColors(imported.colors);
    } catch (err) {
      console.error('Failed to import palette:', err);
    }
  }, []);

  return {
    palette,
    isCompliant,
    addColor,
    removeColor,
    updateColor,
    clearColors,
    setBackground,
    getContrast,
    optimize,
    exportJSON,
    importJSON,
  };
}

/**
 * Hook for generating a random accessible palette
 */
export function useRandomPalette(
  count: number = 5,
  background: RGB = { r: 255, g: 255, b: 255 }
): PaletteColor[] {
  return useMemo(() => {
    const colors: PaletteColor[] = [];

    for (let i = 0; i < count; i++) {
      // Generate random hue
      const hue = (360 / count) * i;

      // Generate random saturation and lightness
      // Ensure sufficient contrast with background
      const isDarkBg =
        background.r < 128 || background.g < 128 || background.b < 128;
      const lightness = isDarkBg ? 70 + Math.random() * 20 : 20 + Math.random() * 30;

      // Convert HSL to RGB (simple approximation)
      const s = 70 + Math.random() * 30;
      const l = lightness;

      const rgb = hslToRgbSimple(hue, s, l);

      colors.push({
        id: `color-${i}`,
        name: `Color ${i + 1}`,
        color: rgb,
        hex: rgbToHex(rgb),
        role: i === 0 ? 'primary' : 'custom',
      });
    }

    return colors;
  }, [count, background]);
}

// Simple HSL to RGB conversion for palette generation
function hslToRgbSimple(h: number, s: number, l: number): RGB {
  h = h / 360;
  s = s / 100;
  l = l / 100;

  const hue2rgb = (p: number, q: number, t: number): number => {
    if (t < 0) t += 1;
    if (t > 1) t -= 1;
    if (t < 1 / 6) return p + (q - p) * 6 * t;
    if (t < 1 / 2) return q;
    if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
    return p;
  };

  let r, g, b;

  if (s === 0) {
    r = g = b = l;
  } else {
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    r = hue2rgb(p, q, h + 1 / 3);
    g = hue2rgb(p, q, h);
    b = hue2rgb(p, q, h - 1 / 3);
  }

  return {
    r: Math.round(r * 255),
    g: Math.round(g * 255),
    b: Math.round(b * 255),
  };
}
