/**
 * Measure Tool - Distance and Angle Measurement
 * Measure distances, angles, areas, and perimeters
 */

import { Point, Measurement } from '../types';
import { GeometryEngine } from '../engine/GeometryEngine';

export class MeasureTool {
  private measurementType: 'distance' | 'angle' | 'area' | 'perimeter' = 'distance';
  private points: Point[] = [];
  private units: string = 'px';
  private onMeasurementCreated?: (measurement: Measurement) => void;

  constructor(units: string = 'px', onMeasurementCreated?: (measurement: Measurement) => void) {
    this.units = units;
    this.onMeasurementCreated = onMeasurementCreated;
  }

  /**
   * Set measurement type
   */
  setType(type: 'distance' | 'angle' | 'area' | 'perimeter'): void {
    this.measurementType = type;
    this.reset();
  }

  /**
   * Set units
   */
  setUnits(units: string): void {
    this.units = units;
  }

  /**
   * Add measurement point
   */
  addPoint(point: Point): void {
    this.points.push(point);

    if (this.canCompleteMeasurement()) {
      this.createMeasurement();
    }
  }

  /**
   * Check if measurement can be completed
   */
  private canCompleteMeasurement(): boolean {
    switch (this.measurementType) {
      case 'distance':
        return this.points.length === 2;
      case 'angle':
        return this.points.length === 3;
      case 'area':
      case 'perimeter':
        return this.points.length >= 3;
      default:
        return false;
    }
  }

  /**
   * Create measurement
   */
  private createMeasurement(): void {
    let measurement: Measurement | null = null;

    switch (this.measurementType) {
      case 'distance':
        measurement = this.measureDistance();
        break;
      case 'angle':
        measurement = this.measureAngle();
        break;
      case 'area':
        measurement = this.measureArea();
        break;
      case 'perimeter':
        measurement = this.measurePerimeter();
        break;
    }

    if (measurement && this.onMeasurementCreated) {
      this.onMeasurementCreated(measurement);
    }

    this.reset();
  }

  /**
   * Measure distance
   */
  private measureDistance(): Measurement | null {
    if (this.points.length < 2) return null;

    const distance = GeometryEngine.distance(this.points[0], this.points[1]);

    return {
      id: `measure_${Date.now()}`,
      type: 'distance',
      value: this.convertUnits(distance),
      units: this.units,
      start: this.points[0],
      end: this.points[1],
      label: `${this.formatValue(distance)} ${this.units}`
    };
  }

  /**
   * Measure angle
   */
  private measureAngle(): Measurement | null {
    if (this.points.length < 3) return null;

    const angle = GeometryEngine.angleBetween(
      this.points[0],
      this.points[1],
      this.points[2]
    );

    const degrees = (angle * 180) / Math.PI;

    return {
      id: `measure_${Date.now()}`,
      type: 'angle',
      value: degrees,
      units: '°',
      points: this.points,
      label: `${this.formatValue(degrees)}°`
    };
  }

  /**
   * Measure area
   */
  private measureArea(): Measurement | null {
    if (this.points.length < 3) return null;

    const area = Math.abs(GeometryEngine.polygonArea(this.points));

    return {
      id: `measure_${Date.now()}`,
      type: 'area',
      value: this.convertUnits(area),
      units: `${this.units}²`,
      points: this.points,
      label: `${this.formatValue(area)} ${this.units}²`
    };
  }

  /**
   * Measure perimeter
   */
  private measurePerimeter(): Measurement | null {
    if (this.points.length < 2) return null;

    let perimeter = 0;
    for (let i = 0; i < this.points.length; i++) {
      const next = (i + 1) % this.points.length;
      perimeter += GeometryEngine.distance(this.points[i], this.points[next]);
    }

    return {
      id: `measure_${Date.now()}`,
      type: 'perimeter',
      value: this.convertUnits(perimeter),
      units: this.units,
      points: this.points,
      label: `${this.formatValue(perimeter)} ${this.units}`
    };
  }

  /**
   * Convert pixel units to target units
   */
  private convertUnits(value: number): number {
    // Conversion factors (px to unit)
    const conversions: Record<string, number> = {
      px: 1,
      mm: 0.264583,
      cm: 0.0264583,
      in: 0.0104167,
      pt: 0.75
    };

    return value * (conversions[this.units] || 1);
  }

  /**
   * Format value for display
   */
  private formatValue(value: number, precision: number = 2): string {
    return value.toFixed(precision);
  }

  /**
   * Reset tool
   */
  reset(): void {
    this.points = [];
  }

  /**
   * Get current points
   */
  getPoints(): Point[] {
    return this.points;
  }
}
