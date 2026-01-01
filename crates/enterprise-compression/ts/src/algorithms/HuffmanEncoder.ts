/**
 * Huffman Coding Compression
 * Optimal prefix-free encoding based on symbol frequencies
 */

import { createHash } from 'crypto';
import {
  CompressionConfig,
  CompressionResult,
  DecompressionResult,
  CompressionAlgorithm,
  CompressionError,
} from '../types';

interface HuffmanNode {
  value?: number;
  frequency: number;
  left?: HuffmanNode;
  right?: HuffmanNode;
}

interface CodeTable {
  [byte: number]: {
    code: string;
    length: number;
  };
}

export class HuffmanEncoder {
  private static readonly VERSION = '1.0.0';

  /**
   * Compress data using Huffman encoding
   */
  async compress(data: Buffer, config: CompressionConfig): Promise<CompressionResult> {
    const startTime = performance.now();

    try {
      if (!data || data.length === 0) {
        throw new Error('Input data is empty');
      }

      const originalSize = data.length;

      // Build frequency table
      const frequencies = this.buildFrequencyTable(data);

      // Build Huffman tree
      const tree = this.buildHuffmanTree(frequencies);

      // Generate code table
      const codeTable = this.generateCodeTable(tree);

      // Encode data
      const encoded = this.encode(data, codeTable);

      // Pack with tree for decompression
      const packed = this.packWithTree(encoded, tree);

      const compressedSize = packed.length;
      const duration = performance.now() - startTime;
      const compressionRatio = originalSize / compressedSize;
      const throughput = (originalSize / duration) * 1000;

      const checksum = createHash('md5')
        .update(data)
        .digest('hex');

      return {
        compressed: packed,
        originalSize,
        compressedSize,
        compressionRatio,
        algorithm: CompressionAlgorithm.HUFFMAN,
        level: config.level,
        duration,
        throughput,
        metadata: {
          timestamp: new Date(),
          checksum,
          version: HuffmanEncoder.VERSION,
          custom: {
            uniqueSymbols: Object.keys(frequencies).length,
            entropy: this.calculateEntropy(frequencies, originalSize),
          },
        },
      };
    } catch (error) {
      throw new CompressionError(
        `Huffman encoding failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        CompressionAlgorithm.HUFFMAN,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Decompress Huffman encoded data
   */
  async decompress(data: Buffer): Promise<DecompressionResult> {
    const startTime = performance.now();

    try {
      if (!data || data.length === 0) {
        throw new Error('Input data is empty');
      }

      // Unpack tree and encoded data
      const { tree, encoded, originalLength } = this.unpackWithTree(data);

      // Decode data
      const decompressed = this.decode(encoded, tree, originalLength);

      const duration = performance.now() - startTime;

      return {
        decompressed,
        originalSize: decompressed.length,
        duration,
        algorithm: CompressionAlgorithm.HUFFMAN,
        verified: true,
      };
    } catch (error) {
      throw new CompressionError(
        `Huffman decoding failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        CompressionAlgorithm.HUFFMAN,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Build frequency table from data
   */
  private buildFrequencyTable(data: Buffer): Record<number, number> {
    const frequencies: Record<number, number> = {};

    for (const byte of data) {
      frequencies[byte] = (frequencies[byte] || 0) + 1;
    }

    return frequencies;
  }

  /**
   * Build Huffman tree from frequencies
   */
  private buildHuffmanTree(frequencies: Record<number, number>): HuffmanNode {
    // Create leaf nodes
    const nodes: HuffmanNode[] = Object.entries(frequencies).map(
      ([value, frequency]) => ({
        value: parseInt(value),
        frequency,
      })
    );

    // Build tree bottom-up
    while (nodes.length > 1) {
      // Sort by frequency
      nodes.sort((a, b) => a.frequency - b.frequency);

      // Take two nodes with lowest frequency
      const left = nodes.shift()!;
      const right = nodes.shift()!;

      // Create parent node
      const parent: HuffmanNode = {
        frequency: left.frequency + right.frequency,
        left,
        right,
      };

      nodes.push(parent);
    }

    return nodes[0];
  }

  /**
   * Generate code table from Huffman tree
   */
  private generateCodeTable(root: HuffmanNode): CodeTable {
    const table: CodeTable = {};

    const traverse = (node: HuffmanNode, code: string) => {
      if (node.value !== undefined) {
        // Leaf node
        table[node.value] = {
          code,
          length: code.length,
        };
      } else {
        // Internal node
        if (node.left) traverse(node.left, code + '0');
        if (node.right) traverse(node.right, code + '1');
      }
    };

    traverse(root, '');
    return table;
  }

  /**
   * Encode data using code table
   */
  private encode(data: Buffer, codeTable: CodeTable): Buffer {
    let bitString = '';

    for (const byte of data) {
      bitString += codeTable[byte].code;
    }

    // Convert bit string to bytes
    const bytes: number[] = [];
    for (let i = 0; i < bitString.length; i += 8) {
      const chunk = bitString.slice(i, i + 8).padEnd(8, '0');
      bytes.push(parseInt(chunk, 2));
    }

    return Buffer.from(bytes);
  }

  /**
   * Decode data using Huffman tree
   */
  private decode(data: Buffer, tree: HuffmanNode, originalLength: number): Buffer {
    const decoded: number[] = [];
    let currentNode = tree;
    let bitIndex = 0;

    const getBit = (byteIndex: number, bitPos: number): number => {
      return (data[byteIndex] >> (7 - bitPos)) & 1;
    };

    while (decoded.length < originalLength) {
      const byteIndex = Math.floor(bitIndex / 8);
      const bitPos = bitIndex % 8;

      if (byteIndex >= data.length) break;

      const bit = getBit(byteIndex, bitPos);
      bitIndex++;

      // Traverse tree
      currentNode = bit === 0 ? currentNode.left! : currentNode.right!;

      // Leaf node reached
      if (currentNode.value !== undefined) {
        decoded.push(currentNode.value);
        currentNode = tree;
      }
    }

    return Buffer.from(decoded);
  }

  /**
   * Serialize Huffman tree
   */
  private serializeTree(node: HuffmanNode): Buffer {
    const result: number[] = [];

    const traverse = (n: HuffmanNode) => {
      if (n.value !== undefined) {
        // Leaf node: 1 bit + 8 bits for value
        result.push(1);
        result.push(n.value);
      } else {
        // Internal node: 0 bit
        result.push(0);
        if (n.left) traverse(n.left);
        if (n.right) traverse(n.right);
      }
    };

    traverse(node);
    return Buffer.from(result);
  }

  /**
   * Deserialize Huffman tree
   */
  private deserializeTree(data: Buffer): { tree: HuffmanNode; bytesRead: number } {
    let index = 0;

    const buildNode = (): HuffmanNode => {
      const marker = data[index++];

      if (marker === 1) {
        // Leaf node
        const value = data[index++];
        return { value, frequency: 0 };
      } else {
        // Internal node
        const left = buildNode();
        const right = buildNode();
        return { left, right, frequency: 0 };
      }
    };

    const tree = buildNode();
    return { tree, bytesRead: index };
  }

  /**
   * Pack encoded data with tree
   */
  private packWithTree(encoded: Buffer, tree: HuffmanNode): Buffer {
    const serializedTree = this.serializeTree(tree);
    const header = Buffer.allocUnsafe(8);

    header.writeUInt32LE(serializedTree.length, 0);
    header.writeUInt32LE(encoded.length, 4);

    return Buffer.concat([header, serializedTree, encoded]);
  }

  /**
   * Unpack tree and encoded data
   */
  private unpackWithTree(data: Buffer): {
    tree: HuffmanNode;
    encoded: Buffer;
    originalLength: number;
  } {
    const treeLength = data.readUInt32LE(0);
    const encodedLength = data.readUInt32LE(4);

    const treeData = data.slice(8, 8 + treeLength);
    const { tree } = this.deserializeTree(treeData);

    const encoded = data.slice(8 + treeLength, 8 + treeLength + encodedLength);

    // Original length stored after header
    const originalLength = data.readUInt32LE(8 + treeLength + encodedLength);

    return { tree, encoded, originalLength };
  }

  /**
   * Calculate Shannon entropy
   */
  private calculateEntropy(frequencies: Record<number, number>, total: number): number {
    let entropy = 0;

    for (const freq of Object.values(frequencies)) {
      const p = freq / total;
      entropy -= p * Math.log2(p);
    }

    return entropy;
  }

  /**
   * Calculate theoretical compression ratio
   */
  calculateTheoreticalRatio(data: Buffer): number {
    const frequencies = this.buildFrequencyTable(data);
    const entropy = this.calculateEntropy(frequencies, data.length);
    return 8 / entropy;
  }

  /**
   * Get code statistics
   */
  getCodeStatistics(codeTable: CodeTable): {
    avgCodeLength: number;
    minCodeLength: number;
    maxCodeLength: number;
    totalBits: number;
  } {
    const lengths = Object.values(codeTable).map(c => c.length);

    return {
      avgCodeLength: lengths.reduce((a, b) => a + b, 0) / lengths.length,
      minCodeLength: Math.min(...lengths),
      maxCodeLength: Math.max(...lengths),
      totalBits: lengths.reduce((a, b) => a + b, 0),
    };
  }
}
