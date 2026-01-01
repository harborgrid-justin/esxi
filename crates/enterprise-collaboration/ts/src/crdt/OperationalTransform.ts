/**
 * Operational Transformation (OT) Implementation
 * Handles concurrent editing and operation transformation
 */

import { Operation, OperationType, Position } from '../types';

export interface TransformResult {
  operation: Operation;
  inverse?: Operation;
}

export class OperationalTransform {
  /**
   * Transform two concurrent operations against each other
   * Returns the transformed versions that maintain consistency
   */
  static transform(op1: Operation, op2: Operation): [Operation, Operation] {
    // If operations are from the same participant, no transformation needed
    if (op1.participantId === op2.participantId) {
      return [op1, op2];
    }

    // Transform based on operation types
    if (op1.type === OperationType.INSERT && op2.type === OperationType.INSERT) {
      return this.transformInsertInsert(op1, op2);
    }
    if (op1.type === OperationType.INSERT && op2.type === OperationType.DELETE) {
      return this.transformInsertDelete(op1, op2);
    }
    if (op1.type === OperationType.DELETE && op2.type === OperationType.INSERT) {
      const [op2Prime, op1Prime] = this.transformInsertDelete(op2, op1);
      return [op1Prime, op2Prime];
    }
    if (op1.type === OperationType.DELETE && op2.type === OperationType.DELETE) {
      return this.transformDeleteDelete(op1, op2);
    }

    // Default: no transformation for other operation types
    return [op1, op2];
  }

  /**
   * Transform two concurrent insert operations
   */
  private static transformInsertInsert(
    op1: Operation,
    op2: Operation
  ): [Operation, Operation] {
    const pos1 = this.positionToOffset(op1.position);
    const pos2 = this.positionToOffset(op2.position);

    let op1Prime = { ...op1 };
    let op2Prime = { ...op2 };

    if (pos1 < pos2) {
      // op1 is before op2, shift op2 position
      op2Prime = {
        ...op2,
        position: this.offsetToPosition(
          pos2 + (this.getContentLength(op1.content) || 0)
        ),
      };
    } else if (pos1 > pos2) {
      // op2 is before op1, shift op1 position
      op1Prime = {
        ...op1,
        position: this.offsetToPosition(
          pos1 + (this.getContentLength(op2.content) || 0)
        ),
      };
    } else {
      // Same position - use tie-breaking (participant ID)
      if (op1.participantId < op2.participantId) {
        op2Prime = {
          ...op2,
          position: this.offsetToPosition(
            pos2 + (this.getContentLength(op1.content) || 0)
          ),
        };
      } else {
        op1Prime = {
          ...op1,
          position: this.offsetToPosition(
            pos1 + (this.getContentLength(op2.content) || 0)
          ),
        };
      }
    }

    return [op1Prime, op2Prime];
  }

  /**
   * Transform insert and delete operations
   */
  private static transformInsertDelete(
    insert: Operation,
    del: Operation
  ): [Operation, Operation] {
    const insertPos = this.positionToOffset(insert.position);
    const deletePos = this.positionToOffset(del.position);
    const deleteLen = del.length || 0;

    let insertPrime = { ...insert };
    let deletePrime = { ...del };

    if (insertPos <= deletePos) {
      // Insert is before or at delete position
      deletePrime = {
        ...del,
        position: this.offsetToPosition(
          deletePos + (this.getContentLength(insert.content) || 0)
        ),
      };
    } else if (insertPos > deletePos + deleteLen) {
      // Insert is after delete range
      insertPrime = {
        ...insert,
        position: this.offsetToPosition(insertPos - deleteLen),
      };
    } else {
      // Insert is within delete range
      deletePrime = {
        ...del,
        length: deleteLen + (this.getContentLength(insert.content) || 0),
      };
    }

    return [insertPrime, deletePrime];
  }

  /**
   * Transform two concurrent delete operations
   */
  private static transformDeleteDelete(
    op1: Operation,
    op2: Operation
  ): [Operation, Operation] {
    const pos1 = this.positionToOffset(op1.position);
    const pos2 = this.positionToOffset(op2.position);
    const len1 = op1.length || 0;
    const len2 = op2.length || 0;

    let op1Prime = { ...op1 };
    let op2Prime = { ...op2 };

    const end1 = pos1 + len1;
    const end2 = pos2 + len2;

    // No overlap
    if (end1 <= pos2) {
      // op1 is completely before op2
      op2Prime = {
        ...op2,
        position: this.offsetToPosition(pos2 - len1),
      };
    } else if (end2 <= pos1) {
      // op2 is completely before op1
      op1Prime = {
        ...op1,
        position: this.offsetToPosition(pos1 - len2),
      };
    } else {
      // Overlapping deletes - compute intersection
      const overlapStart = Math.max(pos1, pos2);
      const overlapEnd = Math.min(end1, end2);
      const overlapLen = overlapEnd - overlapStart;

      if (pos1 <= pos2) {
        op1Prime = {
          ...op1,
          length: len1 - overlapLen,
        };
        op2Prime = {
          ...op2,
          position: this.offsetToPosition(pos1),
          length: Math.max(0, len2 - overlapLen - (pos2 - pos1)),
        };
      } else {
        op2Prime = {
          ...op2,
          length: len2 - overlapLen,
        };
        op1Prime = {
          ...op1,
          position: this.offsetToPosition(pos2),
          length: Math.max(0, len1 - overlapLen - (pos1 - pos2)),
        };
      }
    }

    return [op1Prime, op2Prime];
  }

  /**
   * Compose two sequential operations into a single operation
   */
  static compose(op1: Operation, op2: Operation): Operation | null {
    // Can only compose operations from the same participant
    if (op1.participantId !== op2.participantId) {
      return null;
    }

    // Compose insert + insert at same position
    if (
      op1.type === OperationType.INSERT &&
      op2.type === OperationType.INSERT &&
      this.positionsEqual(op1.position, op2.position)
    ) {
      return {
        ...op1,
        content: this.concatenateContent(op1.content, op2.content),
        timestamp: op2.timestamp,
      };
    }

    // Compose delete + delete at same position
    if (
      op1.type === OperationType.DELETE &&
      op2.type === OperationType.DELETE &&
      this.positionsEqual(op1.position, op2.position)
    ) {
      return {
        ...op1,
        length: (op1.length || 0) + (op2.length || 0),
        timestamp: op2.timestamp,
      };
    }

    return null;
  }

  /**
   * Generate inverse operation (for undo)
   */
  static inverse(op: Operation, deletedContent?: unknown): Operation {
    switch (op.type) {
      case OperationType.INSERT:
        return {
          ...op,
          type: OperationType.DELETE,
          length: this.getContentLength(op.content),
          content: undefined,
        };

      case OperationType.DELETE:
        return {
          ...op,
          type: OperationType.INSERT,
          content: deletedContent,
          length: undefined,
        };

      case OperationType.REPLACE:
        return {
          ...op,
          content: deletedContent,
        };

      default:
        return op;
    }
  }

  /**
   * Apply an operation to content
   */
  static apply(content: string, op: Operation): string {
    const offset = this.positionToOffset(op.position);

    switch (op.type) {
      case OperationType.INSERT:
        return (
          content.slice(0, offset) +
          (op.content as string) +
          content.slice(offset)
        );

      case OperationType.DELETE:
        return (
          content.slice(0, offset) + content.slice(offset + (op.length || 0))
        );

      case OperationType.REPLACE:
        return (
          content.slice(0, offset) +
          (op.content as string) +
          content.slice(offset + (op.length || 0))
        );

      default:
        return content;
    }
  }

  /**
   * Validate operation integrity
   */
  static validate(op: Operation): boolean {
    if (!op.id || !op.participantId || !op.timestamp) {
      return false;
    }

    switch (op.type) {
      case OperationType.INSERT:
        return op.content !== undefined;

      case OperationType.DELETE:
        return (op.length || 0) > 0;

      case OperationType.REPLACE:
        return op.content !== undefined && (op.length || 0) > 0;

      default:
        return true;
    }
  }

  // ============================================================================
  // Helper Methods
  // ============================================================================

  private static positionToOffset(position: Position): number {
    return position.offset || 0;
  }

  private static offsetToPosition(offset: number): Position {
    return {
      line: 0,
      column: 0,
      offset,
    };
  }

  private static positionsEqual(pos1: Position, pos2: Position): boolean {
    return this.positionToOffset(pos1) === this.positionToOffset(pos2);
  }

  private static getContentLength(content: unknown): number {
    if (typeof content === 'string') {
      return content.length;
    }
    if (Array.isArray(content)) {
      return content.length;
    }
    return 1;
  }

  private static concatenateContent(
    content1: unknown,
    content2: unknown
  ): unknown {
    if (typeof content1 === 'string' && typeof content2 === 'string') {
      return content1 + content2;
    }
    if (Array.isArray(content1) && Array.isArray(content2)) {
      return [...content1, ...content2];
    }
    return content2;
  }
}
