/**
 * Shader Manager - GLSL Shader Compilation and Management
 * Handles shader program creation, compilation, and uniform/attribute management
 */

import { Shader } from '../types';

export class ShaderManager {
  private gl: WebGL2RenderingContext;
  private shaders: Map<string, Shader>;
  private activeShader: Shader | null = null;

  constructor(gl: WebGL2RenderingContext) {
    this.gl = gl;
    this.shaders = new Map();
  }

  /**
   * Create and compile a shader program
   */
  createShader(id: string, vertexSource: string, fragmentSource: string): Shader {
    const gl = this.gl;

    // Compile vertex shader
    const vertexShader = this.compileShader(vertexSource, gl.VERTEX_SHADER);
    if (!vertexShader) {
      throw new Error(`Failed to compile vertex shader for "${id}"`);
    }

    // Compile fragment shader
    const fragmentShader = this.compileShader(fragmentSource, gl.FRAGMENT_SHADER);
    if (!fragmentShader) {
      gl.deleteShader(vertexShader);
      throw new Error(`Failed to compile fragment shader for "${id}"`);
    }

    // Link program
    const program = gl.createProgram();
    if (!program) {
      gl.deleteShader(vertexShader);
      gl.deleteShader(fragmentShader);
      throw new Error(`Failed to create shader program for "${id}"`);
    }

    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);

    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      const info = gl.getProgramInfoLog(program);
      gl.deleteProgram(program);
      gl.deleteShader(vertexShader);
      gl.deleteShader(fragmentShader);
      throw new Error(`Failed to link shader program "${id}": ${info}`);
    }

    // Extract uniforms and attributes
    const uniforms = this.extractUniforms(program);
    const attributes = this.extractAttributes(program);

    const shader: Shader = {
      id,
      program,
      vertexShader,
      fragmentShader,
      uniforms,
      attributes,
      use: (gl: WebGL2RenderingContext) => this.useShader(shader),
      setUniform: (gl: WebGL2RenderingContext, name: string, value: any) =>
        this.setUniform(shader, name, value),
      destroy: (gl: WebGL2RenderingContext) => this.destroyShader(id)
    };

    this.shaders.set(id, shader);
    return shader;
  }

  /**
   * Compile a shader
   */
  private compileShader(source: string, type: number): WebGLShader | null {
    const gl = this.gl;
    const shader = gl.createShader(type);

    if (!shader) {
      return null;
    }

    gl.shaderSource(shader, source);
    gl.compileShader(shader);

    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      const info = gl.getShaderInfoLog(shader);
      console.error(`Shader compilation error: ${info}`);
      console.error(`Source:\n${this.addLineNumbers(source)}`);
      gl.deleteShader(shader);
      return null;
    }

    return shader;
  }

  /**
   * Add line numbers to shader source for debugging
   */
  private addLineNumbers(source: string): string {
    return source
      .split('\n')
      .map((line, i) => `${(i + 1).toString().padStart(3, ' ')}: ${line}`)
      .join('\n');
  }

  /**
   * Extract uniform locations from program
   */
  private extractUniforms(program: WebGLProgram): Map<string, WebGLUniformLocation> {
    const gl = this.gl;
    const uniforms = new Map<string, WebGLUniformLocation>();

    const numUniforms = gl.getProgramParameter(program, gl.ACTIVE_UNIFORMS);
    for (let i = 0; i < numUniforms; i++) {
      const info = gl.getActiveUniform(program, i);
      if (!info) continue;

      const location = gl.getUniformLocation(program, info.name);
      if (location) {
        uniforms.set(info.name, location);
      }
    }

    return uniforms;
  }

  /**
   * Extract attribute locations from program
   */
  private extractAttributes(program: WebGLProgram): Map<string, number> {
    const gl = this.gl;
    const attributes = new Map<string, number>();

    const numAttributes = gl.getProgramParameter(program, gl.ACTIVE_ATTRIBUTES);
    for (let i = 0; i < numAttributes; i++) {
      const info = gl.getActiveAttrib(program, i);
      if (!info) continue;

      const location = gl.getAttribLocation(program, info.name);
      if (location >= 0) {
        attributes.set(info.name, location);
      }
    }

    return attributes;
  }

  /**
   * Use a shader program
   */
  useShader(shader: Shader): void {
    if (this.activeShader === shader) {
      return; // Already active
    }

    this.gl.useProgram(shader.program);
    this.activeShader = shader;
  }

  /**
   * Set uniform value
   */
  setUniform(shader: Shader, name: string, value: any): void {
    const location = shader.uniforms.get(name);
    if (!location) {
      console.warn(`Uniform "${name}" not found in shader "${shader.id}"`);
      return;
    }

    const gl = this.gl;

    // Detect type and set uniform
    if (typeof value === 'number') {
      gl.uniform1f(location, value);
    } else if (typeof value === 'boolean') {
      gl.uniform1i(location, value ? 1 : 0);
    } else if (Array.isArray(value)) {
      switch (value.length) {
        case 1:
          gl.uniform1f(location, value[0]);
          break;
        case 2:
          gl.uniform2fv(location, value);
          break;
        case 3:
          gl.uniform3fv(location, value);
          break;
        case 4:
          gl.uniform4fv(location, value);
          break;
        case 9:
          gl.uniformMatrix3fv(location, false, value);
          break;
        case 16:
          gl.uniformMatrix4fv(location, false, value);
          break;
      }
    } else if (value instanceof Float32Array) {
      if (value.length === 9) {
        gl.uniformMatrix3fv(location, false, value);
      } else if (value.length === 16) {
        gl.uniformMatrix4fv(location, false, value);
      }
    }
  }

  /**
   * Get a shader by ID
   */
  getShader(id: string): Shader | undefined {
    return this.shaders.get(id);
  }

  /**
   * Check if shader exists
   */
  hasShader(id: string): boolean {
    return this.shaders.has(id);
  }

  /**
   * Get all shader IDs
   */
  getShaderIds(): string[] {
    return Array.from(this.shaders.keys());
  }

  /**
   * Destroy a shader
   */
  destroyShader(id: string): void {
    const shader = this.shaders.get(id);
    if (!shader) return;

    const gl = this.gl;

    if (this.activeShader === shader) {
      this.activeShader = null;
      gl.useProgram(null);
    }

    gl.deleteProgram(shader.program);
    gl.deleteShader(shader.vertexShader);
    gl.deleteShader(shader.fragmentShader);

    this.shaders.delete(id);
  }

  /**
   * Dispose all shaders
   */
  dispose(): void {
    for (const id of this.shaders.keys()) {
      this.destroyShader(id);
    }
    this.shaders.clear();
    this.activeShader = null;
  }
}
