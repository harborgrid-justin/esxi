/**
 * Pen Tool - Bezier Curve Drawing Tool
 * Create complex paths with Bezier curves and straight lines
 */

import { Point, Path, PathCommand, PathCommandType, Shape, PathShape, ShapeType } from '../types';
import { GeometryEngine } from '../engine/GeometryEngine';
import { TransformEngine } from '../engine/TransformEngine';

export interface PenToolState {
  active: boolean;
  currentPath: PathCommand[];
  currentPoint: Point | null;
  controlPoint1: Point | null;
  controlPoint2: Point | null;
  mode: 'line' | 'curve';
  closed: boolean;
}

export class PenTool {
  private state: PenToolState;
  private onShapeCreated?: (shape: Shape) => void;

  constructor(onShapeCreated?: (shape: Shape) => void) {
    this.state = {
      active: false,
      currentPath: [],
      currentPoint: null,
      controlPoint1: null,
      controlPoint2: null,
      mode: 'line',
      closed: false
    };
    this.onShapeCreated = onShapeCreated;
  }

  /**
   * Start using the pen tool
   */
  activate(): void {
    this.state.active = true;
    this.reset();
  }

  /**
   * Deactivate the pen tool
   */
  deactivate(): void {
    this.state.active = false;
    this.reset();
  }

  /**
   * Handle mouse down
   */
  onMouseDown(point: Point, event: MouseEvent): void {
    if (!this.state.active) return;

    if (event.shiftKey) {
      // Start curve mode
      this.state.mode = 'curve';
      this.addCurvePoint(point);
    } else {
      // Line mode
      this.state.mode = 'line';
      this.addLinePoint(point);
    }
  }

  /**
   * Handle mouse move
   */
  onMouseMove(point: Point, event: MouseEvent): void {
    if (!this.state.active) return;

    if (this.state.mode === 'curve' && this.state.currentPoint) {
      // Update control point for curve
      this.state.controlPoint2 = point;
    }

    this.state.currentPoint = point;
  }

  /**
   * Handle mouse up
   */
  onMouseUp(point: Point, event: MouseEvent): void {
    if (!this.state.active) return;

    if (this.state.mode === 'curve' && this.state.controlPoint1) {
      // Finalize curve
      this.state.controlPoint2 = point;
      this.state.mode = 'line';
    }
  }

  /**
   * Handle double-click to finish path
   */
  onDoubleClick(point: Point, event: MouseEvent): void {
    if (!this.state.active) return;
    this.finishPath(false);
  }

  /**
   * Handle key press
   */
  onKeyDown(event: KeyboardEvent): void {
    if (!this.state.active) return;

    switch (event.key) {
      case 'Enter':
        this.finishPath(false);
        break;
      case 'Escape':
        this.cancel();
        break;
      case 'c':
      case 'C':
        if (event.ctrlKey || event.metaKey) {
          this.finishPath(true); // Close path
        }
        break;
    }
  }

  /**
   * Add a line point
   */
  private addLinePoint(point: Point): void {
    if (this.state.currentPath.length === 0) {
      // First point - MoveTo
      this.state.currentPath.push({
        type: PathCommandType.MoveTo,
        points: [point]
      });
    } else {
      // LineTo
      this.state.currentPath.push({
        type: PathCommandType.LineTo,
        points: [point]
      });
    }

    this.state.currentPoint = point;
  }

  /**
   * Add a curve point
   */
  private addCurvePoint(point: Point): void {
    if (this.state.currentPath.length === 0) {
      // Can't start with a curve
      this.addLinePoint(point);
      return;
    }

    this.state.controlPoint1 = point;
    this.state.currentPoint = point;
  }

  /**
   * Add cubic Bezier curve
   */
  private addCubicCurve(cp1: Point, cp2: Point, end: Point): void {
    this.state.currentPath.push({
      type: PathCommandType.CurveTo,
      points: [cp1, cp2, end]
    });
  }

  /**
   * Add quadratic Bezier curve
   */
  private addQuadraticCurve(cp: Point, end: Point): void {
    this.state.currentPath.push({
      type: PathCommandType.QuadraticCurveTo,
      points: [cp, end]
    });
  }

  /**
   * Finish the current path
   */
  finishPath(close: boolean): void {
    if (this.state.currentPath.length < 2) {
      this.cancel();
      return;
    }

    if (close) {
      this.state.currentPath.push({
        type: PathCommandType.ClosePath,
        points: []
      });
      this.state.closed = true;
    }

    // Create path shape
    const shape = this.createPathShape();
    if (shape && this.onShapeCreated) {
      this.onShapeCreated(shape);
    }

    this.reset();
  }

  /**
   * Cancel the current path
   */
  cancel(): void {
    this.reset();
  }

  /**
   * Reset the tool state
   */
  reset(): void {
    this.state.currentPath = [];
    this.state.currentPoint = null;
    this.state.controlPoint1 = null;
    this.state.controlPoint2 = null;
    this.state.mode = 'line';
    this.state.closed = false;
  }

  /**
   * Create a path shape from current state
   */
  private createPathShape(): PathShape | null {
    if (this.state.currentPath.length === 0) {
      return null;
    }

    const pathId = `path_${Date.now()}`;

    const path: Path = {
      id: pathId,
      commands: this.state.currentPath,
      closed: this.state.closed,
      fillRule: 'nonzero',
      getBounds: function() {
        const points = this.commands.flatMap(cmd => cmd.points);
        return GeometryEngine.getBounds(points);
      },
      getLength: function() {
        let length = 0;
        for (let i = 1; i < this.commands.length; i++) {
          const prev = this.commands[i - 1].points[this.commands[i - 1].points.length - 1];
          const curr = this.commands[i].points[this.commands[i].points.length - 1];
          if (prev && curr) {
            length += GeometryEngine.distance(prev, curr);
          }
        }
        return length;
      },
      getPointAtLength: function(distance: number): Point {
        // Simplified implementation
        return this.commands[0].points[0];
      },
      getTangentAtLength: function(distance: number): any {
        return { x: 1, y: 0 };
      },
      simplify: function(tolerance: number): Path {
        return this;
      },
      reverse: function(): Path {
        const reversed = { ...this };
        reversed.commands = [...this.commands].reverse();
        return reversed;
      },
      offset: function(distance: number): Path[] {
        return [this];
      }
    };

    const shape: PathShape = {
      id: `shape_${pathId}`,
      type: ShapeType.Path,
      layerId: 'default',
      path,
      transform: TransformEngine.identity(),
      style: {
        fill: 'none',
        stroke: '#000000',
        strokeWidth: 2
      },
      locked: false,
      visible: true,
      selected: false,
      getBounds: () => path.getBounds(),
      containsPoint: (point: Point) => {
        // Simplified hit testing
        const bounds = path.getBounds();
        return GeometryEngine.pointInBounds(point, bounds);
      },
      intersects: (other: Shape) => false,
      clone: function() {
        return { ...this };
      }
    };

    return shape;
  }

  /**
   * Get current state
   */
  getState(): PenToolState {
    return this.state;
  }

  /**
   * Get preview path for rendering
   */
  getPreviewPath(): PathCommand[] {
    return [...this.state.currentPath];
  }
}
