/**
 * Tessellation - Polygon Triangulation
 * Convert polygons to triangles for GPU rendering
 */

import { Point } from '../types';

export class Tessellation {
  /**
   * Triangulate polygon using ear clipping algorithm
   */
  static earClipping(vertices: Point[]): number[] {
    if (vertices.length < 3) return [];

    const indices: number[] = [];
    const n = vertices.length;

    // Create index list
    const indexList: number[] = [];
    for (let i = 0; i < n; i++) {
      indexList.push(i);
    }

    let count = 2 * n;

    for (let v = n - 1; indexList.length > 2; ) {
      if (--count <= 0) {
        // Probably bad polygon
        break;
      }

      // Three consecutive vertices
      const u = v;
      if (indexList.length <= u) v = 0;
      v = u + 1;
      if (indexList.length <= v) v = 0;
      let w = v + 1;
      if (indexList.length <= w) w = 0;

      const a = vertices[indexList[u]];
      const b = vertices[indexList[v]];
      const c = vertices[indexList[w]];

      if (this.isEar(a, b, c, vertices, indexList)) {
        // Output triangle
        indices.push(indexList[u], indexList[v], indexList[w]);

        // Remove v from list
        indexList.splice(v, 1);

        count = 2 * indexList.length;
      }
    }

    return indices;
  }

  /**
   * Check if three vertices form an ear
   */
  private static isEar(a: Point, b: Point, c: Point, vertices: Point[], indexList: number[]): boolean {
    // Check if triangle is counter-clockwise
    if (this.area(a, b, c) < 0) return false;

    // Check if any other vertex is inside the triangle
    for (const index of indexList) {
      const p = vertices[index];
      if (p === a || p === b || p === c) continue;

      if (this.isPointInTriangle(p, a, b, c)) {
        return false;
      }
    }

    return true;
  }

  /**
   * Calculate signed area of triangle
   */
  private static area(a: Point, b: Point, c: Point): number {
    return (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y);
  }

  /**
   * Check if point is inside triangle
   */
  private static isPointInTriangle(p: Point, a: Point, b: Point, c: Point): boolean {
    const area1 = this.area(p, a, b);
    const area2 = this.area(p, b, c);
    const area3 = this.area(p, c, a);

    const hasNeg = area1 < 0 || area2 < 0 || area3 < 0;
    const hasPos = area1 > 0 || area2 > 0 || area3 > 0;

    return !(hasNeg && hasPos);
  }

  /**
   * Triangulate using Delaunay triangulation (simplified)
   */
  static delaunay(points: Point[]): number[] {
    if (points.length < 3) return [];

    // Simplified Delaunay - use ear clipping for now
    // Full implementation would use incremental insertion or divide-and-conquer
    return this.earClipping(points);
  }

  /**
   * Triangulate polygon with holes
   */
  static triangulateWithHoles(
    outer: Point[],
    holes: Point[][]
  ): number[] {
    // Combine outer and holes into a single polygon
    // This is a simplified approach - full implementation would use bridge edges

    const allVertices = [...outer];
    const indices: number[] = [];

    // Add holes (simplified - just concatenate)
    for (const hole of holes) {
      allVertices.push(...hole);
    }

    // Triangulate combined polygon
    return this.earClipping(allVertices);
  }

  /**
   * Generate triangle strip from polygon
   */
  static toTriangleStrip(vertices: Point[]): number[] {
    if (vertices.length < 3) return [];

    const strip: number[] = [];
    const n = vertices.length;

    // Simple triangle strip generation
    for (let i = 0; i < n - 2; i++) {
      if (i % 2 === 0) {
        strip.push(i, i + 1, i + 2);
      } else {
        strip.push(i + 1, i, i + 2);
      }
    }

    return strip;
  }

  /**
   * Generate triangle fan from polygon
   */
  static toTriangleFan(vertices: Point[]): number[] {
    if (vertices.length < 3) return [];

    const fan: number[] = [];
    const n = vertices.length;

    // All triangles share the first vertex
    for (let i = 1; i < n - 1; i++) {
      fan.push(0, i, i + 1);
    }

    return fan;
  }

  /**
   * Subdivide triangles for smoother rendering
   */
  static subdivide(vertices: Point[], indices: number[], iterations: number = 1): {
    vertices: Point[];
    indices: number[];
  } {
    let currentVertices = [...vertices];
    let currentIndices = [...indices];

    for (let iter = 0; iter < iterations; iter++) {
      const newVertices = [...currentVertices];
      const newIndices: number[] = [];
      const edgeMap = new Map<string, number>();

      const getEdgeMidpoint = (i1: number, i2: number): number => {
        const key = i1 < i2 ? `${i1},${i2}` : `${i2},${i1}`;

        if (edgeMap.has(key)) {
          return edgeMap.get(key)!;
        }

        const v1 = currentVertices[i1];
        const v2 = currentVertices[i2];
        const midpoint = {
          x: (v1.x + v2.x) / 2,
          y: (v1.y + v2.y) / 2
        };

        const index = newVertices.length;
        newVertices.push(midpoint);
        edgeMap.set(key, index);

        return index;
      };

      // Subdivide each triangle
      for (let i = 0; i < currentIndices.length; i += 3) {
        const i0 = currentIndices[i];
        const i1 = currentIndices[i + 1];
        const i2 = currentIndices[i + 2];

        const m01 = getEdgeMidpoint(i0, i1);
        const m12 = getEdgeMidpoint(i1, i2);
        const m20 = getEdgeMidpoint(i2, i0);

        // Create 4 new triangles
        newIndices.push(i0, m01, m20);
        newIndices.push(i1, m12, m01);
        newIndices.push(i2, m20, m12);
        newIndices.push(m01, m12, m20);
      }

      currentVertices = newVertices;
      currentIndices = newIndices;
    }

    return { vertices: currentVertices, indices: currentIndices };
  }

  /**
   * Calculate triangle quality (aspect ratio)
   */
  static triangleQuality(a: Point, b: Point, c: Point): number {
    const ab = Math.sqrt((b.x - a.x) ** 2 + (b.y - a.y) ** 2);
    const bc = Math.sqrt((c.x - b.x) ** 2 + (c.y - b.y) ** 2);
    const ca = Math.sqrt((a.x - c.x) ** 2 + (a.y - c.y) ** 2);

    const s = (ab + bc + ca) / 2; // Semi-perimeter
    const area = Math.sqrt(s * (s - ab) * (s - bc) * (s - ca)); // Heron's formula

    const longestEdge = Math.max(ab, bc, ca);

    // Quality metric: ratio of inscribed circle radius to longest edge
    const inRadius = area / s;
    return (2 * inRadius) / longestEdge;
  }

  /**
   * Optimize mesh by flipping edges
   */
  static optimizeMesh(vertices: Point[], indices: number[]): number[] {
    const optimized = [...indices];
    let improved = true;
    let iterations = 0;
    const maxIterations = 10;

    while (improved && iterations < maxIterations) {
      improved = false;
      iterations++;

      for (let i = 0; i < optimized.length; i += 3) {
        // Check each edge and consider flipping
        // This is a simplified version
        const quality = this.triangleQuality(
          vertices[optimized[i]],
          vertices[optimized[i + 1]],
          vertices[optimized[i + 2]]
        );

        // If quality is poor, might want to flip edge
        if (quality < 0.5) {
          // Edge flipping logic would go here
        }
      }
    }

    return optimized;
  }
}
