/**
 * Feature Service
 * CRUD operations for spatial features
 */

import {
  Feature,
  FeatureCollection,
  Geometry,
  SpatialQuery,
  Bounds,
} from '../types';
import { TopologyEngine } from '../geometry/TopologyEngine';

export class FeatureService {
  private features: Map<string | number, Feature> = new Map();
  private nextId = 1;

  /**
   * Create a new feature
   */
  create(geometry: Geometry, properties: any = {}): Feature {
    const id = this.nextId++;

    const feature: Feature = {
      type: 'Feature',
      id,
      geometry,
      properties,
    };

    this.features.set(id, feature);

    return feature;
  }

  /**
   * Read feature by ID
   */
  read(id: string | number): Feature | null {
    return this.features.get(id) || null;
  }

  /**
   * Update feature
   */
  update(id: string | number, updates: Partial<Feature>): Feature | null {
    const feature = this.features.get(id);

    if (!feature) {
      return null;
    }

    const updated: Feature = {
      ...feature,
      ...updates,
      id, // Keep original ID
      type: 'Feature', // Keep type
    };

    this.features.set(id, updated);

    return updated;
  }

  /**
   * Delete feature
   */
  delete(id: string | number): boolean {
    return this.features.delete(id);
  }

  /**
   * Get all features
   */
  getAll(): Feature[] {
    return Array.from(this.features.values());
  }

  /**
   * Query features
   */
  query(query: SpatialQuery): Feature[] {
    let results = this.getAll();

    // Filter by geometry if provided
    if (query.geometry && query.spatialRel) {
      results = results.filter((feature) =>
        TopologyEngine.relate(
          feature.geometry,
          query.geometry!,
          query.spatialRel!
        )
      );
    }

    // Filter by WHERE clause
    if (query.where) {
      results = results.filter((feature) =>
        this.evaluateWhere(feature.properties, query.where!)
      );
    }

    // Select fields
    if (query.fields && query.fields.length > 0) {
      results = results.map((feature) => ({
        ...feature,
        properties: this.selectFields(feature.properties, query.fields!),
      }));
    }

    // Optionally remove geometry
    if (query.returnGeometry === false) {
      results = results.map((feature) => ({
        ...feature,
        geometry: { type: 'Point', coordinates: [0, 0] } as any,
      }));
    }

    // Order by
    if (query.orderBy && query.orderBy.length > 0) {
      results = this.orderFeatures(results, query.orderBy);
    }

    // Limit
    if (query.limit) {
      results = results.slice(0, query.limit);
    }

    return results;
  }

  /**
   * Count features matching query
   */
  count(query: SpatialQuery = {}): number {
    return this.query({ ...query, returnGeometry: false }).length;
  }

  /**
   * Get features within bounds
   */
  getInBounds(bounds: Bounds): Feature[] {
    return this.getAll().filter((feature) => {
      const featureBounds = this.getFeatureBounds(feature);
      return this.boundsIntersect(bounds, featureBounds);
    });
  }

  /**
   * Bulk create features
   */
  bulkCreate(features: Array<{ geometry: Geometry; properties?: any }>): Feature[] {
    return features.map((f) => this.create(f.geometry, f.properties));
  }

  /**
   * Bulk update features
   */
  bulkUpdate(updates: Array<{ id: string | number; data: Partial<Feature> }>): Feature[] {
    return updates
      .map((u) => this.update(u.id, u.data))
      .filter((f) => f !== null) as Feature[];
  }

  /**
   * Bulk delete features
   */
  bulkDelete(ids: (string | number)[]): number {
    let deletedCount = 0;
    for (const id of ids) {
      if (this.delete(id)) {
        deletedCount++;
      }
    }
    return deletedCount;
  }

  /**
   * Export to GeoJSON
   */
  toGeoJSON(): FeatureCollection {
    return {
      type: 'FeatureCollection',
      features: this.getAll(),
    };
  }

  /**
   * Import from GeoJSON
   */
  fromGeoJSON(geojson: FeatureCollection): void {
    for (const feature of geojson.features) {
      if (feature.id) {
        this.features.set(feature.id, feature);
        if (typeof feature.id === 'number' && feature.id >= this.nextId) {
          this.nextId = feature.id + 1;
        }
      } else {
        this.create(feature.geometry, feature.properties);
      }
    }
  }

  /**
   * Clear all features
   */
  clear(): void {
    this.features.clear();
    this.nextId = 1;
  }

  /**
   * Evaluate WHERE clause
   */
  private evaluateWhere(properties: any, where: string): boolean {
    // Simplified WHERE clause evaluation
    // Production would use proper SQL parser
    try {
      // Replace property names with actual values
      let expression = where;
      for (const [key, value] of Object.entries(properties)) {
        const safeValue = typeof value === 'string' ? `'${value}'` : value;
        expression = expression.replace(
          new RegExp(`\\b${key}\\b`, 'g'),
          String(safeValue)
        );
      }

      // Evaluate the expression
      return eval(expression);
    } catch (error) {
      console.error('Error evaluating WHERE clause:', error);
      return false;
    }
  }

  /**
   * Select specific fields
   */
  private selectFields(properties: any, fields: string[]): any {
    const selected: any = {};
    for (const field of fields) {
      if (field in properties) {
        selected[field] = properties[field];
      }
    }
    return selected;
  }

  /**
   * Order features
   */
  private orderFeatures(features: Feature[], orderBy: string[]): Feature[] {
    return features.sort((a, b) => {
      for (const field of orderBy) {
        const descending = field.endsWith(' DESC');
        const fieldName = field.replace(/ (ASC|DESC)$/, '');

        const aValue = a.properties[fieldName];
        const bValue = b.properties[fieldName];

        if (aValue < bValue) return descending ? 1 : -1;
        if (aValue > bValue) return descending ? -1 : 1;
      }
      return 0;
    });
  }

  /**
   * Get feature bounds
   */
  private getFeatureBounds(feature: Feature): Bounds {
    // Simplified - would use GeometryFactory
    return {
      minX: -180,
      minY: -90,
      maxX: 180,
      maxY: 90,
    };
  }

  /**
   * Check if bounds intersect
   */
  private boundsIntersect(bounds1: Bounds, bounds2: Bounds): boolean {
    return !(
      bounds1.maxX < bounds2.minX ||
      bounds1.minX > bounds2.maxX ||
      bounds1.maxY < bounds2.minY ||
      bounds1.minY > bounds2.maxY
    );
  }
}
