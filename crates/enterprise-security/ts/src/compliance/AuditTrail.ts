/**
 * Audit Trail - Immutable Audit Logging
 * Tamper-proof audit trail with cryptographic integrity
 */

import { nanoid } from 'nanoid';
import { AuditLog, AuditEventType, AuditSeverity } from '../types';
import { hashingService } from '../encryption/HashingService';

export class AuditTrail {
  private logs: AuditLog[] = [];
  private logIndex: Map<string, number> = new Map();

  /**
   * Log audit event
   */
  async log(
    eventType: AuditEventType,
    severity: AuditSeverity,
    resource: string,
    action: string,
    result: 'SUCCESS' | 'FAILURE' | 'PARTIAL',
    details: Record<string, unknown>,
    context?: {
      userId?: string;
      username?: string;
      ipAddress?: string;
      userAgent?: string;
    }
  ): Promise<AuditLog> {
    const previousHash = this.logs.length > 0 ? this.logs[this.logs.length - 1]!.hash : '';

    const log: AuditLog = {
      id: nanoid(),
      eventType,
      severity,
      timestamp: new Date(),
      userId: context?.userId,
      username: context?.username,
      ipAddress: context?.ipAddress,
      userAgent: context?.userAgent,
      resource,
      action,
      result,
      details,
      metadata: {
        requestId: nanoid(),
      },
      hash: '',
    };

    // Generate hash for integrity (includes previous hash for chaining)
    log.hash = await this.generateHash(log, previousHash);

    this.logs.push(log);
    this.logIndex.set(log.id, this.logs.length - 1);

    return log;
  }

  /**
   * Query audit logs
   */
  query(filters: {
    userId?: string;
    eventType?: AuditEventType;
    severity?: AuditSeverity;
    startDate?: Date;
    endDate?: Date;
    resource?: string;
    limit?: number;
  }): AuditLog[] {
    let results = [...this.logs];

    if (filters.userId) {
      results = results.filter(l => l.userId === filters.userId);
    }
    if (filters.eventType) {
      results = results.filter(l => l.eventType === filters.eventType);
    }
    if (filters.severity) {
      results = results.filter(l => l.severity === filters.severity);
    }
    if (filters.startDate) {
      results = results.filter(l => l.timestamp >= filters.startDate!);
    }
    if (filters.endDate) {
      results = results.filter(l => l.timestamp <= filters.endDate!);
    }
    if (filters.resource) {
      results = results.filter(l => l.resource === filters.resource);
    }

    if (filters.limit) {
      results = results.slice(-filters.limit);
    }

    return results;
  }

  /**
   * Verify audit trail integrity
   */
  async verifyIntegrity(): Promise<{ valid: boolean; errors: string[] }> {
    const errors: string[] = [];
    let previousHash = '';

    for (let i = 0; i < this.logs.length; i++) {
      const log = this.logs[i]!;
      const expectedHash = await this.generateHash(log, previousHash);

      if (log.hash !== expectedHash) {
        errors.push(`Hash mismatch at index ${i}: ${log.id}`);
      }

      previousHash = log.hash;
    }

    return {
      valid: errors.length === 0,
      errors,
    };
  }

  /**
   * Get audit statistics
   */
  getStatistics(): {
    total: number;
    bySeverity: Record<AuditSeverity, number>;
    byEventType: Record<string, number>;
    successRate: number;
  } {
    const bySeverity: Record<AuditSeverity, number> = {
      [AuditSeverity.CRITICAL]: 0,
      [AuditSeverity.HIGH]: 0,
      [AuditSeverity.MEDIUM]: 0,
      [AuditSeverity.LOW]: 0,
      [AuditSeverity.INFO]: 0,
    };

    const byEventType: Record<string, number> = {};
    let successCount = 0;

    for (const log of this.logs) {
      bySeverity[log.severity]++;
      byEventType[log.eventType] = (byEventType[log.eventType] || 0) + 1;
      if (log.result === 'SUCCESS') {
        successCount++;
      }
    }

    return {
      total: this.logs.length,
      bySeverity,
      byEventType,
      successRate: this.logs.length > 0 ? successCount / this.logs.length : 0,
    };
  }

  private async generateHash(log: AuditLog, previousHash: string): Promise<string> {
    const data = JSON.stringify({
      id: log.id,
      eventType: log.eventType,
      timestamp: log.timestamp.toISOString(),
      resource: log.resource,
      action: log.action,
      result: log.result,
      previousHash,
    });

    return hashingService.generateHash(data, 'sha256');
  }
}

export const auditTrail = new AuditTrail();
