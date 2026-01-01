/**
 * Schema Optimizer
 * Optimize data structures using schema-based compression
 */

import { PackSchema, PackField } from '../types';

export class SchemaOptimizer {
  /**
   * Optimize schema by analyzing data patterns
   */
  optimizeSchema(data: any[], currentSchema?: PackSchema): PackSchema {
    const fields = this.analyzeFields(data);
    const optimizedFields = this.optimizeFields(fields);

    return {
      version: currentSchema ? currentSchema.version + 1 : 1,
      fields: optimizedFields,
      checksum: this.calculateChecksum(optimizedFields),
    };
  }

  /**
   * Analyze fields across all data items
   */
  private analyzeFields(data: any[]): Map<string, FieldStats> {
    const stats = new Map<string, FieldStats>();

    for (const item of data) {
      for (const [key, value] of Object.entries(item)) {
        if (!stats.has(key)) {
          stats.set(key, {
            name: key,
            types: new Set(),
            nullCount: 0,
            minValue: Infinity,
            maxValue: -Infinity,
            totalItems: 0,
          });
        }

        const fieldStats = stats.get(key)!;
        fieldStats.totalItems++;

        if (value === null || value === undefined) {
          fieldStats.nullCount++;
        } else {
          fieldStats.types.add(typeof value);
          if (typeof value === 'number') {
            fieldStats.minValue = Math.min(fieldStats.minValue, value);
            fieldStats.maxValue = Math.max(fieldStats.maxValue, value);
          }
        }
      }
    }

    return stats;
  }

  /**
   * Optimize field definitions
   */
  private optimizeFields(stats: Map<string, FieldStats>): PackField[] {
    const fields: PackField[] = [];

    for (const [name, stat] of stats) {
      const field = this.optimizeField(name, stat);
      fields.push(field);
    }

    return fields.sort((a, b) => {
      // Sort required fields first, then by size
      if (a.optional && !b.optional) return 1;
      if (!a.optional && b.optional) return -1;
      return this.getFieldSize(a) - this.getFieldSize(b);
    });
  }

  /**
   * Optimize single field
   */
  private optimizeField(name: string, stats: FieldStats): PackField {
    const field: PackField = { name, type: 'object' };

    // Determine if optional
    field.optional = stats.nullCount > 0;

    // Determine optimal type
    if (stats.types.has('number')) {
      field.type = this.optimizeNumberType(stats);
    } else if (stats.types.has('string')) {
      field.type = 'string';
      field.encoding = 'utf8';
    } else if (stats.types.has('object')) {
      field.type = 'object';
    }

    return field;
  }

  /**
   * Determine optimal number type
   */
  private optimizeNumberType(stats: FieldStats): PackField['type'] {
    const { minValue, maxValue } = stats;
    const isInteger = Number.isInteger(minValue) && Number.isInteger(maxValue);

    if (!isInteger) {
      // Use float32 if range allows, otherwise float64
      if (Math.abs(minValue) < 1e38 && Math.abs(maxValue) < 1e38) {
        return 'float32';
      }
      return 'float64';
    }

    // Optimize integer types
    if (minValue >= 0) {
      if (maxValue <= 255) return 'uint8';
      if (maxValue <= 65535) return 'uint16';
      return 'uint32';
    } else {
      if (minValue >= -128 && maxValue <= 127) return 'int8';
      if (minValue >= -32768 && maxValue <= 32767) return 'int16';
      return 'int32';
    }
  }

  /**
   * Get field size in bytes
   */
  private getFieldSize(field: PackField): number {
    const sizes: Record<string, number> = {
      uint8: 1,
      int8: 1,
      uint16: 2,
      int16: 2,
      uint32: 4,
      int32: 4,
      float32: 4,
      float64: 8,
      string: 100, // Estimate
      buffer: 100, // Estimate
      array: 100, // Estimate
      object: 100, // Estimate
    };

    return sizes[field.type] || 100;
  }

  /**
   * Calculate schema checksum
   */
  private calculateChecksum(fields: PackField[]): string {
    const str = JSON.stringify(fields);
    let hash = 0;

    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash;
    }

    return hash.toString(16);
  }

  /**
   * Compare schemas for compatibility
   */
  areCompatible(schema1: PackSchema, schema2: PackSchema): boolean {
    if (schema1.fields.length !== schema2.fields.length) {
      return false;
    }

    for (let i = 0; i < schema1.fields.length; i++) {
      const field1 = schema1.fields[i];
      const field2 = schema2.fields[i];

      if (field1.name !== field2.name || field1.type !== field2.type) {
        return false;
      }
    }

    return true;
  }

  /**
   * Merge schemas
   */
  mergeSchemas(schemas: PackSchema[]): PackSchema {
    const allFields = new Map<string, PackField>();

    for (const schema of schemas) {
      for (const field of schema.fields) {
        if (!allFields.has(field.name)) {
          allFields.set(field.name, { ...field });
        } else {
          const existing = allFields.get(field.name)!;
          existing.optional = existing.optional || field.optional;
        }
      }
    }

    const fields = Array.from(allFields.values());

    return {
      version: Math.max(...schemas.map(s => s.version)) + 1,
      fields,
      checksum: this.calculateChecksum(fields),
    };
  }
}

interface FieldStats {
  name: string;
  types: Set<string>;
  nullCount: number;
  minValue: number;
  maxValue: number;
  totalItems: number;
}
