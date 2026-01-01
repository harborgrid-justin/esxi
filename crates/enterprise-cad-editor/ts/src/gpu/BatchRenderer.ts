/**
 * Batch Renderer - Instanced Drawing for High Performance
 * Batches similar shapes together and uses instanced rendering
 */

import { mat3 } from 'gl-matrix';
import { Shape, ShapeType, PathShape, RectangleShape, CircleShape } from '../types';
import { ShaderManager } from './ShaderManager';
import { BufferManager } from './BufferManager';

interface BatchGroup {
  shapes: Shape[];
  vertexData: Float32Array;
  indexData: Uint16Array;
  instanceData: Float32Array;
  instanceCount: number;
}

export class BatchRenderer {
  private gl: WebGL2RenderingContext;
  private shaderManager: ShaderManager;
  private bufferManager: BufferManager;

  private batches: Map<string, BatchGroup>;
  private currentBatch: Shape[] = [];
  private maxBatchSize: number = 10000;

  constructor(
    gl: WebGL2RenderingContext,
    shaderManager: ShaderManager,
    bufferManager: BufferManager
  ) {
    this.gl = gl;
    this.shaderManager = shaderManager;
    this.bufferManager = bufferManager;
    this.batches = new Map();

    this.initializeGeometry();
  }

  /**
   * Initialize common geometry (unit shapes for instancing)
   */
  private initializeGeometry(): void {
    // Unit quad (for rectangles, images)
    const quadVertices = new Float32Array([
      0, 0, 0, 0,
      1, 0, 1, 0,
      1, 1, 1, 1,
      0, 1, 0, 1
    ]);
    const quadIndices = new Uint16Array([0, 1, 2, 0, 2, 3]);

    this.bufferManager.createVertexBuffer('quad_vertices', quadVertices);
    this.bufferManager.createIndexBuffer('quad_indices', quadIndices);

    // Unit circle (for circles, ellipses)
    const segments = 64;
    const circleVertices = new Float32Array((segments + 1) * 4); // x, y, u, v
    const circleIndices = new Uint16Array(segments * 3);

    // Center vertex
    circleVertices[0] = 0;
    circleVertices[1] = 0;
    circleVertices[2] = 0.5;
    circleVertices[3] = 0.5;

    // Perimeter vertices
    for (let i = 0; i < segments; i++) {
      const angle = (i / segments) * Math.PI * 2;
      const x = Math.cos(angle);
      const y = Math.sin(angle);

      const offset = (i + 1) * 4;
      circleVertices[offset] = x;
      circleVertices[offset + 1] = y;
      circleVertices[offset + 2] = (x + 1) * 0.5;
      circleVertices[offset + 3] = (y + 1) * 0.5;

      // Triangle indices
      const triOffset = i * 3;
      circleIndices[triOffset] = 0;
      circleIndices[triOffset + 1] = i + 1;
      circleIndices[triOffset + 2] = ((i + 1) % segments) + 1;
    }

    this.bufferManager.createVertexBuffer('circle_vertices', circleVertices);
    this.bufferManager.createIndexBuffer('circle_indices', circleIndices);
  }

  /**
   * Begin a new batch
   */
  beginBatch(): void {
    this.currentBatch = [];
  }

  /**
   * Add a shape to the current batch
   */
  addShape(shape: Shape): void {
    this.currentBatch.push(shape);

    if (this.currentBatch.length >= this.maxBatchSize) {
      this.flush();
    }
  }

  /**
   * End batch and prepare for flushing
   */
  endBatch(): void {
    // Batch is ready to be flushed
  }

  /**
   * Flush all batched shapes
   */
  flush(vpMatrix?: mat3): void {
    if (this.currentBatch.length === 0) return;

    // Group shapes by type for batching
    const groups = this.groupShapesByType(this.currentBatch);

    for (const [type, shapes] of groups.entries()) {
      this.renderBatch(type, shapes, vpMatrix);
    }

    this.currentBatch = [];
  }

  /**
   * Group shapes by type
   */
  private groupShapesByType(shapes: Shape[]): Map<ShapeType, Shape[]> {
    const groups = new Map<ShapeType, Shape[]>();

    for (const shape of shapes) {
      const group = groups.get(shape.type);
      if (group) {
        group.push(shape);
      } else {
        groups.set(shape.type, [shape]);
      }
    }

    return groups;
  }

  /**
   * Render a batch of shapes of the same type
   */
  private renderBatch(type: ShapeType, shapes: Shape[], vpMatrix?: mat3): void {
    switch (type) {
      case ShapeType.Rectangle:
        this.renderRectangles(shapes as RectangleShape[], vpMatrix);
        break;
      case ShapeType.Circle:
        this.renderCircles(shapes as CircleShape[], vpMatrix);
        break;
      case ShapeType.Path:
        this.renderPaths(shapes as PathShape[], vpMatrix);
        break;
      // Add more shape types as needed
      default:
        console.warn(`Unsupported shape type for batching: ${type}`);
    }
  }

  /**
   * Render rectangles using instanced rendering
   */
  private renderRectangles(shapes: RectangleShape[], vpMatrix?: mat3): void {
    const gl = this.gl;
    const shader = this.shaderManager.getShader('instanced');
    if (!shader) return;

    shader.use(gl);

    if (vpMatrix) {
      shader.setUniform(gl, 'u_viewProjection', vpMatrix);
    }

    // Bind quad geometry
    const vertexBuffer = this.bufferManager.getBuffer('quad_vertices');
    const indexBuffer = this.bufferManager.getBuffer('quad_indices');

    if (!vertexBuffer || !indexBuffer) return;

    vertexBuffer.bind(gl);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 16, 0);
    gl.vertexAttribPointer(1, 2, gl.FLOAT, false, 16, 8);
    gl.enableVertexAttribArray(0);
    gl.enableVertexAttribArray(1);

    // Create instance data
    const instanceData = this.createRectangleInstanceData(shapes);
    const instanceBuffer = this.bufferManager.createBuffer(
      'rect_instances',
      instanceData,
      gl.ARRAY_BUFFER,
      gl.DYNAMIC_DRAW
    );

    instanceBuffer.bind(gl);

    // Instance transform matrix (3x3 = 9 floats)
    const stride = 13 * 4; // 9 (matrix) + 4 (color)

    for (let i = 0; i < 3; i++) {
      const location = 2 + i;
      gl.vertexAttribPointer(location, 3, gl.FLOAT, false, stride, i * 3 * 4);
      gl.enableVertexAttribArray(location);
      gl.vertexAttribDivisor(location, 1); // Instance attribute
    }

    // Instance color
    gl.vertexAttribPointer(5, 4, gl.FLOAT, false, stride, 9 * 4);
    gl.enableVertexAttribArray(5);
    gl.vertexAttribDivisor(5, 1);

    // Draw instances
    indexBuffer.bind(gl);
    gl.drawElementsInstanced(gl.TRIANGLES, 6, gl.UNSIGNED_SHORT, 0, shapes.length);

    // Reset divisors
    for (let i = 2; i <= 5; i++) {
      gl.vertexAttribDivisor(i, 0);
      gl.disableVertexAttribArray(i);
    }

    gl.disableVertexAttribArray(0);
    gl.disableVertexAttribArray(1);
  }

  /**
   * Create instance data for rectangles
   */
  private createRectangleInstanceData(shapes: RectangleShape[]): Float32Array {
    const data = new Float32Array(shapes.length * 13); // 9 (matrix) + 4 (color)

    for (let i = 0; i < shapes.length; i++) {
      const shape = shapes[i];
      const offset = i * 13;

      // Create transform matrix
      const transform = mat3.create();
      mat3.fromTranslation(transform, [shape.rect.x, shape.rect.y]);

      if (shape.rect.rotation) {
        mat3.rotate(transform, transform, shape.rect.rotation);
      }

      mat3.scale(transform, transform, [shape.rect.width, shape.rect.height]);

      // Copy matrix (column-major)
      for (let j = 0; j < 9; j++) {
        data[offset + j] = transform[j];
      }

      // Color (parse from style)
      const color = this.parseColor(shape.style.fill || '#000000');
      const opacity = (shape.style.fillOpacity ?? 1) * (shape.style.opacity ?? 1);

      data[offset + 9] = color[0];
      data[offset + 10] = color[1];
      data[offset + 11] = color[2];
      data[offset + 12] = color[3] * opacity;
    }

    return data;
  }

  /**
   * Render circles using instanced rendering
   */
  private renderCircles(shapes: CircleShape[], vpMatrix?: mat3): void {
    const gl = this.gl;
    const shader = this.shaderManager.getShader('instanced');
    if (!shader) return;

    shader.use(gl);

    if (vpMatrix) {
      shader.setUniform(gl, 'u_viewProjection', vpMatrix);
    }

    // Bind circle geometry
    const vertexBuffer = this.bufferManager.getBuffer('circle_vertices');
    const indexBuffer = this.bufferManager.getBuffer('circle_indices');

    if (!vertexBuffer || !indexBuffer) return;

    vertexBuffer.bind(gl);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 16, 0);
    gl.vertexAttribPointer(1, 2, gl.FLOAT, false, 16, 8);
    gl.enableVertexAttribArray(0);
    gl.enableVertexAttribArray(1);

    // Create instance data
    const instanceData = this.createCircleInstanceData(shapes);
    const instanceBuffer = this.bufferManager.createBuffer(
      'circle_instances',
      instanceData,
      gl.ARRAY_BUFFER,
      gl.DYNAMIC_DRAW
    );

    instanceBuffer.bind(gl);

    const stride = 13 * 4;

    for (let i = 0; i < 3; i++) {
      const location = 2 + i;
      gl.vertexAttribPointer(location, 3, gl.FLOAT, false, stride, i * 3 * 4);
      gl.enableVertexAttribArray(location);
      gl.vertexAttribDivisor(location, 1);
    }

    gl.vertexAttribPointer(5, 4, gl.FLOAT, false, stride, 9 * 4);
    gl.enableVertexAttribArray(5);
    gl.vertexAttribDivisor(5, 1);

    // Draw instances
    indexBuffer.bind(gl);
    gl.drawElementsInstanced(gl.TRIANGLES, 64 * 3, gl.UNSIGNED_SHORT, 0, shapes.length);

    // Cleanup
    for (let i = 2; i <= 5; i++) {
      gl.vertexAttribDivisor(i, 0);
      gl.disableVertexAttribArray(i);
    }

    gl.disableVertexAttribArray(0);
    gl.disableVertexAttribArray(1);
  }

  /**
   * Create instance data for circles
   */
  private createCircleInstanceData(shapes: CircleShape[]): Float32Array {
    const data = new Float32Array(shapes.length * 13);

    for (let i = 0; i < shapes.length; i++) {
      const shape = shapes[i];
      const offset = i * 13;

      const transform = mat3.create();
      mat3.fromTranslation(transform, [shape.circle.center.x, shape.circle.center.y]);
      mat3.scale(transform, transform, [shape.circle.radius, shape.circle.radius]);

      for (let j = 0; j < 9; j++) {
        data[offset + j] = transform[j];
      }

      const color = this.parseColor(shape.style.fill || '#000000');
      const opacity = (shape.style.fillOpacity ?? 1) * (shape.style.opacity ?? 1);

      data[offset + 9] = color[0];
      data[offset + 10] = color[1];
      data[offset + 11] = color[2];
      data[offset + 12] = color[3] * opacity;
    }

    return data;
  }

  /**
   * Render paths (basic implementation)
   */
  private renderPaths(shapes: PathShape[], vpMatrix?: mat3): void {
    // Path rendering requires tessellation
    // This is a simplified version - full implementation would use tessellation
    for (const shape of shapes) {
      // Individual path rendering (would be optimized with tessellation)
      console.log('Path rendering not fully implemented in batch mode');
    }
  }

  /**
   * Parse CSS color to RGBA array
   */
  private parseColor(color: string | CanvasGradient | CanvasPattern): [number, number, number, number] {
    if (typeof color !== 'string') {
      return [0, 0, 0, 1];
    }

    // Handle hex colors
    if (color.startsWith('#')) {
      const hex = color.slice(1);
      const r = parseInt(hex.slice(0, 2), 16) / 255;
      const g = parseInt(hex.slice(2, 4), 16) / 255;
      const b = parseInt(hex.slice(4, 6), 16) / 255;
      const a = hex.length === 8 ? parseInt(hex.slice(6, 8), 16) / 255 : 1;
      return [r, g, b, a];
    }

    // Handle rgb/rgba
    const match = color.match(/rgba?\(([^)]+)\)/);
    if (match) {
      const parts = match[1].split(',').map(p => parseFloat(p.trim()));
      return [
        parts[0] / 255,
        parts[1] / 255,
        parts[2] / 255,
        parts[3] ?? 1
      ];
    }

    // Default to black
    return [0, 0, 0, 1];
  }

  /**
   * Dispose resources
   */
  dispose(): void {
    this.batches.clear();
    this.currentBatch = [];
  }
}
