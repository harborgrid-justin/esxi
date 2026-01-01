/**
 * WebGL2 Renderer for GPU-Accelerated CAD Rendering
 * High-performance rendering engine with instanced drawing and batching
 */

import { mat3, vec2 } from 'gl-matrix';
import { CADDocument, RenderBatch, Shape, Viewport } from '../types';
import { ShaderManager } from './ShaderManager';
import { BufferManager } from './BufferManager';
import { TextureAtlas } from './TextureAtlas';
import { BatchRenderer } from './BatchRenderer';

export interface WebGLRendererOptions {
  antialias?: boolean;
  alpha?: boolean;
  preserveDrawingBuffer?: boolean;
  powerPreference?: 'default' | 'high-performance' | 'low-power';
  maxTextureSize?: number;
  maxBatchSize?: number;
}

export class WebGLRenderer {
  private gl: WebGL2RenderingContext;
  private canvas: HTMLCanvasElement;
  private shaderManager: ShaderManager;
  private bufferManager: BufferManager;
  private textureAtlas: TextureAtlas;
  private batchRenderer: BatchRenderer;

  private viewMatrix: mat3;
  private projectionMatrix: mat3;
  private vpMatrix: mat3;

  private options: Required<WebGLRendererOptions>;
  private frameCount: number = 0;
  private lastFrameTime: number = 0;
  private fps: number = 0;

  constructor(canvas: HTMLCanvasElement, options: WebGLRendererOptions = {}) {
    this.canvas = canvas;

    const gl = canvas.getContext('webgl2', {
      antialias: options.antialias ?? true,
      alpha: options.alpha ?? true,
      preserveDrawingBuffer: options.preserveDrawingBuffer ?? false,
      powerPreference: options.powerPreference ?? 'high-performance'
    });

    if (!gl) {
      throw new Error('WebGL2 not supported');
    }

    this.gl = gl;
    this.options = {
      antialias: options.antialias ?? true,
      alpha: options.alpha ?? true,
      preserveDrawingBuffer: options.preserveDrawingBuffer ?? false,
      powerPreference: options.powerPreference ?? 'high-performance',
      maxTextureSize: options.maxTextureSize ?? gl.getParameter(gl.MAX_TEXTURE_SIZE),
      maxBatchSize: options.maxBatchSize ?? 10000
    };

    this.shaderManager = new ShaderManager(gl);
    this.bufferManager = new BufferManager(gl);
    this.textureAtlas = new TextureAtlas(gl, this.options.maxTextureSize);
    this.batchRenderer = new BatchRenderer(gl, this.shaderManager, this.bufferManager);

    this.viewMatrix = mat3.create();
    this.projectionMatrix = mat3.create();
    this.vpMatrix = mat3.create();

    this.initialize();
  }

  /**
   * Initialize WebGL state and compile shaders
   */
  private initialize(): void {
    const gl = this.gl;

    // Enable blending for transparency
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

    // Enable depth testing
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.LEQUAL);

    // Enable multisampling if available
    if (this.options.antialias) {
      const samples = gl.getParameter(gl.SAMPLES);
      console.log(`MSAA enabled with ${samples} samples`);
    }

    // Compile default shaders
    this.compileDefaultShaders();

    // Set initial clear color
    gl.clearColor(1.0, 1.0, 1.0, 1.0);
  }

  /**
   * Compile default shader programs
   */
  private compileDefaultShaders(): void {
    // Basic shape shader
    this.shaderManager.createShader('basic',
      // Vertex shader
      `#version 300 es
      precision highp float;

      layout(location = 0) in vec2 a_position;
      layout(location = 1) in vec4 a_color;
      layout(location = 2) in vec2 a_texCoord;

      uniform mat3 u_viewProjection;
      uniform mat3 u_model;

      out vec4 v_color;
      out vec2 v_texCoord;

      void main() {
        vec3 position = u_viewProjection * u_model * vec3(a_position, 1.0);
        gl_Position = vec4(position.xy, 0.0, 1.0);
        v_color = a_color;
        v_texCoord = a_texCoord;
      }`,

      // Fragment shader
      `#version 300 es
      precision highp float;

      in vec4 v_color;
      in vec2 v_texCoord;

      uniform sampler2D u_texture;
      uniform bool u_useTexture;

      out vec4 fragColor;

      void main() {
        if (u_useTexture) {
          fragColor = texture(u_texture, v_texCoord) * v_color;
        } else {
          fragColor = v_color;
        }
      }`
    );

    // Instanced shape shader for batch rendering
    this.shaderManager.createShader('instanced',
      `#version 300 es
      precision highp float;

      layout(location = 0) in vec2 a_position;
      layout(location = 1) in vec2 a_texCoord;
      layout(location = 2) in mat3 a_instanceTransform;
      layout(location = 5) in vec4 a_instanceColor;

      uniform mat3 u_viewProjection;

      out vec4 v_color;
      out vec2 v_texCoord;

      void main() {
        vec3 position = u_viewProjection * a_instanceTransform * vec3(a_position, 1.0);
        gl_Position = vec4(position.xy, 0.0, 1.0);
        v_color = a_instanceColor;
        v_texCoord = a_texCoord;
      }`,

      `#version 300 es
      precision highp float;

      in vec4 v_color;
      in vec2 v_texCoord;

      out vec4 fragColor;

      void main() {
        fragColor = v_color;
      }`
    );

    // Line shader with antialiasing
    this.shaderManager.createShader('line',
      `#version 300 es
      precision highp float;

      layout(location = 0) in vec2 a_position;
      layout(location = 1) in vec2 a_normal;
      layout(location = 2) in vec4 a_color;
      layout(location = 3) in float a_thickness;

      uniform mat3 u_viewProjection;
      uniform float u_pixelRatio;

      out vec4 v_color;
      out float v_distance;

      void main() {
        vec3 position = u_viewProjection * vec3(a_position, 1.0);
        vec2 offset = a_normal * a_thickness * 0.5 / u_pixelRatio;
        gl_Position = vec4(position.xy + offset, 0.0, 1.0);
        v_color = a_color;
        v_distance = length(a_normal);
      }`,

      `#version 300 es
      precision highp float;

      in vec4 v_color;
      in float v_distance;

      out vec4 fragColor;

      void main() {
        // Antialiased edge
        float alpha = 1.0 - smoothstep(0.0, 1.0, abs(v_distance));
        fragColor = vec4(v_color.rgb, v_color.a * alpha);
      }`
    );

    // Grid shader
    this.shaderManager.createShader('grid',
      `#version 300 es
      precision highp float;

      layout(location = 0) in vec2 a_position;

      uniform mat3 u_viewProjection;

      out vec2 v_worldPos;

      void main() {
        vec3 position = u_viewProjection * vec3(a_position, 1.0);
        gl_Position = vec4(position.xy, 0.0, 1.0);
        v_worldPos = a_position;
      }`,

      `#version 300 es
      precision highp float;

      in vec2 v_worldPos;

      uniform float u_gridSize;
      uniform vec4 u_gridColor;
      uniform vec4 u_majorGridColor;
      uniform float u_majorGridInterval;

      out vec4 fragColor;

      void main() {
        vec2 coord = v_worldPos / u_gridSize;
        vec2 grid = abs(fract(coord - 0.5) - 0.5) / fwidth(coord);
        float line = min(grid.x, grid.y);

        // Major grid lines
        vec2 majorCoord = v_worldPos / (u_gridSize * u_majorGridInterval);
        vec2 majorGrid = abs(fract(majorCoord - 0.5) - 0.5) / fwidth(majorCoord);
        float majorLine = min(majorGrid.x, majorGrid.y);

        float alpha = 1.0 - min(line, 1.0);
        float majorAlpha = 1.0 - min(majorLine, 1.0);

        vec4 color = mix(u_gridColor, u_majorGridColor, majorAlpha);
        fragColor = vec4(color.rgb, max(alpha, majorAlpha) * color.a);
      }`
    );
  }

  /**
   * Resize the canvas and update viewport
   */
  resize(width: number, height: number, pixelRatio: number = window.devicePixelRatio): void {
    this.canvas.width = width * pixelRatio;
    this.canvas.height = height * pixelRatio;
    this.canvas.style.width = `${width}px`;
    this.canvas.style.height = `${height}px`;

    this.gl.viewport(0, 0, this.canvas.width, this.canvas.height);

    // Update projection matrix for orthographic projection
    this.updateProjectionMatrix(width, height);
  }

  /**
   * Update projection matrix
   */
  private updateProjectionMatrix(width: number, height: number): void {
    const left = -width / 2;
    const right = width / 2;
    const bottom = -height / 2;
    const top = height / 2;

    mat3.identity(this.projectionMatrix);
    this.projectionMatrix[0] = 2 / (right - left);
    this.projectionMatrix[4] = 2 / (top - bottom);
    this.projectionMatrix[6] = -(right + left) / (right - left);
    this.projectionMatrix[7] = -(top + bottom) / (top - bottom);
  }

  /**
   * Update view matrix from viewport
   */
  updateViewMatrix(viewport: Viewport): void {
    mat3.identity(this.viewMatrix);

    // Apply zoom
    mat3.scale(this.viewMatrix, this.viewMatrix, [viewport.zoom, viewport.zoom]);

    // Apply rotation
    if (viewport.rotation !== 0) {
      mat3.rotate(this.viewMatrix, this.viewMatrix, viewport.rotation);
    }

    // Apply translation
    mat3.translate(this.viewMatrix, this.viewMatrix, [-viewport.center.x, -viewport.center.y]);

    // Combine view and projection
    mat3.multiply(this.vpMatrix, this.projectionMatrix, this.viewMatrix);
  }

  /**
   * Clear the canvas
   */
  clear(r: number = 1, g: number = 1, b: number = 1, a: number = 1): void {
    this.gl.clearColor(r, g, b, a);
    this.gl.clear(this.gl.COLOR_BUFFER_BIT | this.gl.DEPTH_BUFFER_BIT);
  }

  /**
   * Render a CAD document
   */
  render(document: CADDocument): void {
    const startTime = performance.now();

    this.clear();
    this.updateViewMatrix(document.viewport);

    // Render in layer order
    const layers = Array.from(document.layers.values())
      .sort((a, b) => a.order - b.order);

    for (const layer of layers) {
      if (!layer.visible) continue;

      const shapes = document.getShapesInLayer(layer.id);
      this.batchRenderer.beginBatch();

      for (const shape of shapes) {
        if (!shape.visible) continue;
        this.renderShape(shape);
      }

      this.batchRenderer.endBatch();
      this.batchRenderer.flush(this.vpMatrix);
    }

    // Update FPS counter
    const frameTime = performance.now() - startTime;
    this.frameCount++;
    if (startTime - this.lastFrameTime > 1000) {
      this.fps = this.frameCount / ((startTime - this.lastFrameTime) / 1000);
      this.frameCount = 0;
      this.lastFrameTime = startTime;
    }
  }

  /**
   * Render a single shape
   */
  private renderShape(shape: Shape): void {
    this.batchRenderer.addShape(shape);
  }

  /**
   * Render grid
   */
  renderGrid(viewport: Viewport, gridSize: number, majorInterval: number = 5): void {
    const shader = this.shaderManager.getShader('grid');
    if (!shader) return;

    shader.use(this.gl);
    shader.setUniform(this.gl, 'u_viewProjection', this.vpMatrix);
    shader.setUniform(this.gl, 'u_gridSize', gridSize);
    shader.setUniform(this.gl, 'u_gridColor', [0.9, 0.9, 0.9, 0.5]);
    shader.setUniform(this.gl, 'u_majorGridColor', [0.7, 0.7, 0.7, 0.8]);
    shader.setUniform(this.gl, 'u_majorGridInterval', majorInterval);

    // Create fullscreen quad in world space
    const bounds = viewport.getViewBounds();
    const vertices = new Float32Array([
      bounds.minX, bounds.minY,
      bounds.maxX, bounds.minY,
      bounds.maxX, bounds.maxY,
      bounds.minX, bounds.maxY
    ]);

    const buffer = this.bufferManager.createBuffer('grid_quad', vertices, this.gl.ARRAY_BUFFER);
    buffer.bind(this.gl);

    this.gl.vertexAttribPointer(0, 2, this.gl.FLOAT, false, 0, 0);
    this.gl.enableVertexAttribArray(0);

    this.gl.drawArrays(this.gl.TRIANGLE_FAN, 0, 4);

    this.gl.disableVertexAttribArray(0);
  }

  /**
   * Get current FPS
   */
  getFPS(): number {
    return this.fps;
  }

  /**
   * Get WebGL context
   */
  getContext(): WebGL2RenderingContext {
    return this.gl;
  }

  /**
   * Get shader manager
   */
  getShaderManager(): ShaderManager {
    return this.shaderManager;
  }

  /**
   * Get buffer manager
   */
  getBufferManager(): BufferManager {
    return this.bufferManager;
  }

  /**
   * Get texture atlas
   */
  getTextureAtlas(): TextureAtlas {
    return this.textureAtlas;
  }

  /**
   * Clean up resources
   */
  dispose(): void {
    this.batchRenderer.dispose();
    this.bufferManager.dispose();
    this.textureAtlas.dispose();
    this.shaderManager.dispose();
  }
}
