/**
 * Contrast calculation algorithms
 * Implements WCAG 2.1 and APCA (Advanced Perceptual Contrast Algorithm)
 */

import { RGB, ContrastResult, WCAG_THRESHOLDS, APCA_THRESHOLDS } from '../types';
import { calculateRelativeLuminance, calculateAPCALuminance } from './LuminanceCalculator';
import { round } from '../utils/colorMath';

/**
 * Calculate WCAG 2.1 contrast ratio
 * Formula: (L1 + 0.05) / (L2 + 0.05)
 * where L1 is the lighter color and L2 is the darker color
 *
 * @param foreground Foreground color
 * @param background Background color
 * @returns Contrast ratio (1-21)
 */
export function calculateWCAGContrast(foreground: RGB, background: RGB): number {
  const l1 = calculateRelativeLuminance(foreground);
  const l2 = calculateRelativeLuminance(background);

  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);

  const ratio = (lighter + 0.05) / (darker + 0.05);
  return round(ratio, 2);
}

/**
 * Calculate APCA contrast (Lc value)
 * APCA is perceptually uniform and directional (text vs background matters)
 *
 * @param text Text color
 * @param background Background color
 * @returns APCA Lc value (-108 to 106)
 */
export function calculateAPCAContrast(text: RGB, background: RGB): number {
  const bgY = calculateAPCALuminance(background);
  const txtY = calculateAPCALuminance(text);

  // APCA constants
  const blkThrs = 0.022;
  const blkClmp = 1.414;
  const scaleBoW = 1.14;
  const scaleWoB = 1.14;
  const loConThresh = 0.1;
  const loConFactor = 0.027;
  const loConOffset = 0.027;
  const deltaYmin = 0.0005;

  // Clamp luminances
  let Ybg = bgY >= blkThrs ? bgY : bgY + Math.pow(blkThrs - bgY, blkClmp);
  let Ytxt = txtY >= blkThrs ? txtY : txtY + Math.pow(blkThrs - txtY, blkClmp);

  // Calculate SAPC
  let SAPC = 0;

  if (Math.abs(Ybg - Ytxt) < deltaYmin) {
    return 0;
  }

  if (Ybg > Ytxt) {
    // Light background, dark text (positive Lc)
    SAPC = (Math.pow(Ybg, 0.56) - Math.pow(Ytxt, 0.57)) * scaleBoW;
  } else {
    // Dark background, light text (negative Lc)
    SAPC = (Math.pow(Ybg, 0.65) - Math.pow(Ytxt, 0.62)) * scaleWoB;
  }

  // Soft clamp for low contrast
  if (Math.abs(SAPC) < loConThresh) {
    return 0;
  }

  if (SAPC > 0) {
    SAPC = SAPC - loConOffset;
  } else {
    SAPC = SAPC + loConOffset;
  }

  return round(SAPC * 100, 1);
}

/**
 * Get minimum font size recommendation for APCA contrast
 *
 * @param lc APCA Lc value
 * @returns Minimum font size in pixels (or undefined if not compliant)
 */
export function getAPCAMinFontSize(lc: number): number | undefined {
  const absLc = Math.abs(lc);

  if (absLc >= 90) return 12; // ~9pt
  if (absLc >= 75) return 16; // ~12pt
  if (absLc >= 60) return 20; // ~15pt
  if (absLc >= 45) return 32; // ~24pt (large text only)
  return undefined; // Not compliant for any text
}

/**
 * Check WCAG compliance for all levels
 *
 * @param ratio Contrast ratio
 * @returns Object with compliance status for each level
 */
export function checkWCAGCompliance(ratio: number) {
  return {
    normalTextAA: ratio >= WCAG_THRESHOLDS.NORMAL_TEXT_AA,
    normalTextAAA: ratio >= WCAG_THRESHOLDS.NORMAL_TEXT_AAA,
    largeTextAA: ratio >= WCAG_THRESHOLDS.LARGE_TEXT_AA,
    largeTextAAA: ratio >= WCAG_THRESHOLDS.LARGE_TEXT_AAA,
    uiComponents: ratio >= WCAG_THRESHOLDS.UI_COMPONENTS,
  };
}

/**
 * Check APCA compliance
 *
 * @param lc APCA Lc value
 * @returns Object with compliance status
 */
export function checkAPCACompliance(lc: number) {
  const absLc = Math.abs(lc);

  return {
    score: lc,
    compliant: absLc >= APCA_THRESHOLDS.MIN_BODY_TEXT,
    minFontSize: getAPCAMinFontSize(lc),
  };
}

/**
 * Calculate complete contrast analysis
 *
 * @param foreground Foreground color
 * @param background Background color
 * @returns Complete contrast result with WCAG and APCA
 */
export function calculateContrast(foreground: RGB, background: RGB): ContrastResult {
  const wcagRatio = calculateWCAGContrast(foreground, background);
  const apcaScore = calculateAPCAContrast(foreground, background);

  const foregroundLuminance = calculateRelativeLuminance(foreground);
  const backgroundLuminance = calculateRelativeLuminance(background);

  return {
    ratio: wcagRatio,
    wcag: checkWCAGCompliance(wcagRatio),
    apca: checkAPCACompliance(apcaScore),
    luminance: {
      foreground: round(foregroundLuminance, 4),
      background: round(backgroundLuminance, 4),
    },
  };
}

/**
 * Find minimum contrast ratio needed for a specific WCAG level
 *
 * @param level WCAG level (AA or AAA)
 * @param isLargeText Whether the text is large
 * @returns Minimum contrast ratio
 */
export function getMinimumContrastRatio(level: 'AA' | 'AAA', isLargeText: boolean): number {
  if (level === 'AAA') {
    return isLargeText ? WCAG_THRESHOLDS.LARGE_TEXT_AAA : WCAG_THRESHOLDS.NORMAL_TEXT_AAA;
  }
  return isLargeText ? WCAG_THRESHOLDS.LARGE_TEXT_AA : WCAG_THRESHOLDS.NORMAL_TEXT_AA;
}

/**
 * Calculate contrast grade (A-F based on ratio)
 *
 * @param ratio Contrast ratio
 * @returns Letter grade
 */
export function getContrastGrade(ratio: number): string {
  if (ratio >= 12) return 'A+';
  if (ratio >= 7) return 'A';
  if (ratio >= 4.5) return 'B';
  if (ratio >= 3) return 'C';
  if (ratio >= 2) return 'D';
  return 'F';
}

/**
 * Calculate contrast score (0-100)
 *
 * @param ratio Contrast ratio
 * @returns Score from 0-100
 */
export function getContrastScore(ratio: number): number {
  // Map 1-21 ratio to 0-100 score
  const normalized = Math.min(ratio / 21, 1);
  return Math.round(normalized * 100);
}

/**
 * Check if contrast meets minimum requirements for UI components
 *
 * @param foreground Foreground color
 * @param background Background color
 * @returns true if meets WCAG 2.1 UI component requirements
 */
export function meetsUIComponentRequirements(foreground: RGB, background: RGB): boolean {
  const ratio = calculateWCAGContrast(foreground, background);
  return ratio >= WCAG_THRESHOLDS.UI_COMPONENTS;
}

/**
 * Compare two contrast ratios and determine which is better
 *
 * @param ratio1 First contrast ratio
 * @param ratio2 Second contrast ratio
 * @returns 1 if first is better, -1 if second is better, 0 if equal
 */
export function compareContrast(ratio1: number, ratio2: number): number {
  if (ratio1 > ratio2) return 1;
  if (ratio1 < ratio2) return -1;
  return 0;
}

/**
 * Calculate required luminance for target contrast ratio
 * Given a background luminance and target ratio, find required foreground luminance
 *
 * @param backgroundLuminance Background relative luminance
 * @param targetRatio Target contrast ratio
 * @param lighter Whether to find lighter (true) or darker (false) foreground
 * @returns Required foreground luminance
 */
export function calculateRequiredLuminance(
  backgroundLuminance: number,
  targetRatio: number,
  lighter: boolean = true
): number {
  if (lighter) {
    // Foreground is lighter: (L1 + 0.05) / (L2 + 0.05) = ratio
    return targetRatio * (backgroundLuminance + 0.05) - 0.05;
  } else {
    // Foreground is darker: (L2 + 0.05) / (L1 + 0.05) = ratio
    return (backgroundLuminance + 0.05) / targetRatio - 0.05;
  }
}
