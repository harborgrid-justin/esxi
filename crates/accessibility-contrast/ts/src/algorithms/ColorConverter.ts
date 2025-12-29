/**
 * Color space conversion algorithms
 * Supports RGB, HSL, LAB, LCH, XYZ conversions with scientific accuracy
 */

import { RGB, HSL, LAB, LCH, XYZ, HexColor } from '../types';
import { clamp, normalizeAngle, degToRad, radToDeg, hexToRgb, rgbToHex } from '../utils/colorMath';

/**
 * Convert RGB to HSL
 */
export function rgbToHsl(rgb: RGB): HSL {
  const r = rgb.r / 255;
  const g = rgb.g / 255;
  const b = rgb.b / 255;

  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const delta = max - min;

  let h = 0;
  let s = 0;
  const l = (max + min) / 2;

  if (delta !== 0) {
    s = l > 0.5 ? delta / (2 - max - min) : delta / (max + min);

    switch (max) {
      case r:
        h = ((g - b) / delta + (g < b ? 6 : 0)) / 6;
        break;
      case g:
        h = ((b - r) / delta + 2) / 6;
        break;
      case b:
        h = ((r - g) / delta + 4) / 6;
        break;
    }
  }

  return {
    h: Math.round(h * 360),
    s: Math.round(s * 100),
    l: Math.round(l * 100),
  };
}

/**
 * Convert HSL to RGB
 */
export function hslToRgb(hsl: HSL): RGB {
  const h = hsl.h / 360;
  const s = hsl.s / 100;
  const l = hsl.l / 100;

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

/**
 * Convert RGB to XYZ (D65 illuminant)
 */
export function rgbToXyz(rgb: RGB): XYZ {
  // Convert to linear RGB
  let r = rgb.r / 255;
  let g = rgb.g / 255;
  let b = rgb.b / 255;

  r = r > 0.04045 ? Math.pow((r + 0.055) / 1.055, 2.4) : r / 12.92;
  g = g > 0.04045 ? Math.pow((g + 0.055) / 1.055, 2.4) : g / 12.92;
  b = b > 0.04045 ? Math.pow((b + 0.055) / 1.055, 2.4) : b / 12.92;

  // Convert to XYZ using D65 illuminant
  const x = r * 0.4124564 + g * 0.3575761 + b * 0.1804375;
  const y = r * 0.2126729 + g * 0.7151522 + b * 0.072175;
  const z = r * 0.0193339 + g * 0.119192 + b * 0.9503041;

  return { x: x * 100, y: y * 100, z: z * 100 };
}

/**
 * Convert XYZ to RGB (D65 illuminant)
 */
export function xyzToRgb(xyz: XYZ): RGB {
  const x = xyz.x / 100;
  const y = xyz.y / 100;
  const z = xyz.z / 100;

  // Convert XYZ to linear RGB
  let r = x * 3.2404542 + y * -1.5371385 + z * -0.4985314;
  let g = x * -0.969266 + y * 1.8760108 + z * 0.041556;
  let b = x * 0.0556434 + y * -0.2040259 + z * 1.0572252;

  // Convert linear RGB to sRGB
  r = r > 0.0031308 ? 1.055 * Math.pow(r, 1 / 2.4) - 0.055 : 12.92 * r;
  g = g > 0.0031308 ? 1.055 * Math.pow(g, 1 / 2.4) - 0.055 : 12.92 * g;
  b = b > 0.0031308 ? 1.055 * Math.pow(b, 1 / 2.4) - 0.055 : 12.92 * b;

  return {
    r: Math.round(clamp(r * 255, 0, 255)),
    g: Math.round(clamp(g * 255, 0, 255)),
    b: Math.round(clamp(b * 255, 0, 255)),
  };
}

/**
 * Convert XYZ to LAB (D65 illuminant)
 */
export function xyzToLab(xyz: XYZ): LAB {
  // D65 illuminant reference white
  const xn = 95.047;
  const yn = 100.0;
  const zn = 108.883;

  const fx = (t: number): number => {
    return t > 0.008856 ? Math.pow(t, 1 / 3) : 7.787 * t + 16 / 116;
  };

  const xr = fx(xyz.x / xn);
  const yr = fx(xyz.y / yn);
  const zr = fx(xyz.z / zn);

  const l = 116 * yr - 16;
  const a = 500 * (xr - yr);
  const b = 200 * (yr - zr);

  return { l, a, b };
}

/**
 * Convert LAB to XYZ (D65 illuminant)
 */
export function labToXyz(lab: LAB): XYZ {
  // D65 illuminant reference white
  const xn = 95.047;
  const yn = 100.0;
  const zn = 108.883;

  const fy = (lab.l + 16) / 116;
  const fx = lab.a / 500 + fy;
  const fz = fy - lab.b / 200;

  const xr = Math.pow(fx, 3) > 0.008856 ? Math.pow(fx, 3) : (fx - 16 / 116) / 7.787;
  const yr = Math.pow(fy, 3) > 0.008856 ? Math.pow(fy, 3) : (fy - 16 / 116) / 7.787;
  const zr = Math.pow(fz, 3) > 0.008856 ? Math.pow(fz, 3) : (fz - 16 / 116) / 7.787;

  return {
    x: xr * xn,
    y: yr * yn,
    z: zr * zn,
  };
}

/**
 * Convert RGB to LAB
 */
export function rgbToLab(rgb: RGB): LAB {
  const xyz = rgbToXyz(rgb);
  return xyzToLab(xyz);
}

/**
 * Convert LAB to RGB
 */
export function labToRgb(lab: LAB): RGB {
  const xyz = labToXyz(lab);
  return xyzToRgb(xyz);
}

/**
 * Convert LAB to LCH
 */
export function labToLch(lab: LAB): LCH {
  const c = Math.sqrt(lab.a * lab.a + lab.b * lab.b);
  let h = radToDeg(Math.atan2(lab.b, lab.a));
  h = normalizeAngle(h);

  return { l: lab.l, c, h };
}

/**
 * Convert LCH to LAB
 */
export function lchToLab(lch: LCH): LAB {
  const a = lch.c * Math.cos(degToRad(lch.h));
  const b = lch.c * Math.sin(degToRad(lch.h));

  return { l: lch.l, a, b };
}

/**
 * Convert RGB to LCH
 */
export function rgbToLch(rgb: RGB): LCH {
  const lab = rgbToLab(rgb);
  return labToLch(lab);
}

/**
 * Convert LCH to RGB
 */
export function lchToRgb(lch: LCH): RGB {
  const lab = lchToLab(lch);
  return labToRgb(lab);
}

/**
 * Convert hex to RGB
 */
export function hexToRGB(hex: HexColor): RGB {
  return hexToRgb(hex);
}

/**
 * Convert RGB to hex
 */
export function rgbToHEX(rgb: RGB): HexColor {
  return rgbToHex(rgb);
}

/**
 * Lighten a color by percentage
 */
export function lighten(rgb: RGB, percent: number): RGB {
  const hsl = rgbToHsl(rgb);
  hsl.l = clamp(hsl.l + percent, 0, 100);
  return hslToRgb(hsl);
}

/**
 * Darken a color by percentage
 */
export function darken(rgb: RGB, percent: number): RGB {
  const hsl = rgbToHsl(rgb);
  hsl.l = clamp(hsl.l - percent, 0, 100);
  return hslToRgb(hsl);
}

/**
 * Saturate a color by percentage
 */
export function saturate(rgb: RGB, percent: number): RGB {
  const hsl = rgbToHsl(rgb);
  hsl.s = clamp(hsl.s + percent, 0, 100);
  return hslToRgb(hsl);
}

/**
 * Desaturate a color by percentage
 */
export function desaturate(rgb: RGB, percent: number): RGB {
  const hsl = rgbToHsl(rgb);
  hsl.s = clamp(hsl.s - percent, 0, 100);
  return hslToRgb(hsl);
}

/**
 * Rotate hue by degrees
 */
export function rotateHue(rgb: RGB, degrees: number): RGB {
  const hsl = rgbToHsl(rgb);
  hsl.h = normalizeAngle(hsl.h + degrees);
  return hslToRgb(hsl);
}

/**
 * Convert any color to RGB
 */
export function toRGB(color: RGB | HSL | LAB | LCH | HexColor): RGB {
  if (typeof color === 'string') {
    return hexToRGB(color);
  }

  if ('r' in color && 'g' in color && 'b' in color) {
    return color;
  }

  if ('h' in color && 's' in color && 'l' in color) {
    return hslToRgb(color);
  }

  if ('l' in color && 'a' in color && 'b' in color) {
    return labToRgb(color as LAB);
  }

  if ('l' in color && 'c' in color && 'h' in color) {
    return lchToRgb(color as LCH);
  }

  throw new Error('Invalid color format');
}
