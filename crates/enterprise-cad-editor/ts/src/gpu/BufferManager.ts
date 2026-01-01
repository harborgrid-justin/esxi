/**
 * Buffer Manager - GPU Buffer Management
 * Efficient management of vertex and index buffers
 */

import { GPUBuffer } from '../types';

export class BufferManager {
  private gl: WebGL2RenderingContext;
  private buffers: Map<string, GPUBuffer>;
  private activeBuffer: GPUBuffer | null = null;

  constructor(gl: WebGL2RenderingContext) {
    this.gl = gl;
    this.buffers = new Map();
  }

  /**
   * Create a new GPU buffer
   */
  createBuffer(
    id: string,
    data: Float32Array | Uint16Array | Uint32Array,
    target: number = this.gl.ARRAY_BUFFER,
    usage: number = this.gl.STATIC_DRAW
  ): GPUBuffer {
    const gl = this.gl;

    // Delete existing buffer with same ID
    if (this.buffers.has(id)) {
      this.destroyBuffer(id);
    }

    const buffer = gl.createBuffer();
    if (!buffer) {
      throw new Error(`Failed to create buffer "${id}"`);
    }

    gl.bindBuffer(target, buffer);
    gl.bufferData(target, data, usage);
    gl.bindBuffer(target, null);

    const gpuBuffer: GPUBuffer = {
      id,
      buffer,
      target,
      usage,
      size: data.byteLength,
      data,
      bind: (gl: WebGL2RenderingContext) => this.bindBuffer(gpuBuffer),
      update: (gl: WebGL2RenderingContext, data: ArrayBuffer, offset?: number) =>
        this.updateBuffer(gpuBuffer, data, offset),
      destroy: (gl: WebGL2RenderingContext) => this.destroyBuffer(id)
    };

    this.buffers.set(id, gpuBuffer);
    return gpuBuffer;
  }

  /**
   * Create vertex buffer (convenience method)
   */
  createVertexBuffer(id: string, data: Float32Array, usage: number = this.gl.STATIC_DRAW): GPUBuffer {
    return this.createBuffer(id, data, this.gl.ARRAY_BUFFER, usage);
  }

  /**
   * Create index buffer (convenience method)
   */
  createIndexBuffer(
    id: string,
    data: Uint16Array | Uint32Array,
    usage: number = this.gl.STATIC_DRAW
  ): GPUBuffer {
    return this.createBuffer(id, data, this.gl.ELEMENT_ARRAY_BUFFER, usage);
  }

  /**
   * Bind a buffer
   */
  bindBuffer(buffer: GPUBuffer): void {
    if (this.activeBuffer === buffer) {
      return; // Already bound
    }

    this.gl.bindBuffer(buffer.target, buffer.buffer);
    this.activeBuffer = buffer;
  }

  /**
   * Update buffer data
   */
  updateBuffer(buffer: GPUBuffer, data: ArrayBuffer, offset: number = 0): void {
    const gl = this.gl;

    this.bindBuffer(buffer);

    if (offset === 0 && data.byteLength === buffer.size) {
      // Replace entire buffer
      gl.bufferData(buffer.target, data, buffer.usage);
    } else if (offset + data.byteLength <= buffer.size) {
      // Update subset
      gl.bufferSubData(buffer.target, offset, data);
    } else {
      // Need to resize buffer
      buffer.size = offset + data.byteLength;
      gl.bufferData(buffer.target, buffer.size, buffer.usage);
      gl.bufferSubData(buffer.target, offset, data);
    }

    // Update cached data if it's a typed array
    if (data instanceof Float32Array || data instanceof Uint16Array || data instanceof Uint32Array) {
      buffer.data = data;
    }
  }

  /**
   * Get a buffer by ID
   */
  getBuffer(id: string): GPUBuffer | undefined {
    return this.buffers.get(id);
  }

  /**
   * Check if buffer exists
   */
  hasBuffer(id: string): boolean {
    return this.buffers.has(id);
  }

  /**
   * Get all buffer IDs
   */
  getBufferIds(): string[] {
    return Array.from(this.buffers.keys());
  }

  /**
   * Destroy a buffer
   */
  destroyBuffer(id: string): void {
    const buffer = this.buffers.get(id);
    if (!buffer) return;

    if (this.activeBuffer === buffer) {
      this.activeBuffer = null;
      this.gl.bindBuffer(buffer.target, null);
    }

    this.gl.deleteBuffer(buffer.buffer);
    this.buffers.delete(id);
  }

  /**
   * Dispose all buffers
   */
  dispose(): void {
    for (const id of this.buffers.keys()) {
      this.destroyBuffer(id);
    }
    this.buffers.clear();
    this.activeBuffer = null;
  }

  /**
   * Get total memory usage in bytes
   */
  getMemoryUsage(): number {
    let total = 0;
    for (const buffer of this.buffers.values()) {
      total += buffer.size;
    }
    return total;
  }

  /**
   * Get memory usage statistics
   */
  getMemoryStats(): {
    totalBytes: number;
    totalMB: number;
    bufferCount: number;
    vertexBuffers: number;
    indexBuffers: number;
  } {
    let totalBytes = 0;
    let vertexBuffers = 0;
    let indexBuffers = 0;

    for (const buffer of this.buffers.values()) {
      totalBytes += buffer.size;
      if (buffer.target === this.gl.ARRAY_BUFFER) {
        vertexBuffers++;
      } else if (buffer.target === this.gl.ELEMENT_ARRAY_BUFFER) {
        indexBuffers++;
      }
    }

    return {
      totalBytes,
      totalMB: totalBytes / (1024 * 1024),
      bufferCount: this.buffers.size,
      vertexBuffers,
      indexBuffers
    };
  }
}
