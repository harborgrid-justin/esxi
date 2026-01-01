/**
 * Threat Detection - Anomaly Detection & Threat Intelligence
 * Real-time threat detection and security monitoring
 */

import { nanoid } from 'nanoid';
import { Threat, ThreatType, ThreatSeverity, ThreatIndicator } from '../types';
import { auditTrail } from '../compliance/AuditTrail';
import { AuditEventType, AuditSeverity } from '../types';

export interface DetectionRule {
  id: string;
  name: string;
  type: ThreatType;
  pattern: string;
  severity: ThreatSeverity;
  enabled: boolean;
}

export class ThreatDetection {
  private threats: Map<string, Threat> = new Map();
  private rules: Map<string, DetectionRule> = new Map();

  constructor() {
    this.initializeRules();
  }

  /**
   * Initialize detection rules
   */
  private initializeRules(): void {
    this.addRule({
      id: nanoid(),
      name: 'Brute Force Login Attempts',
      type: ThreatType.BRUTE_FORCE,
      pattern: 'failed_login_count > 5',
      severity: ThreatSeverity.HIGH,
      enabled: true,
    });

    this.addRule({
      id: nanoid(),
      name: 'SQL Injection Detection',
      type: ThreatType.SQL_INJECTION,
      pattern: 'sql_keywords',
      severity: ThreatSeverity.CRITICAL,
      enabled: true,
    });

    this.addRule({
      id: nanoid(),
      name: 'Unusual Data Access Pattern',
      type: ThreatType.ANOMALOUS_BEHAVIOR,
      pattern: 'access_rate > threshold',
      severity: ThreatSeverity.MEDIUM,
      enabled: true,
    });
  }

  private addRule(rule: DetectionRule): void {
    this.rules.set(rule.id, rule);
  }

  /**
   * Detect threat
   */
  async detectThreat(
    type: ThreatType,
    source: string,
    target: string,
    description: string,
    indicators: ThreatIndicator[]
  ): Promise<Threat> {
    const threat: Threat = {
      id: nanoid(),
      type,
      severity: this.calculateSeverity(type, indicators),
      detectedAt: new Date(),
      source,
      target,
      description,
      indicators,
      mitigated: false,
      metadata: {},
    };

    this.threats.set(threat.id, threat);

    // Log to audit trail
    await auditTrail.log(
      AuditEventType.THREAT_DETECTED,
      this.mapSeverity(threat.severity),
      target,
      'THREAT_DETECTION',
      'SUCCESS',
      {
        threatId: threat.id,
        threatType: type,
        indicators: indicators.length,
      }
    );

    return threat;
  }

  /**
   * Mitigate threat
   */
  async mitigateThreat(threatId: string, action: string): Promise<void> {
    const threat = this.threats.get(threatId);
    if (threat) {
      threat.mitigated = true;
      threat.mitigationAction = action;
      threat.mitigatedAt = new Date();
    }
  }

  /**
   * Get active threats
   */
  getActiveThreats(): Threat[] {
    return Array.from(this.threats.values()).filter(t => !t.mitigated);
  }

  /**
   * Get threat by ID
   */
  getThreat(id: string): Threat | undefined {
    return this.threats.get(id);
  }

  private calculateSeverity(type: ThreatType, indicators: ThreatIndicator[]): ThreatSeverity {
    const avgConfidence = indicators.reduce((sum, i) => sum + i.confidence, 0) / indicators.length;

    if (avgConfidence > 80 || type === ThreatType.SQL_INJECTION) {
      return ThreatSeverity.CRITICAL;
    } else if (avgConfidence > 60) {
      return ThreatSeverity.HIGH;
    } else if (avgConfidence > 40) {
      return ThreatSeverity.MEDIUM;
    }
    return ThreatSeverity.LOW;
  }

  private mapSeverity(severity: ThreatSeverity): AuditSeverity {
    switch (severity) {
      case ThreatSeverity.CRITICAL: return AuditSeverity.CRITICAL;
      case ThreatSeverity.HIGH: return AuditSeverity.HIGH;
      case ThreatSeverity.MEDIUM: return AuditSeverity.MEDIUM;
      case ThreatSeverity.LOW: return AuditSeverity.LOW;
      default: return AuditSeverity.INFO;
    }
  }
}

export const threatDetection = new ThreatDetection();
