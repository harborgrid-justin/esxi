/**
 * Color representation types and interfaces for accessibility analysis
 */

/**
 * RGB color space representation (0-255)
 */
export interface RGB {
  r: number;
  g: number;
  b: number;
  alpha?: number;
}

/**
 * HSL color space representation
 * h: 0-360, s: 0-100, l: 0-100
 */
export interface HSL {
  h: number;
  s: number;
  l: number;
  alpha?: number;
}

/**
 * CIE LAB color space representation
 */
export interface LAB {
  l: number; // 0-100
  a: number; // -128 to 127
  b: number; // -128 to 127
  alpha?: number;
}

/**
 * LCH color space representation (cylindrical LAB)
 */
export interface LCH {
  l: number; // 0-100
  c: number; // 0-150+
  h: number; // 0-360
  alpha?: number;
}

/**
 * XYZ color space representation (CIE 1931)
 */
export interface XYZ {
  x: number;
  y: number;
  z: number;
}

/**
 * Hex color string (e.g., "#FF5733")
 */
export type HexColor = string;

/**
 * Union type for all supported color formats
 */
export type Color = RGB | HSL | LAB | LCH | HexColor;

/**
 * WCAG compliance levels
 */
export enum WCAGLevel {
  AA = 'AA',
  AAA = 'AAA',
}

/**
 * WCAG conformance levels for different content types
 */
export enum WCAGConformance {
  NORMAL_TEXT_AA = 'NORMAL_TEXT_AA', // 4.5:1
  NORMAL_TEXT_AAA = 'NORMAL_TEXT_AAA', // 7:1
  LARGE_TEXT_AA = 'LARGE_TEXT_AA', // 3:1
  LARGE_TEXT_AAA = 'LARGE_TEXT_AAA', // 4.5:1
  UI_COMPONENTS = 'UI_COMPONENTS', // 3:1
}

/**
 * Contrast calculation result
 */
export interface ContrastResult {
  /** Contrast ratio (1-21) */
  ratio: number;
  /** WCAG 2.1 compliance */
  wcag: {
    normalTextAA: boolean;
    normalTextAAA: boolean;
    largeTextAA: boolean;
    largeTextAAA: boolean;
    uiComponents: boolean;
  };
  /** APCA (Advanced Perceptual Contrast Algorithm) score */
  apca: {
    score: number; // Lc value
    compliant: boolean;
    minFontSize?: number;
  };
  /** Luminance values */
  luminance: {
    foreground: number;
    background: number;
  };
}

/**
 * Color blindness types (deficiencies)
 */
export enum ColorBlindnessType {
  PROTANOPIA = 'protanopia', // Red-blind
  DEUTERANOPIA = 'deuteranopia', // Green-blind
  TRITANOPIA = 'tritanopia', // Blue-blind
  PROTANOMALY = 'protanomaly', // Red-weak
  DEUTERANOMALY = 'deuteranomaly', // Green-weak
  TRITANOMALY = 'tritanomaly', // Blue-weak
  ACHROMATOPSIA = 'achromatopsia', // Complete color blindness
  ACHROMATOMALY = 'achromatomaly', // Blue cone monochromacy
}

/**
 * Simulation result for color blindness
 */
export interface SimulationResult {
  type: ColorBlindnessType;
  original: RGB;
  simulated: RGB;
  severity: number; // 0-1
}

/**
 * Color palette entry
 */
export interface PaletteColor {
  id: string;
  name: string;
  color: RGB;
  hex: string;
  role?: 'primary' | 'secondary' | 'accent' | 'background' | 'text' | 'custom';
}

/**
 * Palette contrast matrix entry
 */
export interface PaletteContrastPair {
  foreground: PaletteColor;
  background: PaletteColor;
  contrast: ContrastResult;
}

/**
 * Complete palette with contrast analysis
 */
export interface ColorPalette {
  id: string;
  name: string;
  colors: PaletteColor[];
  contrastMatrix: PaletteContrastPair[];
  wcagCompliant: boolean;
}

/**
 * Color suggestion for accessibility improvement
 */
export interface ColorSuggestion {
  color: RGB;
  hex: string;
  contrast: ContrastResult;
  /** Distance from original color (deltaE) */
  distance: number;
  /** Modification type */
  modification: 'lightened' | 'darkened' | 'saturated' | 'desaturated' | 'hue-shifted';
}

/**
 * Optimization options for finding accessible colors
 */
export interface OptimizationOptions {
  /** Target WCAG conformance level */
  target: WCAGConformance;
  /** Maximum color distance allowed (deltaE) */
  maxDistance?: number;
  /** Preserve hue when possible */
  preserveHue?: boolean;
  /** Number of suggestions to generate */
  suggestionCount?: number;
}

/**
 * Color analysis report
 */
export interface ColorAnalysisReport {
  foreground: RGB;
  background: RGB;
  contrast: ContrastResult;
  accessible: boolean;
  suggestions: ColorSuggestion[];
  simulations: SimulationResult[];
}

/**
 * APCA contrast constants
 */
export const APCA_THRESHOLDS = {
  MIN_BODY_TEXT: 75,
  MIN_SUBTEXT: 60,
  MIN_PLACEHOLDER: 45,
  MIN_DISABLED: 30,
  MIN_UI_COMPONENT: 45,
  MIN_LARGE_TEXT: 60,
} as const;

/**
 * WCAG contrast thresholds
 */
export const WCAG_THRESHOLDS = {
  NORMAL_TEXT_AA: 4.5,
  NORMAL_TEXT_AAA: 7.0,
  LARGE_TEXT_AA: 3.0,
  LARGE_TEXT_AAA: 4.5,
  UI_COMPONENTS: 3.0,
} as const;

/**
 * Large text definition (WCAG)
 */
export const LARGE_TEXT = {
  MIN_SIZE_PT: 18,
  MIN_SIZE_PX: 24,
  MIN_SIZE_BOLD_PT: 14,
  MIN_SIZE_BOLD_PX: 18.66,
} as const;

/**
 * Color difference calculation method
 */
export enum DeltaEMethod {
  CIE76 = 'cie76',
  CIE94 = 'cie94',
  CIEDE2000 = 'ciede2000',
}

/**
 * Error types for color operations
 */
export class ColorError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ColorError';
  }
}

export class InvalidColorError extends ColorError {
  constructor(color: unknown) {
    super(`Invalid color value: ${JSON.stringify(color)}`);
    this.name = 'InvalidColorError';
  }
}

export class ContrastCalculationError extends ColorError {
  constructor(message: string) {
    super(`Contrast calculation error: ${message}`);
    this.name = 'ContrastCalculationError';
  }
}
