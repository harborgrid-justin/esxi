/**
 * TypeScript type definitions for accessibility real-time monitoring
 */

export type Severity = 'critical' | 'high' | 'medium' | 'low' | 'info';

export type ScanStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

export type ScanType = 'full' | 'incremental' | 'targeted' | 'scheduled' | 'on_demand';

export type HealthStatus = 'healthy' | 'degraded' | 'unhealthy' | 'unknown';

export interface ScanConfig {
  id: string;
  name: string;
  scan_type: ScanType;
  targets: string[];
  rules: string[];
  schedule?: string; // Cron expression
  timeout_seconds: number;
  retry_count: number;
  metadata: Record<string, string>;
}

export interface ScanResult {
  id: string;
  scan_id: string;
  status: ScanStatus;
  started_at: string;
  completed_at?: string;
  duration_ms?: number;
  issues_found: number;
  issues_by_severity: Record<string, number>;
  pages_scanned: number;
  error?: string;
}

export interface AccessibilityIssue {
  id: string;
  scan_id: string;
  severity: Severity;
  rule_id: string;
  rule_name: string;
  description: string;
  element: string;
  selector: string;
  page_url: string;
  context: string;
  wcag_criteria: string[];
  remediation: string;
  detected_at: string;
}

export interface AlertConditions {
  min_severity: Severity;
  issue_threshold?: number;
  failure_threshold?: number;
  scan_types?: ScanType[];
}

export type AlertChannel =
  | { type: 'email'; recipients: string[] }
  | { type: 'webhook'; url: string; headers: Record<string, string> }
  | { type: 'slack'; webhook_url: string; channel: string }
  | { type: 'pagerduty'; integration_key: string };

export interface AlertConfig {
  id: string;
  name: string;
  enabled: boolean;
  conditions: AlertConditions;
  channels: AlertChannel[];
  throttle_minutes?: number;
}

export interface Alert {
  id: string;
  config_id: string;
  severity: Severity;
  title: string;
  message: string;
  scan_id?: string;
  issues: AccessibilityIssue[];
  created_at: string;
  acknowledged: boolean;
  acknowledged_at?: string;
  acknowledged_by?: string;
}

export interface MonitorMetrics {
  timestamp: string;
  active_scans: number;
  completed_scans: number;
  failed_scans: number;
  total_issues: number;
  issues_by_severity: Record<string, number>;
  average_scan_duration_ms: number;
  system_health: HealthStatus;
}

export interface ScanSchedule {
  id: string;
  name: string;
  cron: string;
  config: ScanConfig;
  enabled: boolean;
  next_run?: string;
  last_run?: string;
}

export interface ScanContext {
  config: ScanConfig;
  started_at: string;
  status: ScanStatus;
  pages_scanned: number;
  total_pages: number;
  issues: AccessibilityIssue[];
}

export type MonitorEvent =
  | { type: 'scan_started'; scan_id: string; config: ScanConfig; timestamp: string }
  | {
      type: 'scan_progress';
      scan_id: string;
      pages_scanned: number;
      total_pages: number;
      issues_found: number;
      percentage: number;
      timestamp: string;
    }
  | { type: 'scan_completed'; scan_id: string; result: ScanResult; timestamp: string }
  | { type: 'scan_failed'; scan_id: string; error: string; timestamp: string }
  | { type: 'issue_detected'; issue: AccessibilityIssue; timestamp: string }
  | { type: 'issue_resolved'; issue_id: string; timestamp: string }
  | { type: 'alert_triggered'; alert: Alert; timestamp: string }
  | { type: 'alert_acknowledged'; alert_id: string; acknowledged_by: string; timestamp: string }
  | {
      type: 'system_health_changed';
      old_status: HealthStatus;
      new_status: HealthStatus;
      reason: string;
      timestamp: string;
    }
  | { type: 'metrics_updated'; metrics: MonitorMetrics; timestamp: string }
  | { type: 'client_connected'; client_id: string; timestamp: string }
  | { type: 'client_disconnected'; client_id: string; timestamp: string };

export interface StateSnapshot {
  active_scans: Array<[string, ScanContext]>;
  metrics: MonitorMetrics;
  health: HealthStatus;
  timestamp: string;
}

export interface WebSocketMessage {
  type: 'event' | 'state' | 'pong' | 'error';
  event?: MonitorEvent;
  data?: StateSnapshot;
  message?: string;
}

export interface WebSocketConfig {
  url: string;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
  heartbeatInterval?: number;
}

export const SEVERITY_COLORS: Record<Severity, string> = {
  critical: '#dc2626',
  high: '#ea580c',
  medium: '#f59e0b',
  low: '#3b82f6',
  info: '#6b7280',
};

export const STATUS_COLORS: Record<ScanStatus, string> = {
  pending: '#6b7280',
  running: '#3b82f6',
  completed: '#10b981',
  failed: '#dc2626',
  cancelled: '#6b7280',
};

export const HEALTH_COLORS: Record<HealthStatus, string> = {
  healthy: '#10b981',
  degraded: '#f59e0b',
  unhealthy: '#dc2626',
  unknown: '#6b7280',
};
