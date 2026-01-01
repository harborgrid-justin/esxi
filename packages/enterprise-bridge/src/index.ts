/**
 * Enterprise WASM Bridge - Main Entry Point
 *
 * TypeScript-Rust WASM Bridge for $983M Enterprise SaaS Platform
 *
 * @packageDocumentation
 */

// Export types
export * from './types';

// Export loaders
export {
  WasmLoader,
  getDefaultLoader,
  initializeWasm,
} from './loader/WasmLoader';

export {
  WasmPool,
  getDefaultPool,
  initializePool,
} from './loader/WasmPool';

// Export services
export { CadBridge } from './services/CadBridge';
export { CompressionBridge } from './services/CompressionBridge';
export { QueryBridge } from './services/QueryBridge';
export { CollaborationBridge } from './services/CollaborationBridge';
export { SecurityBridge } from './services/SecurityBridge';

// Export utilities
export * from './utils/memory';

// Re-export for convenience
import { WasmLoader, initializeWasm } from './loader/WasmLoader';
import { WasmPool, initializePool } from './loader/WasmPool';
import { CadBridge } from './services/CadBridge';
import { CompressionBridge } from './services/CompressionBridge';
import { QueryBridge } from './services/QueryBridge';
import { CollaborationBridge } from './services/CollaborationBridge';
import { SecurityBridge } from './services/SecurityBridge';
import type { WasmInitOptions, WasmPoolConfig } from './types';

/**
 * Enterprise Bridge facade providing easy access to all services.
 *
 * @example
 * ```typescript
 * import { EnterpriseBridge } from '@esxi/enterprise-bridge';
 *
 * const bridge = new EnterpriseBridge();
 * await bridge.initialize();
 *
 * // Use CAD services
 * const result = await bridge.cad.validateGeometry(geometry);
 *
 * // Use compression services
 * const compressed = await bridge.compression.compress(data, params);
 *
 * // Use query services
 * const queryResult = await bridge.query.execute(params);
 *
 * // Use collaboration services
 * const event = await bridge.collaboration.applyLocalOperation('insert', payload);
 *
 * // Use security services
 * const validationResult = await bridge.security.validateXss(input);
 * ```
 */
export class EnterpriseBridge {
  private loader: WasmLoader | null = null;
  private pool: WasmPool | null = null;
  private _cad: CadBridge | null = null;
  private _compression: CompressionBridge | null = null;
  private _query: QueryBridge | null = null;
  private _collaboration: CollaborationBridge | null = null;
  private _security: SecurityBridge | null = null;

  /**
   * Create a new Enterprise Bridge instance.
   *
   * @param options - Initialization options
   */
  constructor(
    private readonly options: {
      wasmOptions?: WasmInitOptions;
      poolOptions?: WasmPoolConfig;
      usePool?: boolean;
      userId?: string;
      queryCache?: boolean;
      strictSecurity?: boolean;
    } = {}
  ) {}

  /**
   * Initialize the bridge.
   *
   * This must be called before using any services.
   */
  async initialize(): Promise<void> {
    if (this.options.usePool) {
      // Use pooled instances for better concurrency
      this.pool = await initializePool(this.options.poolOptions);

      // Create a loader for service initialization
      // Services will use the pool for operations
      this.loader = new WasmLoader(this.options.wasmOptions);
      await this.loader.initialize();
    } else {
      // Use single instance
      this.loader = await initializeWasm(this.options.wasmOptions);
    }

    // Initialize services
    this._cad = new CadBridge(this.loader);
    this._compression = new CompressionBridge(this.loader);
    this._query = new QueryBridge(
      this.loader,
      this.options.queryCache ?? true
    );
    this._collaboration = new CollaborationBridge(
      this.loader,
      this.options.userId || 'default-user'
    );
    this._security = new SecurityBridge(
      this.loader,
      this.options.strictSecurity ?? true
    );
  }

  /**
   * CAD service accessor.
   */
  get cad(): CadBridge {
    if (!this._cad) {
      throw new Error('Bridge not initialized. Call initialize() first.');
    }
    return this._cad;
  }

  /**
   * Compression service accessor.
   */
  get compression(): CompressionBridge {
    if (!this._compression) {
      throw new Error('Bridge not initialized. Call initialize() first.');
    }
    return this._compression;
  }

  /**
   * Query service accessor.
   */
  get query(): QueryBridge {
    if (!this._query) {
      throw new Error('Bridge not initialized. Call initialize() first.');
    }
    return this._query;
  }

  /**
   * Collaboration service accessor.
   */
  get collaboration(): CollaborationBridge {
    if (!this._collaboration) {
      throw new Error('Bridge not initialized. Call initialize() first.');
    }
    return this._collaboration;
  }

  /**
   * Security service accessor.
   */
  get security(): SecurityBridge {
    if (!this._security) {
      throw new Error('Bridge not initialized. Call initialize() first.');
    }
    return this._security;
  }

  /**
   * Get the WASM loader instance.
   */
  getLoader(): WasmLoader {
    if (!this.loader) {
      throw new Error('Bridge not initialized. Call initialize() first.');
    }
    return this.loader;
  }

  /**
   * Get the WASM pool instance (if using pool).
   */
  getPool(): WasmPool | null {
    return this.pool;
  }

  /**
   * Get bridge version.
   */
  getVersion(): string {
    if (!this.loader) {
      throw new Error('Bridge not initialized. Call initialize() first.');
    }
    return this.loader.getVersion();
  }

  /**
   * Perform a health check on the bridge.
   */
  healthCheck(): boolean {
    if (!this.loader) {
      return false;
    }
    return this.loader.healthCheck();
  }

  /**
   * Get bridge statistics.
   */
  async getStats() {
    if (!this.loader) {
      throw new Error('Bridge not initialized. Call initialize() first.');
    }

    const bridgeStats = await this.loader.getStats();
    const poolStats = this.pool?.getStats();

    return {
      bridge: bridgeStats,
      pool: poolStats,
    };
  }

  /**
   * Reset the bridge state.
   */
  async reset(): Promise<void> {
    if (this.loader) {
      await this.loader.reset();
    }
  }

  /**
   * Dispose of the bridge and cleanup resources.
   */
  async dispose(): Promise<void> {
    // Dispose services
    this._cad?.dispose();
    this._compression?.dispose();
    this._query?.dispose();
    this._collaboration?.dispose();
    this._security?.dispose();

    // Dispose loader and pool
    if (this.pool) {
      await this.pool.dispose();
    }

    if (this.loader) {
      this.loader.dispose();
    }

    // Clear references
    this._cad = null;
    this._compression = null;
    this._query = null;
    this._collaboration = null;
    this._security = null;
    this.loader = null;
    this.pool = null;
  }
}

/**
 * Default export for convenience.
 */
export default EnterpriseBridge;
