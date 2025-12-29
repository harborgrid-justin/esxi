/**
 * React hook for contrast analysis
 */

import { useState, useEffect, useMemo, useCallback } from 'react';
import { RGB, ContrastResult, ColorAnalysisReport, OptimizationOptions, WCAGConformance } from '../types';
import { calculateContrast } from '../algorithms/ContrastCalculator';
import { generateColorSuggestions } from '../algorithms/ColorOptimizer';
import { simulateAllTypes } from '../algorithms/ColorBlindness';
import { hexToRgb, isValidHex, rgbToHex } from '../utils/colorMath';

export interface UseContrastOptions {
  /** Foreground color (hex or RGB) */
  foreground: string | RGB;
  /** Background color (hex or RGB) */
  background: string | RGB;
  /** Auto-generate suggestions */
  autoSuggest?: boolean;
  /** Target WCAG conformance level */
  targetLevel?: WCAGConformance;
  /** Maximum color distance for suggestions */
  maxDistance?: number;
}

export interface UseContrastResult {
  /** Contrast analysis result */
  contrast: ContrastResult | null;
  /** Foreground RGB color */
  foregroundRGB: RGB | null;
  /** Background RGB color */
  backgroundRGB: RGB | null;
  /** Foreground hex color */
  foregroundHex: string | null;
  /** Background hex color */
  backgroundHex: string | null;
  /** Is the contrast accessible (AA) */
  isAccessible: boolean;
  /** Is the contrast excellent (AAA) */
  isExcellent: boolean;
  /** Contrast grade (A-F) */
  grade: string;
  /** Color suggestions for improvement */
  suggestions: ReturnType<typeof generateColorSuggestions>;
  /** Full analysis report */
  report: ColorAnalysisReport | null;
  /** Errors if any */
  error: string | null;
  /** Update foreground color */
  setForeground: (color: string | RGB) => void;
  /** Update background color */
  setBackground: (color: string | RGB) => void;
  /** Refresh analysis */
  refresh: () => void;
}

/**
 * Hook for analyzing color contrast
 */
export function useContrast(options: UseContrastOptions): UseContrastResult {
  const [foreground, setForegroundState] = useState<string | RGB>(options.foreground);
  const [background, setBackgroundState] = useState<string | RGB>(options.background);
  const [error, setError] = useState<string | null>(null);

  // Convert colors to RGB
  const foregroundRGB = useMemo(() => {
    try {
      setError(null);
      if (typeof foreground === 'string') {
        if (!isValidHex(foreground)) {
          throw new Error('Invalid foreground hex color');
        }
        return hexToRgb(foreground);
      }
      return foreground;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Invalid foreground color');
      return null;
    }
  }, [foreground]);

  const backgroundRGB = useMemo(() => {
    try {
      setError(null);
      if (typeof background === 'string') {
        if (!isValidHex(background)) {
          throw new Error('Invalid background hex color');
        }
        return hexToRgb(background);
      }
      return background;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Invalid background color');
      return null;
    }
  }, [background]);

  // Calculate contrast
  const contrast = useMemo(() => {
    if (!foregroundRGB || !backgroundRGB) return null;
    try {
      return calculateContrast(foregroundRGB, backgroundRGB);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Contrast calculation failed');
      return null;
    }
  }, [foregroundRGB, backgroundRGB]);

  // Get hex colors
  const foregroundHex = useMemo(() => {
    return foregroundRGB ? rgbToHex(foregroundRGB) : null;
  }, [foregroundRGB]);

  const backgroundHex = useMemo(() => {
    return backgroundRGB ? rgbToHex(backgroundRGB) : null;
  }, [backgroundRGB]);

  // Calculate accessibility
  const isAccessible = useMemo(() => {
    return contrast ? contrast.wcag.normalTextAA : false;
  }, [contrast]);

  const isExcellent = useMemo(() => {
    return contrast ? contrast.wcag.normalTextAAA : false;
  }, [contrast]);

  // Calculate grade
  const grade = useMemo(() => {
    if (!contrast) return 'F';
    const ratio = contrast.ratio;
    if (ratio >= 12) return 'A+';
    if (ratio >= 7) return 'A';
    if (ratio >= 4.5) return 'B';
    if (ratio >= 3) return 'C';
    if (ratio >= 2) return 'D';
    return 'F';
  }, [contrast]);

  // Generate suggestions
  const suggestions = useMemo(() => {
    if (!options.autoSuggest || !foregroundRGB || !backgroundRGB || isAccessible) {
      return [];
    }

    const optimizationOptions: OptimizationOptions = {
      target: options.targetLevel || WCAGConformance.NORMAL_TEXT_AA,
      maxDistance: options.maxDistance,
      preserveHue: true,
      suggestionCount: 10,
    };

    try {
      return generateColorSuggestions(foregroundRGB, backgroundRGB, optimizationOptions);
    } catch (err) {
      return [];
    }
  }, [foregroundRGB, backgroundRGB, isAccessible, options.autoSuggest, options.targetLevel, options.maxDistance]);

  // Generate full report
  const report = useMemo<ColorAnalysisReport | null>(() => {
    if (!foregroundRGB || !backgroundRGB || !contrast) return null;

    const simulations = simulateAllTypes(foregroundRGB);

    return {
      foreground: foregroundRGB,
      background: backgroundRGB,
      contrast,
      accessible: isAccessible,
      suggestions,
      simulations,
    };
  }, [foregroundRGB, backgroundRGB, contrast, isAccessible, suggestions]);

  // Callbacks
  const setForeground = useCallback((color: string | RGB) => {
    setForegroundState(color);
  }, []);

  const setBackground = useCallback((color: string | RGB) => {
    setBackgroundState(color);
  }, []);

  const refresh = useCallback(() => {
    setForegroundState(options.foreground);
    setBackgroundState(options.background);
  }, [options.foreground, options.background]);

  return {
    contrast,
    foregroundRGB,
    backgroundRGB,
    foregroundHex,
    backgroundHex,
    isAccessible,
    isExcellent,
    grade,
    suggestions,
    report,
    error,
    setForeground,
    setBackground,
    refresh,
  };
}

/**
 * Simple hook for quick contrast check
 */
export function useContrastCheck(foreground: string | RGB, background: string | RGB): {
  ratio: number | null;
  isAccessible: boolean;
  isExcellent: boolean;
} {
  const { contrast } = useContrast({ foreground, background });

  return {
    ratio: contrast?.ratio || null,
    isAccessible: contrast?.wcag.normalTextAA || false,
    isExcellent: contrast?.wcag.normalTextAAA || false,
  };
}
