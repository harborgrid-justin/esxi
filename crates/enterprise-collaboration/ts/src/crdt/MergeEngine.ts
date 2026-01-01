/**
 * Merge Engine - Three-way Merge Resolution
 * Handles document merging and conflict resolution
 */

import {
  Operation,
  OperationType,
  Conflict,
  ConflictType,
  ConflictResolution,
  ConflictResolutionStrategy,
  Range,
} from '../types';
import { VectorClock } from './VectorClock';
import { OperationalTransform } from './OperationalTransform';

export interface MergeResult {
  operations: Operation[];
  conflicts: Conflict[];
  resolved: boolean;
}

export interface MergeOptions {
  strategy: ConflictResolutionStrategy;
  participantId: string;
  preferredAuthor?: string;
}

export class MergeEngine {
  /**
   * Perform three-way merge of operations
   * @param base - Common ancestor operations
   * @param local - Local operations
   * @param remote - Remote operations
   * @param options - Merge options
   */
  static merge(
    base: Operation[],
    local: Operation[],
    remote: Operation[],
    options: MergeOptions
  ): MergeResult {
    const result: MergeResult = {
      operations: [],
      conflicts: [],
      resolved: true,
    };

    // Identify operations unique to each branch
    const localOnly = this.diffOperations(local, base);
    const remoteOnly = this.diffOperations(remote, base);

    // Detect conflicts
    const conflicts = this.detectConflicts(localOnly, remoteOnly);

    if (conflicts.length === 0) {
      // No conflicts - simple merge
      result.operations = this.mergeNonConflicting(localOnly, remoteOnly);
      return result;
    }

    // Handle conflicts based on strategy
    const resolution = this.resolveConflicts(conflicts, options);
    result.conflicts = resolution.conflicts;
    result.operations = resolution.operations;
    result.resolved = resolution.resolved;

    return result;
  }

  /**
   * Detect conflicts between two sets of operations
   */
  static detectConflicts(
    ops1: Operation[],
    ops2: Operation[]
  ): Conflict[] {
    const conflicts: Conflict[] = [];

    for (const op1 of ops1) {
      for (const op2 of ops2) {
        const conflict = this.checkConflict(op1, op2);
        if (conflict) {
          conflicts.push(conflict);
        }
      }
    }

    return conflicts;
  }

  /**
   * Check if two operations conflict
   */
  private static checkConflict(
    op1: Operation,
    op2: Operation
  ): Conflict | null {
    // Same participant - no conflict
    if (op1.participantId === op2.participantId) {
      return null;
    }

    // Check for vector clock concurrency
    const clock1 = new VectorClock(op1.vectorClock);
    const clock2 = new VectorClock(op2.vectorClock);

    if (!clock1.isConcurrent(clock2)) {
      return null;
    }

    // Check for overlapping ranges
    const range1 = this.operationToRange(op1);
    const range2 = this.operationToRange(op2);

    if (this.rangesOverlap(range1, range2)) {
      return {
        id: `conflict_${op1.id}_${op2.id}`,
        type: ConflictType.CONCURRENT_EDIT,
        operations: [op1, op2],
        affectedRanges: [range1, range2],
        detectedAt: new Date(),
        resolved: false,
      };
    }

    return null;
  }

  /**
   * Resolve conflicts based on strategy
   */
  static resolveConflicts(
    conflicts: Conflict[],
    options: MergeOptions
  ): {
    conflicts: Conflict[];
    operations: Operation[];
    resolved: boolean;
  } {
    const resolvedConflicts: Conflict[] = [];
    const operations: Operation[] = [];
    let allResolved = true;

    for (const conflict of conflicts) {
      const resolution = this.resolveConflict(conflict, options);

      if (resolution) {
        resolvedConflicts.push({
          ...conflict,
          resolved: true,
          resolution,
        });

        // Add winning operation(s)
        if (resolution.selectedOperation) {
          const selectedOp = conflict.operations.find(
            (op) => op.id === resolution.selectedOperation
          );
          if (selectedOp) {
            operations.push(selectedOp);
          }
        } else if (resolution.mergedResult) {
          // Add merged operation
          operations.push(resolution.mergedResult as Operation);
        }
      } else {
        resolvedConflicts.push(conflict);
        allResolved = false;
      }
    }

    return {
      conflicts: resolvedConflicts,
      operations,
      resolved: allResolved,
    };
  }

  /**
   * Resolve a single conflict
   */
  private static resolveConflict(
    conflict: Conflict,
    options: MergeOptions
  ): ConflictResolution | null {
    switch (options.strategy) {
      case ConflictResolutionStrategy.LAST_WRITE_WINS:
        return this.resolveLastWriteWins(conflict, options);

      case ConflictResolutionStrategy.FIRST_WRITE_WINS:
        return this.resolveFirstWriteWins(conflict, options);

      case ConflictResolutionStrategy.MERGE:
        return this.resolveMerge(conflict, options);

      case ConflictResolutionStrategy.MANUAL:
        return null; // Manual resolution required

      default:
        return this.resolveLastWriteWins(conflict, options);
    }
  }

  /**
   * Resolve conflict using last-write-wins strategy
   */
  private static resolveLastWriteWins(
    conflict: Conflict,
    options: MergeOptions
  ): ConflictResolution {
    const sortedOps = conflict.operations.sort(
      (a, b) => b.timestamp.getTime() - a.timestamp.getTime()
    );

    return {
      conflictId: conflict.id,
      strategy: ConflictResolutionStrategy.LAST_WRITE_WINS,
      selectedOperation: sortedOps[0].id,
      resolvedBy: options.participantId,
      resolvedAt: new Date(),
    };
  }

  /**
   * Resolve conflict using first-write-wins strategy
   */
  private static resolveFirstWriteWins(
    conflict: Conflict,
    options: MergeOptions
  ): ConflictResolution {
    const sortedOps = conflict.operations.sort(
      (a, b) => a.timestamp.getTime() - b.timestamp.getTime()
    );

    return {
      conflictId: conflict.id,
      strategy: ConflictResolutionStrategy.FIRST_WRITE_WINS,
      selectedOperation: sortedOps[0].id,
      resolvedBy: options.participantId,
      resolvedAt: new Date(),
    };
  }

  /**
   * Resolve conflict using merge strategy
   */
  private static resolveMerge(
    conflict: Conflict,
    options: MergeOptions
  ): ConflictResolution {
    const [op1, op2] = conflict.operations;

    // Attempt to merge operations
    if (
      op1.type === OperationType.INSERT &&
      op2.type === OperationType.INSERT
    ) {
      // Merge inserts
      const mergedOp = this.mergeInserts(op1, op2);

      return {
        conflictId: conflict.id,
        strategy: ConflictResolutionStrategy.MERGE,
        mergedResult: mergedOp,
        resolvedBy: options.participantId,
        resolvedAt: new Date(),
      };
    }

    // Fall back to last-write-wins if merge not possible
    return this.resolveLastWriteWins(conflict, options);
  }

  /**
   * Merge two insert operations
   */
  private static mergeInserts(op1: Operation, op2: Operation): Operation {
    const content1 = String(op1.content || '');
    const content2 = String(op2.content || '');

    // Combine content based on timestamp order
    const mergedContent =
      op1.timestamp < op2.timestamp
        ? content1 + content2
        : content2 + content1;

    // Merge vector clocks
    const clock1 = new VectorClock(op1.vectorClock);
    const clock2 = new VectorClock(op2.vectorClock);
    const mergedClock = clock1.merge(clock2);

    return {
      id: `merged_${op1.id}_${op2.id}`,
      type: OperationType.INSERT,
      position: op1.timestamp < op2.timestamp ? op1.position : op2.position,
      content: mergedContent,
      participantId: op1.participantId,
      timestamp: new Date(),
      vectorClock: mergedClock.toObject(),
      metadata: {
        merged: true,
        sourceOperations: [op1.id, op2.id],
      },
    };
  }

  /**
   * Merge non-conflicting operations
   */
  private static mergeNonConflicting(
    ops1: Operation[],
    ops2: Operation[]
  ): Operation[] {
    const merged = [...ops1, ...ops2];

    // Sort by vector clock and timestamp
    merged.sort((a, b) => {
      const clockA = new VectorClock(a.vectorClock);
      const clockB = new VectorClock(b.vectorClock);

      if (clockA.happensBefore(clockB)) return -1;
      if (clockA.happensAfter(clockB)) return 1;

      return a.timestamp.getTime() - b.timestamp.getTime();
    });

    return merged;
  }

  /**
   * Diff two operation sets
   */
  private static diffOperations(
    ops: Operation[],
    base: Operation[]
  ): Operation[] {
    const baseIds = new Set(base.map((op) => op.id));
    return ops.filter((op) => !baseIds.has(op.id));
  }

  /**
   * Convert operation to range
   */
  private static operationToRange(operation: Operation): Range {
    const start = operation.position;
    const end = { ...start };

    if (operation.type === OperationType.DELETE && operation.length) {
      end.offset = (start.offset || 0) + operation.length;
    } else if (
      operation.type === OperationType.INSERT &&
      operation.content
    ) {
      const length =
        typeof operation.content === 'string'
          ? operation.content.length
          : 1;
      end.offset = (start.offset || 0) + length;
    }

    return { start, end };
  }

  /**
   * Check if ranges overlap
   */
  private static rangesOverlap(range1: Range, range2: Range): boolean {
    const start1 = range1.start.offset || 0;
    const end1 = range1.end.offset || 0;
    const start2 = range2.start.offset || 0;
    const end2 = range2.end.offset || 0;

    return start1 < end2 && start2 < end1;
  }

  /**
   * Apply a merge result to a document
   */
  static applyMergeResult(
    content: string,
    result: MergeResult
  ): string {
    let updatedContent = content;

    for (const operation of result.operations) {
      updatedContent = OperationalTransform.apply(updatedContent, operation);
    }

    return updatedContent;
  }

  /**
   * Create a checkpoint from current state
   */
  static createCheckpoint(
    operations: Operation[],
    participantId: string
  ): Operation {
    const vectorClocks = operations.map((op) => new VectorClock(op.vectorClock));
    const mergedClock = vectorClocks.reduce(
      (acc, clock) => acc.merge(clock),
      VectorClock.create()
    );

    return {
      id: `checkpoint_${Date.now()}`,
      type: OperationType.CUSTOM,
      position: { line: 0, column: 0, offset: 0 },
      participantId,
      timestamp: new Date(),
      vectorClock: mergedClock.toObject(),
      metadata: {
        checkpoint: true,
        operationCount: operations.length,
      },
    };
  }
}
