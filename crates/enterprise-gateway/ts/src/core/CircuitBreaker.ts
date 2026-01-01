/**
 * Enterprise API Gateway - Circuit Breaker
 *
 * Fault tolerance and cascade failure prevention
 */

import type { CircuitBreakerConfig, CircuitBreakerState, CircuitState } from '../types';
import { CircuitBreakerError } from '../types';

export class CircuitBreaker {
  private states: Map<string, CircuitBreakerState> = new Map();
  private config: CircuitBreakerConfig;

  constructor(config: CircuitBreakerConfig) {
    this.config = config;
  }

  /**
   * Check if a request can be executed
   */
  public canExecute(serviceId: string): boolean {
    const state = this.getState(serviceId);

    switch (state.state) {
      case 'CLOSED':
        return true;

      case 'OPEN':
        // Check if we should transition to HALF_OPEN
        if (state.nextAttemptTime && Date.now() >= state.nextAttemptTime) {
          this.transitionTo(serviceId, 'HALF_OPEN');
          return true;
        }
        return false;

      case 'HALF_OPEN':
        return true;

      default:
        return true;
    }
  }

  /**
   * Record a successful request
   */
  public recordSuccess(serviceId: string): void {
    const state = this.getState(serviceId);
    state.successes++;
    state.totalRequests++;

    switch (state.state) {
      case 'HALF_OPEN':
        // Check if we've had enough successes to close the circuit
        if (state.successes >= this.config.successThreshold) {
          this.transitionTo(serviceId, 'CLOSED');
          state.failures = 0;
          state.successes = 0;
        }
        break;

      case 'CLOSED':
        // Reset failure count on success
        state.failures = 0;
        break;
    }

    this.updateState(serviceId, state);
  }

  /**
   * Record a failed request
   */
  public recordFailure(serviceId: string): void {
    const state = this.getState(serviceId);
    state.failures++;
    state.totalRequests++;
    state.lastFailureTime = Date.now();

    switch (state.state) {
      case 'CLOSED':
        // Check if we should open the circuit
        if (this.shouldOpen(state)) {
          this.transitionTo(serviceId, 'OPEN');
          state.nextAttemptTime = Date.now() + this.config.timeout;
        }
        break;

      case 'HALF_OPEN':
        // Any failure in HALF_OPEN state reopens the circuit
        this.transitionTo(serviceId, 'OPEN');
        state.nextAttemptTime = Date.now() + this.config.timeout;
        state.successes = 0;
        break;
    }

    this.updateState(serviceId, state);
  }

  /**
   * Determine if circuit should open
   */
  private shouldOpen(state: CircuitBreakerState): boolean {
    // Need minimum volume before we can open
    if (state.totalRequests < this.config.volumeThreshold) {
      return false;
    }

    // Check failure rate within monitoring period
    const failureRate = state.failures / state.totalRequests;
    const thresholdRate = this.config.failureThreshold / 100;

    return failureRate >= thresholdRate;
  }

  /**
   * Transition to a new state
   */
  private transitionTo(serviceId: string, newState: CircuitState): void {
    const state = this.getState(serviceId);
    const oldState = state.state;

    state.state = newState;

    console.log(`Circuit breaker for ${serviceId}: ${oldState} -> ${newState}`);

    // Reset counters on state transition
    if (newState === 'CLOSED') {
      state.failures = 0;
      state.successes = 0;
      state.totalRequests = 0;
      state.lastFailureTime = undefined;
      state.nextAttemptTime = undefined;
    } else if (newState === 'HALF_OPEN') {
      state.successes = 0;
      state.failures = 0;
    }

    this.updateState(serviceId, state);
  }

  /**
   * Get state for a service
   */
  public getState(serviceId: string): CircuitBreakerState {
    let state = this.states.get(serviceId);

    if (!state) {
      state = {
        state: 'CLOSED',
        failures: 0,
        successes: 0,
        totalRequests: 0,
      };
      this.states.set(serviceId, state);
    }

    return state;
  }

  /**
   * Update state
   */
  private updateState(serviceId: string, state: CircuitBreakerState): void {
    this.states.set(serviceId, state);
  }

  /**
   * Force open a circuit
   */
  public forceOpen(serviceId: string): void {
    const state = this.getState(serviceId);
    state.state = 'OPEN';
    state.nextAttemptTime = Date.now() + this.config.timeout;
    this.updateState(serviceId, state);
  }

  /**
   * Force close a circuit
   */
  public forceClose(serviceId: string): void {
    const state = this.getState(serviceId);
    state.state = 'CLOSED';
    state.failures = 0;
    state.successes = 0;
    state.totalRequests = 0;
    state.lastFailureTime = undefined;
    state.nextAttemptTime = undefined;
    this.updateState(serviceId, state);
  }

  /**
   * Get all states
   */
  public getStates(): Map<string, CircuitBreakerState> {
    return new Map(this.states);
  }

  /**
   * Reset all circuits
   */
  public reset(): void {
    this.states.clear();
  }

  /**
   * Get statistics
   */
  public getStatistics(): {
    total: number;
    closed: number;
    open: number;
    halfOpen: number;
  } {
    const stats = {
      total: this.states.size,
      closed: 0,
      open: 0,
      halfOpen: 0,
    };

    for (const state of this.states.values()) {
      switch (state.state) {
        case 'CLOSED':
          stats.closed++;
          break;
        case 'OPEN':
          stats.open++;
          break;
        case 'HALF_OPEN':
          stats.halfOpen++;
          break;
      }
    }

    return stats;
  }
}
