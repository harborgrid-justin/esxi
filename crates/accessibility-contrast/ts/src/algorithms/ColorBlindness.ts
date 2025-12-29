/**
 * Color blindness simulation algorithms
 * Implements scientifically accurate simulations for various types of color vision deficiencies
 */

import { RGB, ColorBlindnessType, SimulationResult } from '../types';
import { clamp } from '../utils/colorMath';

/**
 * Transformation matrices for color blindness simulation
 * Based on Viénot, Brettel, and Mollon (1999) research
 */

// Protanopia (red-blind) transformation matrix
const PROTANOPIA_MATRIX = [
  [0.56667, 0.43333, 0.0],
  [0.55833, 0.44167, 0.0],
  [0.0, 0.24167, 0.75833],
];

// Deuteranopia (green-blind) transformation matrix
const DEUTERANOPIA_MATRIX = [
  [0.625, 0.375, 0.0],
  [0.7, 0.3, 0.0],
  [0.0, 0.3, 0.7],
];

// Tritanopia (blue-blind) transformation matrix
const TRITANOPIA_MATRIX = [
  [0.95, 0.05, 0.0],
  [0.0, 0.43333, 0.56667],
  [0.0, 0.475, 0.525],
];

// Achromatopsia (complete color blindness) - grayscale
const ACHROMATOPSIA_MATRIX = [
  [0.299, 0.587, 0.114],
  [0.299, 0.587, 0.114],
  [0.299, 0.587, 0.114],
];

/**
 * Apply transformation matrix to RGB color
 */
function applyMatrix(rgb: RGB, matrix: number[][]): RGB {
  const r = rgb.r / 255;
  const g = rgb.g / 255;
  const b = rgb.b / 255;

  const newR = matrix[0][0] * r + matrix[0][1] * g + matrix[0][2] * b;
  const newG = matrix[1][0] * r + matrix[1][1] * g + matrix[1][2] * b;
  const newB = matrix[2][0] * r + matrix[2][1] * g + matrix[2][2] * b;

  return {
    r: Math.round(clamp(newR * 255, 0, 255)),
    g: Math.round(clamp(newG * 255, 0, 255)),
    b: Math.round(clamp(newB * 255, 0, 255)),
  };
}

/**
 * Interpolate between normal vision and color blind vision
 * Used for simulating anomalous trichromacy (partial color blindness)
 */
function interpolateVision(normal: RGB, colorBlind: RGB, severity: number): RGB {
  const r = Math.round(normal.r * (1 - severity) + colorBlind.r * severity);
  const g = Math.round(normal.g * (1 - severity) + colorBlind.g * severity);
  const b = Math.round(normal.b * (1 - severity) + colorBlind.b * severity);

  return { r, g, b };
}

/**
 * Simulate protanopia (red-blind)
 */
export function simulateProtanopia(rgb: RGB): RGB {
  return applyMatrix(rgb, PROTANOPIA_MATRIX);
}

/**
 * Simulate deuteranopia (green-blind)
 */
export function simulateDeuteranopia(rgb: RGB): RGB {
  return applyMatrix(rgb, DEUTERANOPIA_MATRIX);
}

/**
 * Simulate tritanopia (blue-blind)
 */
export function simulateTritanopia(rgb: RGB): RGB {
  return applyMatrix(rgb, TRITANOPIA_MATRIX);
}

/**
 * Simulate achromatopsia (complete color blindness)
 */
export function simulateAchromatopsia(rgb: RGB): RGB {
  return applyMatrix(rgb, ACHROMATOPSIA_MATRIX);
}

/**
 * Simulate protanomaly (red-weak, severity 0-1)
 */
export function simulateProtanomaly(rgb: RGB, severity: number = 0.6): RGB {
  const protanopia = simulateProtanopia(rgb);
  return interpolateVision(rgb, protanopia, severity);
}

/**
 * Simulate deuteranomaly (green-weak, severity 0-1)
 */
export function simulateDeuteranomaly(rgb: RGB, severity: number = 0.6): RGB {
  const deuteranopia = simulateDeuteranopia(rgb);
  return interpolateVision(rgb, deuteranopia, severity);
}

/**
 * Simulate tritanomaly (blue-weak, severity 0-1)
 */
export function simulateTritanomaly(rgb: RGB, severity: number = 0.6): RGB {
  const tritanopia = simulateTritanopia(rgb);
  return interpolateVision(rgb, tritanopia, severity);
}

/**
 * Simulate achromatomaly (blue cone monochromacy, severity 0-1)
 */
export function simulateAchromatomaly(rgb: RGB, severity: number = 0.8): RGB {
  const achromatopsia = simulateAchromatopsia(rgb);
  return interpolateVision(rgb, achromatopsia, severity);
}

/**
 * Simulate color blindness based on type
 */
export function simulateColorBlindness(
  rgb: RGB,
  type: ColorBlindnessType,
  severity: number = 1.0
): RGB {
  switch (type) {
    case ColorBlindnessType.PROTANOPIA:
      return simulateProtanopia(rgb);
    case ColorBlindnessType.DEUTERANOPIA:
      return simulateDeuteranopia(rgb);
    case ColorBlindnessType.TRITANOPIA:
      return simulateTritanopia(rgb);
    case ColorBlindnessType.ACHROMATOPSIA:
      return simulateAchromatopsia(rgb);
    case ColorBlindnessType.PROTANOMALY:
      return simulateProtanomaly(rgb, severity);
    case ColorBlindnessType.DEUTERANOMALY:
      return simulateDeuteranomaly(rgb, severity);
    case ColorBlindnessType.TRITANOMALY:
      return simulateTritanomaly(rgb, severity);
    case ColorBlindnessType.ACHROMATOMALY:
      return simulateAchromatomaly(rgb, severity);
    default:
      return rgb;
  }
}

/**
 * Simulate color blindness with full result details
 */
export function simulateColorBlindnessWithResult(
  rgb: RGB,
  type: ColorBlindnessType,
  severity: number = 1.0
): SimulationResult {
  const simulated = simulateColorBlindness(rgb, type, severity);

  return {
    type,
    original: rgb,
    simulated,
    severity,
  };
}

/**
 * Simulate all types of color blindness for a given color
 */
export function simulateAllTypes(rgb: RGB): SimulationResult[] {
  return Object.values(ColorBlindnessType).map((type) =>
    simulateColorBlindnessWithResult(rgb, type)
  );
}

/**
 * Calculate color difference between original and simulated
 * Simple Euclidean distance in RGB space
 */
export function calculateSimulationDifference(original: RGB, simulated: RGB): number {
  const dr = original.r - simulated.r;
  const dg = original.g - simulated.g;
  const db = original.b - simulated.b;

  return Math.sqrt(dr * dr + dg * dg + db * db);
}

/**
 * Check if two colors are distinguishable for a specific color blindness type
 * Colors should have sufficient difference after simulation
 */
export function areColorsDistinguishable(
  color1: RGB,
  color2: RGB,
  type: ColorBlindnessType,
  threshold: number = 30
): boolean {
  const sim1 = simulateColorBlindness(color1, type);
  const sim2 = simulateColorBlindness(color2, type);

  const difference = calculateSimulationDifference(sim1, sim2);
  return difference >= threshold;
}

/**
 * Get human-readable name for color blindness type
 */
export function getColorBlindnessName(type: ColorBlindnessType): string {
  const names: Record<ColorBlindnessType, string> = {
    [ColorBlindnessType.PROTANOPIA]: 'Protanopia (Red-Blind)',
    [ColorBlindnessType.DEUTERANOPIA]: 'Deuteranopia (Green-Blind)',
    [ColorBlindnessType.TRITANOPIA]: 'Tritanopia (Blue-Blind)',
    [ColorBlindnessType.PROTANOMALY]: 'Protanomaly (Red-Weak)',
    [ColorBlindnessType.DEUTERANOMALY]: 'Deuteranomaly (Green-Weak)',
    [ColorBlindnessType.TRITANOMALY]: 'Tritanomaly (Blue-Weak)',
    [ColorBlindnessType.ACHROMATOPSIA]: 'Achromatopsia (Complete Color Blindness)',
    [ColorBlindnessType.ACHROMATOMALY]: 'Achromatomaly (Blue Cone Monochromacy)',
  };

  return names[type];
}

/**
 * Get description for color blindness type
 */
export function getColorBlindnessDescription(type: ColorBlindnessType): string {
  const descriptions: Record<ColorBlindnessType, string> = {
    [ColorBlindnessType.PROTANOPIA]:
      'Inability to perceive red light. Affects ~1% of males. Red appears dark gray.',
    [ColorBlindnessType.DEUTERANOPIA]:
      'Inability to perceive green light. Affects ~1% of males. Most common form.',
    [ColorBlindnessType.TRITANOPIA]:
      'Inability to perceive blue light. Very rare (~0.001%). Blue appears green.',
    [ColorBlindnessType.PROTANOMALY]:
      'Reduced sensitivity to red light. Affects ~1% of males. Mild red deficiency.',
    [ColorBlindnessType.DEUTERANOMALY]:
      'Reduced sensitivity to green light. Affects ~5% of males. Most common deficiency.',
    [ColorBlindnessType.TRITANOMALY]:
      'Reduced sensitivity to blue light. Extremely rare. Mild blue deficiency.',
    [ColorBlindnessType.ACHROMATOPSIA]:
      'Complete inability to see color. Extremely rare (~0.003%). Only shades of gray.',
    [ColorBlindnessType.ACHROMATOMALY]:
      'Severely reduced color vision. Very rare. Only blue cones function.',
  };

  return descriptions[type];
}

/**
 * Get prevalence (percentage of population) for color blindness type
 */
export function getColorBlindnessPrevalence(type: ColorBlindnessType): {
  male: number;
  female: number;
} {
  const prevalence: Record<ColorBlindnessType, { male: number; female: number }> = {
    [ColorBlindnessType.PROTANOPIA]: { male: 1.0, female: 0.01 },
    [ColorBlindnessType.DEUTERANOPIA]: { male: 1.0, female: 0.01 },
    [ColorBlindnessType.TRITANOPIA]: { male: 0.001, female: 0.001 },
    [ColorBlindnessType.PROTANOMALY]: { male: 1.0, female: 0.01 },
    [ColorBlindnessType.DEUTERANOMALY]: { male: 5.0, female: 0.4 },
    [ColorBlindnessType.TRITANOMALY]: { male: 0.001, female: 0.001 },
    [ColorBlindnessType.ACHROMATOPSIA]: { male: 0.003, female: 0.003 },
    [ColorBlindnessType.ACHROMATOMALY]: { male: 0.001, female: 0.001 },
  };

  return prevalence[type];
}
