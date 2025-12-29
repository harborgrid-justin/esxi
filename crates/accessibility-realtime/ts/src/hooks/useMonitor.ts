import { useState, useEffect, useCallback } from 'react';
import type {
  MonitorEvent,
  ScanContext,
  MonitorMetrics,
  HealthStatus,
  StateSnapshot,
} from '../types';

export interface UseMonitorReturn {
  activeScans: Map<string, ScanContext>;
  metrics: MonitorMetrics | null;
  health: HealthStatus;
  scanProgress: Map<string, number>;
  updateFromEvent: (event: MonitorEvent) => void;
  updateFromState: (state: StateSnapshot) => void;
}

/**
 * React hook for managing monitor state
 */
export function useMonitor(
  lastEvent: MonitorEvent | null,
  lastState: StateSnapshot | null
): UseMonitorReturn {
  const [activeScans, setActiveScans] = useState<Map<string, ScanContext>>(new Map());
  const [metrics, setMetrics] = useState<MonitorMetrics | null>(null);
  const [health, setHealth] = useState<HealthStatus>('unknown');
  const [scanProgress, setScanProgress] = useState<Map<string, number>>(new Map());

  // Update from events
  useEffect(() => {
    if (lastEvent) {
      updateFromEvent(lastEvent);
    }
  }, [lastEvent]);

  // Update from state snapshots
  useEffect(() => {
    if (lastState) {
      updateFromState(lastState);
    }
  }, [lastState]);

  const updateFromEvent = useCallback((event: MonitorEvent) => {
    switch (event.type) {
      case 'scan_started':
        setActiveScans((prev) => {
          const next = new Map(prev);
          next.set(event.scan_id, {
            config: event.config,
            started_at: event.timestamp,
            status: 'running',
            pages_scanned: 0,
            total_pages: event.config.targets.length,
            issues: [],
          });
          return next;
        });
        break;

      case 'scan_progress':
        setActiveScans((prev) => {
          const next = new Map(prev);
          const scan = next.get(event.scan_id);
          if (scan) {
            next.set(event.scan_id, {
              ...scan,
              pages_scanned: event.pages_scanned,
              total_pages: event.total_pages,
            });
          }
          return next;
        });
        setScanProgress((prev) => {
          const next = new Map(prev);
          next.set(event.scan_id, event.percentage);
          return next;
        });
        break;

      case 'scan_completed':
      case 'scan_failed':
        setActiveScans((prev) => {
          const next = new Map(prev);
          next.delete(event.scan_id);
          return next;
        });
        setScanProgress((prev) => {
          const next = new Map(prev);
          next.delete(event.scan_id);
          return next;
        });
        break;

      case 'metrics_updated':
        setMetrics(event.metrics);
        setHealth(event.metrics.system_health);
        break;

      case 'system_health_changed':
        setHealth(event.new_status);
        break;
    }
  }, []);

  const updateFromState = useCallback((state: StateSnapshot) => {
    // Update active scans
    const scansMap = new Map<string, ScanContext>();
    for (const [scanId, context] of state.active_scans) {
      scansMap.set(scanId, context);
    }
    setActiveScans(scansMap);

    // Update metrics
    setMetrics(state.metrics);
    setHealth(state.health);
  }, []);

  return {
    activeScans,
    metrics,
    health,
    scanProgress,
    updateFromEvent,
    updateFromState,
  };
}
