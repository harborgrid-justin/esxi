/**
 * WebSocket Connection Manager
 * Manages WebSocket lifecycle, reconnection, and message handling
 */

import {
  ConnectionState,
  ConnectionConfig,
  ConnectionMetrics,
  Message,
  MessageType,
  CollaborationError,
  ErrorCode,
} from '../types';
import { ReconnectionStrategy } from './ReconnectionStrategy';
import { MessageProtocol } from './MessageProtocol';

export type MessageHandler = (message: Message) => void;
export type StateChangeHandler = (state: ConnectionState) => void;
export type ErrorHandler = (error: Error) => void;

export class ConnectionManager {
  private ws: WebSocket | null = null;
  private config: Required<ConnectionConfig>;
  private state: ConnectionState = ConnectionState.DISCONNECTED;
  private reconnectionStrategy: ReconnectionStrategy;
  private messageProtocol: MessageProtocol;

  private messageHandlers: Set<MessageHandler> = new Set();
  private stateHandlers: Set<StateChangeHandler> = new Set();
  private errorHandlers: Set<ErrorHandler> = new Set();

  private heartbeatInterval: NodeJS.Timeout | null = null;
  private connectionTimeout: NodeJS.Timeout | null = null;
  private lastHeartbeat: Date | null = null;

  private metrics: ConnectionMetrics = {
    latency: 0,
    messagesSent: 0,
    messagesReceived: 0,
    byteseSent: 0,
    bytesReceived: 0,
    reconnections: 0,
    errors: 0,
  };

  constructor(config: ConnectionConfig) {
    this.config = {
      url: config.url,
      protocols: config.protocols || [],
      reconnect: config.reconnect ?? true,
      reconnectAttempts: config.reconnectAttempts ?? 5,
      reconnectInterval: config.reconnectInterval ?? 1000,
      reconnectBackoff: config.reconnectBackoff ?? true,
      heartbeatInterval: config.heartbeatInterval ?? 30000,
      timeout: config.timeout ?? 10000,
      headers: config.headers || {},
    };

    this.reconnectionStrategy = new ReconnectionStrategy({
      maxAttempts: this.config.reconnectAttempts,
      baseInterval: this.config.reconnectInterval,
      backoff: this.config.reconnectBackoff,
    });

    this.messageProtocol = new MessageProtocol();
  }

  /**
   * Connect to WebSocket server
   */
  async connect(): Promise<void> {
    if (this.state === ConnectionState.CONNECTED ||
        this.state === ConnectionState.CONNECTING) {
      return;
    }

    this.setState(ConnectionState.CONNECTING);

    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(this.config.url, this.config.protocols);

        // Set up event handlers
        this.ws.onopen = () => {
          this.handleOpen();
          resolve();
        };

        this.ws.onmessage = (event) => {
          this.handleMessage(event);
        };

        this.ws.onerror = (event) => {
          this.handleError(event);
          reject(new CollaborationError(
            ErrorCode.CONNECTION_FAILED,
            'WebSocket connection failed'
          ));
        };

        this.ws.onclose = (event) => {
          this.handleClose(event);
        };

        // Set connection timeout
        this.connectionTimeout = setTimeout(() => {
          if (this.state === ConnectionState.CONNECTING) {
            this.ws?.close();
            reject(new CollaborationError(
              ErrorCode.TIMEOUT,
              'Connection timeout'
            ));
          }
        }, this.config.timeout);

      } catch (error) {
        this.handleError(error as Error);
        reject(error);
      }
    });
  }

  /**
   * Disconnect from WebSocket server
   */
  disconnect(): void {
    this.stopHeartbeat();

    if (this.connectionTimeout) {
      clearTimeout(this.connectionTimeout);
      this.connectionTimeout = null;
    }

    if (this.ws) {
      this.ws.close(1000, 'Client disconnect');
      this.ws = null;
    }

    this.setState(ConnectionState.DISCONNECTED);
    this.reconnectionStrategy.reset();
  }

  /**
   * Send a message
   */
  send<T = unknown>(message: Message<T>): void {
    if (!this.ws || this.state !== ConnectionState.CONNECTED) {
      throw new CollaborationError(
        ErrorCode.INVALID_STATE,
        'WebSocket is not connected'
      );
    }

    try {
      const encoded = this.messageProtocol.encode(message);
      this.ws.send(encoded);

      this.metrics.messagesSent++;
      this.metrics.byteseSent += encoded.byteLength;
    } catch (error) {
      this.handleError(error as Error);
      throw error;
    }
  }

  /**
   * Handle WebSocket open event
   */
  private handleOpen(): void {
    if (this.connectionTimeout) {
      clearTimeout(this.connectionTimeout);
      this.connectionTimeout = null;
    }

    this.setState(ConnectionState.CONNECTED);
    this.reconnectionStrategy.reset();
    this.startHeartbeat();
  }

  /**
   * Handle WebSocket message event
   */
  private handleMessage(event: MessageEvent): void {
    try {
      const data = event.data;
      const size = data instanceof ArrayBuffer ?
        data.byteLength :
        new Blob([data]).size;

      this.metrics.messagesReceived++;
      this.metrics.bytesReceived += size;

      const message = this.messageProtocol.decode(data);

      // Handle heartbeat
      if (message.type === MessageType.HEARTBEAT) {
        this.handleHeartbeat(message);
        return;
      }

      // Notify handlers
      this.messageHandlers.forEach((handler) => {
        try {
          handler(message);
        } catch (error) {
          console.error('Message handler error:', error);
        }
      });

    } catch (error) {
      this.handleError(error as Error);
    }
  }

  /**
   * Handle WebSocket error event
   */
  private handleError(error: Error | Event): void {
    this.metrics.errors++;

    const err = error instanceof Error ?
      error :
      new CollaborationError(ErrorCode.UNKNOWN, 'WebSocket error');

    this.metrics.lastError = err;

    this.errorHandlers.forEach((handler) => {
      try {
        handler(err);
      } catch (e) {
        console.error('Error handler error:', e);
      }
    });
  }

  /**
   * Handle WebSocket close event
   */
  private async handleClose(event: CloseEvent): Promise<void> {
    this.stopHeartbeat();

    if (this.state === ConnectionState.DISCONNECTED) {
      return;
    }

    // Clean close - don't reconnect
    if (event.code === 1000 || !this.config.reconnect) {
      this.setState(ConnectionState.DISCONNECTED);
      return;
    }

    // Attempt reconnection
    this.setState(ConnectionState.RECONNECTING);
    this.metrics.reconnections++;

    const shouldReconnect = await this.reconnectionStrategy.shouldReconnect();

    if (shouldReconnect) {
      const delay = this.reconnectionStrategy.getNextDelay();

      setTimeout(async () => {
        try {
          await this.connect();
        } catch (error) {
          console.error('Reconnection failed:', error);
          this.handleClose(event);
        }
      }, delay);
    } else {
      this.setState(ConnectionState.FAILED);
      this.handleError(
        new CollaborationError(
          ErrorCode.CONNECTION_FAILED,
          'Max reconnection attempts reached'
        )
      );
    }
  }

  /**
   * Start heartbeat
   */
  private startHeartbeat(): void {
    this.stopHeartbeat();

    this.heartbeatInterval = setInterval(() => {
      if (this.state === ConnectionState.CONNECTED) {
        this.sendHeartbeat();
      }
    }, this.config.heartbeatInterval);
  }

  /**
   * Stop heartbeat
   */
  private stopHeartbeat(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }
  }

  /**
   * Send heartbeat message
   */
  private sendHeartbeat(): void {
    const heartbeat: Message = {
      id: `heartbeat_${Date.now()}`,
      type: MessageType.HEARTBEAT,
      payload: { timestamp: Date.now() },
      senderId: 'client',
      timestamp: new Date(),
    };

    try {
      this.send(heartbeat);
      this.lastHeartbeat = new Date();
    } catch (error) {
      console.error('Heartbeat failed:', error);
    }
  }

  /**
   * Handle heartbeat response
   */
  private handleHeartbeat(message: Message): void {
    if (this.lastHeartbeat) {
      const latency = Date.now() - this.lastHeartbeat.getTime();
      this.metrics.latency = latency;
    }
  }

  /**
   * Set connection state
   */
  private setState(state: ConnectionState): void {
    if (this.state === state) {
      return;
    }

    this.state = state;

    this.stateHandlers.forEach((handler) => {
      try {
        handler(state);
      } catch (error) {
        console.error('State handler error:', error);
      }
    });
  }

  /**
   * Register message handler
   */
  onMessage(handler: MessageHandler): () => void {
    this.messageHandlers.add(handler);
    return () => this.messageHandlers.delete(handler);
  }

  /**
   * Register state change handler
   */
  onStateChange(handler: StateChangeHandler): () => void {
    this.stateHandlers.add(handler);
    return () => this.stateHandlers.delete(handler);
  }

  /**
   * Register error handler
   */
  onError(handler: ErrorHandler): () => void {
    this.errorHandlers.add(handler);
    return () => this.errorHandlers.delete(handler);
  }

  /**
   * Get current state
   */
  getState(): ConnectionState {
    return this.state;
  }

  /**
   * Get metrics
   */
  getMetrics(): ConnectionMetrics {
    return { ...this.metrics };
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.state === ConnectionState.CONNECTED;
  }
}
