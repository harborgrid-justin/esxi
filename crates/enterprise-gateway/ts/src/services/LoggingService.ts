/**
 * Enterprise API Gateway - Logging Service
 *
 * Request logging and audit trails
 */

import type { GatewayRequest, GatewayResponse, LogEntry, LogLevel } from '../types';

export class LoggingService {
  private logs: LogEntry[] = [];
  private readonly maxLogs = 5000;
  private readonly retentionMs = 3600000; // 1 hour
  private logLevel: LogLevel = 'info';

  /**
   * Set log level
   */
  public setLogLevel(level: LogLevel): void {
    this.logLevel = level;
  }

  /**
   * Log a message
   */
  public log(level: LogLevel, message: string, metadata?: Record<string, unknown>): void {
    if (!this.shouldLog(level)) {
      return;
    }

    const entry: LogEntry = {
      timestamp: Date.now(),
      level,
      message,
      metadata,
    };

    this.logs.push(entry);
    this.cleanup();

    // In production, send to external logging service
    this.output(entry);
  }

  /**
   * Log incoming request
   */
  public logRequest(request: GatewayRequest): void {
    this.log('info', 'Incoming request', {
      requestId: request.id,
      method: request.method,
      path: request.path,
      ip: request.ip,
      consumer: request.consumer?.username,
    });
  }

  /**
   * Log response
   */
  public logResponse(request: GatewayRequest, response: GatewayResponse): void {
    const level = response.statusCode >= 500 ? 'error' : response.statusCode >= 400 ? 'warn' : 'info';

    this.log(level, 'Response sent', {
      requestId: request.id,
      statusCode: response.statusCode,
      duration: response.duration,
      upstream: response.upstream,
    });
  }

  /**
   * Log error
   */
  public logError(request: GatewayRequest, error: Error): void {
    this.log('error', `Error processing request: ${error.message}`, {
      requestId: request.id,
      method: request.method,
      path: request.path,
      error: {
        name: error.name,
        message: error.message,
        stack: error.stack,
      },
    });
  }

  /**
   * Log rate limit event
   */
  public logRateLimit(request: GatewayRequest, limit: number, remaining: number): void {
    this.log('warn', 'Rate limit applied', {
      requestId: request.id,
      consumer: request.consumer?.username,
      ip: request.ip,
      limit,
      remaining,
    });
  }

  /**
   * Log authentication failure
   */
  public logAuthFailure(request: GatewayRequest, reason: string): void {
    this.log('warn', `Authentication failed: ${reason}`, {
      requestId: request.id,
      method: request.method,
      path: request.path,
      ip: request.ip,
    });
  }

  /**
   * Log WAF block
   */
  public logWAFBlock(request: GatewayRequest, rules: string[]): void {
    this.log('warn', 'Request blocked by WAF', {
      requestId: request.id,
      method: request.method,
      path: request.path,
      ip: request.ip,
      matchedRules: rules,
    });
  }

  /**
   * Check if should log at this level
   */
  private shouldLog(level: LogLevel): boolean {
    const levels: LogLevel[] = ['debug', 'info', 'warn', 'error', 'fatal'];
    const currentIndex = levels.indexOf(this.logLevel);
    const messageIndex = levels.indexOf(level);

    return messageIndex >= currentIndex;
  }

  /**
   * Output log entry
   */
  private output(entry: LogEntry): void {
    const timestamp = new Date(entry.timestamp).toISOString();
    const level = entry.level.toUpperCase().padEnd(5);
    const message = entry.message;

    let logMessage = `[${timestamp}] ${level} ${message}`;

    if (entry.metadata) {
      logMessage += ` ${JSON.stringify(entry.metadata)}`;
    }

    switch (entry.level) {
      case 'debug':
      case 'info':
        console.log(logMessage);
        break;
      case 'warn':
        console.warn(logMessage);
        break;
      case 'error':
      case 'fatal':
        console.error(logMessage);
        break;
    }
  }

  /**
   * Get logs
   */
  public getLogs(filter?: {
    level?: LogLevel;
    startTime?: number;
    endTime?: number;
    requestId?: string;
  }): LogEntry[] {
    let filtered = [...this.logs];

    if (filter) {
      if (filter.level) {
        filtered = filtered.filter((log) => log.level === filter.level);
      }

      if (filter.startTime) {
        filtered = filtered.filter((log) => log.timestamp >= filter.startTime!);
      }

      if (filter.endTime) {
        filtered = filtered.filter((log) => log.timestamp <= filter.endTime!);
      }

      if (filter.requestId) {
        filtered = filtered.filter((log) => log.requestId === filter.requestId);
      }
    }

    return filtered.sort((a, b) => b.timestamp - a.timestamp);
  }

  /**
   * Get logs by level
   */
  public getLogsByLevel(): Record<LogLevel, number> {
    const counts: Record<string, number> = {};

    for (const log of this.logs) {
      counts[log.level] = (counts[log.level] || 0) + 1;
    }

    return counts as Record<LogLevel, number>;
  }

  /**
   * Search logs
   */
  public searchLogs(query: string): LogEntry[] {
    const lowerQuery = query.toLowerCase();

    return this.logs.filter((log) => {
      // Search in message
      if (log.message.toLowerCase().includes(lowerQuery)) {
        return true;
      }

      // Search in metadata
      if (log.metadata) {
        const metadataStr = JSON.stringify(log.metadata).toLowerCase();
        if (metadataStr.includes(lowerQuery)) {
          return true;
        }
      }

      return false;
    });
  }

  /**
   * Clean up old logs
   */
  private cleanup(): void {
    const now = Date.now();

    // Remove logs older than retention period
    this.logs = this.logs.filter((log) => now - log.timestamp < this.retentionMs);

    // Limit total number of logs
    if (this.logs.length > this.maxLogs) {
      this.logs = this.logs.slice(-this.maxLogs);
    }
  }

  /**
   * Clear all logs
   */
  public clear(): void {
    this.logs = [];
  }

  /**
   * Export logs
   */
  public exportLogs(): LogEntry[] {
    return [...this.logs];
  }

  /**
   * Get statistics
   */
  public getStatistics(): {
    totalLogs: number;
    logsByLevel: Record<LogLevel, number>;
    oldestLog?: number;
    newestLog?: number;
  } {
    const logsByLevel = this.getLogsByLevel();
    const timestamps = this.logs.map((log) => log.timestamp);

    return {
      totalLogs: this.logs.length,
      logsByLevel,
      oldestLog: timestamps.length > 0 ? Math.min(...timestamps) : undefined,
      newestLog: timestamps.length > 0 ? Math.max(...timestamps) : undefined,
    };
  }
}
