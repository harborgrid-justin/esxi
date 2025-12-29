/**
 * Color optimization algorithms
 * Find nearest accessible colors that meet WCAG requirements
 */

import { RGB, LAB, LCH, ColorSuggestion, OptimizationOptions, WCAGConformance } from '../types';
import { calculateContrast, getMinimumContrastRatio, calculateWCAGContrast } from './ContrastCalculator';
import { rgbToLab, rgbToLch, lchToRgb, labToRgb, lighten, darken } from './ColorConverter';
import { calculateRequiredLuminance } from './ContrastCalculator';
import { calculateRelativeLuminance } from './LuminanceCalculator';
import { deltaE2000 } from '../utils/colorMath';
import { rgbToHex, clamp } from '../utils/colorMath';

/**
 * Get target contrast ratio for conformance level
 */
function getTargetRatio(target: WCAGConformance): number {
  switch (target) {
    case WCAGConformance.NORMAL_TEXT_AA:
      return 4.5;
    case WCAGConformance.NORMAL_TEXT_AAA:
      return 7.0;
    case WCAGConformance.LARGE_TEXT_AA:
      return 3.0;
    case WCAGConformance.LARGE_TEXT_AAA:
      return 4.5;
    case WCAGConformance.UI_COMPONENTS:
      return 3.0;
    default:
      return 4.5;
  }
}

/**
 * Find nearest accessible color by adjusting lightness
 */
export function findAccessibleByLightness(
  foreground: RGB,
  background: RGB,
  targetRatio: number,
  preserveHue: boolean = true
): RGB | null {
  const bgLuminance = calculateRelativeLuminance(background);
  const fgLuminance = calculateRelativeLuminance(foreground);

  // Determine if we should lighten or darken
  const shouldLighten = fgLuminance < bgLuminance;

  // Calculate required luminance
  const requiredLuminance = calculateRequiredLuminance(bgLuminance, targetRatio, shouldLighten);

  // If required luminance is out of bounds, try opposite direction
  if (requiredLuminance < 0 || requiredLuminance > 1) {
    const altLuminance = calculateRequiredLuminance(bgLuminance, targetRatio, !shouldLighten);
    if (altLuminance < 0 || altLuminance > 1) {
      return null; // Cannot achieve target ratio
    }
    return adjustColorToLuminance(foreground, altLuminance, preserveHue);
  }

  return adjustColorToLuminance(foreground, requiredLuminance, preserveHue);
}

/**
 * Adjust color to target luminance while preserving hue/saturation
 */
function adjustColorToLuminance(color: RGB, targetLuminance: number, preserveHue: boolean): RGB {
  if (!preserveHue) {
    // Simple grayscale approach
    const gray = Math.round(targetLuminance * 255);
    return { r: gray, g: gray, b: gray };
  }

  const lch = rgbToLch(color);

  // Binary search for correct lightness value
  let minL = 0;
  let maxL = 100;
  let iterations = 0;
  const maxIterations = 20;

  while (iterations < maxIterations && maxL - minL > 0.1) {
    const testL = (minL + maxL) / 2;
    const testColor = lchToRgb({ ...lch, l: testL });
    const testLuminance = calculateRelativeLuminance(testColor);

    if (testLuminance < targetLuminance) {
      minL = testL;
    } else {
      maxL = testL;
    }

    iterations++;
  }

  return lchToRgb({ ...lch, l: (minL + maxL) / 2 });
}

/**
 * Generate color suggestions by various modifications
 */
export function generateColorSuggestions(
  foreground: RGB,
  background: RGB,
  options: OptimizationOptions
): ColorSuggestion[] {
  const targetRatio = getTargetRatio(options.target);
  const suggestions: ColorSuggestion[] = [];
  const originalLab = rgbToLab(foreground);

  // Try lightening at various levels
  for (let i = 5; i <= 95; i += 5) {
    const lightened = lighten(foreground, i);
    const contrast = calculateContrast(lightened, background);

    if (contrast.ratio >= targetRatio) {
      const lab = rgbToLab(lightened);
      const distance = deltaE2000(originalLab, lab);

      if (!options.maxDistance || distance <= options.maxDistance) {
        suggestions.push({
          color: lightened,
          hex: rgbToHex(lightened),
          contrast,
          distance,
          modification: 'lightened',
        });
      }
    }
  }

  // Try darkening at various levels
  for (let i = 5; i <= 95; i += 5) {
    const darkened = darken(foreground, i);
    const contrast = calculateContrast(darkened, background);

    if (contrast.ratio >= targetRatio) {
      const lab = rgbToLab(darkened);
      const distance = deltaE2000(originalLab, lab);

      if (!options.maxDistance || distance <= options.maxDistance) {
        suggestions.push({
          color: darkened,
          hex: rgbToHex(darkened),
          contrast,
          distance,
          modification: 'darkened',
        });
      }
    }
  }

  // Try adjusting lightness in LCH space (preserves hue better)
  if (options.preserveHue !== false) {
    const lch = rgbToLch(foreground);

    for (let l = 0; l <= 100; l += 5) {
      const adjusted = lchToRgb({ ...lch, l });
      const contrast = calculateContrast(adjusted, background);

      if (contrast.ratio >= targetRatio) {
        const lab = rgbToLab(adjusted);
        const distance = deltaE2000(originalLab, lab);

        if (!options.maxDistance || distance <= options.maxDistance) {
          suggestions.push({
            color: adjusted,
            hex: rgbToHex(adjusted),
            contrast,
            distance,
            modification: l > lch.l ? 'lightened' : 'darkened',
          });
        }
      }
    }

    // Try hue rotation
    for (let h = -180; h <= 180; h += 15) {
      if (h === 0) continue;

      const rotated = lchToRgb({ ...lch, h: (lch.h + h + 360) % 360 });
      const contrast = calculateContrast(rotated, background);

      if (contrast.ratio >= targetRatio) {
        const lab = rgbToLab(rotated);
        const distance = deltaE2000(originalLab, lab);

        if (!options.maxDistance || distance <= options.maxDistance) {
          suggestions.push({
            color: rotated,
            hex: rgbToHex(rotated),
            contrast,
            distance,
            modification: 'hue-shifted',
          });
        }
      }
    }

    // Try saturation changes
    for (let c = 0; c <= 150; c += 10) {
      const saturated = lchToRgb({ ...lch, c });
      const contrast = calculateContrast(saturated, background);

      if (contrast.ratio >= targetRatio) {
        const lab = rgbToLab(saturated);
        const distance = deltaE2000(originalLab, lab);

        if (!options.maxDistance || distance <= options.maxDistance) {
          suggestions.push({
            color: saturated,
            hex: rgbToHex(saturated),
            contrast,
            distance,
            modification: c > lch.c ? 'saturated' : 'desaturated',
          });
        }
      }
    }
  }

  // Sort by distance (closest first) and remove duplicates
  const uniqueSuggestions = removeDuplicateSuggestions(suggestions);
  uniqueSuggestions.sort((a, b) => a.distance - b.distance);

  // Limit to requested count
  const count = options.suggestionCount || 10;
  return uniqueSuggestions.slice(0, count);
}

/**
 * Remove duplicate color suggestions
 */
function removeDuplicateSuggestions(suggestions: ColorSuggestion[]): ColorSuggestion[] {
  const seen = new Set<string>();
  return suggestions.filter((suggestion) => {
    const key = `${suggestion.color.r}-${suggestion.color.g}-${suggestion.color.b}`;
    if (seen.has(key)) {
      return false;
    }
    seen.add(key);
    return true;
  });
}

/**
 * Find the single best accessible color
 */
export function findBestAccessibleColor(
  foreground: RGB,
  background: RGB,
  options: OptimizationOptions
): RGB | null {
  const suggestions = generateColorSuggestions(foreground, background, options);

  if (suggestions.length === 0) {
    return null;
  }

  // Return closest color
  return suggestions[0].color;
}

/**
 * Optimize an entire color palette for accessibility
 */
export function optimizePalette(
  colors: RGB[],
  background: RGB,
  targetRatio: number = 4.5
): Map<RGB, RGB> {
  const optimized = new Map<RGB, RGB>();

  for (const color of colors) {
    const ratio = calculateWCAGContrast(color, background);

    if (ratio >= targetRatio) {
      // Already accessible
      optimized.set(color, color);
    } else {
      // Find accessible version
      const accessible = findAccessibleByLightness(color, background, targetRatio, true);
      optimized.set(color, accessible || color);
    }
  }

  return optimized;
}

/**
 * Check if a color can be made accessible
 */
export function canBeAccessible(
  foreground: RGB,
  background: RGB,
  targetRatio: number,
  maxDistance: number = 50
): boolean {
  const suggestions = generateColorSuggestions(foreground, background, {
    target: WCAGConformance.NORMAL_TEXT_AA,
    maxDistance,
    preserveHue: true,
    suggestionCount: 1,
  });

  return suggestions.length > 0;
}

/**
 * Calculate accessibility score (0-100)
 * Higher score means better accessibility
 */
export function calculateAccessibilityScore(
  foreground: RGB,
  background: RGB,
  targetLevel: 'AA' | 'AAA' = 'AA',
  isLargeText: boolean = false
): number {
  const ratio = calculateWCAGContrast(foreground, background);
  const targetRatio = getMinimumContrastRatio(targetLevel, isLargeText);

  if (ratio >= targetRatio) {
    // Exceeds requirement - calculate bonus
    const excess = ratio - targetRatio;
    const bonus = Math.min(excess / targetRatio, 1) * 20;
    return Math.min(80 + bonus, 100);
  } else {
    // Below requirement - calculate penalty
    const percentage = ratio / targetRatio;
    return Math.round(percentage * 80);
  }
}

/**
 * Find complementary accessible color
 * Finds a color that contrasts well with both given colors
 */
export function findComplementaryAccessibleColor(
  color1: RGB,
  color2: RGB,
  targetRatio: number = 4.5
): RGB | null {
  // Try grayscale options first
  const options = [
    { r: 255, g: 255, b: 255 }, // white
    { r: 0, g: 0, b: 0 }, // black
    { r: 128, g: 128, b: 128 }, // gray
  ];

  for (const option of options) {
    const ratio1 = calculateWCAGContrast(option, color1);
    const ratio2 = calculateWCAGContrast(option, color2);

    if (ratio1 >= targetRatio && ratio2 >= targetRatio) {
      return option;
    }
  }

  // Try finding a color in between
  const lch1 = rgbToLch(color1);
  const lch2 = rgbToLch(color2);

  for (let l = 0; l <= 100; l += 5) {
    const testColor = lchToRgb({ l, c: 0, h: 0 });
    const ratio1 = calculateWCAGContrast(testColor, color1);
    const ratio2 = calculateWCAGContrast(testColor, color2);

    if (ratio1 >= targetRatio && ratio2 >= targetRatio) {
      return testColor;
    }
  }

  return null;
}
