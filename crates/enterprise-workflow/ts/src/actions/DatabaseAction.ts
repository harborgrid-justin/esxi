/**
 * Database Action - Execute database operations
 */

import { DatabaseActionConfig, Context } from '../types';

export class DatabaseAction {
  private connections: Map<string, any>;

  constructor() {
    this.connections = new Map();
  }

  /**
   * Execute database operation
   */
  async execute(config: DatabaseActionConfig, context: Context): Promise<any> {
    // Interpolate query parameters
    const parameters = config.parameters
      ? this.interpolateParameters(config.parameters, context)
      : {};

    // Execute based on operation type
    switch (config.operation) {
      case 'query':
        return this.executeQuery(config, parameters);
      case 'insert':
        return this.executeInsert(config, parameters);
      case 'update':
        return this.executeUpdate(config, parameters);
      case 'delete':
        return this.executeDelete(config, parameters);
      default:
        throw new Error(`Unsupported database operation: ${config.operation}`);
    }
  }

  /**
   * Execute SELECT query
   */
  private async executeQuery(config: DatabaseActionConfig, parameters: any): Promise<any> {
    const connection = await this.getConnection(config.connection);

    if (config.query) {
      // Execute raw SQL query
      const query = this.interpolateQuery(config.query, parameters);
      return this.executeRawQuery(connection, query, parameters, config.transaction);
    }

    if (config.table) {
      // Simple SELECT query
      return this.selectFromTable(connection, config.table, parameters);
    }

    throw new Error('Database query requires either query or table');
  }

  /**
   * Execute INSERT operation
   */
  private async executeInsert(config: DatabaseActionConfig, parameters: any): Promise<any> {
    if (!config.table) {
      throw new Error('INSERT operation requires table name');
    }

    const connection = await this.getConnection(config.connection);
    return this.insertIntoTable(connection, config.table, parameters, config.transaction);
  }

  /**
   * Execute UPDATE operation
   */
  private async executeUpdate(config: DatabaseActionConfig, parameters: any): Promise<any> {
    if (!config.table) {
      throw new Error('UPDATE operation requires table name');
    }

    const connection = await this.getConnection(config.connection);
    return this.updateTable(connection, config.table, parameters, config.transaction);
  }

  /**
   * Execute DELETE operation
   */
  private async executeDelete(config: DatabaseActionConfig, parameters: any): Promise<any> {
    if (!config.table) {
      throw new Error('DELETE operation requires table name');
    }

    const connection = await this.getConnection(config.connection);
    return this.deleteFromTable(connection, config.table, parameters, config.transaction);
  }

  /**
   * Get database connection
   */
  private async getConnection(connectionString: string): Promise<any> {
    // In production, this would manage actual database connections
    // Support for: PostgreSQL, MySQL, MongoDB, SQL Server, etc.

    if (this.connections.has(connectionString)) {
      return this.connections.get(connectionString);
    }

    // Create new connection (placeholder)
    const connection = {
      connectionString,
      connected: true
    };

    this.connections.set(connectionString, connection);
    return connection;
  }

  /**
   * Execute raw SQL query
   */
  private async executeRawQuery(
    connection: any,
    query: string,
    parameters: any,
    transaction?: boolean
  ): Promise<any> {
    // Placeholder implementation
    console.log('Executing query:', query, parameters);

    return {
      success: true,
      rows: [],
      rowCount: 0,
      timestamp: new Date()
    };
  }

  /**
   * SELECT from table
   */
  private async selectFromTable(
    connection: any,
    table: string,
    parameters: any
  ): Promise<any> {
    // Build WHERE clause from parameters
    const whereClause = Object.keys(parameters).length > 0
      ? 'WHERE ' + Object.keys(parameters).map(k => `${k} = ?`).join(' AND ')
      : '';

    const query = `SELECT * FROM ${table} ${whereClause}`;

    return this.executeRawQuery(connection, query, parameters);
  }

  /**
   * INSERT into table
   */
  private async insertIntoTable(
    connection: any,
    table: string,
    data: any,
    transaction?: boolean
  ): Promise<any> {
    const columns = Object.keys(data).join(', ');
    const placeholders = Object.keys(data).map(() => '?').join(', ');
    const query = `INSERT INTO ${table} (${columns}) VALUES (${placeholders})`;

    return this.executeRawQuery(connection, query, data, transaction);
  }

  /**
   * UPDATE table
   */
  private async updateTable(
    connection: any,
    table: string,
    data: any,
    transaction?: boolean
  ): Promise<any> {
    const { where, ...values } = data;

    const setClause = Object.keys(values).map(k => `${k} = ?`).join(', ');
    const whereClause = where
      ? 'WHERE ' + Object.keys(where).map(k => `${k} = ?`).join(' AND ')
      : '';

    const query = `UPDATE ${table} SET ${setClause} ${whereClause}`;

    return this.executeRawQuery(connection, query, { ...values, ...where }, transaction);
  }

  /**
   * DELETE from table
   */
  private async deleteFromTable(
    connection: any,
    table: string,
    parameters: any,
    transaction?: boolean
  ): Promise<any> {
    const whereClause = Object.keys(parameters).length > 0
      ? 'WHERE ' + Object.keys(parameters).map(k => `${k} = ?`).join(' AND ')
      : '';

    const query = `DELETE FROM ${table} ${whereClause}`;

    return this.executeRawQuery(connection, query, parameters, transaction);
  }

  /**
   * Interpolate query with parameters
   */
  private interpolateQuery(query: string, parameters: any): string {
    let interpolated = query;

    for (const [key, value] of Object.entries(parameters)) {
      const regex = new RegExp(`:${key}\\b`, 'g');
      interpolated = interpolated.replace(regex, this.escapeValue(value));
    }

    return interpolated;
  }

  /**
   * Interpolate parameters with context variables
   */
  private interpolateParameters(parameters: Record<string, any>, context: Context): any {
    const interpolated: any = {};

    for (const [key, value] of Object.entries(parameters)) {
      interpolated[key] = this.interpolateValue(value, context);
    }

    return interpolated;
  }

  /**
   * Interpolate single value
   */
  private interpolateValue(value: any, context: Context): any {
    if (typeof value === 'string' && value.startsWith('${') && value.endsWith('}')) {
      const varName = value.slice(2, -1);
      return context.variables.get(varName) ?? value;
    }

    return value;
  }

  /**
   * Escape value for SQL
   */
  private escapeValue(value: any): string {
    if (value === null || value === undefined) {
      return 'NULL';
    }

    if (typeof value === 'number') {
      return String(value);
    }

    if (typeof value === 'boolean') {
      return value ? 'TRUE' : 'FALSE';
    }

    // Escape string (basic implementation)
    return `'${String(value).replace(/'/g, "''")}'`;
  }

  /**
   * Validate database action configuration
   */
  validate(config: DatabaseActionConfig): string[] {
    const errors: string[] = [];

    if (!config.operation) {
      errors.push('Database operation is required');
    } else if (!['query', 'insert', 'update', 'delete'].includes(config.operation)) {
      errors.push('Invalid database operation');
    }

    if (!config.connection) {
      errors.push('Database connection is required');
    }

    if (config.operation === 'query' && !config.query && !config.table) {
      errors.push('Query operation requires either query or table');
    }

    if (['insert', 'update', 'delete'].includes(config.operation) && !config.table) {
      errors.push(`${config.operation.toUpperCase()} operation requires table name`);
    }

    return errors;
  }

  /**
   * Close all connections
   */
  async closeAll(): Promise<void> {
    // In production, properly close all database connections
    this.connections.clear();
  }
}
