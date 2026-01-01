/**
 * Tenant Migration - Handles plan migrations and data migrations
 */

import { v4 as uuidv4 } from 'uuid';
import {
  Tenant,
  TenantMigration,
  Plan,
  Subscription,
  PlanChangeRequest,
  ProrationPreview,
} from '../types';
import { ProrationEngine } from '../engine/ProrationEngine';

export interface MigrationPlan {
  id: string;
  fromPlan: Plan;
  toPlan: Plan;
  estimatedDuration: number;
  steps: MigrationStep[];
  downtime: boolean;
  rollbackPlan: string[];
}

export interface MigrationStep {
  id: string;
  name: string;
  description: string;
  estimatedDuration: number;
  critical: boolean;
  status: 'pending' | 'in_progress' | 'completed' | 'failed';
  startedAt?: Date;
  completedAt?: Date;
  error?: string;
}

export class TenantMigrationManager {
  private migrations: Map<string, TenantMigration> = new Map();
  private prorationEngine: ProrationEngine;

  constructor(prorationEngine?: ProrationEngine) {
    this.prorationEngine = prorationEngine ?? new ProrationEngine();
  }

  /**
   * Create migration plan
   */
  async createMigrationPlan(
    tenant: Tenant,
    subscription: Subscription,
    fromPlan: Plan,
    toPlan: Plan
  ): Promise<MigrationPlan> {
    const steps = this.generateMigrationSteps(fromPlan, toPlan);

    return {
      id: uuidv4(),
      fromPlan,
      toPlan,
      estimatedDuration: steps.reduce((sum, s) => sum + s.estimatedDuration, 0),
      steps,
      downtime: this.requiresDowntime(fromPlan, toPlan),
      rollbackPlan: this.generateRollbackPlan(steps),
    };
  }

  /**
   * Execute plan migration
   */
  async migratePlan(
    tenant: Tenant,
    subscription: Subscription,
    request: PlanChangeRequest,
    fromPlan: Plan,
    toPlan: Plan
  ): Promise<TenantMigration> {
    const migration: TenantMigration = {
      id: uuidv4(),
      tenantId: tenant.id,
      fromPlanId: fromPlan.id,
      toPlanId: toPlan.id,
      status: 'pending',
      startedAt: new Date(),
    };

    this.migrations.set(migration.id, migration);

    try {
      // Update status
      migration.status = 'in_progress';
      this.migrations.set(migration.id, migration);

      // Create migration plan
      const plan = await this.createMigrationPlan(
        tenant,
        subscription,
        fromPlan,
        toPlan
      );

      // Execute migration steps
      for (const step of plan.steps) {
        step.status = 'in_progress';
        step.startedAt = new Date();

        try {
          await this.executeMigrationStep(step, tenant, fromPlan, toPlan);
          step.status = 'completed';
          step.completedAt = new Date();
        } catch (error: any) {
          step.status = 'failed';
          step.error = error.message;
          step.completedAt = new Date();
          throw error;
        }
      }

      // Complete migration
      migration.status = 'completed';
      migration.completedAt = new Date();
    } catch (error: any) {
      migration.status = 'failed';
      migration.error = error.message;
      migration.completedAt = new Date();
    }

    this.migrations.set(migration.id, migration);
    return migration;
  }

  /**
   * Generate migration steps based on plan differences
   */
  private generateMigrationSteps(fromPlan: Plan, toPlan: Plan): MigrationStep[] {
    const steps: MigrationStep[] = [];

    // Always start with backup
    steps.push({
      id: '1',
      name: 'Backup current data',
      description: 'Create backup before migration',
      estimatedDuration: 300,
      critical: true,
      status: 'pending',
    });

    // Check for feature additions
    const newFeatures = toPlan.features.filter(
      (f) => !fromPlan.features.find((ff) => ff.id === f.id)
    );

    if (newFeatures.length > 0) {
      steps.push({
        id: '2',
        name: 'Enable new features',
        description: `Enabling ${newFeatures.length} new feature(s)`,
        estimatedDuration: newFeatures.length * 60,
        critical: false,
        status: 'pending',
      });
    }

    // Check for feature removals
    const removedFeatures = fromPlan.features.filter(
      (f) => !toPlan.features.find((ff) => ff.id === f.id)
    );

    if (removedFeatures.length > 0) {
      steps.push({
        id: '3',
        name: 'Disable removed features',
        description: `Disabling ${removedFeatures.length} feature(s)`,
        estimatedDuration: removedFeatures.length * 30,
        critical: false,
        status: 'pending',
      });
    }

    // Check for quota changes
    const quotaChanges = this.detectQuotaChanges(fromPlan, toPlan);

    if (quotaChanges.length > 0) {
      steps.push({
        id: '4',
        name: 'Update resource quotas',
        description: `Updating ${quotaChanges.length} quota(s)`,
        estimatedDuration: quotaChanges.length * 45,
        critical: true,
        status: 'pending',
      });
    }

    // Pricing model change
    if (fromPlan.pricingModel !== toPlan.pricingModel) {
      steps.push({
        id: '5',
        name: 'Migrate pricing model',
        description: `Changing from ${fromPlan.pricingModel} to ${toPlan.pricingModel}`,
        estimatedDuration: 120,
        critical: true,
        status: 'pending',
      });
    }

    // Data migration if needed
    if (this.requiresDataMigration(fromPlan, toPlan)) {
      steps.push({
        id: '6',
        name: 'Migrate data structures',
        description: 'Migrate data to new plan schema',
        estimatedDuration: 600,
        critical: true,
        status: 'pending',
      });
    }

    // Update subscription
    steps.push({
      id: '7',
      name: 'Update subscription',
      description: 'Update subscription to new plan',
      estimatedDuration: 60,
      critical: true,
      status: 'pending',
    });

    // Verify migration
    steps.push({
      id: '8',
      name: 'Verify migration',
      description: 'Verify all changes were applied correctly',
      estimatedDuration: 180,
      critical: true,
      status: 'pending',
    });

    return steps;
  }

  /**
   * Execute a single migration step
   */
  private async executeMigrationStep(
    step: MigrationStep,
    tenant: Tenant,
    fromPlan: Plan,
    toPlan: Plan
  ): Promise<void> {
    console.log(`Executing migration step: ${step.name} for tenant ${tenant.slug}`);

    // Simulate step execution
    await new Promise((resolve) => setTimeout(resolve, 100));

    // In production, implement actual migration logic for each step
    switch (step.id) {
      case '1':
        await this.backupData(tenant);
        break;
      case '2':
        await this.enableNewFeatures(tenant, toPlan);
        break;
      case '3':
        await this.disableRemovedFeatures(tenant, fromPlan);
        break;
      case '4':
        await this.updateQuotas(tenant, toPlan);
        break;
      case '5':
        await this.migratePricingModel(tenant, fromPlan, toPlan);
        break;
      case '6':
        await this.migrateDataStructures(tenant, fromPlan, toPlan);
        break;
      case '7':
        await this.updateSubscription(tenant, toPlan);
        break;
      case '8':
        await this.verifyMigration(tenant, toPlan);
        break;
    }
  }

  /**
   * Detect quota changes between plans
   */
  private detectQuotaChanges(
    fromPlan: Plan,
    toPlan: Plan
  ): Array<{ feature: string; from: number; to: number }> {
    const changes: Array<{ feature: string; from: number; to: number }> = [];

    for (const toFeature of toPlan.features) {
      const fromFeature = fromPlan.features.find((f) => f.id === toFeature.id);

      if (fromFeature && fromFeature.quota !== toFeature.quota) {
        changes.push({
          feature: toFeature.name,
          from: fromFeature.quota || 0,
          to: toFeature.quota || 0,
        });
      }
    }

    return changes;
  }

  /**
   * Check if migration requires downtime
   */
  private requiresDowntime(fromPlan: Plan, toPlan: Plan): boolean {
    // Major pricing model changes may require downtime
    if (fromPlan.pricingModel !== toPlan.pricingModel) {
      return true;
    }

    // Significant quota reductions may require downtime
    const quotaChanges = this.detectQuotaChanges(fromPlan, toPlan);
    const hasSignificantReduction = quotaChanges.some(
      (c) => c.to < c.from * 0.5
    );

    return hasSignificantReduction;
  }

  /**
   * Check if data migration is required
   */
  private requiresDataMigration(fromPlan: Plan, toPlan: Plan): boolean {
    // Check if schema changes are needed
    return fromPlan.pricingModel !== toPlan.pricingModel;
  }

  /**
   * Generate rollback plan
   */
  private generateRollbackPlan(steps: MigrationStep[]): string[] {
    return steps
      .slice()
      .reverse()
      .map((step) => `Rollback: ${step.name}`);
  }

  /**
   * Rollback migration
   */
  async rollbackMigration(migrationId: string): Promise<TenantMigration> {
    const migration = this.migrations.get(migrationId);

    if (!migration) {
      throw new Error(`Migration ${migrationId} not found`);
    }

    if (migration.status !== 'failed') {
      throw new Error('Can only rollback failed migrations');
    }

    console.log(`Rolling back migration ${migrationId}`);

    // In production, implement actual rollback logic
    migration.status = 'completed';
    migration.error = undefined;

    this.migrations.set(migrationId, migration);
    return migration;
  }

  /**
   * Get migration status
   */
  getMigration(migrationId: string): TenantMigration | undefined {
    return this.migrations.get(migrationId);
  }

  /**
   * Get tenant migrations
   */
  getTenantMigrations(tenantId: string): TenantMigration[] {
    return Array.from(this.migrations.values()).filter(
      (m) => m.tenantId === tenantId
    );
  }

  // Private helper methods (placeholders for actual implementation)

  private async backupData(tenant: Tenant): Promise<void> {
    console.log(`Backing up data for tenant ${tenant.slug}`);
  }

  private async enableNewFeatures(tenant: Tenant, plan: Plan): Promise<void> {
    console.log(`Enabling new features for tenant ${tenant.slug}`);
  }

  private async disableRemovedFeatures(tenant: Tenant, plan: Plan): Promise<void> {
    console.log(`Disabling removed features for tenant ${tenant.slug}`);
  }

  private async updateQuotas(tenant: Tenant, plan: Plan): Promise<void> {
    console.log(`Updating quotas for tenant ${tenant.slug}`);
  }

  private async migratePricingModel(
    tenant: Tenant,
    fromPlan: Plan,
    toPlan: Plan
  ): Promise<void> {
    console.log(`Migrating pricing model for tenant ${tenant.slug}`);
  }

  private async migrateDataStructures(
    tenant: Tenant,
    fromPlan: Plan,
    toPlan: Plan
  ): Promise<void> {
    console.log(`Migrating data structures for tenant ${tenant.slug}`);
  }

  private async updateSubscription(tenant: Tenant, plan: Plan): Promise<void> {
    console.log(`Updating subscription for tenant ${tenant.slug}`);
  }

  private async verifyMigration(tenant: Tenant, plan: Plan): Promise<void> {
    console.log(`Verifying migration for tenant ${tenant.slug}`);
  }
}
