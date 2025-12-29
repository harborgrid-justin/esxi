/**
 * React hook for color blindness simulation
 */

import { useState, useMemo, useCallback } from 'react';
import { RGB, ColorBlindnessType, SimulationResult } from '../types';
import {
  simulateColorBlindness,
  simulateAllTypes,
  areColorsDistinguishable,
  getColorBlindnessName,
  getColorBlindnessDescription,
} from '../algorithms/ColorBlindness';
import { hexToRgb, isValidHex, rgbToHex } from '../utils/colorMath';

export interface UseColorBlindnessOptions {
  /** Color to simulate */
  color: string | RGB;
  /** Type of color blindness */
  type?: ColorBlindnessType;
  /** Severity (0-1) for anomalous types */
  severity?: number;
  /** Simulate all types */
  simulateAll?: boolean;
}

export interface UseColorBlindnessResult {
  /** Original RGB color */
  original: RGB | null;
  /** Simulated RGB color */
  simulated: RGB | null;
  /** Original hex color */
  originalHex: string | null;
  /** Simulated hex color */
  simulatedHex: string | null;
  /** Current simulation type */
  type: ColorBlindnessType;
  /** Current severity */
  severity: number;
  /** Simulation result */
  result: SimulationResult | null;
  /** All simulations (if simulateAll is true) */
  allSimulations: SimulationResult[];
  /** Human-readable name */
  typeName: string;
  /** Description */
  typeDescription: string;
  /** Error if any */
  error: string | null;
  /** Change simulation type */
  setType: (type: ColorBlindnessType) => void;
  /** Change severity */
  setSeverity: (severity: number) => void;
  /** Update color */
  setColor: (color: string | RGB) => void;
}

/**
 * Hook for color blindness simulation
 */
export function useColorBlindness(options: UseColorBlindnessOptions): UseColorBlindnessResult {
  const [color, setColorState] = useState<string | RGB>(options.color);
  const [type, setType] = useState<ColorBlindnessType>(
    options.type || ColorBlindnessType.DEUTERANOPIA
  );
  const [severity, setSeverity] = useState<number>(options.severity || 1.0);
  const [error, setError] = useState<string | null>(null);

  // Convert color to RGB
  const original = useMemo(() => {
    try {
      setError(null);
      if (typeof color === 'string') {
        if (!isValidHex(color)) {
          throw new Error('Invalid hex color');
        }
        return hexToRgb(color);
      }
      return color;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Invalid color');
      return null;
    }
  }, [color]);

  // Simulate current type
  const simulated = useMemo(() => {
    if (!original) return null;
    try {
      return simulateColorBlindness(original, type, severity);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Simulation failed');
      return null;
    }
  }, [original, type, severity]);

  // Get hex colors
  const originalHex = useMemo(() => {
    return original ? rgbToHex(original) : null;
  }, [original]);

  const simulatedHex = useMemo(() => {
    return simulated ? rgbToHex(simulated) : null;
  }, [simulated]);

  // Create simulation result
  const result = useMemo<SimulationResult | null>(() => {
    if (!original || !simulated) return null;
    return {
      type,
      original,
      simulated,
      severity,
    };
  }, [original, simulated, type, severity]);

  // Simulate all types
  const allSimulations = useMemo(() => {
    if (!options.simulateAll || !original) return [];
    try {
      return simulateAllTypes(original);
    } catch (err) {
      return [];
    }
  }, [original, options.simulateAll]);

  // Get type info
  const typeName = useMemo(() => getColorBlindnessName(type), [type]);
  const typeDescription = useMemo(() => getColorBlindnessDescription(type), [type]);

  // Callbacks
  const setColor = useCallback((newColor: string | RGB) => {
    setColorState(newColor);
  }, []);

  return {
    original,
    simulated,
    originalHex,
    simulatedHex,
    type,
    severity,
    result,
    allSimulations,
    typeName,
    typeDescription,
    error,
    setType,
    setSeverity,
    setColor,
  };
}

/**
 * Hook for checking if two colors are distinguishable for color blind users
 */
export function useColorDistinguishability(
  color1: string | RGB,
  color2: string | RGB,
  type?: ColorBlindnessType
): {
  distinguishable: boolean;
  results: Record<ColorBlindnessType, boolean>;
} {
  const rgb1 = useMemo(() => {
    return typeof color1 === 'string' ? hexToRgb(color1) : color1;
  }, [color1]);

  const rgb2 = useMemo(() => {
    return typeof color2 === 'string' ? hexToRgb(color2) : color2;
  }, [color2]);

  const results = useMemo(() => {
    const allTypes = Object.values(ColorBlindnessType);
    const resultsMap: Record<ColorBlindnessType, boolean> = {} as any;

    for (const cbType of allTypes) {
      resultsMap[cbType] = areColorsDistinguishable(rgb1, rgb2, cbType);
    }

    return resultsMap;
  }, [rgb1, rgb2]);

  const distinguishable = useMemo(() => {
    if (type) {
      return results[type];
    }
    // Check if distinguishable for all common types
    return (
      results[ColorBlindnessType.DEUTERANOPIA] &&
      results[ColorBlindnessType.PROTANOPIA] &&
      results[ColorBlindnessType.TRITANOPIA]
    );
  }, [results, type]);

  return { distinguishable, results };
}
