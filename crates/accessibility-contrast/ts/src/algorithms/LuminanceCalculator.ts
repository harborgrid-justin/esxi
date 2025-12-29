/**
 * Luminance calculation algorithms for WCAG and APCA
 */

import { RGB } from '../types';
import { sRGBToLinear, apcaCompanding } from '../utils/colorMath';

/**
 * Calculate relative luminance according to WCAG 2.1
 * Formula: L = 0.2126 * R + 0.7152 * G + 0.0722 * B
 * where R, G, B are linearized sRGB values
 *
 * @param rgb RGB color
 * @returns Relative luminance (0-1)
 */
export function calculateRelativeLuminance(rgb: RGB): number {
  const r = sRGBToLinear(rgb.r);
  const g = sRGBToLinear(rgb.g);
  const b = sRGBToLinear(rgb.b);

  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/**
 * Calculate Y (luminance) for APCA
 * Uses different coefficients than WCAG
 *
 * @param rgb RGB color
 * @returns Luminance for APCA (0-1)
 */
export function calculateAPCALuminance(rgb: RGB): number {
  const r = apcaCompanding(rgb.r);
  const g = apcaCompanding(rgb.g);
  const b = apcaCompanding(rgb.b);

  // APCA uses these coefficients (from SAPC)
  return 0.2126729 * r + 0.7151522 * g + 0.072175 * b;
}

/**
 * Calculate perceived lightness (L*) from relative luminance
 * Uses CIE L* formula
 *
 * @param luminance Relative luminance (0-1)
 * @returns Perceived lightness (0-100)
 */
export function luminanceToLightness(luminance: number): number {
  if (luminance <= 216 / 24389) {
    return (luminance * 24389) / 27;
  }
  return Math.pow(luminance, 1 / 3) * 116 - 16;
}

/**
 * Calculate relative luminance from perceived lightness
 *
 * @param lightness Perceived lightness (0-100)
 * @returns Relative luminance (0-1)
 */
export function lightnessToLuminance(lightness: number): number {
  if (lightness <= 8) {
    return (lightness * 27) / 24389;
  }
  return Math.pow((lightness + 16) / 116, 3);
}

/**
 * Estimate perceived brightness (simple approximation)
 * Not scientifically accurate but useful for quick estimates
 *
 * @param rgb RGB color
 * @returns Perceived brightness (0-255)
 */
export function perceivedBrightness(rgb: RGB): number {
  // HSP color model approximation
  return Math.sqrt(0.299 * rgb.r * rgb.r + 0.587 * rgb.g * rgb.g + 0.114 * rgb.b * rgb.b);
}

/**
 * Check if a color is considered "light" or "dark"
 * Uses relative luminance with 0.5 threshold
 *
 * @param rgb RGB color
 * @returns true if light, false if dark
 */
export function isLightColor(rgb: RGB): boolean {
  return calculateRelativeLuminance(rgb) > 0.5;
}

/**
 * Get appropriate text color (black or white) for a background
 * Uses WCAG relative luminance
 *
 * @param background Background color
 * @returns RGB color for text (black or white)
 */
export function getContrastingTextColor(background: RGB): RGB {
  const luminance = calculateRelativeLuminance(background);
  return luminance > 0.5 ? { r: 0, g: 0, b: 0 } : { r: 255, g: 255, b: 255 };
}

/**
 * Calculate the average luminance of multiple colors
 *
 * @param colors Array of RGB colors
 * @returns Average relative luminance
 */
export function averageLuminance(colors: RGB[]): number {
  if (colors.length === 0) return 0;

  const sum = colors.reduce((acc, color) => acc + calculateRelativeLuminance(color), 0);
  return sum / colors.length;
}

/**
 * Normalize luminance to 0-100 scale
 *
 * @param luminance Relative luminance (0-1)
 * @returns Normalized luminance (0-100)
 */
export function normalizeLuminance(luminance: number): number {
  return luminance * 100;
}

/**
 * Calculate luma (Y') from RGB using Rec. 709 coefficients
 * This is different from relative luminance - it's calculated in gamma-compressed space
 *
 * @param rgb RGB color
 * @returns Luma value (0-255)
 */
export function calculateLuma(rgb: RGB): number {
  return 0.2126 * rgb.r + 0.7152 * rgb.g + 0.0722 * rgb.b;
}

/**
 * Calculate weighted luminance for specific use cases
 * Allows custom weighting of RGB channels
 *
 * @param rgb RGB color
 * @param weights Custom weights for R, G, B channels
 * @returns Weighted luminance
 */
export function calculateWeightedLuminance(
  rgb: RGB,
  weights: { r: number; g: number; b: number } = { r: 0.299, g: 0.587, b: 0.114 }
): number {
  const r = sRGBToLinear(rgb.r);
  const g = sRGBToLinear(rgb.g);
  const b = sRGBToLinear(rgb.b);

  return weights.r * r + weights.g * g + weights.b * b;
}
