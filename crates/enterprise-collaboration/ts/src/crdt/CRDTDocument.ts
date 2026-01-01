/**
 * CRDT Document Implementation
 * Conflict-free Replicated Data Type for collaborative document editing
 */

import { Operation, OperationType, DocumentState, DocumentMetadata } from '../types';
import { VectorClock, ClockComparison } from './VectorClock';
import { OperationalTransform } from './OperationalTransform';

export interface CRDTNode {
  id: string;
  participantId: string;
  value: string | unknown;
  timestamp: Date;
  vectorClock: VectorClock;
  deleted: boolean;
  prev?: string;
  next?: string;
}

export interface CRDTConfig {
  nodeId: string;
  preserveHistory?: boolean;
  maxHistorySize?: number;
  autoGC?: boolean;
}

export class CRDTDocument {
  private nodeId: string;
  private content: Map<string, CRDTNode>;
  private head: string | null;
  private tail: string | null;
  private vectorClock: VectorClock;
  private operationHistory: Operation[];
  private config: Required<CRDTConfig>;

  constructor(config: CRDTConfig) {
    this.nodeId = config.nodeId;
    this.content = new Map();
    this.head = null;
    this.tail = null;
    this.vectorClock = VectorClock.create(config.nodeId);
    this.operationHistory = [];
    this.config = {
      nodeId: config.nodeId,
      preserveHistory: config.preserveHistory ?? true,
      maxHistorySize: config.maxHistorySize ?? 1000,
      autoGC: config.autoGC ?? true,
    };
  }

  /**
   * Insert content at a specific position
   */
  insert(content: string, position: number, participantId?: string): Operation {
    const id = this.generateNodeId();
    const pid = participantId || this.nodeId;

    this.vectorClock.increment(this.nodeId);

    const operation: Operation = {
      id,
      type: OperationType.INSERT,
      position: { line: 0, column: position, offset: position },
      content,
      participantId: pid,
      timestamp: new Date(),
      vectorClock: this.vectorClock.toObject(),
    };

    this.applyOperation(operation);

    if (this.config.preserveHistory) {
      this.addToHistory(operation);
    }

    return operation;
  }

  /**
   * Delete content at a specific position
   */
  delete(position: number, length: number, participantId?: string): Operation {
    const id = this.generateNodeId();
    const pid = participantId || this.nodeId;

    this.vectorClock.increment(this.nodeId);

    const operation: Operation = {
      id,
      type: OperationType.DELETE,
      position: { line: 0, column: position, offset: position },
      length,
      participantId: pid,
      timestamp: new Date(),
      vectorClock: this.vectorClock.toObject(),
    };

    this.applyOperation(operation);

    if (this.config.preserveHistory) {
      this.addToHistory(operation);
    }

    return operation;
  }

  /**
   * Apply a remote operation to this document
   */
  applyRemoteOperation(operation: Operation): void {
    const remoteVectorClock = new VectorClock(operation.vectorClock);

    // Update our vector clock with the remote one
    this.vectorClock = this.vectorClock.merge(remoteVectorClock);

    // Transform against concurrent operations if needed
    const transformedOp = this.transformAgainstHistory(operation);

    this.applyOperation(transformedOp);

    if (this.config.preserveHistory) {
      this.addToHistory(transformedOp);
    }

    // Auto garbage collection
    if (this.config.autoGC && this.content.size > this.config.maxHistorySize) {
      this.garbageCollect();
    }
  }

  /**
   * Apply an operation to the document content
   */
  private applyOperation(operation: Operation): void {
    switch (operation.type) {
      case OperationType.INSERT:
        this.applyInsert(operation);
        break;

      case OperationType.DELETE:
        this.applyDelete(operation);
        break;

      case OperationType.REPLACE:
        this.applyReplace(operation);
        break;

      default:
        console.warn('Unknown operation type:', operation.type);
    }
  }

  /**
   * Apply insert operation
   */
  private applyInsert(operation: Operation): void {
    const position = operation.position.offset || 0;
    const content = String(operation.content || '');

    for (let i = 0; i < content.length; i++) {
      const nodeId = `${operation.id}_${i}`;
      const node: CRDTNode = {
        id: nodeId,
        participantId: operation.participantId,
        value: content[i],
        timestamp: operation.timestamp,
        vectorClock: new VectorClock(operation.vectorClock),
        deleted: false,
      };

      // Insert into linked list at position
      this.insertNodeAtPosition(node, position + i);
      this.content.set(nodeId, node);
    }
  }

  /**
   * Apply delete operation
   */
  private applyDelete(operation: Operation): void {
    const position = operation.position.offset || 0;
    const length = operation.length || 0;

    let currentPos = 0;
    let deletedCount = 0;
    let currentId = this.head;

    while (currentId && deletedCount < length) {
      const node = this.content.get(currentId);
      if (!node) break;

      if (!node.deleted) {
        if (currentPos >= position && deletedCount < length) {
          node.deleted = true;
          deletedCount++;
        }
        currentPos++;
      }

      currentId = node.next || null;
    }
  }

  /**
   * Apply replace operation
   */
  private applyReplace(operation: Operation): void {
    // Replace is delete + insert
    this.applyDelete(operation);
    this.applyInsert(operation);
  }

  /**
   * Transform an operation against concurrent operations in history
   */
  private transformAgainstHistory(operation: Operation): Operation {
    let transformedOp = operation;
    const opVectorClock = new VectorClock(operation.vectorClock);

    // Find concurrent operations
    const concurrentOps = this.operationHistory.filter((histOp) => {
      const histVectorClock = new VectorClock(histOp.vectorClock);
      return opVectorClock.isConcurrent(histVectorClock);
    });

    // Transform against each concurrent operation
    for (const concurrentOp of concurrentOps) {
      const [transformed] = OperationalTransform.transform(
        transformedOp,
        concurrentOp
      );
      transformedOp = transformed;
    }

    return transformedOp;
  }

  /**
   * Insert a node at a specific position in the linked list
   */
  private insertNodeAtPosition(node: CRDTNode, position: number): void {
    if (this.head === null) {
      // Empty list
      this.head = node.id;
      this.tail = node.id;
      return;
    }

    let currentPos = 0;
    let currentId = this.head;
    let prevId: string | null = null;

    // Find insertion point
    while (currentId && currentPos < position) {
      const currentNode = this.content.get(currentId);
      if (!currentNode) break;

      if (!currentNode.deleted) {
        currentPos++;
      }

      prevId = currentId;
      currentId = currentNode.next || null;
    }

    // Insert node
    if (prevId === null) {
      // Insert at head
      node.next = this.head || undefined;
      const headNode = this.head ? this.content.get(this.head) : null;
      if (headNode) {
        headNode.prev = node.id;
      }
      this.head = node.id;
    } else {
      const prevNode = this.content.get(prevId);
      if (prevNode) {
        node.prev = prevId;
        node.next = prevNode.next;
        prevNode.next = node.id;

        if (node.next) {
          const nextNode = this.content.get(node.next);
          if (nextNode) {
            nextNode.prev = node.id;
          }
        } else {
          this.tail = node.id;
        }
      }
    }
  }

  /**
   * Get the current document content as a string
   */
  toString(): string {
    const chars: string[] = [];
    let currentId = this.head;

    while (currentId) {
      const node = this.content.get(currentId);
      if (!node) break;

      if (!node.deleted && typeof node.value === 'string') {
        chars.push(node.value);
      }

      currentId = node.next || null;
    }

    return chars.join('');
  }

  /**
   * Get the current document state
   */
  getState(): DocumentState {
    const metadata: DocumentMetadata = {
      id: this.nodeId,
      type: 'text' as any,
      version: this.operationHistory.length,
      createdAt: new Date(),
      updatedAt: new Date(),
      createdBy: this.nodeId,
      lastModifiedBy: this.nodeId,
    };

    return {
      metadata,
      content: this.toString(),
      checksum: this.calculateChecksum(),
      vectorClock: this.vectorClock.toObject(),
    };
  }

  /**
   * Add operation to history
   */
  private addToHistory(operation: Operation): void {
    this.operationHistory.push(operation);

    // Trim history if needed
    if (this.operationHistory.length > this.config.maxHistorySize) {
      this.operationHistory = this.operationHistory.slice(
        -this.config.maxHistorySize
      );
    }
  }

  /**
   * Garbage collect deleted nodes
   */
  private garbageCollect(): void {
    const nodesToDelete: string[] = [];

    for (const [nodeId, node] of this.content.entries()) {
      if (node.deleted) {
        nodesToDelete.push(nodeId);
      }
    }

    for (const nodeId of nodesToDelete) {
      const node = this.content.get(nodeId);
      if (!node) continue;

      // Update linked list pointers
      if (node.prev) {
        const prevNode = this.content.get(node.prev);
        if (prevNode) {
          prevNode.next = node.next;
        }
      } else {
        this.head = node.next || null;
      }

      if (node.next) {
        const nextNode = this.content.get(node.next);
        if (nextNode) {
          nextNode.prev = node.prev;
        }
      } else {
        this.tail = node.prev || null;
      }

      this.content.delete(nodeId);
    }
  }

  /**
   * Calculate document checksum
   */
  private calculateChecksum(): string {
    const content = this.toString();
    let hash = 0;

    for (let i = 0; i < content.length; i++) {
      const char = content.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash; // Convert to 32-bit integer
    }

    return hash.toString(16);
  }

  /**
   * Generate unique node ID
   */
  private generateNodeId(): string {
    return `${this.nodeId}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Get operation history
   */
  getHistory(): Operation[] {
    return [...this.operationHistory];
  }

  /**
   * Get vector clock
   */
  getVectorClock(): VectorClock {
    return this.vectorClock.clone();
  }

  /**
   * Merge with another CRDT document
   */
  merge(other: CRDTDocument): void {
    // Merge vector clocks
    this.vectorClock = this.vectorClock.merge(other.getVectorClock());

    // Apply remote operations
    const remoteHistory = other.getHistory();
    for (const operation of remoteHistory) {
      const opVectorClock = new VectorClock(operation.vectorClock);

      // Only apply operations we haven't seen
      if (!this.hasSeenOperation(opVectorClock)) {
        this.applyRemoteOperation(operation);
      }
    }
  }

  /**
   * Check if we've seen an operation based on its vector clock
   */
  private hasSeenOperation(opVectorClock: VectorClock): boolean {
    const comparison = this.vectorClock.compare(opVectorClock);
    return (
      comparison === ClockComparison.EQUAL ||
      comparison === ClockComparison.AFTER
    );
  }

  /**
   * Clear document
   */
  clear(): void {
    this.content.clear();
    this.head = null;
    this.tail = null;
    this.operationHistory = [];
  }
}
