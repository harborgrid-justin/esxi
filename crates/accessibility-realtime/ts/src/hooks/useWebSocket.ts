import { useEffect, useRef, useState, useCallback } from 'react';
import { WebSocketService } from '../services/WebSocketService';
import type { MonitorEvent, StateSnapshot, WebSocketConfig } from '../types';

export interface UseWebSocketOptions extends WebSocketConfig {
  autoConnect?: boolean;
}

export interface UseWebSocketReturn {
  isConnected: boolean;
  connectionState: 'connecting' | 'open' | 'closing' | 'closed';
  lastEvent: MonitorEvent | null;
  lastState: StateSnapshot | null;
  error: Error | null;
  connect: () => void;
  disconnect: () => void;
  subscribe: (eventTypes: string[]) => void;
  unsubscribe: (eventTypes: string[]) => void;
  getState: () => void;
}

/**
 * React hook for WebSocket connection management
 */
export function useWebSocket(options: UseWebSocketOptions): UseWebSocketReturn {
  const { autoConnect = true, ...config } = options;
  const serviceRef = useRef<WebSocketService | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [connectionState, setConnectionState] = useState<'connecting' | 'open' | 'closing' | 'closed'>('closed');
  const [lastEvent, setLastEvent] = useState<MonitorEvent | null>(null);
  const [lastState, setLastState] = useState<StateSnapshot | null>(null);
  const [error, setError] = useState<Error | null>(null);

  // Initialize service
  useEffect(() => {
    serviceRef.current = new WebSocketService(config);

    const unsubscribeEvent = serviceRef.current.onEvent((event) => {
      setLastEvent(event);
    });

    const unsubscribeState = serviceRef.current.onState((state) => {
      setLastState(state);
    });

    const unsubscribeError = serviceRef.current.onError((err) => {
      setError(err);
    });

    const unsubscribeConnect = serviceRef.current.onConnect(() => {
      setIsConnected(true);
      setError(null);
    });

    const unsubscribeDisconnect = serviceRef.current.onDisconnect(() => {
      setIsConnected(false);
    });

    // Auto-connect if enabled
    if (autoConnect) {
      serviceRef.current.connect();
    }

    // Update connection state periodically
    const stateInterval = setInterval(() => {
      if (serviceRef.current) {
        setConnectionState(serviceRef.current.getState());
      }
    }, 1000);

    return () => {
      clearInterval(stateInterval);
      unsubscribeEvent();
      unsubscribeState();
      unsubscribeError();
      unsubscribeConnect();
      unsubscribeDisconnect();
      serviceRef.current?.destroy();
    };
  }, [config.url, config.reconnectInterval, config.maxReconnectAttempts, config.heartbeatInterval, autoConnect]);

  const connect = useCallback(() => {
    serviceRef.current?.connect();
  }, []);

  const disconnect = useCallback(() => {
    serviceRef.current?.disconnect();
  }, []);

  const subscribe = useCallback((eventTypes: string[]) => {
    serviceRef.current?.subscribe(eventTypes);
  }, []);

  const unsubscribe = useCallback((eventTypes: string[]) => {
    serviceRef.current?.unsubscribe(eventTypes);
  }, []);

  const getState = useCallback(() => {
    serviceRef.current?.getState();
  }, []);

  return {
    isConnected,
    connectionState,
    lastEvent,
    lastState,
    error,
    connect,
    disconnect,
    subscribe,
    unsubscribe,
    getState,
  };
}
