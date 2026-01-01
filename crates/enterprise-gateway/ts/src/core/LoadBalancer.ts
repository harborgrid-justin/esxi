/**
 * Enterprise API Gateway - Load Balancer
 *
 * Multiple load balancing algorithms for traffic distribution
 */

import type { Upstream, Target, GatewayRequest, LoadBalancingAlgorithm } from '../types';
import { createHash } from 'crypto';

export class LoadBalancer {
  private roundRobinCounters: Map<string, number> = new Map();
  private connectionCounts: Map<string, number> = new Map();
  private consistentHashRing: Map<string, Target[]> = new Map();
  private readonly VIRTUAL_NODES = 150;

  /**
   * Select a target based on the upstream's load balancing algorithm
   */
  public selectTarget(upstream: Upstream, request?: GatewayRequest): Target | null {
    const healthyTargets = upstream.targets.filter((t) => t.healthy);

    if (healthyTargets.length === 0) {
      return null;
    }

    if (healthyTargets.length === 1) {
      return healthyTargets[0]!;
    }

    switch (upstream.algorithm) {
      case 'round-robin':
        return this.roundRobin(upstream.id, healthyTargets);

      case 'weighted-round-robin':
        return this.weightedRoundRobin(upstream.id, healthyTargets);

      case 'least-connections':
        return this.leastConnections(healthyTargets);

      case 'ip-hash':
        return this.ipHash(healthyTargets, request?.ip || '');

      case 'random':
        return this.random(healthyTargets);

      case 'consistent-hash':
        return this.consistentHash(upstream.id, healthyTargets, request?.path || '');

      default:
        return this.roundRobin(upstream.id, healthyTargets);
    }
  }

  /**
   * Round-robin load balancing
   */
  private roundRobin(upstreamId: string, targets: Target[]): Target {
    const counter = this.roundRobinCounters.get(upstreamId) || 0;
    const index = counter % targets.length;
    this.roundRobinCounters.set(upstreamId, counter + 1);
    return targets[index]!;
  }

  /**
   * Weighted round-robin load balancing
   */
  private weightedRoundRobin(upstreamId: string, targets: Target[]): Target {
    // Calculate total weight
    const totalWeight = targets.reduce((sum, t) => sum + t.weight, 0);

    // Get current counter
    const counter = this.roundRobinCounters.get(upstreamId) || 0;

    // Find target based on weighted distribution
    let cumulativeWeight = 0;
    const point = counter % totalWeight;

    for (const target of targets) {
      cumulativeWeight += target.weight;
      if (point < cumulativeWeight) {
        this.roundRobinCounters.set(upstreamId, counter + 1);
        return target;
      }
    }

    // Fallback (shouldn't reach here)
    this.roundRobinCounters.set(upstreamId, counter + 1);
    return targets[0]!;
  }

  /**
   * Least connections load balancing
   */
  private leastConnections(targets: Target[]): Target {
    return targets.reduce((min, target) =>
      target.activeConnections < min.activeConnections ? target : min
    );
  }

  /**
   * IP hash load balancing
   */
  private ipHash(targets: Target[], ip: string): Target {
    const hash = this.hashString(ip);
    const index = hash % targets.length;
    return targets[index]!;
  }

  /**
   * Random load balancing
   */
  private random(targets: Target[]): Target {
    const index = Math.floor(Math.random() * targets.length);
    return targets[index]!;
  }

  /**
   * Consistent hash load balancing
   */
  private consistentHash(upstreamId: string, targets: Target[], key: string): Target {
    // Build or get hash ring
    let ring = this.consistentHashRing.get(upstreamId);
    if (!ring || ring.length !== targets.length * this.VIRTUAL_NODES) {
      ring = this.buildHashRing(targets);
      this.consistentHashRing.set(upstreamId, ring);
    }

    // Hash the key
    const hash = this.hashString(key);

    // Find the first virtual node >= hash
    let selectedTarget = ring[0]!;
    for (const target of ring) {
      const targetHash = this.hashString(`${target.id}-${target.url}`);
      if (targetHash >= hash) {
        selectedTarget = target;
        break;
      }
    }

    return selectedTarget;
  }

  /**
   * Build consistent hash ring with virtual nodes
   */
  private buildHashRing(targets: Target[]): Target[] {
    const ring: Array<{ hash: number; target: Target }> = [];

    for (const target of targets) {
      // Create virtual nodes for better distribution
      for (let i = 0; i < this.VIRTUAL_NODES; i++) {
        const virtualKey = `${target.id}-${i}`;
        const hash = this.hashString(virtualKey);
        ring.push({ hash, target });
      }
    }

    // Sort by hash
    ring.sort((a, b) => a.hash - b.hash);

    return ring.map((node) => node.target);
  }

  /**
   * Hash a string to a number
   */
  private hashString(str: string): number {
    const hash = createHash('md5').update(str).digest('hex');
    return parseInt(hash.substring(0, 8), 16);
  }

  /**
   * Increment connection count for a target
   */
  public incrementConnections(targetId: string): void {
    const count = this.connectionCounts.get(targetId) || 0;
    this.connectionCounts.set(targetId, count + 1);
  }

  /**
   * Decrement connection count for a target
   */
  public decrementConnections(targetId: string): void {
    const count = this.connectionCounts.get(targetId) || 0;
    this.connectionCounts.set(targetId, Math.max(0, count - 1));
  }

  /**
   * Get connection count for a target
   */
  public getConnectionCount(targetId: string): number {
    return this.connectionCounts.get(targetId) || 0;
  }

  /**
   * Clear all state
   */
  public clear(): void {
    this.roundRobinCounters.clear();
    this.connectionCounts.clear();
    this.consistentHashRing.clear();
  }

  /**
   * Get statistics
   */
  public getStatistics(): {
    upstreams: number;
    connections: Map<string, number>;
  } {
    return {
      upstreams: this.roundRobinCounters.size,
      connections: new Map(this.connectionCounts),
    };
  }
}
