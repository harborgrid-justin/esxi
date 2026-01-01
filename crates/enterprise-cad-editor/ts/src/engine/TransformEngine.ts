/**
 * Transform Engine - Affine Transformations for Shapes
 * Handles translation, rotation, scaling, and matrix operations
 */

import { mat3, vec2 } from 'gl-matrix';
import { Point, Transform, Shape } from '../types';

export class TransformEngine {
  /**
   * Create identity transform
   */
  static identity(): Transform {
    return {
      matrix: mat3.create(),
      position: vec2.fromValues(0, 0),
      rotation: 0,
      scale: vec2.fromValues(1, 1),
      pivot: vec2.fromValues(0, 0),
      apply: function(point: Point): Point {
        const v = vec2.fromValues(point.x, point.y);
        vec2.transformMat3(v, v, this.matrix);
        return { x: v[0], y: v[1] };
      },
      inverse: function(): Transform {
        const invMatrix = mat3.create();
        mat3.invert(invMatrix, this.matrix);
        return TransformEngine.fromMatrix(invMatrix);
      },
      compose: function(other: Transform): Transform {
        const composed = mat3.create();
        mat3.multiply(composed, this.matrix, other.matrix);
        return TransformEngine.fromMatrix(composed);
      },
      decompose: function() {
        return TransformEngine.decompose(this.matrix);
      }
    };
  }

  /**
   * Create transform from matrix
   */
  static fromMatrix(matrix: mat3): Transform {
    const { position, rotation, scale } = this.decompose(matrix);

    return {
      matrix: mat3.clone(matrix),
      position,
      rotation,
      scale,
      pivot: vec2.fromValues(0, 0),
      apply: function(point: Point): Point {
        const v = vec2.fromValues(point.x, point.y);
        vec2.transformMat3(v, v, this.matrix);
        return { x: v[0], y: v[1] };
      },
      inverse: function(): Transform {
        const invMatrix = mat3.create();
        mat3.invert(invMatrix, this.matrix);
        return TransformEngine.fromMatrix(invMatrix);
      },
      compose: function(other: Transform): Transform {
        const composed = mat3.create();
        mat3.multiply(composed, this.matrix, other.matrix);
        return TransformEngine.fromMatrix(composed);
      },
      decompose: function() {
        return TransformEngine.decompose(this.matrix);
      }
    };
  }

  /**
   * Decompose matrix into components
   */
  static decompose(matrix: mat3): { position: vec2; rotation: number; scale: vec2 } {
    const position = vec2.fromValues(matrix[6], matrix[7]);

    // Extract scale and rotation from the 2x2 upper-left submatrix
    const a = matrix[0];
    const b = matrix[1];
    const c = matrix[3];
    const d = matrix[4];

    const scaleX = Math.sqrt(a * a + b * b);
    const scaleY = Math.sqrt(c * c + d * d);
    const scale = vec2.fromValues(scaleX, scaleY);

    // Compute rotation (handle scale)
    const rotation = Math.atan2(b / scaleX, a / scaleX);

    return { position, rotation, scale };
  }

  /**
   * Translate shape
   */
  static translate(shape: Shape, dx: number, dy: number): void {
    vec2.add(shape.transform.position, shape.transform.position, vec2.fromValues(dx, dy));
    this.updateMatrix(shape.transform);
  }

  /**
   * Rotate shape around pivot
   */
  static rotate(shape: Shape, angle: number, pivot?: Point): void {
    if (pivot) {
      // Rotate around custom pivot
      const pivotVec = vec2.fromValues(pivot.x, pivot.y);

      // Translate to pivot
      const m1 = mat3.create();
      mat3.fromTranslation(m1, vec2.negate(vec2.create(), pivotVec));

      // Rotate
      const m2 = mat3.create();
      mat3.fromRotation(m2, angle);

      // Translate back
      const m3 = mat3.create();
      mat3.fromTranslation(m3, pivotVec);

      // Compose: m3 * m2 * m1
      const temp = mat3.create();
      mat3.multiply(temp, m2, m1);
      const rotation = mat3.create();
      mat3.multiply(rotation, m3, temp);

      // Apply to current matrix
      mat3.multiply(shape.transform.matrix, rotation, shape.transform.matrix);

      // Update components
      const decomposed = this.decompose(shape.transform.matrix);
      shape.transform.position = decomposed.position;
      shape.transform.rotation = decomposed.rotation;
      shape.transform.scale = decomposed.scale;
    } else {
      // Rotate around origin
      shape.transform.rotation += angle;
      this.updateMatrix(shape.transform);
    }
  }

  /**
   * Scale shape
   */
  static scale(shape: Shape, sx: number, sy: number, pivot?: Point): void {
    if (pivot) {
      // Scale around custom pivot
      const pivotVec = vec2.fromValues(pivot.x, pivot.y);

      const m1 = mat3.create();
      mat3.fromTranslation(m1, vec2.negate(vec2.create(), pivotVec));

      const m2 = mat3.create();
      mat3.fromScaling(m2, vec2.fromValues(sx, sy));

      const m3 = mat3.create();
      mat3.fromTranslation(m3, pivotVec);

      const temp = mat3.create();
      mat3.multiply(temp, m2, m1);
      const scaling = mat3.create();
      mat3.multiply(scaling, m3, temp);

      mat3.multiply(shape.transform.matrix, scaling, shape.transform.matrix);

      const decomposed = this.decompose(shape.transform.matrix);
      shape.transform.position = decomposed.position;
      shape.transform.rotation = decomposed.rotation;
      shape.transform.scale = decomposed.scale;
    } else {
      vec2.multiply(shape.transform.scale, shape.transform.scale, vec2.fromValues(sx, sy));
      this.updateMatrix(shape.transform);
    }
  }

  /**
   * Update transform matrix from components
   */
  static updateMatrix(transform: Transform): void {
    mat3.identity(transform.matrix);

    // Apply translation
    mat3.translate(transform.matrix, transform.matrix, transform.position);

    // Apply rotation
    if (transform.rotation !== 0) {
      mat3.rotate(transform.matrix, transform.matrix, transform.rotation);
    }

    // Apply scale
    if (transform.scale[0] !== 1 || transform.scale[1] !== 1) {
      mat3.scale(transform.matrix, transform.matrix, transform.scale);
    }
  }

  /**
   * Transform point
   */
  static transformPoint(point: Point, transform: Transform): Point {
    return transform.apply(point);
  }

  /**
   * Transform multiple points
   */
  static transformPoints(points: Point[], transform: Transform): Point[] {
    return points.map(p => transform.apply(p));
  }

  /**
   * Get inverse transform
   */
  static inverse(transform: Transform): Transform {
    return transform.inverse();
  }

  /**
   * Compose transforms
   */
  static compose(t1: Transform, t2: Transform): Transform {
    return t1.compose(t2);
  }

  /**
   * Set transform from TRS (Translation-Rotation-Scale)
   */
  static fromTRS(
    translation: vec2,
    rotation: number,
    scale: vec2
  ): Transform {
    const transform = this.identity();
    transform.position = vec2.clone(translation);
    transform.rotation = rotation;
    transform.scale = vec2.clone(scale);
    this.updateMatrix(transform);
    return transform;
  }

  /**
   * Apply transform to shape
   */
  static applyToShape(shape: Shape, transform: Transform): void {
    shape.transform = transform.compose(shape.transform);
  }

  /**
   * Reset transform to identity
   */
  static reset(shape: Shape): void {
    shape.transform = this.identity();
  }

  /**
   * Clone transform
   */
  static clone(transform: Transform): Transform {
    return {
      matrix: mat3.clone(transform.matrix),
      position: vec2.clone(transform.position),
      rotation: transform.rotation,
      scale: vec2.clone(transform.scale),
      pivot: vec2.clone(transform.pivot),
      apply: transform.apply,
      inverse: transform.inverse,
      compose: transform.compose,
      decompose: transform.decompose
    };
  }

  /**
   * Interpolate between two transforms
   */
  static lerp(t1: Transform, t2: Transform, alpha: number): Transform {
    const position = vec2.create();
    vec2.lerp(position, t1.position, t2.position, alpha);

    const rotation = t1.rotation + (t2.rotation - t1.rotation) * alpha;

    const scale = vec2.create();
    vec2.lerp(scale, t1.scale, t2.scale, alpha);

    return this.fromTRS(position, rotation, scale);
  }
}
