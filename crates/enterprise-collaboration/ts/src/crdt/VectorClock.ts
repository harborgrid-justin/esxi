/**
 * Vector Clock Implementation for Distributed Systems
 * Provides causality tracking and conflict detection
 */

export type VectorClockMap = Record<string, number>;

export enum ClockComparison {
  BEFORE = 'before',
  AFTER = 'after',
  CONCURRENT = 'concurrent',
  EQUAL = 'equal',
}

export class VectorClock {
  private clock: VectorClockMap;

  constructor(initial: VectorClockMap = {}) {
    this.clock = { ...initial };
  }

  /**
   * Increment the clock for a specific node
   */
  increment(nodeId: string): void {
    this.clock[nodeId] = (this.clock[nodeId] || 0) + 1;
  }

  /**
   * Get the current value for a node
   */
  get(nodeId: string): number {
    return this.clock[nodeId] || 0;
  }

  /**
   * Set the value for a specific node
   */
  set(nodeId: string, value: number): void {
    this.clock[nodeId] = value;
  }

  /**
   * Merge with another vector clock (take maximum of each component)
   */
  merge(other: VectorClock): VectorClock {
    const merged: VectorClockMap = { ...this.clock };

    for (const [nodeId, value] of Object.entries(other.toObject())) {
      merged[nodeId] = Math.max(merged[nodeId] || 0, value);
    }

    return new VectorClock(merged);
  }

  /**
   * Compare this clock with another to determine causal relationship
   */
  compare(other: VectorClock): ClockComparison {
    const thisClock = this.clock;
    const otherClock = other.toObject();

    const allNodes = new Set([
      ...Object.keys(thisClock),
      ...Object.keys(otherClock),
    ]);

    let hasGreater = false;
    let hasLess = false;

    for (const nodeId of allNodes) {
      const thisValue = thisClock[nodeId] || 0;
      const otherValue = otherClock[nodeId] || 0;

      if (thisValue > otherValue) {
        hasGreater = true;
      } else if (thisValue < otherValue) {
        hasLess = true;
      }
    }

    if (!hasGreater && !hasLess) {
      return ClockComparison.EQUAL;
    }
    if (hasGreater && !hasLess) {
      return ClockComparison.AFTER;
    }
    if (hasLess && !hasGreater) {
      return ClockComparison.BEFORE;
    }
    return ClockComparison.CONCURRENT;
  }

  /**
   * Check if this clock happens before another
   */
  happensBefore(other: VectorClock): boolean {
    return this.compare(other) === ClockComparison.BEFORE;
  }

  /**
   * Check if this clock happens after another
   */
  happensAfter(other: VectorClock): boolean {
    return this.compare(other) === ClockComparison.AFTER;
  }

  /**
   * Check if this clock is concurrent with another
   */
  isConcurrent(other: VectorClock): boolean {
    return this.compare(other) === ClockComparison.CONCURRENT;
  }

  /**
   * Check if this clock is equal to another
   */
  isEqual(other: VectorClock): boolean {
    return this.compare(other) === ClockComparison.EQUAL;
  }

  /**
   * Clone this vector clock
   */
  clone(): VectorClock {
    return new VectorClock({ ...this.clock });
  }

  /**
   * Convert to plain object
   */
  toObject(): VectorClockMap {
    return { ...this.clock };
  }

  /**
   * Convert to JSON string
   */
  toJSON(): string {
    return JSON.stringify(this.clock);
  }

  /**
   * Create from JSON string
   */
  static fromJSON(json: string): VectorClock {
    return new VectorClock(JSON.parse(json));
  }

  /**
   * Get all node IDs in the clock
   */
  getNodeIds(): string[] {
    return Object.keys(this.clock);
  }

  /**
   * Get the sum of all clock values
   */
  getSum(): number {
    return Object.values(this.clock).reduce((sum, val) => sum + val, 0);
  }

  /**
   * Reset the clock
   */
  reset(): void {
    this.clock = {};
  }

  /**
   * Create a new vector clock
   */
  static create(nodeId?: string): VectorClock {
    const clock = new VectorClock();
    if (nodeId) {
      clock.increment(nodeId);
    }
    return clock;
  }
}
