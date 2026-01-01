/**
 * TypeScript type definitions matching Rust WASM bridge types.
 *
 * These types must stay in sync with the Rust types defined in
 * crates/meridian-wasm-bridge/src/types.rs
 */

/**
 * Configuration for the WASM bridge.
 */
export interface BridgeConfig {
  enablePerformanceMonitoring: boolean;
  memoryConfig: MemoryConfig;
  maxConcurrentOperations?: number;
  debugMode?: boolean;
}

/**
 * Memory configuration for the WASM bridge.
 */
export interface MemoryConfig {
  initialPoolSize?: number; // Bytes
  maxPoolSize?: number; // Bytes
  aggressiveGc?: boolean;
}

/**
 * Bridge statistics and performance metrics.
 */
export interface BridgeStats {
  version: string;
  memoryUsage: number; // Bytes
  activeOperations: number;
  uptimeMs: number;
}

/**
 * Memory usage information.
 */
export interface MemoryUsage {
  totalAllocated: number;
  used: number;
  available: number;
  allocations: number;
}

/**
 * Generic operation result wrapper.
 */
export interface OperationResult<T> {
  success: boolean;
  data?: T;
  error?: string;
  durationMs?: number;
}

/**
 * CAD geometry representation.
 */
export interface CadGeometry {
  geometryType: string;
  coordinates: number[][];
  properties: Record<string, unknown>;
  bbox?: number[]; // [minX, minY, maxX, maxY]
}

/**
 * Compression operation parameters.
 */
export interface CompressionParams {
  algorithm: 'gzip' | 'brotli' | 'zstd' | 'lz4';
  level: number; // 1-9
  useDictionary?: boolean;
  dictionary?: Uint8Array;
}

/**
 * Query optimization parameters.
 */
export interface QueryParams {
  query: string;
  params: unknown[];
  optimize?: boolean;
  timeoutMs?: number;
}

/**
 * Query execution result.
 */
export interface QueryExecutionResult {
  rows: Record<string, unknown>[];
  rowCount: number;
  columns: string[];
  executionTimeMs: number;
  optimized: boolean;
}

/**
 * Query plan information.
 */
export interface QueryPlan {
  optimizedQuery: string;
  estimatedCost: number;
  estimatedRows: number;
  planSteps: PlanStep[];
}

/**
 * Query plan step.
 */
export interface PlanStep {
  stepType: string;
  description: string;
  estimatedCost: number;
}

/**
 * Collaboration event.
 */
export interface CollaborationEvent {
  eventType: string;
  userId: string;
  timestamp: number;
  payload: unknown;
  vectorClock?: number[];
}

/**
 * Transformed operation result.
 */
export interface TransformedOperation {
  eventType: string;
  payload: unknown;
  transformed: boolean;
  originalUser: string;
}

/**
 * Merged operation result.
 */
export interface MergedOperation {
  eventType: string;
  payload: unknown;
  mergedFrom: string[];
  resolutionStrategy: string;
}

/**
 * Presence update event.
 */
export interface PresenceUpdate {
  userId: string;
  timestamp: number;
  data: unknown;
}

/**
 * Security validation parameters.
 */
export interface SecurityParams {
  checkType: 'xss' | 'sql_injection' | 'csrf' | 'path_traversal' | 'command_injection';
  input: string;
  context?: Record<string, unknown>;
}

/**
 * Security validation result.
 */
export interface SecurityResult {
  isSafe: boolean;
  threats: SecurityThreat[];
  sanitized?: string;
}

/**
 * Security threat information.
 */
export interface SecurityThreat {
  threatType: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  location?: LocationInfo;
}

/**
 * Location information for errors and threats.
 */
export interface LocationInfo {
  line: number;
  column: number;
  length: number;
}

/**
 * Password validation result.
 */
export interface PasswordValidationResult {
  valid: boolean;
  strength: 'weak' | 'medium' | 'strong' | 'very_strong';
  issues: string[];
  score: number; // 0-100
}

/**
 * CSP validation result.
 */
export interface CspValidationResult {
  valid: boolean;
  warnings: string[];
  errors: string[];
  directives: string[];
}

/**
 * Compression recommendation.
 */
export interface CompressionRecommendation {
  algorithm: string;
  level: number;
  entropy: number;
}

/**
 * Query statistics.
 */
export interface QueryStats {
  totalQueries: number;
  cacheHits: number;
  cacheMisses: number;
  avgExecutionTimeMs: number;
}

/**
 * Query validation result.
 */
export interface QueryValidationResult {
  valid: boolean;
  errors: string[];
}

/**
 * Query explanation.
 */
export interface QueryExplanation {
  query: string;
  plan: ExplanationStep[];
  totalCost: number;
}

/**
 * Query explanation step.
 */
export interface ExplanationStep {
  operation: string;
  details: string;
  cost: number;
  rows: number;
}

/**
 * WASM module initialization options.
 */
export interface WasmInitOptions {
  /**
   * Path to the WASM file.
   * @default './wasm/meridian_wasm_bridge_bg.wasm'
   */
  wasmPath?: string;

  /**
   * Bridge configuration.
   */
  config?: BridgeConfig;

  /**
   * Enable verbose logging.
   * @default false
   */
  verbose?: boolean;
}

/**
 * WASM pool configuration.
 */
export interface WasmPoolConfig {
  /**
   * Initial number of WASM instances to create.
   * @default 2
   */
  initialSize?: number;

  /**
   * Maximum number of WASM instances in the pool.
   * @default 10
   */
  maxSize?: number;

  /**
   * Timeout for acquiring a WASM instance from the pool (ms).
   * @default 5000
   */
  acquireTimeoutMs?: number;

  /**
   * Bridge configuration for each instance.
   */
  bridgeConfig?: BridgeConfig;
}

/**
 * Error types that can be thrown by the bridge.
 */
export class BridgeError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly details?: unknown
  ) {
    super(message);
    this.name = 'BridgeError';
  }
}

/**
 * WASM instance interface exported by the Rust crate.
 */
export interface WasmInstance {
  initialize(config: unknown): Promise<boolean>;
  version(): string;
  get_stats(): Promise<unknown>;
  health_check(): boolean;
  reset(): Promise<void>;
  memory: WebAssembly.Memory;
}
