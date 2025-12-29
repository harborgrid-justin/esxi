/**
 * Accessibility Real-time Monitoring - Frontend Library
 *
 * Enterprise-grade React components and hooks for real-time accessibility monitoring
 */

// Types
export * from './types';

// Services
export { WebSocketService } from './services/WebSocketService';
export type { EventCallback, StateCallback, ErrorCallback, ConnectionCallback } from './services/WebSocketService';

// Hooks
export { useWebSocket } from './hooks/useWebSocket';
export type { UseWebSocketOptions, UseWebSocketReturn } from './hooks/useWebSocket';

export { useAlerts } from './hooks/useAlerts';
export type { UseAlertsOptions, UseAlertsReturn } from './hooks/useAlerts';

export { useMonitor } from './hooks/useMonitor';
export type { UseMonitorReturn } from './hooks/useMonitor';

// Monitor Components
export { LiveMonitor } from './components/Monitor/LiveMonitor';
export type { LiveMonitorProps } from './components/Monitor/LiveMonitor';

export { StatusIndicator } from './components/Monitor/StatusIndicator';
export type { StatusIndicatorProps } from './components/Monitor/StatusIndicator';

export { ScanProgress } from './components/Monitor/ScanProgress';
export type { ScanProgressProps } from './components/Monitor/ScanProgress';

export { AlertFeed } from './components/Monitor/AlertFeed';
export type { AlertFeedProps } from './components/Monitor/AlertFeed';

// Alert Components
export { AlertCard } from './components/Alerts/AlertCard';
export type { AlertCardProps } from './components/Alerts/AlertCard';

export { AlertHistory } from './components/Alerts/AlertHistory';
export type { AlertHistoryProps } from './components/Alerts/AlertHistory';

export { AlertRules } from './components/Alerts/AlertRules';
export type { AlertRulesProps } from './components/Alerts/AlertRules';

// Scheduler Components
export { ScanScheduler } from './components/Scheduler/ScanScheduler';
export type { ScanSchedulerProps } from './components/Scheduler/ScanScheduler';

export { ScheduleCalendar } from './components/Scheduler/ScheduleCalendar';
export type { ScheduleCalendarProps } from './components/Scheduler/ScheduleCalendar';
