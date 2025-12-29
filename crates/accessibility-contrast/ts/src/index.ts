/**
 * @harborgrid/accessibility-contrast
 * Enterprise-grade color contrast analyzer with WCAG 2.1 and APCA compliance
 *
 * @author HarborGrid
 * @license MIT
 */

// ============================================================================
// Types
// ============================================================================
export * from './types';

// ============================================================================
// Utilities
// ============================================================================
export * from './utils/colorMath';

// ============================================================================
// Algorithms
// ============================================================================
export * from './algorithms/ColorConverter';
export * from './algorithms/LuminanceCalculator';
export * from './algorithms/ContrastCalculator';
export * from './algorithms/ColorBlindness';
export * from './algorithms/ColorOptimizer';

// ============================================================================
// Hooks
// ============================================================================
export * from './hooks/useContrast';
export * from './hooks/useColorBlindness';
export * from './hooks/usePalette';

// ============================================================================
// Components - Analyzer
// ============================================================================
export { ContrastAnalyzer } from './components/Analyzer/ContrastAnalyzer';
export type { ContrastAnalyzerProps } from './components/Analyzer/ContrastAnalyzer';

export { ColorPicker } from './components/Analyzer/ColorPicker';
export type { ColorPickerProps } from './components/Analyzer/ColorPicker';

export { ContrastPreview } from './components/Analyzer/ContrastPreview';
export type { ContrastPreviewProps } from './components/Analyzer/ContrastPreview';

export { ContrastRatio } from './components/Analyzer/ContrastRatio';
export type { ContrastRatioProps } from './components/Analyzer/ContrastRatio';

// ============================================================================
// Components - Palette
// ============================================================================
export { PaletteBuilder } from './components/Palette/PaletteBuilder';
export type { PaletteBuilderProps } from './components/Palette/PaletteBuilder';

export { ColorSwatch } from './components/Palette/ColorSwatch';
export type { ColorSwatchProps } from './components/Palette/ColorSwatch';

export { PaletteMatrix } from './components/Palette/PaletteMatrix';
export type { PaletteMatrixProps } from './components/Palette/PaletteMatrix';

// ============================================================================
// Components - Simulation
// ============================================================================
export { ColorBlindSimulator } from './components/Simulation/ColorBlindSimulator';
export type { ColorBlindSimulatorProps } from './components/Simulation/ColorBlindSimulator';

export { SimulationPreview } from './components/Simulation/SimulationPreview';
export type { SimulationPreviewProps } from './components/Simulation/SimulationPreview';

// ============================================================================
// Components - Suggestions
// ============================================================================
export { ContrastSuggestions } from './components/Suggestions/ContrastSuggestions';
export type { ContrastSuggestionsProps } from './components/Suggestions/ContrastSuggestions';

export { AlternativeColors } from './components/Suggestions/AlternativeColors';
export type { AlternativeColorsProps } from './components/Suggestions/AlternativeColors';

// ============================================================================
// Convenience Functions
// ============================================================================

import { RGB } from './types';
import { calculateContrast } from './algorithms/ContrastCalculator';
import { hexToRgb } from './utils/colorMath';

/**
 * Quick contrast check between two colors
 * @param foreground - Foreground color (hex or RGB)
 * @param background - Background color (hex or RGB)
 * @returns Contrast ratio
 */
export function checkContrast(
  foreground: string | RGB,
  background: string | RGB
): number {
  const fg = typeof foreground === 'string' ? hexToRgb(foreground) : foreground;
  const bg = typeof background === 'string' ? hexToRgb(background) : background;
  return calculateContrast(fg, bg).ratio;
}

/**
 * Check if two colors meet WCAG AA requirements
 * @param foreground - Foreground color (hex or RGB)
 * @param background - Background color (hex or RGB)
 * @param isLargeText - Whether the text is large (18pt+ or 14pt+ bold)
 * @returns true if meets AA requirements
 */
export function isAccessible(
  foreground: string | RGB,
  background: string | RGB,
  isLargeText: boolean = false
): boolean {
  const ratio = checkContrast(foreground, background);
  return isLargeText ? ratio >= 3.0 : ratio >= 4.5;
}

/**
 * Check if two colors meet WCAG AAA requirements
 * @param foreground - Foreground color (hex or RGB)
 * @param background - Background color (hex or RGB)
 * @param isLargeText - Whether the text is large (18pt+ or 14pt+ bold)
 * @returns true if meets AAA requirements
 */
export function isAAA(
  foreground: string | RGB,
  background: string | RGB,
  isLargeText: boolean = false
): boolean {
  const ratio = checkContrast(foreground, background);
  return isLargeText ? ratio >= 4.5 : ratio >= 7.0;
}

/**
 * Version information
 */
export const VERSION = '1.0.0';

/**
 * Package metadata
 */
export const PACKAGE_INFO = {
  name: '@harborgrid/accessibility-contrast',
  version: VERSION,
  description: 'Enterprise-grade color contrast analyzer with WCAG 2.1 and APCA compliance',
  author: 'HarborGrid',
  license: 'MIT',
};
