import type { MonitorEvent, StateSnapshot, WebSocketConfig, WebSocketMessage } from '../types';

export type EventCallback = (event: MonitorEvent) => void;
export type StateCallback = (state: StateSnapshot) => void;
export type ErrorCallback = (error: Error) => void;
export type ConnectionCallback = () => void;

/**
 * WebSocket service for real-time accessibility monitoring
 * Handles reconnection, heartbeat, and message routing
 */
export class WebSocketService {
  private ws: WebSocket | null = null;
  private config: Required<WebSocketConfig>;
  private reconnectAttempts = 0;
  private reconnectTimer: NodeJS.Timeout | null = null;
  private heartbeatTimer: NodeJS.Timeout | null = null;
  private eventCallbacks: Set<EventCallback> = new Set();
  private stateCallbacks: Set<StateCallback> = new Set();
  private errorCallbacks: Set<ErrorCallback> = new Set();
  private connectCallbacks: Set<ConnectionCallback> = new Set();
  private disconnectCallbacks: Set<ConnectionCallback> = new Set();
  private isIntentionalClose = false;

  constructor(config: WebSocketConfig) {
    this.config = {
      url: config.url,
      reconnectInterval: config.reconnectInterval ?? 5000,
      maxReconnectAttempts: config.maxReconnectAttempts ?? 10,
      heartbeatInterval: config.heartbeatInterval ?? 30000,
    };
  }

  /**
   * Connect to WebSocket server
   */
  connect(): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      console.warn('WebSocket already connected');
      return;
    }

    this.isIntentionalClose = false;

    try {
      this.ws = new WebSocket(this.config.url);

      this.ws.onopen = () => {
        console.log('WebSocket connected');
        this.reconnectAttempts = 0;
        this.startHeartbeat();
        this.connectCallbacks.forEach((cb) => cb());
      };

      this.ws.onmessage = (event) => {
        this.handleMessage(event.data);
      };

      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        const err = new Error('WebSocket connection error');
        this.errorCallbacks.forEach((cb) => cb(err));
      };

      this.ws.onclose = () => {
        console.log('WebSocket disconnected');
        this.stopHeartbeat();
        this.disconnectCallbacks.forEach((cb) => cb());

        if (!this.isIntentionalClose) {
          this.scheduleReconnect();
        }
      };
    } catch (error) {
      console.error('Failed to create WebSocket:', error);
      const err = error instanceof Error ? error : new Error('Failed to create WebSocket');
      this.errorCallbacks.forEach((cb) => cb(err));
      this.scheduleReconnect();
    }
  }

  /**
   * Disconnect from WebSocket server
   */
  disconnect(): void {
    this.isIntentionalClose = true;
    this.clearReconnectTimer();
    this.stopHeartbeat();

    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  /**
   * Subscribe to specific event types
   */
  subscribe(eventTypes: string[]): void {
    this.send({
      type: 'subscribe',
      event_types: eventTypes,
    });
  }

  /**
   * Unsubscribe from event types
   */
  unsubscribe(eventTypes: string[]): void {
    this.send({
      type: 'unsubscribe',
      event_types: eventTypes,
    });
  }

  /**
   * Request current state snapshot
   */
  getState(): void {
    this.send({
      type: 'get_state',
    });
  }

  /**
   * Register event callback
   */
  onEvent(callback: EventCallback): () => void {
    this.eventCallbacks.add(callback);
    return () => this.eventCallbacks.delete(callback);
  }

  /**
   * Register state callback
   */
  onState(callback: StateCallback): () => void {
    this.stateCallbacks.add(callback);
    return () => this.stateCallbacks.delete(callback);
  }

  /**
   * Register error callback
   */
  onError(callback: ErrorCallback): () => void {
    this.errorCallbacks.add(callback);
    return () => this.errorCallbacks.delete(callback);
  }

  /**
   * Register connect callback
   */
  onConnect(callback: ConnectionCallback): () => void {
    this.connectCallbacks.add(callback);
    return () => this.connectCallbacks.delete(callback);
  }

  /**
   * Register disconnect callback
   */
  onDisconnect(callback: ConnectionCallback): () => void {
    this.disconnectCallbacks.add(callback);
    return () => this.disconnectCallbacks.delete(callback);
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  /**
   * Get connection state
   */
  getState(): 'connecting' | 'open' | 'closing' | 'closed' {
    if (!this.ws) return 'closed';

    switch (this.ws.readyState) {
      case WebSocket.CONNECTING:
        return 'connecting';
      case WebSocket.OPEN:
        return 'open';
      case WebSocket.CLOSING:
        return 'closing';
      case WebSocket.CLOSED:
        return 'closed';
      default:
        return 'closed';
    }
  }

  private handleMessage(data: string): void {
    try {
      const message: WebSocketMessage = JSON.parse(data);

      switch (message.type) {
        case 'event':
          if (message.event) {
            this.eventCallbacks.forEach((cb) => cb(message.event!));
          }
          break;

        case 'state':
          if (message.data) {
            this.stateCallbacks.forEach((cb) => cb(message.data!));
          }
          break;

        case 'error':
          const error = new Error(message.message ?? 'Unknown error');
          this.errorCallbacks.forEach((cb) => cb(error));
          break;

        case 'pong':
          // Heartbeat response received
          break;
      }
    } catch (error) {
      console.error('Failed to parse WebSocket message:', error);
      const err = error instanceof Error ? error : new Error('Failed to parse message');
      this.errorCallbacks.forEach((cb) => cb(err));
    }
  }

  private send(data: unknown): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      try {
        this.ws.send(JSON.stringify(data));
      } catch (error) {
        console.error('Failed to send WebSocket message:', error);
        const err = error instanceof Error ? error : new Error('Failed to send message');
        this.errorCallbacks.forEach((cb) => cb(err));
      }
    } else {
      console.warn('WebSocket not connected, cannot send message');
    }
  }

  private startHeartbeat(): void {
    this.stopHeartbeat();
    this.heartbeatTimer = setInterval(() => {
      this.send({ type: 'ping' });
    }, this.config.heartbeatInterval);
  }

  private stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }

  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.config.maxReconnectAttempts) {
      console.error('Max reconnect attempts reached');
      const error = new Error('Max reconnect attempts reached');
      this.errorCallbacks.forEach((cb) => cb(error));
      return;
    }

    this.clearReconnectTimer();

    const delay = this.config.reconnectInterval * Math.pow(1.5, this.reconnectAttempts);
    console.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts + 1})`);

    this.reconnectTimer = setTimeout(() => {
      this.reconnectAttempts++;
      this.connect();
    }, delay);
  }

  private clearReconnectTimer(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }

  /**
   * Cleanup resources
   */
  destroy(): void {
    this.disconnect();
    this.eventCallbacks.clear();
    this.stateCallbacks.clear();
    this.errorCallbacks.clear();
    this.connectCallbacks.clear();
    this.disconnectCallbacks.clear();
  }
}
