/**
 * Error logging service
 * @module errors/ErrorLogger
 */

import { AccessibilityError } from './AccessibilityError';
import type { Logger } from '../types';

/**
 * Log level
 */
export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

/**
 * Log entry
 */
export interface LogEntry {
  level: LogLevel;
  message: string;
  timestamp: Date;
  context?: Record<string, unknown>;
  error?: Error;
}

/**
 * Logger configuration
 */
export interface LoggerConfig {
  /** Minimum log level to output */
  minLevel?: LogLevel;
  /** Whether to include timestamps */
  timestamps?: boolean;
  /** Whether to include stack traces */
  stackTraces?: boolean;
  /** Custom log formatter */
  formatter?: (entry: LogEntry) => string;
  /** Custom log handler */
  handler?: (entry: LogEntry) => void;
}

/**
 * Error logging service
 */
export class ErrorLogger implements Logger {
  private config: Required<LoggerConfig>;
  private logs: LogEntry[] = [];
  private maxLogs = 1000;

  private readonly levelPriority: Record<LogLevel, number> = {
    debug: 0,
    info: 1,
    warn: 2,
    error: 3,
  };

  constructor(config: LoggerConfig = {}) {
    this.config = {
      minLevel: config.minLevel ?? 'info',
      timestamps: config.timestamps ?? true,
      stackTraces: config.stackTraces ?? true,
      formatter: config.formatter ?? this.defaultFormatter.bind(this),
      handler: config.handler ?? this.defaultHandler.bind(this),
    };
  }

  /**
   * Log debug message
   */
  public debug(message: string, context?: Record<string, unknown>): void {
    this.log('debug', message, context);
  }

  /**
   * Log info message
   */
  public info(message: string, context?: Record<string, unknown>): void {
    this.log('info', message, context);
  }

  /**
   * Log warning message
   */
  public warn(message: string, context?: Record<string, unknown>): void {
    this.log('warn', message, context);
  }

  /**
   * Log error message
   */
  public error(message: string, error?: Error, context?: Record<string, unknown>): void {
    this.log('error', message, context, error);
  }

  /**
   * Log message with level
   */
  private log(
    level: LogLevel,
    message: string,
    context?: Record<string, unknown>,
    error?: Error
  ): void {
    // Check if level meets minimum threshold
    if (this.levelPriority[level] < this.levelPriority[this.config.minLevel]) {
      return;
    }

    const entry: LogEntry = {
      level,
      message,
      timestamp: new Date(),
      context,
      error,
    };

    // Store log entry
    this.storeLogs(entry);

    // Handle log output
    this.config.handler(entry);
  }

  /**
   * Store log entry
   */
  private storeLogs(entry: LogEntry): void {
    this.logs.push(entry);

    // Trim logs if exceeds max
    if (this.logs.length > this.maxLogs) {
      this.logs = this.logs.slice(-this.maxLogs);
    }
  }

  /**
   * Default log formatter
   */
  private defaultFormatter(entry: LogEntry): string {
    const parts: string[] = [];

    // Timestamp
    if (this.config.timestamps) {
      parts.push(`[${entry.timestamp.toISOString()}]`);
    }

    // Level
    parts.push(`[${entry.level.toUpperCase()}]`);

    // Message
    parts.push(entry.message);

    // Context
    if (entry.context && Object.keys(entry.context).length > 0) {
      parts.push(`\nContext: ${JSON.stringify(entry.context, null, 2)}`);
    }

    // Error
    if (entry.error) {
      parts.push(`\nError: ${entry.error.message}`);

      if (this.config.stackTraces && entry.error.stack) {
        parts.push(`\nStack: ${entry.error.stack}`);
      }

      if (entry.error instanceof AccessibilityError) {
        parts.push(`\nError Details: ${JSON.stringify(entry.error.toJSON(), null, 2)}`);
      }
    }

    return parts.join(' ');
  }

  /**
   * Default log handler
   */
  private defaultHandler(entry: LogEntry): void {
    const formatted = this.config.formatter(entry);

    switch (entry.level) {
      case 'debug':
        console.debug(formatted);
        break;
      case 'info':
        console.info(formatted);
        break;
      case 'warn':
        console.warn(formatted);
        break;
      case 'error':
        console.error(formatted);
        break;
    }
  }

  /**
   * Get all logs
   */
  public getLogs(): ReadonlyArray<LogEntry> {
    return [...this.logs];
  }

  /**
   * Get logs by level
   */
  public getLogsByLevel(level: LogLevel): ReadonlyArray<LogEntry> {
    return this.logs.filter((log) => log.level === level);
  }

  /**
   * Get error logs
   */
  public getErrors(): ReadonlyArray<LogEntry> {
    return this.getLogsByLevel('error');
  }

  /**
   * Clear all logs
   */
  public clearLogs(): void {
    this.logs = [];
  }

  /**
   * Export logs as JSON
   */
  public exportLogs(): string {
    return JSON.stringify(this.logs, null, 2);
  }

  /**
   * Set maximum number of logs to store
   */
  public setMaxLogs(max: number): void {
    this.maxLogs = max;
    if (this.logs.length > max) {
      this.logs = this.logs.slice(-max);
    }
  }

  /**
   * Update logger configuration
   */
  public updateConfig(config: Partial<LoggerConfig>): void {
    this.config = {
      ...this.config,
      ...config,
      formatter: config.formatter ?? this.config.formatter,
      handler: config.handler ?? this.config.handler,
    };
  }

  /**
   * Create a child logger with additional context
   */
  public createChildLogger(context: Record<string, unknown>): ErrorLogger {
    const childLogger = new ErrorLogger(this.config);

    // Override handler to merge context
    const originalHandler = this.config.handler;
    childLogger.updateConfig({
      handler: (entry) => {
        originalHandler({
          ...entry,
          context: {
            ...context,
            ...entry.context,
          },
        });
      },
    });

    return childLogger;
  }

  /**
   * Create console logger
   */
  public static createConsoleLogger(config?: LoggerConfig): ErrorLogger {
    return new ErrorLogger({
      ...config,
      handler: (entry) => {
        const message = config?.formatter
          ? config.formatter(entry)
          : new ErrorLogger().defaultFormatter(entry);

        switch (entry.level) {
          case 'debug':
            console.debug(message);
            break;
          case 'info':
            console.info(message);
            break;
          case 'warn':
            console.warn(message);
            break;
          case 'error':
            console.error(message);
            break;
        }
      },
    });
  }

  /**
   * Create silent logger (no output)
   */
  public static createSilentLogger(): ErrorLogger {
    return new ErrorLogger({
      handler: () => {
        // Do nothing
      },
    });
  }
}

/**
 * Create default logger instance
 */
export function createLogger(config?: LoggerConfig): ErrorLogger {
  return new ErrorLogger(config);
}
