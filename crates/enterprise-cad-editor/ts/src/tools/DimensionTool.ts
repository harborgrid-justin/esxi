/**
 * Dimension Tool - CAD Dimensioning
 * Create linear, angular, radial, and diameter dimensions
 */

import { Point, Dimension, DimensionStyle } from '../types';
import { GeometryEngine } from '../engine/GeometryEngine';

export class DimensionTool {
  private dimensionType: 'linear' | 'angular' | 'radial' | 'diameter' = 'linear';
  private points: Point[] = [];
  private offset: number = 20;
  private style: DimensionStyle;
  private onDimensionCreated?: (dimension: Dimension) => void;

  constructor(onDimensionCreated?: (dimension: Dimension) => void) {
    this.style = {
      arrowSize: 10,
      arrowType: 'arrow',
      lineColor: '#000000',
      lineWidth: 1,
      textSize: 12,
      textColor: '#000000',
      textFont: 'Arial',
      precision: 2,
      extensionLineOffset: 2,
      extensionLineExtension: 3
    };
    this.onDimensionCreated = onDimensionCreated;
  }

  /**
   * Set dimension type
   */
  setType(type: 'linear' | 'angular' | 'radial' | 'diameter'): void {
    this.dimensionType = type;
    this.reset();
  }

  /**
   * Set dimension offset
   */
  setOffset(offset: number): void {
    this.offset = offset;
  }

  /**
   * Update style
   */
  updateStyle(style: Partial<DimensionStyle>): void {
    Object.assign(this.style, style);
  }

  /**
   * Add dimension point
   */
  addPoint(point: Point): void {
    this.points.push(point);

    if (this.canCompleteDimension()) {
      this.createDimension();
    }
  }

  /**
   * Check if dimension can be completed
   */
  private canCompleteDimension(): boolean {
    switch (this.dimensionType) {
      case 'linear':
        return this.points.length === 2;
      case 'angular':
        return this.points.length === 3;
      case 'radial':
      case 'diameter':
        return this.points.length === 2;
      default:
        return false;
    }
  }

  /**
   * Create dimension
   */
  private createDimension(): void {
    let dimension: Dimension | null = null;

    switch (this.dimensionType) {
      case 'linear':
        dimension = this.createLinearDimension();
        break;
      case 'angular':
        dimension = this.createAngularDimension();
        break;
      case 'radial':
        dimension = this.createRadialDimension();
        break;
      case 'diameter':
        dimension = this.createDiameterDimension();
        break;
    }

    if (dimension && this.onDimensionCreated) {
      this.onDimensionCreated(dimension);
    }

    this.reset();
  }

  /**
   * Create linear dimension
   */
  private createLinearDimension(): Dimension | null {
    if (this.points.length < 2) return null;

    const distance = GeometryEngine.distance(this.points[0], this.points[1]);

    return {
      id: `dim_${Date.now()}`,
      type: 'linear',
      value: distance,
      points: this.points,
      offset: this.offset,
      style: this.style,
      text: `${distance.toFixed(this.style.precision)}`
    };
  }

  /**
   * Create angular dimension
   */
  private createAngularDimension(): Dimension | null {
    if (this.points.length < 3) return null;

    const angle = GeometryEngine.angleBetween(
      this.points[0],
      this.points[1],
      this.points[2]
    );

    const degrees = (angle * 180) / Math.PI;

    return {
      id: `dim_${Date.now()}`,
      type: 'angular',
      value: degrees,
      points: this.points,
      offset: this.offset,
      style: this.style,
      text: `${degrees.toFixed(this.style.precision)}°`
    };
  }

  /**
   * Create radial dimension
   */
  private createRadialDimension(): Dimension | null {
    if (this.points.length < 2) return null;

    const radius = GeometryEngine.distance(this.points[0], this.points[1]);

    return {
      id: `dim_${Date.now()}`,
      type: 'radial',
      value: radius,
      points: this.points,
      offset: this.offset,
      style: this.style,
      text: `R${radius.toFixed(this.style.precision)}`
    };
  }

  /**
   * Create diameter dimension
   */
  private createDiameterDimension(): Dimension | null {
    if (this.points.length < 2) return null;

    const diameter = GeometryEngine.distance(this.points[0], this.points[1]) * 2;

    return {
      id: `dim_${Date.now()}`,
      type: 'diameter',
      value: diameter,
      points: this.points,
      offset: this.offset,
      style: this.style,
      text: `Ø${diameter.toFixed(this.style.precision)}`
    };
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
