/**
 * Color mathematics utilities for color space calculations
 */

import { RGB, LAB, XYZ } from '../types';

/**
 * Clamps a value between min and max
 */
export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

/**
 * Rounds a number to specified decimal places
 */
export function round(value: number, decimals: number = 2): number {
  const factor = Math.pow(10, decimals);
  return Math.round(value * factor) / factor;
}

/**
 * Converts degrees to radians
 */
export function degToRad(degrees: number): number {
  return (degrees * Math.PI) / 180;
}

/**
 * Converts radians to degrees
 */
export function radToDeg(radians: number): number {
  return (radians * 180) / Math.PI;
}

/**
 * Normalizes an angle to 0-360 range
 */
export function normalizeAngle(angle: number): number {
  angle = angle % 360;
  return angle < 0 ? angle + 360 : angle;
}

/**
 * Calculates the distance between two angles (in degrees)
 */
export function angleDifference(a1: number, a2: number): number {
  const diff = Math.abs(a1 - a2) % 360;
  return diff > 180 ? 360 - diff : diff;
}

/**
 * Linear interpolation between two values
 */
export function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

/**
 * Inverse linear interpolation (finds t for a value between a and b)
 */
export function inverseLerp(a: number, b: number, value: number): number {
  return (value - a) / (b - a);
}

/**
 * Maps a value from one range to another
 */
export function mapRange(
  value: number,
  inMin: number,
  inMax: number,
  outMin: number,
  outMax: number
): number {
  return ((value - inMin) * (outMax - outMin)) / (inMax - inMin) + outMin;
}

/**
 * Gamma correction for sRGB
 */
export function sRGBToLinear(value: number): number {
  const v = value / 255;
  return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
}

/**
 * Inverse gamma correction for sRGB
 */
export function linearToSRGB(value: number): number {
  const v = value <= 0.0031308 ? value * 12.92 : 1.055 * Math.pow(value, 1 / 2.4) - 0.055;
  return clamp(Math.round(v * 255), 0, 255);
}

/**
 * Companding function for APCA
 */
export function apcaCompanding(value: number): number {
  const v = value / 255;
  return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
}

/**
 * Calculate deltaE CIE76 (Euclidean distance in LAB space)
 */
export function deltaE76(lab1: LAB, lab2: LAB): number {
  const dL = lab1.l - lab2.l;
  const da = lab1.a - lab2.a;
  const db = lab1.b - lab2.b;
  return Math.sqrt(dL * dL + da * da + db * db);
}

/**
 * Calculate deltaE CIE94 (weighted distance)
 */
export function deltaE94(lab1: LAB, lab2: LAB): number {
  const dL = lab1.l - lab2.l;
  const da = lab1.a - lab2.a;
  const db = lab1.b - lab2.b;

  const c1 = Math.sqrt(lab1.a * lab1.a + lab1.b * lab1.b);
  const c2 = Math.sqrt(lab2.a * lab2.a + lab2.b * lab2.b);
  const dC = c1 - c2;

  const dH2 = da * da + db * db - dC * dC;
  const dH = dH2 > 0 ? Math.sqrt(dH2) : 0;

  const kL = 1;
  const kC = 1;
  const kH = 1;
  const k1 = 0.045;
  const k2 = 0.015;

  const sL = 1;
  const sC = 1 + k1 * c1;
  const sH = 1 + k2 * c1;

  const dLk = dL / (kL * sL);
  const dCk = dC / (kC * sC);
  const dHk = dH / (kH * sH);

  return Math.sqrt(dLk * dLk + dCk * dCk + dHk * dHk);
}

/**
 * Calculate deltaE CIEDE2000 (most accurate perceptual distance)
 */
export function deltaE2000(lab1: LAB, lab2: LAB): number {
  const kL = 1;
  const kC = 1;
  const kH = 1;

  const dL = lab2.l - lab1.l;
  const avgL = (lab1.l + lab2.l) / 2;

  const c1 = Math.sqrt(lab1.a * lab1.a + lab1.b * lab1.b);
  const c2 = Math.sqrt(lab2.a * lab2.a + lab2.b * lab2.b);
  const avgC = (c1 + c2) / 2;

  const g = 0.5 * (1 - Math.sqrt(Math.pow(avgC, 7) / (Math.pow(avgC, 7) + Math.pow(25, 7))));

  const a1p = lab1.a * (1 + g);
  const a2p = lab2.a * (1 + g);

  const c1p = Math.sqrt(a1p * a1p + lab1.b * lab1.b);
  const c2p = Math.sqrt(a2p * a2p + lab2.b * lab2.b);
  const avgCp = (c1p + c2p) / 2;
  const dCp = c2p - c1p;

  const h1p = Math.abs(a1p) + Math.abs(lab1.b) === 0 ? 0 : Math.atan2(lab1.b, a1p);
  const h2p = Math.abs(a2p) + Math.abs(lab2.b) === 0 ? 0 : Math.atan2(lab2.b, a2p);

  let dhp = h2p - h1p;
  if (Math.abs(dhp) > Math.PI) {
    dhp = dhp > 0 ? dhp - 2 * Math.PI : dhp + 2 * Math.PI;
  }

  const dHp = 2 * Math.sqrt(c1p * c2p) * Math.sin(dhp / 2);

  let avgHp = h1p + h2p;
  if (Math.abs(h1p - h2p) > Math.PI) {
    avgHp = avgHp < 2 * Math.PI ? avgHp + 2 * Math.PI : avgHp - 2 * Math.PI;
  }
  avgHp = avgHp / 2;

  const t =
    1 -
    0.17 * Math.cos(avgHp - degToRad(30)) +
    0.24 * Math.cos(2 * avgHp) +
    0.32 * Math.cos(3 * avgHp + degToRad(6)) -
    0.2 * Math.cos(4 * avgHp - degToRad(63));

  const sL = 1 + (0.015 * (avgL - 50) * (avgL - 50)) / Math.sqrt(20 + (avgL - 50) * (avgL - 50));
  const sC = 1 + 0.045 * avgCp;
  const sH = 1 + 0.015 * avgCp * t;

  const dTheta = degToRad(30) * Math.exp(-Math.pow((radToDeg(avgHp) - 275) / 25, 2));
  const rC = 2 * Math.sqrt(Math.pow(avgCp, 7) / (Math.pow(avgCp, 7) + Math.pow(25, 7)));
  const rT = -rC * Math.sin(2 * dTheta);

  return Math.sqrt(
    Math.pow(dL / (kL * sL), 2) +
      Math.pow(dCp / (kC * sC), 2) +
      Math.pow(dHp / (kH * sH), 2) +
      rT * (dCp / (kC * sC)) * (dHp / (kH * sH))
  );
}

/**
 * Mix two RGB colors
 */
export function mixColors(color1: RGB, color2: RGB, weight: number = 0.5): RGB {
  return {
    r: Math.round(lerp(color1.r, color2.r, weight)),
    g: Math.round(lerp(color1.g, color2.g, weight)),
    b: Math.round(lerp(color1.b, color2.b, weight)),
  };
}

/**
 * Check if two colors are equal
 */
export function colorsEqual(color1: RGB, color2: RGB, tolerance: number = 0): boolean {
  return (
    Math.abs(color1.r - color2.r) <= tolerance &&
    Math.abs(color1.g - color2.g) <= tolerance &&
    Math.abs(color1.b - color2.b) <= tolerance
  );
}

/**
 * Parse hex color to RGB
 */
export function hexToRgb(hex: string): RGB {
  const cleaned = hex.replace(/^#/, '');

  if (cleaned.length === 3) {
    const r = parseInt(cleaned[0] + cleaned[0], 16);
    const g = parseInt(cleaned[1] + cleaned[1], 16);
    const b = parseInt(cleaned[2] + cleaned[2], 16);
    return { r, g, b };
  }

  if (cleaned.length === 6) {
    const r = parseInt(cleaned.substring(0, 2), 16);
    const g = parseInt(cleaned.substring(2, 4), 16);
    const b = parseInt(cleaned.substring(4, 6), 16);
    return { r, g, b };
  }

  throw new Error(`Invalid hex color: ${hex}`);
}

/**
 * Convert RGB to hex
 */
export function rgbToHex(rgb: RGB): string {
  const toHex = (n: number) => {
    const hex = Math.round(clamp(n, 0, 255)).toString(16);
    return hex.length === 1 ? '0' + hex : hex;
  };

  return `#${toHex(rgb.r)}${toHex(rgb.g)}${toHex(rgb.b)}`.toUpperCase();
}

/**
 * Check if a value is a valid RGB color
 */
export function isValidRGB(color: any): color is RGB {
  return (
    color &&
    typeof color === 'object' &&
    typeof color.r === 'number' &&
    typeof color.g === 'number' &&
    typeof color.b === 'number' &&
    color.r >= 0 &&
    color.r <= 255 &&
    color.g >= 0 &&
    color.g <= 255 &&
    color.b >= 0 &&
    color.b <= 255
  );
}

/**
 * Check if a string is a valid hex color
 */
export function isValidHex(hex: string): boolean {
  return /^#?([0-9A-Fa-f]{3}|[0-9A-Fa-f]{6})$/.test(hex);
}
