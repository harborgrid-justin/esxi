/**
 * WASM module loader for the enterprise bridge.
 *
 * Handles loading and initializing the WASM module with proper error handling
 * and configuration.
 */

import type { BridgeConfig, WasmInitOptions, WasmInstance, BridgeStats } from '../types';
import { BridgeError } from '../types';

/**
 * Default WASM module path.
 */
const DEFAULT_WASM_PATH = './wasm/meridian_wasm_bridge_bg.wasm';

/**
 * Default bridge configuration.
 */
const DEFAULT_CONFIG: BridgeConfig = {
  enablePerformanceMonitoring: true,
  memoryConfig: {
    initialPoolSize: 10 * 1024 * 1024, // 10 MB
    maxPoolSize: 100 * 1024 * 1024, // 100 MB
    aggressiveGc: false,
  },
  maxConcurrentOperations: 100,
  debugMode: false,
};

/**
 * WASM module loader and manager.
 */
export class WasmLoader {
  private wasmModule: WasmInstance | null = null;
  private initialized = false;
  private initPromise: Promise<void> | null = null;

  /**
   * Create a new WASM loader instance.
   *
   * @param options - Initialization options
   */
  constructor(private readonly options: WasmInitOptions = {}) {}

  /**
   * Initialize the WASM module.
   *
   * This method is idempotent - calling it multiple times will only initialize once.
   *
   * @throws {BridgeError} If initialization fails
   */
  async initialize(): Promise<void> {
    // Return existing initialization promise if already initializing
    if (this.initPromise) {
      return this.initPromise;
    }

    // Return immediately if already initialized
    if (this.initialized && this.wasmModule) {
      return;
    }

    // Create new initialization promise
    this.initPromise = this.doInitialize();

    try {
      await this.initPromise;
    } finally {
      this.initPromise = null;
    }
  }

  /**
   * Internal initialization logic.
   */
  private async doInitialize(): Promise<void> {
    try {
      const wasmPath = this.options.wasmPath || DEFAULT_WASM_PATH;

      if (this.options.verbose) {
        console.log(`[WasmLoader] Loading WASM module from: ${wasmPath}`);
      }

      // Load the WASM module
      // In production, this would use the generated wasm-pack bindings
      const wasmModule = await this.loadWasmModule(wasmPath);

      this.wasmModule = wasmModule;

      // Initialize with configuration
      const config = {
        ...DEFAULT_CONFIG,
        ...this.options.config,
        memoryConfig: {
          ...DEFAULT_CONFIG.memoryConfig,
          ...this.options.config?.memoryConfig,
        },
      };

      const success = await wasmModule.initialize(config);

      if (!success) {
        throw new BridgeError(
          'WASM module initialization returned false',
          'INIT_FAILED'
        );
      }

      this.initialized = true;

      if (this.options.verbose) {
        console.log(`[WasmLoader] WASM module initialized successfully`);
        console.log(`[WasmLoader] Version: ${wasmModule.version()}`);
      }
    } catch (error) {
      this.initialized = false;
      this.wasmModule = null;

      throw new BridgeError(
        `Failed to initialize WASM module: ${error instanceof Error ? error.message : String(error)}`,
        'INIT_ERROR',
        error
      );
    }
  }

  /**
   * Load the WASM module from the specified path.
   *
   * This is a placeholder - in production, this would import the actual
   * wasm-pack generated module.
   */
  private async loadWasmModule(path: string): Promise<WasmInstance> {
    try {
      // In production, this would be:
      // import init, * as wasm from '../../wasm/meridian_wasm_bridge';
      // await init(path);
      // return wasm;

      // For now, create a mock instance for type checking
      throw new Error(
        'WASM module not built. Run `npm run build:wasm` first.'
      );
    } catch (error) {
      throw new BridgeError(
        `Failed to load WASM module: ${error instanceof Error ? error.message : String(error)}`,
        'LOAD_ERROR',
        error
      );
    }
  }

  /**
   * Get the WASM module instance.
   *
   * @throws {BridgeError} If the module is not initialized
   */
  getInstance(): WasmInstance {
    if (!this.initialized || !this.wasmModule) {
      throw new BridgeError(
        'WASM module not initialized. Call initialize() first.',
        'NOT_INITIALIZED'
      );
    }

    return this.wasmModule;
  }

  /**
   * Check if the WASM module is initialized.
   */
  isInitialized(): boolean {
    return this.initialized && this.wasmModule !== null;
  }

  /**
   * Get the WASM module version.
   *
   * @throws {BridgeError} If the module is not initialized
   */
  getVersion(): string {
    return this.getInstance().version();
  }

  /**
   * Get bridge statistics.
   *
   * @throws {BridgeError} If the module is not initialized
   */
  async getStats(): Promise<BridgeStats> {
    const instance = this.getInstance();
    const stats = await instance.get_stats();
    return stats as BridgeStats;
  }

  /**
   * Perform a health check on the WASM module.
   *
   * @throws {BridgeError} If the module is not initialized
   */
  healthCheck(): boolean {
    return this.getInstance().health_check();
  }

  /**
   * Reset the WASM module state.
   *
   * This clears all caches and resets memory pools.
   *
   * @throws {BridgeError} If the module is not initialized
   */
  async reset(): Promise<void> {
    const instance = this.getInstance();
    await instance.reset();

    if (this.options.verbose) {
      console.log('[WasmLoader] WASM module reset completed');
    }
  }

  /**
   * Cleanup and dispose of the WASM module.
   */
  dispose(): void {
    this.wasmModule = null;
    this.initialized = false;
    this.initPromise = null;

    if (this.options.verbose) {
      console.log('[WasmLoader] WASM module disposed');
    }
  }
}

/**
 * Singleton instance for convenience.
 */
let defaultLoader: WasmLoader | null = null;

/**
 * Get or create the default WASM loader instance.
 */
export function getDefaultLoader(options?: WasmInitOptions): WasmLoader {
  if (!defaultLoader) {
    defaultLoader = new WasmLoader(options);
  }
  return defaultLoader;
}

/**
 * Initialize the default WASM loader.
 *
 * This is a convenience function for simple use cases.
 */
export async function initializeWasm(options?: WasmInitOptions): Promise<WasmLoader> {
  const loader = getDefaultLoader(options);
  await loader.initialize();
  return loader;
}
