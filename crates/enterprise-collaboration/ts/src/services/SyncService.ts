/**
 * Sync Service
 * Handles document synchronization and operation management
 */

import {
  Operation,
  SyncMessage,
  SyncStatus,
  SyncState,
  DocumentState,
  CollaborationError,
  ErrorCode,
} from '../types';
import { VectorClock } from '../crdt/VectorClock';
import { OperationalTransform } from '../crdt/OperationalTransform';

export interface SyncServiceConfig {
  syncInterval?: number;
  batchSize?: number;
  retryAttempts?: number;
  retryDelay?: number;
}

export class SyncService {
  private pendingOperations: Operation[] = [];
  private vectorClock: VectorClock;
  private config: Required<SyncServiceConfig>;
  private syncTimer: NodeJS.Timeout | null = null;
  private sequenceNumber: number = 0;

  constructor(config: SyncServiceConfig = {}) {
    this.config = {
      syncInterval: config.syncInterval ?? 1000,
      batchSize: config.batchSize ?? 50,
      retryAttempts: config.retryAttempts ?? 3,
      retryDelay: config.retryDelay ?? 1000,
    };

    this.vectorClock = VectorClock.create();
  }

  /**
   * Add an operation to the pending queue
   */
  addOperation(operation: Operation): void {
    this.pendingOperations.push(operation);
    this.vectorClock.increment(operation.participantId);
  }

  /**
   * Get pending operations
   */
  getPendingOperations(): Operation[] {
    return [...this.pendingOperations];
  }

  /**
   * Clear pending operations
   */
  clearPendingOperations(): void {
    this.pendingOperations = [];
  }

  /**
   * Create a sync message
   */
  createSyncMessage(participantId: string): SyncMessage {
    const operations = this.pendingOperations.slice(0, this.config.batchSize);

    const message: SyncMessage = {
      type: 'sync',
      operations,
      vectorClock: this.vectorClock.toObject(),
      timestamp: new Date(),
      senderId: participantId,
      sequenceNumber: this.sequenceNumber++,
    };

    return message;
  }

  /**
   * Process incoming sync message
   */
  processSyncMessage(message: SyncMessage): {
    operations: Operation[];
    conflicts: Set<string>;
  } {
    const conflicts = new Set<string>();
    const processedOps: Operation[] = [];

    if (!message.operations) {
      return { operations: processedOps, conflicts };
    }

    const remoteVectorClock = new VectorClock(message.vectorClock);

    for (const operation of message.operations) {
      // Check if we've already seen this operation
      const opVectorClock = new VectorClock(operation.vectorClock);
      if (this.vectorClock.happensAfter(opVectorClock)) {
        continue;
      }

      // Transform against pending operations
      let transformedOp = operation;
      for (const pendingOp of this.pendingOperations) {
        const [transformed] = OperationalTransform.transform(
          transformedOp,
          pendingOp
        );
        transformedOp = transformed;
      }

      processedOps.push(transformedOp);

      // Merge vector clocks
      this.vectorClock = this.vectorClock.merge(opVectorClock);
    }

    return { operations: processedOps, conflicts };
  }

  /**
   * Acknowledge received operations
   */
  acknowledgeOperations(operationIds: string[]): void {
    this.pendingOperations = this.pendingOperations.filter(
      (op) => !operationIds.includes(op.id)
    );
  }

  /**
   * Get current sync status
   */
  getSyncStatus(): SyncStatus {
    const state =
      this.pendingOperations.length === 0
        ? SyncState.SYNCED
        : SyncState.SYNCING;

    return {
      state,
      pendingOperations: this.pendingOperations.length,
      unresolvedConflicts: 0,
      lastSyncAt: new Date(),
    };
  }

  /**
   * Start auto-sync
   */
  startAutoSync(callback: () => void): void {
    this.stopAutoSync();

    this.syncTimer = setInterval(() => {
      if (this.pendingOperations.length > 0) {
        callback();
      }
    }, this.config.syncInterval);
  }

  /**
   * Stop auto-sync
   */
  stopAutoSync(): void {
    if (this.syncTimer) {
      clearInterval(this.syncTimer);
      this.syncTimer = null;
    }
  }

  /**
   * Create checkpoint
   */
  createCheckpoint(documentState: DocumentState): Operation {
    return {
      id: `checkpoint_${Date.now()}`,
      type: 'custom' as any,
      position: { line: 0, column: 0, offset: 0 },
      participantId: 'system',
      timestamp: new Date(),
      vectorClock: this.vectorClock.toObject(),
      metadata: {
        checkpoint: true,
        documentState,
      },
    };
  }

  /**
   * Validate operation integrity
   */
  validateOperation(operation: Operation): boolean {
    return OperationalTransform.validate(operation);
  }

  /**
   * Get vector clock
   */
  getVectorClock(): VectorClock {
    return this.vectorClock.clone();
  }

  /**
   * Set vector clock
   */
  setVectorClock(clock: VectorClock): void {
    this.vectorClock = clock.clone();
  }

  /**
   * Merge with remote vector clock
   */
  mergeVectorClock(remoteClock: VectorClock): void {
    this.vectorClock = this.vectorClock.merge(remoteClock);
  }

  /**
   * Get operation by ID
   */
  getOperation(operationId: string): Operation | undefined {
    return this.pendingOperations.find((op) => op.id === operationId);
  }

  /**
   * Retry failed sync with exponential backoff
   */
  async retrySyncWithBackoff(
    syncFn: () => Promise<void>,
    attempt: number = 0
  ): Promise<void> {
    try {
      await syncFn();
    } catch (error) {
      if (attempt >= this.config.retryAttempts) {
        throw new CollaborationError(
          ErrorCode.SYNC_FAILED,
          `Sync failed after ${this.config.retryAttempts} attempts`,
          { lastError: error }
        );
      }

      const delay = this.config.retryDelay * Math.pow(2, attempt);
      await new Promise((resolve) => setTimeout(resolve, delay));

      return this.retrySyncWithBackoff(syncFn, attempt + 1);
    }
  }

  /**
   * Calculate sync latency
   */
  calculateLatency(sentAt: Date): number {
    return Date.now() - sentAt.getTime();
  }

  /**
   * Reset service state
   */
  reset(): void {
    this.pendingOperations = [];
    this.vectorClock = VectorClock.create();
    this.sequenceNumber = 0;
    this.stopAutoSync();
  }
}
