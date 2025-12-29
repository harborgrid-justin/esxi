import { useState, useEffect, useCallback } from 'react';
import type { Alert, MonitorEvent } from '../types';

export interface UseAlertsOptions {
  maxAlerts?: number;
  autoAcknowledge?: boolean;
  autoAcknowledgeDelay?: number;
}

export interface UseAlertsReturn {
  alerts: Alert[];
  unacknowledgedCount: number;
  addAlert: (alert: Alert) => void;
  acknowledgeAlert: (alertId: string, acknowledgedBy: string) => void;
  clearAlerts: () => void;
  removeAlert: (alertId: string) => void;
}

/**
 * React hook for managing alerts
 */
export function useAlerts(
  lastEvent: MonitorEvent | null,
  options: UseAlertsOptions = {}
): UseAlertsReturn {
  const { maxAlerts = 100, autoAcknowledge = false, autoAcknowledgeDelay = 30000 } = options;
  const [alerts, setAlerts] = useState<Alert[]>([]);

  // Handle incoming alert events
  useEffect(() => {
    if (!lastEvent) return;

    if (lastEvent.type === 'alert_triggered') {
      addAlert(lastEvent.alert);
    } else if (lastEvent.type === 'alert_acknowledged') {
      acknowledgeAlert(lastEvent.alert_id, lastEvent.acknowledged_by);
    }
  }, [lastEvent]);

  const addAlert = useCallback(
    (alert: Alert) => {
      setAlerts((prev) => {
        const newAlerts = [alert, ...prev];
        // Limit number of alerts
        if (newAlerts.length > maxAlerts) {
          newAlerts.splice(maxAlerts);
        }
        return newAlerts;
      });

      // Auto-acknowledge if enabled
      if (autoAcknowledge) {
        setTimeout(() => {
          acknowledgeAlert(alert.id, 'auto');
        }, autoAcknowledgeDelay);
      }
    },
    [maxAlerts, autoAcknowledge, autoAcknowledgeDelay]
  );

  const acknowledgeAlert = useCallback((alertId: string, acknowledgedBy: string) => {
    setAlerts((prev) =>
      prev.map((alert) =>
        alert.id === alertId
          ? {
              ...alert,
              acknowledged: true,
              acknowledged_at: new Date().toISOString(),
              acknowledged_by: acknowledgedBy,
            }
          : alert
      )
    );
  }, []);

  const clearAlerts = useCallback(() => {
    setAlerts([]);
  }, []);

  const removeAlert = useCallback((alertId: string) => {
    setAlerts((prev) => prev.filter((alert) => alert.id !== alertId));
  }, []);

  const unacknowledgedCount = alerts.filter((alert) => !alert.acknowledged).length;

  return {
    alerts,
    unacknowledgedCount,
    addAlert,
    acknowledgeAlert,
    clearAlerts,
    removeAlert,
  };
}
