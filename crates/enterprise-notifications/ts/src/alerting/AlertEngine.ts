/**
 * AlertEngine - Alert processing and management
 * Handles alert lifecycle, routing, and correlation
 */

import { EventEmitter } from 'events';
import {
  Alert,
  AlertStatus,
  AlertSeverity,
  AlertRule,
  Notification,
  NotificationPriority,
} from '../types';

export interface AlertEngineConfig {
  enableDeduplication: boolean;
  deduplicationWindow: number; // milliseconds
  enableAutoResolve: boolean;
  autoResolveTimeout: number; // milliseconds
  enableGrouping: boolean;
  groupingWindow: number; // milliseconds
  maxAlertsPerRule: number;
}

export class AlertEngine extends EventEmitter {
  private config: AlertEngineConfig;
  private alerts: Map<string, Alert>;
  private alertsByFingerprint: Map<string, string>; // fingerprint -> alertId
  private alertsByRule: Map<string, Set<string>>; // ruleId -> alertIds
  private autoResolveTimers: Map<string, NodeJS.Timeout>;
  private isRunning: boolean;

  constructor(config: Partial<AlertEngineConfig> = {}) {
    super();
    this.config = {
      enableDeduplication: config.enableDeduplication ?? true,
      deduplicationWindow: config.deduplicationWindow ?? 300000, // 5 minutes
      enableAutoResolve: config.enableAutoResolve ?? true,
      autoResolveTimeout: config.autoResolveTimeout ?? 3600000, // 1 hour
      enableGrouping: config.enableGrouping ?? true,
      groupingWindow: config.groupingWindow ?? 600000, // 10 minutes
      maxAlertsPerRule: config.maxAlertsPerRule ?? 1000,
    };

    this.alerts = new Map();
    this.alertsByFingerprint = new Map();
    this.alertsByRule = new Map();
    this.autoResolveTimers = new Map();
    this.isRunning = false;
  }

  /**
   * Start alert engine
   */
  start(): void {
    if (this.isRunning) {
      return;
    }

    this.isRunning = true;
    this.emit('started');
  }

  /**
   * Stop alert engine
   */
  stop(): void {
    if (!this.isRunning) {
      return;
    }

    this.isRunning = false;

    // Clear all timers
    for (const timer of this.autoResolveTimers.values()) {
      clearTimeout(timer);
    }
    this.autoResolveTimers.clear();

    this.emit('stopped');
  }

  /**
   * Create new alert
   */
  async createAlert(alert: Partial<Alert>, rule?: AlertRule): Promise<Alert> {
    const fingerprint = this.generateFingerprint(alert);

    // Check for duplicate
    if (this.config.enableDeduplication) {
      const existingAlertId = this.alertsByFingerprint.get(fingerprint);
      if (existingAlertId) {
        const existingAlert = this.alerts.get(existingAlertId);
        if (existingAlert && this.isWithinDeduplicationWindow(existingAlert)) {
          return this.updateExistingAlert(existingAlert);
        }
      }
    }

    // Create new alert
    const newAlert: Alert = {
      id: this.generateAlertId(),
      tenantId: alert.tenantId ?? '',
      ruleId: alert.ruleId ?? rule?.id,
      name: alert.name ?? 'Unnamed Alert',
      description: alert.description,
      severity: alert.severity ?? AlertSeverity.WARNING,
      status: AlertStatus.OPEN,
      source: alert.source ?? 'unknown',
      sourceId: alert.sourceId,
      sourceType: alert.sourceType,
      message: alert.message ?? '',
      details: alert.details,
      metrics: alert.metrics ?? [],
      notificationIds: [],
      escalationLevel: 0,
      fingerprint,
      count: 1,
      firstOccurrenceAt: new Date(),
      lastOccurrenceAt: new Date(),
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    // Store alert
    this.alerts.set(newAlert.id, newAlert);
    this.alertsByFingerprint.set(fingerprint, newAlert.id);

    // Index by rule
    if (newAlert.ruleId) {
      let ruleAlerts = this.alertsByRule.get(newAlert.ruleId);
      if (!ruleAlerts) {
        ruleAlerts = new Set();
        this.alertsByRule.set(newAlert.ruleId, ruleAlerts);
      }
      ruleAlerts.add(newAlert.id);

      // Check max alerts per rule
      if (ruleAlerts.size > this.config.maxAlertsPerRule) {
        this.pruneOldAlerts(newAlert.ruleId);
      }
    }

    // Schedule auto-resolve if enabled
    if (this.config.enableAutoResolve && rule?.autoResolve) {
      this.scheduleAutoResolve(newAlert, rule.autoResolveAfter ?? this.config.autoResolveTimeout);
    }

    this.emit('alert:created', newAlert);
    return newAlert;
  }

  /**
   * Update existing alert (increment count)
   */
  private updateExistingAlert(alert: Alert): Alert {
    alert.count++;
    alert.lastOccurrenceAt = new Date();
    alert.updatedAt = new Date();

    this.emit('alert:updated', alert);
    return alert;
  }

  /**
   * Get alert by ID
   */
  getAlert(alertId: string): Alert | undefined {
    return this.alerts.get(alertId);
  }

  /**
   * Get alerts by rule
   */
  getAlertsByRule(ruleId: string): Alert[] {
    const alertIds = this.alertsByRule.get(ruleId);
    if (!alertIds) {
      return [];
    }

    return Array.from(alertIds)
      .map(id => this.alerts.get(id))
      .filter((a): a is Alert => a !== undefined);
  }

  /**
   * Get all alerts with filters
   */
  getAlerts(filter?: {
    tenantId?: string;
    status?: AlertStatus[];
    severity?: AlertSeverity[];
    source?: string;
    assignedTo?: string;
    startDate?: Date;
    endDate?: Date;
  }): Alert[] {
    let alerts = Array.from(this.alerts.values());

    if (filter) {
      alerts = alerts.filter(alert => {
        if (filter.tenantId && alert.tenantId !== filter.tenantId) {
          return false;
        }
        if (filter.status && !filter.status.includes(alert.status)) {
          return false;
        }
        if (filter.severity && !filter.severity.includes(alert.severity)) {
          return false;
        }
        if (filter.source && alert.source !== filter.source) {
          return false;
        }
        if (filter.assignedTo && alert.assignedTo !== filter.assignedTo) {
          return false;
        }
        if (filter.startDate && alert.createdAt < filter.startDate) {
          return false;
        }
        if (filter.endDate && alert.createdAt > filter.endDate) {
          return false;
        }
        return true;
      });
    }

    return alerts.sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime());
  }

  /**
   * Acknowledge alert
   */
  async acknowledgeAlert(alertId: string, userId: string): Promise<Alert | undefined> {
    const alert = this.alerts.get(alertId);
    if (!alert || alert.status !== AlertStatus.OPEN) {
      return undefined;
    }

    alert.status = AlertStatus.ACKNOWLEDGED;
    alert.acknowledgedBy = userId;
    alert.acknowledgedAt = new Date();
    alert.updatedAt = new Date();

    this.emit('alert:acknowledged', alert);
    return alert;
  }

  /**
   * Assign alert
   */
  async assignAlert(alertId: string, userId: string): Promise<Alert | undefined> {
    const alert = this.alerts.get(alertId);
    if (!alert) {
      return undefined;
    }

    alert.assignedTo = userId;
    alert.assignedAt = new Date();
    alert.updatedAt = new Date();

    if (alert.status === AlertStatus.OPEN) {
      alert.status = AlertStatus.IN_PROGRESS;
    }

    this.emit('alert:assigned', alert);
    return alert;
  }

  /**
   * Resolve alert
   */
  async resolveAlert(alertId: string, userId?: string): Promise<Alert | undefined> {
    const alert = this.alerts.get(alertId);
    if (!alert) {
      return undefined;
    }

    alert.status = AlertStatus.RESOLVED;
    alert.resolvedBy = userId;
    alert.resolvedAt = new Date();
    alert.updatedAt = new Date();

    // Clear auto-resolve timer
    const timer = this.autoResolveTimers.get(alertId);
    if (timer) {
      clearTimeout(timer);
      this.autoResolveTimers.delete(alertId);
    }

    this.emit('alert:resolved', alert);
    return alert;
  }

  /**
   * Close alert
   */
  async closeAlert(alertId: string): Promise<Alert | undefined> {
    const alert = this.alerts.get(alertId);
    if (!alert || alert.status !== AlertStatus.RESOLVED) {
      return undefined;
    }

    alert.status = AlertStatus.CLOSED;
    alert.updatedAt = new Date();

    this.emit('alert:closed', alert);
    return alert;
  }

  /**
   * Suppress alert
   */
  async suppressAlert(
    alertId: string,
    suppressUntil: Date,
    reason?: string
  ): Promise<Alert | undefined> {
    const alert = this.alerts.get(alertId);
    if (!alert) {
      return undefined;
    }

    alert.status = AlertStatus.SUPPRESSED;
    alert.suppressedUntil = suppressUntil;
    alert.suppressionReason = reason;
    alert.updatedAt = new Date();

    this.emit('alert:suppressed', alert);
    return alert;
  }

  /**
   * Schedule auto-resolve
   */
  private scheduleAutoResolve(alert: Alert, timeout: number): void {
    const timer = setTimeout(() => {
      this.resolveAlert(alert.id, 'system').catch(error => {
        this.emit('error', error);
      });
      this.autoResolveTimers.delete(alert.id);
    }, timeout);

    this.autoResolveTimers.set(alert.id, timer);
  }

  /**
   * Generate alert fingerprint
   */
  private generateFingerprint(alert: Partial<Alert>): string {
    const parts = [
      alert.tenantId ?? '',
      alert.source ?? '',
      alert.sourceId ?? '',
      alert.name ?? '',
      alert.message ?? '',
    ];

    return this.hash(parts.join(':'));
  }

  /**
   * Check if alert is within deduplication window
   */
  private isWithinDeduplicationWindow(alert: Alert): boolean {
    const age = Date.now() - alert.lastOccurrenceAt.getTime();
    return age < this.config.deduplicationWindow;
  }

  /**
   * Prune old alerts for a rule
   */
  private pruneOldAlerts(ruleId: string): void {
    const alertIds = this.alertsByRule.get(ruleId);
    if (!alertIds) {
      return;
    }

    const alerts = Array.from(alertIds)
      .map(id => this.alerts.get(id))
      .filter((a): a is Alert => a !== undefined)
      .sort((a, b) => a.createdAt.getTime() - b.createdAt.getTime());

    // Keep only the most recent alerts
    const toRemove = alerts.slice(0, alerts.length - this.config.maxAlertsPerRule);

    for (const alert of toRemove) {
      if (alert.status === AlertStatus.RESOLVED || alert.status === AlertStatus.CLOSED) {
        this.alerts.delete(alert.id);
        alertIds.delete(alert.id);
        if (alert.fingerprint) {
          this.alertsByFingerprint.delete(alert.fingerprint);
        }
      }
    }
  }

  /**
   * Hash string
   */
  private hash(input: string): string {
    let hash = 0;
    for (let i = 0; i < input.length; i++) {
      const char = input.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash;
    }
    return hash.toString(36);
  }

  /**
   * Generate alert ID
   */
  private generateAlertId(): string {
    return `alert_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Get statistics
   */
  getStats(): {
    total: number;
    byStatus: Record<AlertStatus, number>;
    bySeverity: Record<AlertSeverity, number>;
  } {
    const stats = {
      total: this.alerts.size,
      byStatus: {
        [AlertStatus.OPEN]: 0,
        [AlertStatus.ACKNOWLEDGED]: 0,
        [AlertStatus.IN_PROGRESS]: 0,
        [AlertStatus.RESOLVED]: 0,
        [AlertStatus.CLOSED]: 0,
        [AlertStatus.SUPPRESSED]: 0,
      },
      bySeverity: {
        [AlertSeverity.INFO]: 0,
        [AlertSeverity.WARNING]: 0,
        [AlertSeverity.ERROR]: 0,
        [AlertSeverity.CRITICAL]: 0,
        [AlertSeverity.FATAL]: 0,
      },
    };

    for (const alert of this.alerts.values()) {
      stats.byStatus[alert.status]++;
      stats.bySeverity[alert.severity]++;
    }

    return stats;
  }

  /**
   * Clear resolved alerts
   */
  clearResolved(olderThan?: Date): number {
    let cleared = 0;

    for (const [id, alert] of this.alerts.entries()) {
      if (alert.status === AlertStatus.RESOLVED || alert.status === AlertStatus.CLOSED) {
        if (!olderThan || (alert.resolvedAt && alert.resolvedAt < olderThan)) {
          this.alerts.delete(id);
          if (alert.fingerprint) {
            this.alertsByFingerprint.delete(alert.fingerprint);
          }
          if (alert.ruleId) {
            this.alertsByRule.get(alert.ruleId)?.delete(id);
          }
          cleared++;
        }
      }
    }

    return cleared;
  }
}

export default AlertEngine;
