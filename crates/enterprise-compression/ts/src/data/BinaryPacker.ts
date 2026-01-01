/**
 * Binary Packer
 * Efficient binary serialization for structured data
 */

import { createHash } from 'crypto';
import {
  BinaryPackResult,
  PackSchema,
  PackField,
  OptimizationError,
} from '../types';

export class BinaryPacker {
  /**
   * Pack data to binary format using schema
   */
  async pack(data: any, schema: PackSchema): Promise<BinaryPackResult> {
    try {
      const originalSize = JSON.stringify(data).length;
      const packed = this.packData(data, schema);

      const packedSize = packed.length;
      const savingsPercent = ((originalSize - packedSize) / originalSize) * 100;

      return {
        packed,
        schema,
        originalSize,
        packedSize,
        savingsPercent,
      };
    } catch (error) {
      throw new OptimizationError(
        `Binary packing failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'binary-pack',
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Unpack binary data using schema
   */
  async unpack(packed: Buffer, schema: PackSchema): Promise<any> {
    try {
      return this.unpackData(packed, schema);
    } catch (error) {
      throw new OptimizationError(
        `Binary unpacking failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'binary-unpack',
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Pack data according to schema
   */
  private packData(data: any, schema: PackSchema): Buffer {
    const buffers: Buffer[] = [];

    for (const field of schema.fields) {
      const value = data[field.name];
      const fieldBuffer = this.packField(value, field);
      buffers.push(fieldBuffer);
    }

    return Buffer.concat(buffers);
  }

  /**
   * Pack single field
   */
  private packField(value: any, field: PackField): Buffer {
    if (value === undefined && field.optional) {
      return Buffer.from([0]); // Null marker
    }

    const buffer: Buffer[] = [Buffer.from([1])]; // Present marker

    switch (field.type) {
      case 'uint8':
        buffer.push(this.packUInt8(value));
        break;
      case 'uint16':
        buffer.push(this.packUInt16(value));
        break;
      case 'uint32':
        buffer.push(this.packUInt32(value));
        break;
      case 'int8':
        buffer.push(this.packInt8(value));
        break;
      case 'int16':
        buffer.push(this.packInt16(value));
        break;
      case 'int32':
        buffer.push(this.packInt32(value));
        break;
      case 'float32':
        buffer.push(this.packFloat32(value));
        break;
      case 'float64':
        buffer.push(this.packFloat64(value));
        break;
      case 'string':
        buffer.push(this.packString(value, field.encoding || 'utf8'));
        break;
      case 'buffer':
        buffer.push(this.packBuffer(value));
        break;
      case 'array':
        buffer.push(this.packArray(value));
        break;
      case 'object':
        buffer.push(this.packObject(value));
        break;
    }

    return Buffer.concat(buffer);
  }

  /**
   * Unpack data according to schema
   */
  private unpackData(packed: Buffer, schema: PackSchema): any {
    const result: any = {};
    let offset = 0;

    for (const field of schema.fields) {
      const { value, bytesRead } = this.unpackField(packed, offset, field);
      result[field.name] = value;
      offset += bytesRead;
    }

    return result;
  }

  /**
   * Unpack single field
   */
  private unpackField(
    buffer: Buffer,
    offset: number,
    field: PackField
  ): { value: any; bytesRead: number } {
    const marker = buffer.readUInt8(offset);

    if (marker === 0) {
      return { value: field.default, bytesRead: 1 };
    }

    offset += 1;
    let value: any;
    let bytesRead = 1;

    switch (field.type) {
      case 'uint8':
        value = buffer.readUInt8(offset);
        bytesRead += 1;
        break;
      case 'uint16':
        value = buffer.readUInt16LE(offset);
        bytesRead += 2;
        break;
      case 'uint32':
        value = buffer.readUInt32LE(offset);
        bytesRead += 4;
        break;
      case 'int8':
        value = buffer.readInt8(offset);
        bytesRead += 1;
        break;
      case 'int16':
        value = buffer.readInt16LE(offset);
        bytesRead += 2;
        break;
      case 'int32':
        value = buffer.readInt32LE(offset);
        bytesRead += 4;
        break;
      case 'float32':
        value = buffer.readFloatLE(offset);
        bytesRead += 4;
        break;
      case 'float64':
        value = buffer.readDoubleLE(offset);
        bytesRead += 8;
        break;
      case 'string':
        const strResult = this.unpackString(buffer, offset, field.encoding || 'utf8');
        value = strResult.value;
        bytesRead += strResult.bytesRead;
        break;
      case 'buffer':
        const bufResult = this.unpackBuffer(buffer, offset);
        value = bufResult.value;
        bytesRead += bufResult.bytesRead;
        break;
      case 'array':
        const arrResult = this.unpackArray(buffer, offset);
        value = arrResult.value;
        bytesRead += arrResult.bytesRead;
        break;
      case 'object':
        const objResult = this.unpackObject(buffer, offset);
        value = objResult.value;
        bytesRead += objResult.bytesRead;
        break;
    }

    return { value, bytesRead };
  }

  // Pack helpers
  private packUInt8(value: number): Buffer {
    const buf = Buffer.allocUnsafe(1);
    buf.writeUInt8(value, 0);
    return buf;
  }

  private packUInt16(value: number): Buffer {
    const buf = Buffer.allocUnsafe(2);
    buf.writeUInt16LE(value, 0);
    return buf;
  }

  private packUInt32(value: number): Buffer {
    const buf = Buffer.allocUnsafe(4);
    buf.writeUInt32LE(value, 0);
    return buf;
  }

  private packInt8(value: number): Buffer {
    const buf = Buffer.allocUnsafe(1);
    buf.writeInt8(value, 0);
    return buf;
  }

  private packInt16(value: number): Buffer {
    const buf = Buffer.allocUnsafe(2);
    buf.writeInt16LE(value, 0);
    return buf;
  }

  private packInt32(value: number): Buffer {
    const buf = Buffer.allocUnsafe(4);
    buf.writeInt32LE(value, 0);
    return buf;
  }

  private packFloat32(value: number): Buffer {
    const buf = Buffer.allocUnsafe(4);
    buf.writeFloatLE(value, 0);
    return buf;
  }

  private packFloat64(value: number): Buffer {
    const buf = Buffer.allocUnsafe(8);
    buf.writeDoubleLE(value, 0);
    return buf;
  }

  private packString(value: string, encoding: 'utf8' | 'ascii' | 'base64'): Buffer {
    const strBuf = Buffer.from(value, encoding);
    const lenBuf = Buffer.allocUnsafe(4);
    lenBuf.writeUInt32LE(strBuf.length, 0);
    return Buffer.concat([lenBuf, strBuf]);
  }

  private packBuffer(value: Buffer): Buffer {
    const lenBuf = Buffer.allocUnsafe(4);
    lenBuf.writeUInt32LE(value.length, 0);
    return Buffer.concat([lenBuf, value]);
  }

  private packArray(value: any[]): Buffer {
    const jsonStr = JSON.stringify(value);
    return this.packString(jsonStr, 'utf8');
  }

  private packObject(value: any): Buffer {
    const jsonStr = JSON.stringify(value);
    return this.packString(jsonStr, 'utf8');
  }

  // Unpack helpers
  private unpackString(
    buffer: Buffer,
    offset: number,
    encoding: 'utf8' | 'ascii' | 'base64'
  ): { value: string; bytesRead: number } {
    const length = buffer.readUInt32LE(offset);
    const value = buffer.toString(encoding, offset + 4, offset + 4 + length);
    return { value, bytesRead: 4 + length };
  }

  private unpackBuffer(
    buffer: Buffer,
    offset: number
  ): { value: Buffer; bytesRead: number } {
    const length = buffer.readUInt32LE(offset);
    const value = buffer.slice(offset + 4, offset + 4 + length);
    return { value, bytesRead: 4 + length };
  }

  private unpackArray(
    buffer: Buffer,
    offset: number
  ): { value: any[]; bytesRead: number } {
    const strResult = this.unpackString(buffer, offset, 'utf8');
    return { value: JSON.parse(strResult.value), bytesRead: strResult.bytesRead };
  }

  private unpackObject(
    buffer: Buffer,
    offset: number
  ): { value: any; bytesRead: number } {
    const strResult = this.unpackString(buffer, offset, 'utf8');
    return { value: JSON.parse(strResult.value), bytesRead: strResult.bytesRead };
  }

  /**
   * Generate schema from data
   */
  generateSchema(data: any, name: string = 'auto-schema'): PackSchema {
    const fields: PackField[] = [];

    for (const [key, value] of Object.entries(data)) {
      fields.push(this.inferField(key, value));
    }

    const checksum = createHash('md5')
      .update(JSON.stringify(fields))
      .digest('hex');

    return {
      version: 1,
      fields,
      checksum,
    };
  }

  /**
   * Infer field type from value
   */
  private inferField(name: string, value: any): PackField {
    const field: PackField = { name, type: 'object' };

    if (typeof value === 'number') {
      if (Number.isInteger(value)) {
        if (value >= 0 && value <= 255) field.type = 'uint8';
        else if (value >= 0 && value <= 65535) field.type = 'uint16';
        else if (value >= 0) field.type = 'uint32';
        else if (value >= -128 && value <= 127) field.type = 'int8';
        else if (value >= -32768 && value <= 32767) field.type = 'int16';
        else field.type = 'int32';
      } else {
        field.type = 'float64';
      }
    } else if (typeof value === 'string') {
      field.type = 'string';
      field.encoding = 'utf8';
    } else if (Buffer.isBuffer(value)) {
      field.type = 'buffer';
    } else if (Array.isArray(value)) {
      field.type = 'array';
    } else if (typeof value === 'object') {
      field.type = 'object';
    }

    return field;
  }
}
