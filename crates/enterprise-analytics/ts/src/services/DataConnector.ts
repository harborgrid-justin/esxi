/**
 * Multi-Source Data Connector Service
 * @module @harborgrid/enterprise-analytics/services
 */

import type { DataSource, DataSourceType, QueryResult } from '../types';

export interface ConnectionOptions {
  timeout?: number;
  retryAttempts?: number;
  retryDelay?: number;
  poolSize?: number;
}

export class DataConnector {
  private connections: Map<string, Connection>;
  private connectionPool: Map<string, ConnectionPool>;

  constructor() {
    this.connections = new Map();
    this.connectionPool = new Map();
  }

  // ============================================================================
  // Connection Management
  // ============================================================================

  async connect(dataSource: DataSource, options?: ConnectionOptions): Promise<void> {
    const connection = await this.createConnection(dataSource, options);
    this.connections.set(dataSource.id, connection);

    // Initialize connection pool if configured
    if (options?.poolSize && options.poolSize > 1) {
      const pool = await this.createConnectionPool(dataSource, options);
      this.connectionPool.set(dataSource.id, pool);
    }
  }

  async disconnect(dataSourceId: string): Promise<void> {
    const connection = this.connections.get(dataSourceId);
    if (connection) {
      await connection.close();
      this.connections.delete(dataSourceId);
    }

    const pool = this.connectionPool.get(dataSourceId);
    if (pool) {
      await pool.close();
      this.connectionPool.delete(dataSourceId);
    }
  }

  async disconnectAll(): Promise<void> {
    const disconnectPromises = Array.from(this.connections.keys()).map((id) =>
      this.disconnect(id)
    );
    await Promise.all(disconnectPromises);
  }

  isConnected(dataSourceId: string): boolean {
    return this.connections.has(dataSourceId);
  }

  // ============================================================================
  // Query Execution
  // ============================================================================

  async execute<T = Record<string, unknown>>(
    dataSourceId: string,
    query: string,
    params?: unknown[]
  ): Promise<QueryResult<T>> {
    const connection = this.getConnection(dataSourceId);
    if (!connection) {
      throw new Error(`No connection found for data source: ${dataSourceId}`);
    }

    const startTime = Date.now();

    try {
      const result = await connection.execute<T>(query, params);
      const executionTime = Date.now() - startTime;

      return {
        ...result,
        executionTime,
      };
    } catch (error) {
      throw new DataConnectorError(
        `Query execution failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        dataSourceId,
        error
      );
    }
  }

  async executeMany<T = Record<string, unknown>>(
    dataSourceId: string,
    queries: Array<{ query: string; params?: unknown[] }>
  ): Promise<QueryResult<T>[]> {
    const connection = this.getConnection(dataSourceId);
    if (!connection) {
      throw new Error(`No connection found for data source: ${dataSourceId}`);
    }

    return Promise.all(
      queries.map(({ query, params }) => this.execute<T>(dataSourceId, query, params))
    );
  }

  async stream<T = Record<string, unknown>>(
    dataSourceId: string,
    query: string,
    params?: unknown[],
    onData?: (data: T) => void
  ): Promise<void> {
    const connection = this.getConnection(dataSourceId);
    if (!connection) {
      throw new Error(`No connection found for data source: ${dataSourceId}`);
    }

    await connection.stream<T>(query, params, onData);
  }

  // ============================================================================
  // Connection Pooling
  // ============================================================================

  async getPooledConnection(dataSourceId: string): Promise<Connection> {
    const pool = this.connectionPool.get(dataSourceId);
    if (pool) {
      return pool.acquire();
    }

    // Fall back to direct connection
    const connection = this.connections.get(dataSourceId);
    if (!connection) {
      throw new Error(`No connection found for data source: ${dataSourceId}`);
    }

    return connection;
  }

  async releasePooledConnection(dataSourceId: string, connection: Connection): Promise<void> {
    const pool = this.connectionPool.get(dataSourceId);
    if (pool) {
      await pool.release(connection);
    }
  }

  // ============================================================================
  // Schema Discovery
  // ============================================================================

  async discoverSchema(dataSourceId: string): Promise<SchemaInfo> {
    const connection = this.getConnection(dataSourceId);
    if (!connection) {
      throw new Error(`No connection found for data source: ${dataSourceId}`);
    }

    return connection.discoverSchema();
  }

  async getTables(dataSourceId: string): Promise<string[]> {
    const schema = await this.discoverSchema(dataSourceId);
    return schema.tables.map((t) => t.name);
  }

  async getColumns(dataSourceId: string, tableName: string): Promise<ColumnInfo[]> {
    const schema = await this.discoverSchema(dataSourceId);
    const table = schema.tables.find((t) => t.name === tableName);
    return table?.columns || [];
  }

  // ============================================================================
  // Health Checks
  // ============================================================================

  async ping(dataSourceId: string): Promise<boolean> {
    const connection = this.getConnection(dataSourceId);
    if (!connection) {
      return false;
    }

    try {
      await connection.ping();
      return true;
    } catch {
      return false;
    }
  }

  async getConnectionStats(dataSourceId: string): Promise<ConnectionStats> {
    const connection = this.getConnection(dataSourceId);
    if (!connection) {
      throw new Error(`No connection found for data source: ${dataSourceId}`);
    }

    return connection.getStats();
  }

  // ============================================================================
  // Private Methods
  // ============================================================================

  private getConnection(dataSourceId: string): Connection | undefined {
    return this.connections.get(dataSourceId);
  }

  private async createConnection(
    dataSource: DataSource,
    options?: ConnectionOptions
  ): Promise<Connection> {
    switch (dataSource.type) {
      case 'sql':
        return new SQLConnection(dataSource, options);
      case 'rest_api':
        return new RESTConnection(dataSource, options);
      case 'graphql':
        return new GraphQLConnection(dataSource, options);
      case 'websocket':
        return new WebSocketConnection(dataSource, options);
      default:
        throw new Error(`Unsupported data source type: ${dataSource.type}`);
    }
  }

  private async createConnectionPool(
    dataSource: DataSource,
    options: ConnectionOptions
  ): Promise<ConnectionPool> {
    return new ConnectionPool(dataSource, options);
  }
}

// ============================================================================
// Connection Interfaces
// ============================================================================

export interface Connection {
  execute<T = Record<string, unknown>>(query: string, params?: unknown[]): Promise<QueryResult<T>>;
  stream<T = Record<string, unknown>>(
    query: string,
    params?: unknown[],
    onData?: (data: T) => void
  ): Promise<void>;
  discoverSchema(): Promise<SchemaInfo>;
  ping(): Promise<void>;
  getStats(): Promise<ConnectionStats>;
  close(): Promise<void>;
}

export interface SchemaInfo {
  tables: Array<{
    name: string;
    columns: ColumnInfo[];
  }>;
}

export interface ColumnInfo {
  name: string;
  type: string;
  nullable: boolean;
  primaryKey: boolean;
}

export interface ConnectionStats {
  queriesExecuted: number;
  totalExecutionTime: number;
  averageExecutionTime: number;
  lastQueryTime?: Date;
  errors: number;
}

// ============================================================================
// Connection Implementations
// ============================================================================

class BaseConnection implements Connection {
  protected dataSource: DataSource;
  protected options: ConnectionOptions;
  protected stats: ConnectionStats;

  constructor(dataSource: DataSource, options?: ConnectionOptions) {
    this.dataSource = dataSource;
    this.options = options || {};
    this.stats = {
      queriesExecuted: 0,
      totalExecutionTime: 0,
      averageExecutionTime: 0,
      errors: 0,
    };
  }

  async execute<T = Record<string, unknown>>(
    query: string,
    params?: unknown[]
  ): Promise<QueryResult<T>> {
    throw new Error('Not implemented');
  }

  async stream<T = Record<string, unknown>>(
    query: string,
    params?: unknown[],
    onData?: (data: T) => void
  ): Promise<void> {
    throw new Error('Not implemented');
  }

  async discoverSchema(): Promise<SchemaInfo> {
    throw new Error('Not implemented');
  }

  async ping(): Promise<void> {
    // Default ping implementation
  }

  async getStats(): Promise<ConnectionStats> {
    return this.stats;
  }

  async close(): Promise<void> {
    // Default close implementation
  }
}

class SQLConnection extends BaseConnection {
  async execute<T = Record<string, unknown>>(
    query: string,
    params?: unknown[]
  ): Promise<QueryResult<T>> {
    const startTime = Date.now();

    // Mock implementation
    const data: T[] = [];
    const metadata = {
      rowCount: 0,
      columnCount: 0,
      columns: [],
      executedAt: new Date(),
    };

    this.stats.queriesExecuted++;
    this.stats.totalExecutionTime += Date.now() - startTime;
    this.stats.averageExecutionTime = this.stats.totalExecutionTime / this.stats.queriesExecuted;
    this.stats.lastQueryTime = new Date();

    return { data, metadata };
  }
}

class RESTConnection extends BaseConnection {
  async execute<T = Record<string, unknown>>(
    query: string,
    params?: unknown[]
  ): Promise<QueryResult<T>> {
    const startTime = Date.now();

    const response = await fetch(this.dataSource.connectionString || '', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...this.dataSource.headers,
      },
      body: JSON.stringify({ query, params }),
    });

    if (!response.ok) {
      this.stats.errors++;
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const result = await response.json();

    this.stats.queriesExecuted++;
    this.stats.totalExecutionTime += Date.now() - startTime;
    this.stats.averageExecutionTime = this.stats.totalExecutionTime / this.stats.queriesExecuted;
    this.stats.lastQueryTime = new Date();

    return result;
  }
}

class GraphQLConnection extends BaseConnection {}
class WebSocketConnection extends BaseConnection {}

class ConnectionPool {
  private connections: Connection[];
  private available: Connection[];
  private inUse: Set<Connection>;

  constructor(private dataSource: DataSource, private options: ConnectionOptions) {
    this.connections = [];
    this.available = [];
    this.inUse = new Set();
  }

  async initialize(): Promise<void> {
    const poolSize = this.options.poolSize || 5;
    for (let i = 0; i < poolSize; i++) {
      const connection = new SQLConnection(this.dataSource, this.options);
      this.connections.push(connection);
      this.available.push(connection);
    }
  }

  async acquire(): Promise<Connection> {
    if (this.available.length === 0) {
      // Wait for a connection to become available
      await new Promise((resolve) => setTimeout(resolve, 100));
      return this.acquire();
    }

    const connection = this.available.pop()!;
    this.inUse.add(connection);
    return connection;
  }

  async release(connection: Connection): Promise<void> {
    this.inUse.delete(connection);
    this.available.push(connection);
  }

  async close(): Promise<void> {
    await Promise.all(this.connections.map((c) => c.close()));
    this.connections = [];
    this.available = [];
    this.inUse.clear();
  }
}

// ============================================================================
// Errors
// ============================================================================

export class DataConnectorError extends Error {
  constructor(message: string, public dataSourceId: string, public cause?: unknown) {
    super(message);
    this.name = 'DataConnectorError';
  }
}

// Factory function
export function createDataConnector(): DataConnector {
  return new DataConnector();
}
