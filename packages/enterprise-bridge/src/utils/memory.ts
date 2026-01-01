/**
 * Memory management utilities for WASM bridge.
 *
 * Provides helper functions for managing memory transfer between
 * JavaScript and WASM.
 */

import type { MemoryUsage } from '../types';

/**
 * Convert a JavaScript string to a Uint8Array (UTF-8 encoded).
 */
export function stringToBytes(str: string): Uint8Array {
  const encoder = new TextEncoder();
  return encoder.encode(str);
}

/**
 * Convert a Uint8Array to a JavaScript string (UTF-8 decoded).
 */
export function bytesToString(bytes: Uint8Array): string {
  const decoder = new TextDecoder();
  return decoder.decode(bytes);
}

/**
 * Copy a JavaScript array to WASM memory.
 *
 * @param wasmMemory - The WASM memory instance
 * @param data - Data to copy
 * @param ptr - Pointer in WASM memory
 */
export function copyToWasm(
  wasmMemory: WebAssembly.Memory,
  data: Uint8Array,
  ptr: number
): void {
  const memoryArray = new Uint8Array(wasmMemory.buffer);
  memoryArray.set(data, ptr);
}

/**
 * Copy data from WASM memory to JavaScript.
 *
 * @param wasmMemory - The WASM memory instance
 * @param ptr - Pointer in WASM memory
 * @param length - Number of bytes to copy
 */
export function copyFromWasm(
  wasmMemory: WebAssembly.Memory,
  ptr: number,
  length: number
): Uint8Array {
  const memoryArray = new Uint8Array(wasmMemory.buffer);
  return memoryArray.slice(ptr, ptr + length);
}

/**
 * Format bytes as a human-readable string.
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

/**
 * Format memory usage information.
 */
export function formatMemoryUsage(usage: MemoryUsage): string {
  const used = formatBytes(usage.used);
  const total = formatBytes(usage.totalAllocated);
  const available = formatBytes(usage.available);
  const percentage = ((usage.used / usage.totalAllocated) * 100).toFixed(1);

  return `${used} / ${total} (${percentage}%) - ${available} available, ${usage.allocations} allocations`;
}

/**
 * Monitor memory usage and log warnings if usage is high.
 */
export class MemoryMonitor {
  private checkInterval: NodeJS.Timeout | null = null;
  private warningThreshold: number;

  constructor(
    private readonly getUsage: () => Promise<MemoryUsage>,
    warningThresholdPercent = 80
  ) {
    this.warningThreshold = warningThresholdPercent / 100;
  }

  /**
   * Start monitoring memory usage.
   *
   * @param intervalMs - Check interval in milliseconds
   */
  start(intervalMs = 5000): void {
    if (this.checkInterval) {
      return;
    }

    this.checkInterval = setInterval(async () => {
      try {
        const usage = await this.getUsage();
        const usagePercent = usage.used / usage.totalAllocated;

        if (usagePercent >= this.warningThreshold) {
          console.warn(
            `[MemoryMonitor] High memory usage: ${formatMemoryUsage(usage)}`
          );
        }
      } catch (error) {
        console.error('[MemoryMonitor] Error checking memory:', error);
      }
    }, intervalMs);
  }

  /**
   * Stop monitoring memory usage.
   */
  stop(): void {
    if (this.checkInterval) {
      clearInterval(this.checkInterval);
      this.checkInterval = null;
    }
  }

  /**
   * Check if monitoring is active.
   */
  isActive(): boolean {
    return this.checkInterval !== null;
  }
}

/**
 * Allocate a buffer for zero-copy data transfer.
 *
 * This creates a SharedArrayBuffer that can be used for efficient
 * data transfer between JavaScript and WASM.
 */
export function allocateSharedBuffer(size: number): SharedArrayBuffer {
  return new SharedArrayBuffer(size);
}

/**
 * Check if SharedArrayBuffer is supported.
 */
export function isSharedArrayBufferSupported(): boolean {
  return typeof SharedArrayBuffer !== 'undefined';
}

/**
 * Calculate the size needed for a string in UTF-8 encoding.
 */
export function calculateUtf8Size(str: string): number {
  let size = 0;
  for (let i = 0; i < str.length; i++) {
    const code = str.charCodeAt(i);
    if (code < 0x80) {
      size += 1;
    } else if (code < 0x800) {
      size += 2;
    } else if (code < 0x10000) {
      size += 3;
    } else {
      size += 4;
    }
  }
  return size;
}

/**
 * Batch transfer multiple arrays to WASM memory.
 *
 * Returns an array of pointers to the transferred data.
 */
export function batchCopyToWasm(
  wasmMemory: WebAssembly.Memory,
  dataArrays: Uint8Array[],
  startPtr: number
): number[] {
  const pointers: number[] = [];
  let currentPtr = startPtr;

  for (const data of dataArrays) {
    copyToWasm(wasmMemory, data, currentPtr);
    pointers.push(currentPtr);
    currentPtr += data.length;
  }

  return pointers;
}

/**
 * Create a memory view for efficient data access.
 */
export class MemoryView {
  private view: DataView;

  constructor(private readonly memory: WebAssembly.Memory, private offset: number) {
    this.view = new DataView(memory.buffer, offset);
  }

  /**
   * Read a 32-bit unsigned integer.
   */
  readUint32(byteOffset: number): number {
    return this.view.getUint32(byteOffset, true);
  }

  /**
   * Write a 32-bit unsigned integer.
   */
  writeUint32(byteOffset: number, value: number): void {
    this.view.setUint32(byteOffset, value, true);
  }

  /**
   * Read a 64-bit unsigned integer (as bigint).
   */
  readUint64(byteOffset: number): bigint {
    return this.view.getBigUint64(byteOffset, true);
  }

  /**
   * Write a 64-bit unsigned integer (as bigint).
   */
  writeUint64(byteOffset: number, value: bigint): void {
    this.view.setBigUint64(byteOffset, value, true);
  }

  /**
   * Read a 64-bit float.
   */
  readFloat64(byteOffset: number): number {
    return this.view.getFloat64(byteOffset, true);
  }

  /**
   * Write a 64-bit float.
   */
  writeFloat64(byteOffset: number, value: number): void {
    this.view.setFloat64(byteOffset, value, true);
  }

  /**
   * Read bytes into a new Uint8Array.
   */
  readBytes(byteOffset: number, length: number): Uint8Array {
    const buffer = new Uint8Array(this.memory.buffer, this.offset + byteOffset, length);
    return new Uint8Array(buffer);
  }

  /**
   * Write bytes from a Uint8Array.
   */
  writeBytes(byteOffset: number, bytes: Uint8Array): void {
    const buffer = new Uint8Array(this.memory.buffer, this.offset + byteOffset);
    buffer.set(bytes);
  }
}
