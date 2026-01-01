/**
 * Data Retention - Data Lifecycle Management
 * Automated data retention and deletion policies
 */

import { nanoid } from 'nanoid';

export interface RetentionPolicy {
  id: string;
  name: string;
  dataType: string;
  retentionPeriod: number; // days
  deleteAfter: boolean;
  archiveAfter?: number; // days
  enabled: boolean;
  createdAt: Date;
}

export interface DataRecord {
  id: string;
  type: string;
  createdAt: Date;
  archivedAt?: Date;
  deletedAt?: Date;
  metadata: Record<string, unknown>;
}

export class DataRetention {
  private policies: Map<string, RetentionPolicy> = new Map();
  private records: Map<string, DataRecord> = new Map();

  /**
   * Create retention policy
   */
  async createPolicy(data: {
    name: string;
    dataType: string;
    retentionPeriod: number;
    deleteAfter?: boolean;
    archiveAfter?: number;
  }): Promise<RetentionPolicy> {
    const policy: RetentionPolicy = {
      id: nanoid(),
      name: data.name,
      dataType: data.dataType,
      retentionPeriod: data.retentionPeriod,
      deleteAfter: data.deleteAfter ?? true,
      archiveAfter: data.archiveAfter,
      enabled: true,
      createdAt: new Date(),
    };

    this.policies.set(policy.id, policy);
    return policy;
  }

  /**
   * Apply retention policies
   */
  async applyPolicies(): Promise<{
    archived: number;
    deleted: number;
  }> {
    let archived = 0;
    let deleted = 0;
    const now = Date.now();

    for (const policy of this.policies.values()) {
      if (!policy.enabled) continue;

      for (const record of this.records.values()) {
        if (record.type !== policy.dataType) continue;

        const age = (now - record.createdAt.getTime()) / (1000 * 60 * 60 * 24);

        // Archive if applicable
        if (policy.archiveAfter && age >= policy.archiveAfter && !record.archivedAt) {
          record.archivedAt = new Date();
          archived++;
        }

        // Delete if retention period exceeded
        if (policy.deleteAfter && age >= policy.retentionPeriod && !record.deletedAt) {
          record.deletedAt = new Date();
          deleted++;
        }
      }
    }

    return { archived, deleted };
  }

  /**
   * Get all policies
   */
  getAllPolicies(): RetentionPolicy[] {
    return Array.from(this.policies.values());
  }

  /**
   * Get active policies
   */
  getActivePolicies(): RetentionPolicy[] {
    return Array.from(this.policies.values()).filter(p => p.enabled);
  }
}

export const dataRetention = new DataRetention();
