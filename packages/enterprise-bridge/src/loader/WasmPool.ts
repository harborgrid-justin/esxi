/**
 * WASM instance pool for high-performance concurrent operations.
 *
 * Maintains a pool of WASM instances to enable parallel processing
 * and avoid blocking on single-threaded WASM operations.
 */

import type { WasmPoolConfig, WasmInstance, BridgeConfig } from '../types';
import { BridgeError } from '../types';
import { WasmLoader } from './WasmLoader';

/**
 * Default pool configuration.
 */
const DEFAULT_POOL_CONFIG: Required<WasmPoolConfig> = {
  initialSize: 2,
  maxSize: 10,
  acquireTimeoutMs: 5000,
  bridgeConfig: {
    enablePerformanceMonitoring: true,
    memoryConfig: {
      initialPoolSize: 10 * 1024 * 1024,
      maxPoolSize: 100 * 1024 * 1024,
      aggressiveGc: false,
    },
    maxConcurrentOperations: 100,
    debugMode: false,
  },
};

/**
 * Pool entry tracking a WASM instance.
 */
interface PoolEntry {
  loader: WasmLoader;
  instance: WasmInstance;
  inUse: boolean;
  createdAt: number;
  lastUsed: number;
  useCount: number;
}

/**
 * WASM instance pool manager.
 */
export class WasmPool {
  private pool: PoolEntry[] = [];
  private config: Required<WasmPoolConfig>;
  private initialized = false;
  private waitQueue: Array<{
    resolve: (instance: WasmInstance) => void;
    reject: (error: Error) => void;
    timeoutId: NodeJS.Timeout;
  }> = [];

  /**
   * Create a new WASM pool.
   *
   * @param config - Pool configuration
   */
  constructor(config: WasmPoolConfig = {}) {
    this.config = {
      ...DEFAULT_POOL_CONFIG,
      ...config,
      bridgeConfig: {
        ...DEFAULT_POOL_CONFIG.bridgeConfig,
        ...config.bridgeConfig,
      },
    };
  }

  /**
   * Initialize the pool with the configured number of instances.
   */
  async initialize(): Promise<void> {
    if (this.initialized) {
      return;
    }

    console.log(
      `[WasmPool] Initializing pool with ${this.config.initialSize} instances`
    );

    try {
      // Create initial instances in parallel
      const createPromises = Array.from(
        { length: this.config.initialSize },
        () => this.createInstance()
      );

      await Promise.all(createPromises);

      this.initialized = true;

      console.log(
        `[WasmPool] Pool initialized successfully with ${this.pool.length} instances`
      );
    } catch (error) {
      throw new BridgeError(
        `Failed to initialize WASM pool: ${error instanceof Error ? error.message : String(error)}`,
        'POOL_INIT_ERROR',
        error
      );
    }
  }

  /**
   * Create a new WASM instance and add it to the pool.
   */
  private async createInstance(): Promise<PoolEntry> {
    const loader = new WasmLoader({
      config: this.config.bridgeConfig,
      verbose: this.config.bridgeConfig.debugMode,
    });

    await loader.initialize();
    const instance = loader.getInstance();

    const entry: PoolEntry = {
      loader,
      instance,
      inUse: false,
      createdAt: Date.now(),
      lastUsed: Date.now(),
      useCount: 0,
    };

    this.pool.push(entry);

    return entry;
  }

  /**
   * Acquire a WASM instance from the pool.
   *
   * If no instances are available, waits for one to become available
   * or creates a new one if the pool is not at max capacity.
   *
   * @throws {BridgeError} If acquisition times out or fails
   */
  async acquire(): Promise<WasmInstance> {
    if (!this.initialized) {
      throw new BridgeError(
        'Pool not initialized. Call initialize() first.',
        'POOL_NOT_INITIALIZED'
      );
    }

    // Try to find an available instance
    const available = this.pool.find(entry => !entry.inUse);

    if (available) {
      available.inUse = true;
      available.lastUsed = Date.now();
      available.useCount++;
      return available.instance;
    }

    // Try to create a new instance if under max capacity
    if (this.pool.length < this.config.maxSize) {
      console.log(
        `[WasmPool] Creating new instance (${this.pool.length + 1}/${this.config.maxSize})`
      );

      const entry = await this.createInstance();
      entry.inUse = true;
      entry.lastUsed = Date.now();
      entry.useCount++;
      return entry.instance;
    }

    // Wait for an instance to become available
    return this.waitForInstance();
  }

  /**
   * Wait for an instance to become available.
   */
  private waitForInstance(): Promise<WasmInstance> {
    return new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        // Remove from wait queue
        const index = this.waitQueue.findIndex(
          item => item.timeoutId === timeoutId
        );
        if (index !== -1) {
          this.waitQueue.splice(index, 1);
        }

        reject(
          new BridgeError(
            `Timeout waiting for WASM instance after ${this.config.acquireTimeoutMs}ms`,
            'ACQUIRE_TIMEOUT'
          )
        );
      }, this.config.acquireTimeoutMs);

      this.waitQueue.push({ resolve, reject, timeoutId });
    });
  }

  /**
   * Release a WASM instance back to the pool.
   *
   * @param instance - The instance to release
   */
  release(instance: WasmInstance): void {
    const entry = this.pool.find(e => e.instance === instance);

    if (!entry) {
      console.warn('[WasmPool] Attempted to release unknown instance');
      return;
    }

    if (!entry.inUse) {
      console.warn('[WasmPool] Attempted to release instance that was not in use');
      return;
    }

    entry.inUse = false;
    entry.lastUsed = Date.now();

    // Service wait queue if any
    if (this.waitQueue.length > 0) {
      const waiter = this.waitQueue.shift();
      if (waiter) {
        clearTimeout(waiter.timeoutId);
        entry.inUse = true;
        entry.useCount++;
        waiter.resolve(instance);
      }
    }
  }

  /**
   * Execute a function with an acquired WASM instance.
   *
   * Automatically acquires and releases the instance.
   *
   * @param fn - Function to execute with the instance
   * @returns Promise resolving to the function's return value
   */
  async execute<T>(
    fn: (instance: WasmInstance) => Promise<T>
  ): Promise<T> {
    const instance = await this.acquire();

    try {
      return await fn(instance);
    } finally {
      this.release(instance);
    }
  }

  /**
   * Get pool statistics.
   */
  getStats(): {
    totalInstances: number;
    availableInstances: number;
    inUseInstances: number;
    waitingRequests: number;
    totalUseCount: number;
  } {
    const inUse = this.pool.filter(e => e.inUse).length;
    const totalUseCount = this.pool.reduce((sum, e) => sum + e.useCount, 0);

    return {
      totalInstances: this.pool.length,
      availableInstances: this.pool.length - inUse,
      inUseInstances: inUse,
      waitingRequests: this.waitQueue.length,
      totalUseCount,
    };
  }

  /**
   * Get detailed information about pool entries.
   */
  getDetailedStats(): Array<{
    inUse: boolean;
    createdAt: number;
    lastUsed: number;
    useCount: number;
    ageMs: number;
  }> {
    const now = Date.now();

    return this.pool.map(entry => ({
      inUse: entry.inUse,
      createdAt: entry.createdAt,
      lastUsed: entry.lastUsed,
      useCount: entry.useCount,
      ageMs: now - entry.createdAt,
    }));
  }

  /**
   * Perform health checks on all instances.
   */
  healthCheck(): {
    healthy: number;
    unhealthy: number;
    details: Array<{ index: number; healthy: boolean }>;
  } {
    const details = this.pool.map((entry, index) => ({
      index,
      healthy: entry.instance.health_check(),
    }));

    const healthy = details.filter(d => d.healthy).length;
    const unhealthy = details.filter(d => !d.healthy).length;

    return { healthy, unhealthy, details };
  }

  /**
   * Cleanup idle instances to free resources.
   *
   * Removes instances that haven't been used recently, keeping
   * at least the minimum configured number.
   *
   * @param maxIdleMs - Maximum idle time before cleanup (default: 5 minutes)
   */
  async cleanup(maxIdleMs = 5 * 60 * 1000): Promise<number> {
    const now = Date.now();
    let removed = 0;

    // Filter out idle instances
    const toRemove = this.pool.filter(
      (entry, index) =>
        !entry.inUse &&
        now - entry.lastUsed > maxIdleMs &&
        this.pool.length - removed > this.config.initialSize
    );

    for (const entry of toRemove) {
      const index = this.pool.indexOf(entry);
      if (index !== -1) {
        this.pool.splice(index, 1);
        entry.loader.dispose();
        removed++;
      }
    }

    if (removed > 0) {
      console.log(`[WasmPool] Cleaned up ${removed} idle instances`);
    }

    return removed;
  }

  /**
   * Dispose of all instances and cleanup the pool.
   */
  async dispose(): Promise<void> {
    console.log('[WasmPool] Disposing pool...');

    // Clear wait queue
    for (const waiter of this.waitQueue) {
      clearTimeout(waiter.timeoutId);
      waiter.reject(new BridgeError('Pool is being disposed', 'POOL_DISPOSED'));
    }
    this.waitQueue = [];

    // Dispose all instances
    for (const entry of this.pool) {
      entry.loader.dispose();
    }

    this.pool = [];
    this.initialized = false;

    console.log('[WasmPool] Pool disposed');
  }
}

/**
 * Singleton pool instance for convenience.
 */
let defaultPool: WasmPool | null = null;

/**
 * Get or create the default WASM pool instance.
 */
export function getDefaultPool(config?: WasmPoolConfig): WasmPool {
  if (!defaultPool) {
    defaultPool = new WasmPool(config);
  }
  return defaultPool;
}

/**
 * Initialize the default WASM pool.
 */
export async function initializePool(config?: WasmPoolConfig): Promise<WasmPool> {
  const pool = getDefaultPool(config);
  await pool.initialize();
  return pool;
}
