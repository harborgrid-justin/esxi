/**
 * Enterprise API Gateway - Health Checker
 *
 * Active and passive health checking for upstream targets
 */

import type { Upstream, Target, HealthCheck, HealthStatus } from '../types';

export class HealthChecker {
  private upstreams: Map<string, Upstream> = new Map();
  private healthStatuses: Map<string, HealthStatus> = new Map();
  private intervals: Map<string, NodeJS.Timeout> = new Map();
  private running = false;

  /**
   * Add an upstream for health checking
   */
  public addUpstream(upstream: Upstream): void {
    this.upstreams.set(upstream.id, upstream);

    // Initialize health status for each target
    for (const target of upstream.targets) {
      this.healthStatuses.set(target.id, {
        targetId: target.id,
        healthy: true,
        consecutiveSuccesses: 0,
        consecutiveFailures: 0,
        lastCheck: Date.now(),
      });
    }

    // Start health checking if configured
    if (upstream.healthChecks && this.running) {
      this.startHealthCheck(upstream);
    }
  }

  /**
   * Remove an upstream from health checking
   */
  public removeUpstream(upstreamId: string): void {
    const upstream = this.upstreams.get(upstreamId);
    if (!upstream) return;

    // Stop health checking
    this.stopHealthCheck(upstreamId);

    // Remove health statuses
    for (const target of upstream.targets) {
      this.healthStatuses.delete(target.id);
    }

    this.upstreams.delete(upstreamId);
  }

  /**
   * Start health checking for all upstreams
   */
  public start(): void {
    this.running = true;

    for (const upstream of this.upstreams.values()) {
      if (upstream.healthChecks) {
        this.startHealthCheck(upstream);
      }
    }
  }

  /**
   * Stop all health checking
   */
  public stop(): void {
    this.running = false;

    for (const upstreamId of this.upstreams.keys()) {
      this.stopHealthCheck(upstreamId);
    }
  }

  /**
   * Start health checking for a specific upstream
   */
  private startHealthCheck(upstream: Upstream): void {
    if (!upstream.healthChecks) return;

    const interval = setInterval(() => {
      this.performHealthChecks(upstream);
    }, upstream.healthChecks.interval);

    this.intervals.set(upstream.id, interval);

    // Perform initial check immediately
    this.performHealthChecks(upstream);
  }

  /**
   * Stop health checking for a specific upstream
   */
  private stopHealthCheck(upstreamId: string): void {
    const interval = this.intervals.get(upstreamId);
    if (interval) {
      clearInterval(interval);
      this.intervals.delete(upstreamId);
    }
  }

  /**
   * Perform health checks on all targets of an upstream
   */
  private async performHealthChecks(upstream: Upstream): Promise<void> {
    if (!upstream.healthChecks) return;

    const checks = upstream.targets.map((target) =>
      this.checkTarget(target, upstream.healthChecks!)
    );

    await Promise.all(checks);
  }

  /**
   * Check health of a single target
   */
  private async checkTarget(target: Target, healthCheck: HealthCheck): Promise<void> {
    const startTime = Date.now();
    const status = this.healthStatuses.get(target.id);

    if (!status) return;

    try {
      const result = await this.performHealthCheck(target, healthCheck);
      const responseTime = Date.now() - startTime;

      status.lastCheck = Date.now();
      status.responseTime = responseTime;

      if (result) {
        status.consecutiveSuccesses++;
        status.consecutiveFailures = 0;
        status.lastError = undefined;

        // Mark as healthy if we've reached the threshold
        if (status.consecutiveSuccesses >= healthCheck.healthyThreshold) {
          if (!status.healthy) {
            console.log(`Target ${target.id} is now healthy`);
          }
          status.healthy = true;
          target.healthy = true;
        }
      } else {
        status.consecutiveFailures++;
        status.consecutiveSuccesses = 0;

        // Mark as unhealthy if we've reached the threshold
        if (status.consecutiveFailures >= healthCheck.unhealthyThreshold) {
          if (status.healthy) {
            console.log(`Target ${target.id} is now unhealthy`);
          }
          status.healthy = false;
          target.healthy = false;
        }
      }
    } catch (error) {
      status.consecutiveFailures++;
      status.consecutiveSuccesses = 0;
      status.lastCheck = Date.now();
      status.lastError = error instanceof Error ? error.message : 'Unknown error';

      // Mark as unhealthy if we've reached the threshold
      if (status.consecutiveFailures >= healthCheck.unhealthyThreshold) {
        if (status.healthy) {
          console.log(`Target ${target.id} is now unhealthy: ${status.lastError}`);
        }
        status.healthy = false;
        target.healthy = false;
      }
    }

    this.healthStatuses.set(target.id, status);
  }

  /**
   * Perform actual health check
   */
  private async performHealthCheck(
    target: Target,
    healthCheck: HealthCheck
  ): Promise<boolean> {
    // This is a simplified version - in production, use axios or node-fetch
    return new Promise((resolve) => {
      // Simulate health check
      setTimeout(() => {
        // 90% success rate for simulation
        resolve(Math.random() > 0.1);
      }, 50);
    });
  }

  /**
   * Record passive health check result (from actual request)
   */
  public recordRequestResult(targetId: string, success: boolean, error?: string): void {
    const status = this.healthStatuses.get(targetId);
    if (!status) return;

    if (success) {
      status.consecutiveSuccesses++;
      status.consecutiveFailures = 0;
    } else {
      status.consecutiveFailures++;
      status.consecutiveSuccesses = 0;
      if (error) {
        status.lastError = error;
      }
    }

    // Update health status based on passive checks
    const upstream = this.findUpstreamByTarget(targetId);
    if (upstream?.healthChecks) {
      const healthCheck = upstream.healthChecks;

      if (success && status.consecutiveSuccesses >= healthCheck.healthyThreshold) {
        status.healthy = true;
        const target = this.findTarget(targetId);
        if (target) target.healthy = true;
      } else if (!success && status.consecutiveFailures >= healthCheck.unhealthyThreshold) {
        status.healthy = false;
        const target = this.findTarget(targetId);
        if (target) target.healthy = false;
      }
    }

    this.healthStatuses.set(targetId, status);
  }

  /**
   * Find upstream by target ID
   */
  private findUpstreamByTarget(targetId: string): Upstream | undefined {
    for (const upstream of this.upstreams.values()) {
      if (upstream.targets.some((t) => t.id === targetId)) {
        return upstream;
      }
    }
    return undefined;
  }

  /**
   * Find target by ID
   */
  private findTarget(targetId: string): Target | undefined {
    for (const upstream of this.upstreams.values()) {
      const target = upstream.targets.find((t) => t.id === targetId);
      if (target) return target;
    }
    return undefined;
  }

  /**
   * Get health status for a target
   */
  public getTargetHealth(targetId: string): HealthStatus | undefined {
    return this.healthStatuses.get(targetId);
  }

  /**
   * Get health status for all targets
   */
  public getHealthStatus(): Map<string, boolean> {
    const status = new Map<string, boolean>();
    for (const [targetId, health] of this.healthStatuses) {
      status.set(targetId, health.healthy);
    }
    return status;
  }

  /**
   * Get detailed health status
   */
  public getDetailedStatus(): Map<string, HealthStatus> {
    return new Map(this.healthStatuses);
  }

  /**
   * Get statistics
   */
  public getStatistics(): {
    total: number;
    healthy: number;
    unhealthy: number;
    upstreams: number;
  } {
    let healthy = 0;
    let unhealthy = 0;

    for (const status of this.healthStatuses.values()) {
      if (status.healthy) {
        healthy++;
      } else {
        unhealthy++;
      }
    }

    return {
      total: this.healthStatuses.size,
      healthy,
      unhealthy,
      upstreams: this.upstreams.size,
    };
  }
}
