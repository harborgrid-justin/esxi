/**
 * Custom Interpolators for Advanced Animations
 * Provides specialized interpolation functions for complex data types
 */

import * as d3 from 'd3';
import { InterpolatorFunction, CustomInterpolator, Position, Color } from '../types';

/**
 * Interpolate between two colors
 */
export const colorInterpolator: CustomInterpolator<Color> = {
  interpolate: (start: Color, end: Color): InterpolatorFunction<Color> => {
    return (t: number): Color => ({
      r: start.r + (end.r - start.r) * t,
      g: start.g + (end.g - start.g) * t,
      b: start.b + (end.b - start.b) * t,
      a: start.a !== undefined && end.a !== undefined
        ? start.a + (end.a - start.a) * t
        : undefined,
    });
  },
};

/**
 * Interpolate between two positions
 */
export const positionInterpolator: CustomInterpolator<Position> = {
  interpolate: (start: Position, end: Position): InterpolatorFunction<Position> => {
    return (t: number): Position => ({
      x: start.x + (end.x - start.x) * t,
      y: start.y + (end.y - start.y) * t,
    });
  },
};

/**
 * Circular interpolation (arc path)
 */
export const circularInterpolator = (
  start: Position,
  end: Position,
  radius: number
): InterpolatorFunction<Position> => {
  const startAngle = Math.atan2(start.y, start.x);
  const endAngle = Math.atan2(end.y, end.x);

  // Choose shortest arc
  let deltaAngle = endAngle - startAngle;
  if (deltaAngle > Math.PI) deltaAngle -= 2 * Math.PI;
  if (deltaAngle < -Math.PI) deltaAngle += 2 * Math.PI;

  return (t: number): Position => {
    const angle = startAngle + deltaAngle * t;
    return {
      x: radius * Math.cos(angle),
      y: radius * Math.sin(angle),
    };
  };
};

/**
 * Bezier curve interpolation
 */
export const bezierInterpolator = (
  start: Position,
  control1: Position,
  control2: Position,
  end: Position
): InterpolatorFunction<Position> => {
  return (t: number): Position => {
    const u = 1 - t;
    const tt = t * t;
    const uu = u * u;
    const uuu = uu * u;
    const ttt = tt * t;

    return {
      x: uuu * start.x + 3 * uu * t * control1.x + 3 * u * tt * control2.x + ttt * end.x,
      y: uuu * start.y + 3 * uu * t * control1.y + 3 * u * tt * control2.y + ttt * end.y,
    };
  };
};

/**
 * Spring physics interpolator
 */
export const springInterpolator = (
  stiffness: number = 100,
  damping: number = 10,
  mass: number = 1
): ((t: number) => number) => {
  return (t: number): number => {
    const w0 = Math.sqrt(stiffness / mass);
    const zeta = damping / (2 * Math.sqrt(stiffness * mass));

    if (zeta < 1) {
      // Underdamped
      const wd = w0 * Math.sqrt(1 - zeta * zeta);
      const A = 1;
      const B = (zeta * w0) / wd;
      return 1 - Math.exp(-zeta * w0 * t) * (A * Math.cos(wd * t) + B * Math.sin(wd * t));
    } else if (zeta === 1) {
      // Critically damped
      return 1 - Math.exp(-w0 * t) * (1 + w0 * t);
    } else {
      // Overdamped
      const r1 = -w0 * (zeta + Math.sqrt(zeta * zeta - 1));
      const r2 = -w0 * (zeta - Math.sqrt(zeta * zeta - 1));
      const A = r2 / (r2 - r1);
      const B = -r1 / (r2 - r1);
      return 1 - (A * Math.exp(r1 * t) + B * Math.exp(r2 * t));
    }
  };
};

/**
 * Elastic interpolator with customizable parameters
 */
export const elasticInterpolator = (
  amplitude: number = 1,
  period: number = 0.3
): ((t: number) => number) => {
  return (t: number): number => {
    if (t === 0 || t === 1) return t;
    const s = (period / (2 * Math.PI)) * Math.asin(1 / amplitude);
    return amplitude * Math.pow(2, -10 * t) * Math.sin(((t - s) * (2 * Math.PI)) / period) + 1;
  };
};

/**
 * Interpolate array of numbers
 */
export const arrayInterpolator = (
  start: number[],
  end: number[]
): InterpolatorFunction<number[]> => {
  const length = Math.max(start.length, end.length);
  const interpolators: Array<(t: number) => number> = [];

  for (let i = 0; i < length; i++) {
    const a = start[i] || 0;
    const b = end[i] || 0;
    interpolators.push(d3.interpolateNumber(a, b));
  }

  return (t: number): number[] => interpolators.map((interp) => interp(t));
};

/**
 * Interpolate object properties
 */
export const objectInterpolator = <T extends Record<string, any>>(
  start: T,
  end: T
): InterpolatorFunction<T> => {
  const keys = Array.from(new Set([...Object.keys(start), ...Object.keys(end)]));
  const interpolators: Record<string, (t: number) => any> = {};

  keys.forEach((key) => {
    const startVal = start[key];
    const endVal = end[key];

    if (typeof startVal === 'number' && typeof endVal === 'number') {
      interpolators[key] = d3.interpolateNumber(startVal, endVal);
    } else if (typeof startVal === 'string' && typeof endVal === 'string') {
      interpolators[key] = d3.interpolateString(startVal, endVal);
    } else {
      interpolators[key] = () => endVal;
    }
  });

  return (t: number): T => {
    const result: any = {};
    keys.forEach((key) => {
      result[key] = interpolators[key]!(t);
    });
    return result as T;
  };
};

/**
 * Path morphing interpolator
 */
export const pathInterpolator = (
  start: string,
  end: string
): InterpolatorFunction<string> => {
  return d3.interpolateString(start, end) as InterpolatorFunction<string>;
};

/**
 * Scale interpolator (logarithmic)
 */
export const logInterpolator = (
  start: number,
  end: number
): InterpolatorFunction<number> => {
  const logStart = Math.log(start);
  const logEnd = Math.log(end);

  return (t: number): number => {
    return Math.exp(logStart + (logEnd - logStart) * t);
  };
};

/**
 * Angle interpolator (shortest path)
 */
export const angleInterpolator = (
  start: number,
  end: number
): InterpolatorFunction<number> => {
  let delta = ((end - start + Math.PI) % (2 * Math.PI)) - Math.PI;
  if (delta < -Math.PI) delta += 2 * Math.PI;

  return (t: number): number => {
    return start + delta * t;
  };
};

/**
 * Custom easing function factory
 */
export const createCustomEasing = (
  controlPoints: number[]
): ((t: number) => number) => {
  // Cubic Bezier easing
  const [x1, y1, x2, y2] = controlPoints;

  const sampleCurveX = (t: number): number => {
    return ((1 - t) ** 3) * 0 + 3 * ((1 - t) ** 2) * t * x1 + 3 * (1 - t) * (t ** 2) * x2 + t ** 3 * 1;
  };

  const sampleCurveY = (t: number): number => {
    return ((1 - t) ** 3) * 0 + 3 * ((1 - t) ** 2) * t * y1 + 3 * (1 - t) * (t ** 2) * y2 + t ** 3 * 1;
  };

  const solveCurveX = (x: number, epsilon: number = 1e-6): number => {
    let t0 = 0;
    let t1 = 1;
    let t2 = x;
    let x2: number;
    let i: number;

    // Binary subdivision
    for (i = 0; i < 8; i++) {
      x2 = sampleCurveX(t2) - x;
      if (Math.abs(x2) < epsilon) return t2;
      if (x2 > 0) {
        t1 = t2;
      } else {
        t0 = t2;
      }
      t2 = (t1 + t0) / 2;
    }

    return t2;
  };

  return (t: number): number => {
    if (t === 0 || t === 1) return t;
    const tSolved = solveCurveX(t);
    return sampleCurveY(tSolved);
  };
};

/**
 * Stepped interpolator (discrete steps)
 */
export const steppedInterpolator = (
  steps: number
): ((t: number) => number) => {
  return (t: number): number => {
    return Math.floor(t * steps) / steps;
  };
};

/**
 * Overshoot interpolator (goes beyond target then returns)
 */
export const overshootInterpolator = (
  amount: number = 0.7
): ((t: number) => number) => {
  return (t: number): number => {
    const s = amount;
    return t * t * ((s + 1) * t - s);
  };
};

/**
 * Anticipate interpolator (backs up before moving forward)
 */
export const anticipateInterpolator = (
  tension: number = 2
): ((t: number) => number) => {
  return (t: number): number => {
    return t * t * ((tension + 1) * t - tension);
  };
};

/**
 * Wiggle interpolator (oscillates around target)
 */
export const wiggleInterpolator = (
  frequency: number = 3,
  amplitude: number = 0.1
): ((t: number) => number) => {
  return (t: number): number => {
    return t + amplitude * Math.sin(t * frequency * Math.PI * 2) * (1 - t);
  };
};

/**
 * Smooth step interpolator (smoother than linear)
 */
export const smoothStepInterpolator = (
  start: number,
  end: number
): InterpolatorFunction<number> => {
  return (t: number): number => {
    const smoothed = t * t * (3 - 2 * t);
    return start + (end - start) * smoothed;
  };
};

/**
 * Smoother step interpolator (even smoother)
 */
export const smootherStepInterpolator = (
  start: number,
  end: number
): InterpolatorFunction<number> => {
  return (t: number): number => {
    const smoothed = t * t * t * (t * (t * 6 - 15) + 10);
    return start + (end - start) * smoothed;
  };
};

/**
 * Perlin noise-based interpolator
 */
export const noiseInterpolator = (
  amplitude: number = 0.1,
  frequency: number = 2
): ((t: number) => number) => {
  return (t: number): number => {
    // Simplified Perlin noise
    const noise = Math.sin(t * frequency * Math.PI) * amplitude;
    return t + noise;
  };
};

// Export all interpolators as a collection
export const Interpolators = {
  color: colorInterpolator,
  position: positionInterpolator,
  circular: circularInterpolator,
  bezier: bezierInterpolator,
  spring: springInterpolator,
  elastic: elasticInterpolator,
  array: arrayInterpolator,
  object: objectInterpolator,
  path: pathInterpolator,
  log: logInterpolator,
  angle: angleInterpolator,
  stepped: steppedInterpolator,
  overshoot: overshootInterpolator,
  anticipate: anticipateInterpolator,
  wiggle: wiggleInterpolator,
  smoothStep: smoothStepInterpolator,
  smootherStep: smootherStepInterpolator,
  noise: noiseInterpolator,
  createCustomEasing,
};

export default Interpolators;
