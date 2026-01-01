/**
 * Collaboration service bridge for real-time collaboration.
 *
 * Provides TypeScript wrapper around WASM collaboration engine with
 * CRDT and Operational Transform support.
 */

import type {
  CollaborationEvent,
  TransformedOperation,
  MergedOperation,
  PresenceUpdate,
  OperationResult,
} from '../types';
import { BridgeError } from '../types';
import { WasmLoader } from '../loader/WasmLoader';

/**
 * Collaboration service bridge.
 */
export class CollaborationBridge {
  private collaborationEngine: any = null;
  private crdtText: any = null;

  constructor(
    private readonly loader: WasmLoader,
    private readonly userId: string,
    private readonly numUsers = 10
  ) {}

  /**
   * Initialize the collaboration engine.
   */
  private async ensureInitialized(): Promise<void> {
    if (!this.collaborationEngine) {
      const instance = this.loader.getInstance();
      // In production:
      // this.collaborationEngine = new instance.CollaborationEngine(this.userId, this.numUsers);
      // this.crdtText = new instance.CrdtText(this.userId);
      throw new BridgeError(
        'Collaboration engine not available. Build WASM module first.',
        'COLLABORATION_NOT_AVAILABLE'
      );
    }
  }

  /**
   * Apply a local operation and generate an event.
   *
   * @param operationType - Type of operation (insert, delete, update, etc.)
   * @param payload - Operation payload
   * @returns Collaboration event to broadcast
   */
  async applyLocalOperation(
    operationType: string,
    payload: unknown
  ): Promise<OperationResult<CollaborationEvent>> {
    await this.ensureInitialized();

    try {
      const result = await this.collaborationEngine.apply_local_operation(
        operationType,
        payload
      );
      return result as OperationResult<CollaborationEvent>;
    } catch (error) {
      throw new BridgeError(
        `Local operation failed: ${error instanceof Error ? error.message : String(error)}`,
        'LOCAL_OP_ERROR',
        error
      );
    }
  }

  /**
   * Apply a remote operation from another user.
   *
   * @param event - Collaboration event from remote user
   * @returns Transformed operation
   */
  async applyRemoteOperation(
    event: CollaborationEvent
  ): Promise<OperationResult<TransformedOperation>> {
    await this.ensureInitialized();

    try {
      const result = await this.collaborationEngine.apply_remote_operation(event);
      return result as OperationResult<TransformedOperation>;
    } catch (error) {
      throw new BridgeError(
        `Remote operation failed: ${error instanceof Error ? error.message : String(error)}`,
        'REMOTE_OP_ERROR',
        error
      );
    }
  }

  /**
   * Merge two operations that happened concurrently.
   *
   * @param op1 - First operation
   * @param op2 - Second operation
   * @returns Merged operation
   */
  async mergeOperations(
    op1: CollaborationEvent,
    op2: CollaborationEvent
  ): Promise<OperationResult<MergedOperation>> {
    await this.ensureInitialized();

    try {
      const result = await this.collaborationEngine.merge_operations(op1, op2);
      return result as OperationResult<MergedOperation>;
    } catch (error) {
      throw new BridgeError(
        `Merge operations failed: ${error instanceof Error ? error.message : String(error)}`,
        'MERGE_ERROR',
        error
      );
    }
  }

  /**
   * Check if two operations are causally ordered.
   *
   * @param clock1 - First vector clock
   * @param clock2 - Second vector clock
   * @returns True if op1 happened before op2
   */
  async isCausallyBefore(clock1: number[], clock2: number[]): Promise<boolean> {
    await this.ensureInitialized();

    try {
      return this.collaborationEngine.is_causally_before(clock1, clock2);
    } catch (error) {
      throw new BridgeError(
        `Causal ordering check failed: ${error instanceof Error ? error.message : String(error)}`,
        'CAUSAL_ERROR',
        error
      );
    }
  }

  /**
   * Get the current vector clock.
   *
   * @returns Current vector clock
   */
  async getVectorClock(): Promise<number[]> {
    await this.ensureInitialized();

    try {
      return this.collaborationEngine.get_vector_clock();
    } catch (error) {
      throw new BridgeError(
        `Failed to get vector clock: ${error instanceof Error ? error.message : String(error)}`,
        'CLOCK_ERROR',
        error
      );
    }
  }

  /**
   * Set the vector clock.
   *
   * @param clock - New vector clock
   */
  async setVectorClock(clock: number[]): Promise<void> {
    await this.ensureInitialized();

    try {
      this.collaborationEngine.set_vector_clock(clock);
    } catch (error) {
      throw new BridgeError(
        `Failed to set vector clock: ${error instanceof Error ? error.message : String(error)}`,
        'CLOCK_ERROR',
        error
      );
    }
  }

  /**
   * Create a presence update event.
   *
   * @param presenceData - Presence data (cursor, selection, etc.)
   * @returns Presence update event
   */
  async createPresenceEvent(presenceData: unknown): Promise<PresenceUpdate> {
    await this.ensureInitialized();

    try {
      const result = await this.collaborationEngine.create_presence_event(
        presenceData
      );
      return result as PresenceUpdate;
    } catch (error) {
      throw new BridgeError(
        `Presence event creation failed: ${error instanceof Error ? error.message : String(error)}`,
        'PRESENCE_ERROR',
        error
      );
    }
  }

  /**
   * CRDT text operations
   */

  /**
   * Insert text at a position (CRDT).
   *
   * @param position - Insert position
   * @param text - Text to insert
   */
  async insertText(position: number, text: string): Promise<void> {
    await this.ensureInitialized();

    try {
      await this.crdtText.insert(position, text);
    } catch (error) {
      throw new BridgeError(
        `Text insertion failed: ${error instanceof Error ? error.message : String(error)}`,
        'INSERT_TEXT_ERROR',
        error
      );
    }
  }

  /**
   * Delete text at a position (CRDT).
   *
   * @param position - Delete position
   * @param length - Number of characters to delete
   */
  async deleteText(position: number, length: number): Promise<void> {
    await this.ensureInitialized();

    try {
      await this.crdtText.delete(position, length);
    } catch (error) {
      throw new BridgeError(
        `Text deletion failed: ${error instanceof Error ? error.message : String(error)}`,
        'DELETE_TEXT_ERROR',
        error
      );
    }
  }

  /**
   * Get the current text content (CRDT).
   *
   * @returns Current text content
   */
  async getTextContent(): Promise<string> {
    await this.ensureInitialized();

    try {
      return this.crdtText.content();
    } catch (error) {
      throw new BridgeError(
        `Failed to get text content: ${error instanceof Error ? error.message : String(error)}`,
        'GET_TEXT_ERROR',
        error
      );
    }
  }

  /**
   * Get the text length (CRDT).
   *
   * @returns Text length
   */
  async getTextLength(): Promise<number> {
    await this.ensureInitialized();

    try {
      return this.crdtText.length();
    } catch (error) {
      throw new BridgeError(
        `Failed to get text length: ${error instanceof Error ? error.message : String(error)}`,
        'GET_LENGTH_ERROR',
        error
      );
    }
  }

  /**
   * Clean up resources.
   */
  dispose(): void {
    this.collaborationEngine = null;
    this.crdtText = null;
  }
}
