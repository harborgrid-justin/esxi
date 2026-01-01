/**
 * Shape Tool - Drawing Primitive Shapes
 * Create rectangles, circles, ellipses, polygons, and lines
 */

import { Point, Shape, ShapeType, RectangleShape, CircleShape, PolygonShape, LineShape } from '../types';
import { GeometryEngine } from '../engine/GeometryEngine';
import { TransformEngine } from '../engine/TransformEngine';

export class ShapeTool {
  private shapeType: ShapeType;
  private startPoint: Point | null = null;
  private currentPoint: Point | null = null;
  private polygonPoints: Point[] = [];
  private onShapeCreated?: (shape: Shape) => void;

  constructor(shapeType: ShapeType, onShapeCreated?: (shape: Shape) => void) {
    this.shapeType = shapeType;
    this.onShapeCreated = onShapeCreated;
  }

  /**
   * Set shape type
   */
  setShapeType(type: ShapeType): void {
    this.shapeType = type;
    this.reset();
  }

  /**
   * Handle mouse down
   */
  onMouseDown(point: Point, event: MouseEvent): void {
    if (this.shapeType === ShapeType.Polygon) {
      this.polygonPoints.push(point);
    } else {
      this.startPoint = point;
      this.currentPoint = point;
    }
  }

  /**
   * Handle mouse move
   */
  onMouseMove(point: Point, event: MouseEvent): void {
    this.currentPoint = point;
  }

  /**
   * Handle mouse up
   */
  onMouseUp(point: Point, event: MouseEvent): void {
    this.currentPoint = point;

    if (this.shapeType !== ShapeType.Polygon) {
      const shape = this.createShape();
      if (shape && this.onShapeCreated) {
        this.onShapeCreated(shape);
      }
      this.reset();
    }
  }

  /**
   * Handle double-click (finish polygon)
   */
  onDoubleClick(point: Point, event: MouseEvent): void {
    if (this.shapeType === ShapeType.Polygon && this.polygonPoints.length >= 3) {
      const shape = this.createPolygon();
      if (shape && this.onShapeCreated) {
        this.onShapeCreated(shape);
      }
      this.reset();
    }
  }

  /**
   * Create shape based on type
   */
  private createShape(): Shape | null {
    if (!this.startPoint || !this.currentPoint) return null;

    switch (this.shapeType) {
      case ShapeType.Rectangle:
        return this.createRectangle();
      case ShapeType.Circle:
        return this.createCircle();
      case ShapeType.Ellipse:
        return this.createEllipse();
      case ShapeType.Line:
        return this.createLine();
      default:
        return null;
    }
  }

  /**
   * Create rectangle
   */
  private createRectangle(): RectangleShape | null {
    if (!this.startPoint || !this.currentPoint) return null;

    const x = Math.min(this.startPoint.x, this.currentPoint.x);
    const y = Math.min(this.startPoint.y, this.currentPoint.y);
    const width = Math.abs(this.currentPoint.x - this.startPoint.x);
    const height = Math.abs(this.currentPoint.y - this.startPoint.y);

    return {
      id: `rect_${Date.now()}`,
      type: ShapeType.Rectangle,
      layerId: 'default',
      rect: { x, y, width, height },
      transform: TransformEngine.identity(),
      style: { fill: '#cccccc', stroke: '#000000', strokeWidth: 1 },
      locked: false,
      visible: true,
      selected: false,
      getBounds: function() {
        return {
          minX: this.rect.x,
          minY: this.rect.y,
          maxX: this.rect.x + this.rect.width,
          maxY: this.rect.y + this.rect.height,
          width: this.rect.width,
          height: this.rect.height,
          center: { x: this.rect.x + this.rect.width / 2, y: this.rect.y + this.rect.height / 2 }
        };
      },
      containsPoint: function(point: Point) {
        return point.x >= this.rect.x && point.x <= this.rect.x + this.rect.width &&
               point.y >= this.rect.y && point.y <= this.rect.y + this.rect.height;
      },
      intersects: () => false,
      clone: function() { return { ...this }; }
    };
  }

  /**
   * Create circle
   */
  private createCircle(): CircleShape | null {
    if (!this.startPoint || !this.currentPoint) return null;

    const radius = GeometryEngine.distance(this.startPoint, this.currentPoint);

    return {
      id: `circle_${Date.now()}`,
      type: ShapeType.Circle,
      layerId: 'default',
      circle: { center: this.startPoint, radius },
      transform: TransformEngine.identity(),
      style: { fill: '#cccccc', stroke: '#000000', strokeWidth: 1 },
      locked: false,
      visible: true,
      selected: false,
      getBounds: function() {
        return {
          minX: this.circle.center.x - this.circle.radius,
          minY: this.circle.center.y - this.circle.radius,
          maxX: this.circle.center.x + this.circle.radius,
          maxY: this.circle.center.y + this.circle.radius,
          width: this.circle.radius * 2,
          height: this.circle.radius * 2,
          center: this.circle.center
        };
      },
      containsPoint: function(point: Point) {
        return GeometryEngine.distance(point, this.circle.center) <= this.circle.radius;
      },
      intersects: () => false,
      clone: function() { return { ...this }; }
    };
  }

  /**
   * Create ellipse
   */
  private createEllipse(): Shape | null {
    if (!this.startPoint || !this.currentPoint) return null;

    const radiusX = Math.abs(this.currentPoint.x - this.startPoint.x);
    const radiusY = Math.abs(this.currentPoint.y - this.startPoint.y);

    return {
      id: `ellipse_${Date.now()}`,
      type: ShapeType.Ellipse,
      layerId: 'default',
      center: this.startPoint,
      radiusX,
      radiusY,
      transform: TransformEngine.identity(),
      style: { fill: '#cccccc', stroke: '#000000', strokeWidth: 1 },
      locked: false,
      visible: true,
      selected: false,
      getBounds: function() {
        return {
          minX: this.center.x - this.radiusX,
          minY: this.center.y - this.radiusY,
          maxX: this.center.x + this.radiusX,
          maxY: this.center.y + this.radiusY,
          width: this.radiusX * 2,
          height: this.radiusY * 2,
          center: this.center
        };
      },
      containsPoint: function(point: Point) {
        const dx = (point.x - this.center.x) / this.radiusX;
        const dy = (point.y - this.center.y) / this.radiusY;
        return dx * dx + dy * dy <= 1;
      },
      intersects: () => false,
      clone: function() { return { ...this }; }
    } as any;
  }

  /**
   * Create line
   */
  private createLine(): LineShape | null {
    if (!this.startPoint || !this.currentPoint) return null;

    return {
      id: `line_${Date.now()}`,
      type: ShapeType.Line,
      layerId: 'default',
      start: this.startPoint,
      end: this.currentPoint,
      transform: TransformEngine.identity(),
      style: { stroke: '#000000', strokeWidth: 2 },
      locked: false,
      visible: true,
      selected: false,
      getBounds: function() {
        const points = [this.start, this.end];
        return GeometryEngine.getBounds(points);
      },
      containsPoint: function(point: Point) {
        return GeometryEngine.isPointOnLine(point, this.start, this.end, 5);
      },
      intersects: () => false,
      clone: function() { return { ...this }; }
    };
  }

  /**
   * Create polygon
   */
  private createPolygon(): PolygonShape | null {
    if (this.polygonPoints.length < 3) return null;

    return {
      id: `polygon_${Date.now()}`,
      type: ShapeType.Polygon,
      layerId: 'default',
      points: [...this.polygonPoints],
      transform: TransformEngine.identity(),
      style: { fill: '#cccccc', stroke: '#000000', strokeWidth: 1 },
      locked: false,
      visible: true,
      selected: false,
      getBounds: function() {
        return GeometryEngine.getBounds(this.points);
      },
      containsPoint: function(point: Point) {
        return GeometryEngine.pointInPolygon(point, this.points);
      },
      intersects: () => false,
      clone: function() { return { ...this }; }
    };
  }

  /**
   * Get preview shape
   */
  getPreview(): Shape | null {
    if (this.shapeType === ShapeType.Polygon) {
      return this.polygonPoints.length > 0 ? this.createPolygon() : null;
    }
    return this.createShape();
  }

  /**
   * Reset tool
   */
  reset(): void {
    this.startPoint = null;
    this.currentPoint = null;
    this.polygonPoints = [];
  }
}
