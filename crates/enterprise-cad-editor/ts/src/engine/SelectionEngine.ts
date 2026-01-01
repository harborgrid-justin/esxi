/**
 * Selection Engine - Shape Selection and Hit Testing
 * Handles selection logic, hit testing, and selection box operations
 */

import { Point, Shape, SelectionState, SelectionBox, BoundingBox } from '../types';
import { GeometryEngine } from './GeometryEngine';

export class SelectionEngine {
  private selectionState: SelectionState;
  private shapes: Map<string, Shape>;

  constructor() {
    this.selectionState = {
      selectedIds: new Set(),
      mode: 'normal',
      select: (ids: string | string[]) => this.select(ids),
      deselect: (ids: string | string[]) => this.deselect(ids),
      clear: () => this.clear(),
      toggle: (id: string) => this.toggle(id),
      contains: (id: string) => this.contains(id)
    };
    this.shapes = new Map();
  }

  /**
   * Register shapes for selection
   */
  registerShapes(shapes: Shape[]): void {
    this.shapes.clear();
    for (const shape of shapes) {
      this.shapes.set(shape.id, shape);
    }
  }

  /**
   * Hit test at a point
   */
  hitTest(point: Point, threshold: number = 5): Shape | null {
    const shapes = Array.from(this.shapes.values())
      .filter(s => s.visible && !s.locked)
      .reverse(); // Test top to bottom

    for (const shape of shapes) {
      if (this.testShape(shape, point, threshold)) {
        return shape;
      }
    }

    return null;
  }

  /**
   * Test if a point hits a shape
   */
  private testShape(shape: Shape, point: Point, threshold: number): boolean {
    // First check bounding box
    const bounds = shape.getBounds();
    const expandedBounds: BoundingBox = {
      minX: bounds.minX - threshold,
      minY: bounds.minY - threshold,
      maxX: bounds.maxX + threshold,
      maxY: bounds.maxY + threshold,
      width: bounds.width + threshold * 2,
      height: bounds.height + threshold * 2,
      center: bounds.center
    };

    if (!GeometryEngine.pointInBounds(point, expandedBounds)) {
      return false;
    }

    // Detailed hit testing based on shape type
    return shape.containsPoint(point);
  }

  /**
   * Select shapes
   */
  select(ids: string | string[]): void {
    const idArray = Array.isArray(ids) ? ids : [ids];

    for (const id of idArray) {
      this.selectionState.selectedIds.add(id);
      const shape = this.shapes.get(id);
      if (shape) {
        shape.selected = true;
      }
    }

    this.updateSelectionBounds();
  }

  /**
   * Deselect shapes
   */
  deselect(ids: string | string[]): void {
    const idArray = Array.isArray(ids) ? ids : [ids];

    for (const id of idArray) {
      this.selectionState.selectedIds.delete(id);
      const shape = this.shapes.get(id);
      if (shape) {
        shape.selected = false;
      }
    }

    this.updateSelectionBounds();
  }

  /**
   * Clear all selections
   */
  clear(): void {
    for (const id of this.selectionState.selectedIds) {
      const shape = this.shapes.get(id);
      if (shape) {
        shape.selected = false;
      }
    }

    this.selectionState.selectedIds.clear();
    this.selectionState.bounds = undefined;
  }

  /**
   * Toggle selection
   */
  toggle(id: string): void {
    if (this.selectionState.selectedIds.has(id)) {
      this.deselect(id);
    } else {
      this.select(id);
    }
  }

  /**
   * Check if shape is selected
   */
  contains(id: string): boolean {
    return this.selectionState.selectedIds.has(id);
  }

  /**
   * Select shapes in box
   */
  selectInBox(box: SelectionBox): void {
    const bounds = box.getBounds();

    for (const shape of this.shapes.values()) {
      if (!shape.visible || shape.locked) continue;

      const selected = box.mode === 'contain'
        ? this.shapeContainedInBounds(shape, bounds)
        : this.shapeIntersectsBounds(shape, bounds);

      if (selected) {
        this.select(shape.id);
      }
    }
  }

  /**
   * Check if shape is contained in bounds
   */
  private shapeContainedInBounds(shape: Shape, bounds: BoundingBox): boolean {
    const shapeBounds = shape.getBounds();

    return (
      shapeBounds.minX >= bounds.minX &&
      shapeBounds.maxX <= bounds.maxX &&
      shapeBounds.minY >= bounds.minY &&
      shapeBounds.maxY <= bounds.maxY
    );
  }

  /**
   * Check if shape intersects bounds
   */
  private shapeIntersectsBounds(shape: Shape, bounds: BoundingBox): boolean {
    const shapeBounds = shape.getBounds();
    return GeometryEngine.boundsIntersect(shapeBounds, bounds);
  }

  /**
   * Update selection bounds
   */
  private updateSelectionBounds(): void {
    if (this.selectionState.selectedIds.size === 0) {
      this.selectionState.bounds = undefined;
      return;
    }

    const selectedShapes = Array.from(this.selectionState.selectedIds)
      .map(id => this.shapes.get(id))
      .filter(s => s !== undefined) as Shape[];

    if (selectedShapes.length === 0) {
      this.selectionState.bounds = undefined;
      return;
    }

    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;

    for (const shape of selectedShapes) {
      const bounds = shape.getBounds();
      minX = Math.min(minX, bounds.minX);
      minY = Math.min(minY, bounds.minY);
      maxX = Math.max(maxX, bounds.maxX);
      maxY = Math.max(maxY, bounds.maxY);
    }

    this.selectionState.bounds = {
      minX,
      minY,
      maxX,
      maxY,
      width: maxX - minX,
      height: maxY - minY,
      center: {
        x: (minX + maxX) / 2,
        y: (minY + maxY) / 2
      }
    };
  }

  /**
   * Set hover state
   */
  setHovered(id: string | undefined): void {
    this.selectionState.hoveredId = id;
  }

  /**
   * Set focused state
   */
  setFocused(id: string | undefined): void {
    this.selectionState.focusedId = id;
  }

  /**
   * Get selection state
   */
  getState(): SelectionState {
    return this.selectionState;
  }

  /**
   * Get selected shapes
   */
  getSelectedShapes(): Shape[] {
    return Array.from(this.selectionState.selectedIds)
      .map(id => this.shapes.get(id))
      .filter(s => s !== undefined) as Shape[];
  }

  /**
   * Get selection count
   */
  getSelectionCount(): number {
    return this.selectionState.selectedIds.size;
  }
}
