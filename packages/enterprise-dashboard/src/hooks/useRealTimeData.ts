/**
 * useRealTimeData Hook
 * WebSocket-based real-time data subscription
 */

import { useEffect, useRef, useCallback, useState } from 'react';
import { dashboardService } from '../services/DashboardService';
import type { DataSubscription } from '../types';

export interface UseRealTimeDataOptions {
  channels: string[];
  enabled?: boolean;
  reconnect?: boolean;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
  onConnect?: () => void;
  onDisconnect?: () => void;
  onError?: (error: Event) => void;
  onMessage?: (data: unknown) => void;
}

export interface RealTimeDataState {
  connected: boolean;
  reconnecting: boolean;
  error: string | null;
  lastMessage: unknown | null;
  messageCount: number;
}

export function useRealTimeData(options: UseRealTimeDataOptions) {
  const {
    channels,
    enabled = true,
    reconnect = true,
    reconnectInterval = 5000,
    maxReconnectAttempts = 10,
    onConnect,
    onDisconnect,
    onError,
    onMessage,
  } = options;

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const shouldConnectRef = useRef(enabled);

  const [state, setState] = useState<RealTimeDataState>({
    connected: false,
    reconnecting: false,
    error: null,
    lastMessage: null,
    messageCount: 0,
  });

  // ============================================================================
  // WebSocket Management
  // ============================================================================

  const connect = useCallback(() => {
    if (!shouldConnectRef.current || channels.length === 0) {
      return;
    }

    // Close existing connection
    if (wsRef.current) {
      wsRef.current.close();
    }

    try {
      const ws = dashboardService.createWebSocket(channels);

      ws.addEventListener('open', () => {
        console.log('WebSocket connected to channels:', channels);
        reconnectAttemptsRef.current = 0;
        setState((prev) => ({
          ...prev,
          connected: true,
          reconnecting: false,
          error: null,
        }));
        onConnect?.();
      });

      ws.addEventListener('message', (event) => {
        try {
          const data = JSON.parse(event.data);
          setState((prev) => ({
            ...prev,
            lastMessage: data,
            messageCount: prev.messageCount + 1,
          }));
          onMessage?.(data);
        } catch (err) {
          console.error('Failed to parse WebSocket message:', err);
        }
      });

      ws.addEventListener('error', (error) => {
        console.error('WebSocket error:', error);
        setState((prev) => ({
          ...prev,
          error: 'WebSocket connection error',
        }));
        onError?.(error);
      });

      ws.addEventListener('close', (event) => {
        console.log('WebSocket disconnected:', event.code, event.reason);
        setState((prev) => ({
          ...prev,
          connected: false,
        }));
        onDisconnect?.();

        // Attempt reconnection
        if (
          shouldConnectRef.current &&
          reconnect &&
          reconnectAttemptsRef.current < maxReconnectAttempts
        ) {
          reconnectAttemptsRef.current += 1;
          setState((prev) => ({
            ...prev,
            reconnecting: true,
          }));

          reconnectTimeoutRef.current = setTimeout(() => {
            console.log(
              `Reconnecting... (attempt ${reconnectAttemptsRef.current}/${maxReconnectAttempts})`
            );
            connect();
          }, reconnectInterval);
        } else if (reconnectAttemptsRef.current >= maxReconnectAttempts) {
          setState((prev) => ({
            ...prev,
            error: 'Max reconnection attempts reached',
            reconnecting: false,
          }));
        }
      });

      wsRef.current = ws;
    } catch (err) {
      console.error('Failed to create WebSocket:', err);
      setState((prev) => ({
        ...prev,
        error: err instanceof Error ? err.message : 'Connection failed',
      }));
    }
  }, [channels, reconnect, reconnectInterval, maxReconnectAttempts, onConnect, onDisconnect, onError, onMessage]);

  const disconnect = useCallback(() => {
    shouldConnectRef.current = false;

    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    setState((prev) => ({
      ...prev,
      connected: false,
      reconnecting: false,
    }));
  }, []);

  const send = useCallback((data: unknown) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(data));
    } else {
      console.warn('WebSocket is not connected. Cannot send data.');
    }
  }, []);

  const reconnectNow = useCallback(() => {
    reconnectAttemptsRef.current = 0;
    disconnect();
    shouldConnectRef.current = true;
    connect();
  }, [connect, disconnect]);

  // ============================================================================
  // Effects
  // ============================================================================

  useEffect(() => {
    shouldConnectRef.current = enabled;

    if (enabled) {
      connect();
    } else {
      disconnect();
    }

    return () => {
      disconnect();
    };
  }, [enabled, channels.join(',')]); // Reconnect if channels change

  // ============================================================================
  // Return Hook Interface
  // ============================================================================

  return {
    ...state,
    send,
    reconnect: reconnectNow,
    disconnect,
  };
}

/**
 * Specialized hook for KPI real-time updates
 */
export function useRealTimeKPIs(onUpdate: (kpis: unknown) => void) {
  return useRealTimeData({
    channels: ['kpis', 'metrics'],
    onMessage: (data) => {
      if (data && typeof data === 'object' && 'type' in data && data.type === 'kpi_update') {
        onUpdate(data);
      }
    },
  });
}

/**
 * Specialized hook for alert real-time updates
 */
export function useRealTimeAlerts(onAlert: (alert: unknown) => void) {
  return useRealTimeData({
    channels: ['alerts'],
    onMessage: (data) => {
      if (data && typeof data === 'object' && 'type' in data && data.type === 'new_alert') {
        onAlert(data);
      }
    },
  });
}

/**
 * Specialized hook for activity real-time updates
 */
export function useRealTimeActivity(onActivity: (activity: unknown) => void) {
  return useRealTimeData({
    channels: ['activities', 'audit'],
    onMessage: (data) => {
      if (data && typeof data === 'object' && 'type' in data && data.type === 'activity') {
        onActivity(data);
      }
    },
  });
}

/**
 * Multi-channel subscription hook
 */
export function useRealTimeMultiChannel(
  subscriptions: Array<{
    channels: string[];
    handler: (data: unknown) => void;
  }>
) {
  const allChannels = Array.from(
    new Set(subscriptions.flatMap((sub) => sub.channels))
  );

  return useRealTimeData({
    channels: allChannels,
    onMessage: (data) => {
      subscriptions.forEach((sub) => {
        sub.handler(data);
      });
    },
  });
}

export default useRealTimeData;
