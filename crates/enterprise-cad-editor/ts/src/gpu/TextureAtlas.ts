/**
 * Texture Atlas - Efficient Texture Management
 * Packs multiple textures into a single atlas for reduced draw calls
 */

import { Texture } from '../types';

interface AtlasNode {
  x: number;
  y: number;
  width: number;
  height: number;
  used: boolean;
  right?: AtlasNode;
  down?: AtlasNode;
}

interface AtlasEntry {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
  uvs: [number, number, number, number]; // [u0, v0, u1, v1]
}

export class TextureAtlas {
  private gl: WebGL2RenderingContext;
  private texture: WebGLTexture;
  private width: number;
  private height: number;
  private root: AtlasNode;
  private entries: Map<string, AtlasEntry>;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;

  constructor(gl: WebGL2RenderingContext, size: number = 2048) {
    this.gl = gl;
    this.width = size;
    this.height = size;
    this.entries = new Map();

    // Create off-screen canvas for texture packing
    this.canvas = document.createElement('canvas');
    this.canvas.width = size;
    this.canvas.height = size;
    const ctx = this.canvas.getContext('2d', { willReadFrequently: true });
    if (!ctx) {
      throw new Error('Failed to get 2D context for texture atlas');
    }
    this.ctx = ctx;

    // Clear to transparent
    this.ctx.clearRect(0, 0, size, size);

    // Initialize bin packing tree
    this.root = {
      x: 0,
      y: 0,
      width: size,
      height: size,
      used: false
    };

    // Create WebGL texture
    const texture = gl.createTexture();
    if (!texture) {
      throw new Error('Failed to create texture atlas');
    }
    this.texture = texture;

    // Initialize texture
    gl.bindTexture(gl.TEXTURE_2D, this.texture);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.RGBA,
      size,
      size,
      0,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      null
    );

    // Set texture parameters
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

    gl.bindTexture(gl.TEXTURE_2D, null);
  }

  /**
   * Add an image to the atlas
   */
  addImage(id: string, image: HTMLImageElement | HTMLCanvasElement): AtlasEntry | null {
    // Check if already added
    if (this.entries.has(id)) {
      return this.entries.get(id)!;
    }

    const width = image.width;
    const height = image.height;

    // Find space in atlas
    const node = this.findNode(this.root, width + 2, height + 2); // Add padding
    if (!node) {
      console.warn(`No space in atlas for image "${id}" (${width}x${height})`);
      return null;
    }

    // Split node
    this.splitNode(node, width + 2, height + 2);

    // Draw image to canvas (with 1px padding)
    this.ctx.drawImage(image, node.x + 1, node.y + 1, width, height);

    // Calculate UV coordinates
    const u0 = (node.x + 1) / this.width;
    const v0 = (node.y + 1) / this.height;
    const u1 = (node.x + 1 + width) / this.width;
    const v1 = (node.y + 1 + height) / this.height;

    const entry: AtlasEntry = {
      id,
      x: node.x + 1,
      y: node.y + 1,
      width,
      height,
      uvs: [u0, v0, u1, v1]
    };

    this.entries.set(id, entry);

    // Update GPU texture
    this.updateTexture(node.x, node.y, width + 2, height + 2);

    return entry;
  }

  /**
   * Find a node that can fit the given size
   */
  private findNode(node: AtlasNode, width: number, height: number): AtlasNode | null {
    if (node.used) {
      // Try right then down
      const right = node.right ? this.findNode(node.right, width, height) : null;
      if (right) return right;
      return node.down ? this.findNode(node.down, width, height) : null;
    } else if (width <= node.width && height <= node.height) {
      return node;
    }
    return null;
  }

  /**
   * Split a node to fit the given size
   */
  private splitNode(node: AtlasNode, width: number, height: number): void {
    node.used = true;

    // Create right and down nodes
    node.right = {
      x: node.x + width,
      y: node.y,
      width: node.width - width,
      height: height,
      used: false
    };

    node.down = {
      x: node.x,
      y: node.y + height,
      width: node.width,
      height: node.height - height,
      used: false
    };
  }

  /**
   * Update GPU texture region
   */
  private updateTexture(x: number, y: number, width: number, height: number): void {
    const gl = this.gl;

    // Get image data from canvas
    const imageData = this.ctx.getImageData(x, y, width, height);

    // Upload to GPU
    gl.bindTexture(gl.TEXTURE_2D, this.texture);
    gl.texSubImage2D(
      gl.TEXTURE_2D,
      0,
      x,
      y,
      width,
      height,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      imageData
    );
    gl.bindTexture(gl.TEXTURE_2D, null);
  }

  /**
   * Get atlas entry by ID
   */
  getEntry(id: string): AtlasEntry | undefined {
    return this.entries.get(id);
  }

  /**
   * Get UV coordinates for an entry
   */
  getUVs(id: string): [number, number, number, number] | null {
    const entry = this.entries.get(id);
    return entry ? entry.uvs : null;
  }

  /**
   * Bind the atlas texture
   */
  bind(unit: number = 0): void {
    const gl = this.gl;
    gl.activeTexture(gl.TEXTURE0 + unit);
    gl.bindTexture(gl.TEXTURE_2D, this.texture);
  }

  /**
   * Get WebGL texture
   */
  getTexture(): WebGLTexture {
    return this.texture;
  }

  /**
   * Get atlas size
   */
  getSize(): { width: number; height: number } {
    return { width: this.width, height: this.height };
  }

  /**
   * Get fill percentage
   */
  getFillPercentage(): number {
    let usedPixels = 0;
    for (const entry of this.entries.values()) {
      usedPixels += entry.width * entry.height;
    }
    return (usedPixels / (this.width * this.height)) * 100;
  }

  /**
   * Clear the atlas
   */
  clear(): void {
    this.entries.clear();
    this.ctx.clearRect(0, 0, this.width, this.height);

    // Reset bin packing tree
    this.root = {
      x: 0,
      y: 0,
      width: this.width,
      height: this.height,
      used: false
    };

    // Clear GPU texture
    const gl = this.gl;
    gl.bindTexture(gl.TEXTURE_2D, this.texture);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.RGBA,
      this.width,
      this.height,
      0,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      null
    );
    gl.bindTexture(gl.TEXTURE_2D, null);
  }

  /**
   * Get debug canvas for visualization
   */
  getDebugCanvas(): HTMLCanvasElement {
    return this.canvas;
  }

  /**
   * Dispose resources
   */
  dispose(): void {
    this.gl.deleteTexture(this.texture);
    this.entries.clear();
  }
}
