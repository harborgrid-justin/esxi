/**
 * Collaboration Provider - React Context Provider
 * Provides collaboration state and operations to child components
 */

import React, { createContext, useContext, useEffect, useState, useCallback, ReactNode } from 'react';
import {
  CollaborationSession,
  Participant,
  PresenceState,
  SyncStatus,
  SyncState,
  Operation,
  Conflict,
  ConnectionState,
  Message,
} from '../types';
import { ConnectionManager } from '../websocket/ConnectionManager';
import { CRDTDocument } from '../crdt/CRDTDocument';
import { SyncService } from '../services/SyncService';
import { PermissionService } from '../services/PermissionService';

export interface CollaborationContextValue {
  // Session
  session: CollaborationSession | null;
  participants: Map<string, Participant>;
  currentParticipant: Participant | null;

  // Connection
  connectionState: ConnectionState;
  isConnected: boolean;

  // Document
  document: CRDTDocument | null;
  content: string;

  // Sync
  syncStatus: SyncStatus;

  // Presence
  presenceStates: Map<string, PresenceState>;

  // Conflicts
  conflicts: Conflict[];

  // Operations
  insert: (content: string, position: number) => void;
  delete: (position: number, length: number) => void;
  updatePresence: (state: Partial<PresenceState>) => void;
  resolveConflict: (conflictId: string, resolution: any) => void;

  // Connection
  connect: () => Promise<void>;
  disconnect: () => void;
}

const CollaborationContext = createContext<CollaborationContextValue | null>(null);

export interface CollaborationProviderProps {
  children: ReactNode;
  sessionId: string;
  participantId: string;
  websocketUrl: string;
  onError?: (error: Error) => void;
}

export const CollaborationProvider: React.FC<CollaborationProviderProps> = ({
  children,
  sessionId,
  participantId,
  websocketUrl,
  onError,
}) => {
  // State
  const [session, setSession] = useState<CollaborationSession | null>(null);
  const [participants, setParticipants] = useState<Map<string, Participant>>(new Map());
  const [currentParticipant, setCurrentParticipant] = useState<Participant | null>(null);
  const [connectionState, setConnectionState] = useState<ConnectionState>(ConnectionState.DISCONNECTED);
  const [document, setDocument] = useState<CRDTDocument | null>(null);
  const [content, setContent] = useState<string>('');
  const [syncStatus, setSyncStatus] = useState<SyncStatus>({
    state: SyncState.OUT_OF_SYNC,
    pendingOperations: 0,
    unresolvedConflicts: 0,
  });
  const [presenceStates, setPresenceStates] = useState<Map<string, PresenceState>>(new Map());
  const [conflicts, setConflicts] = useState<Conflict[]>([]);

  // Services
  const [connectionManager] = useState(
    () => new ConnectionManager({ url: websocketUrl })
  );
  const [syncService] = useState(() => new SyncService());
  const [permissionService] = useState(() => new PermissionService());

  // Initialize document
  useEffect(() => {
    const doc = new CRDTDocument({ nodeId: participantId });
    setDocument(doc);
  }, [participantId]);

  // Set up connection event handlers
  useEffect(() => {
    const unsubscribeState = connectionManager.onStateChange((state) => {
      setConnectionState(state);
    });

    const unsubscribeMessage = connectionManager.onMessage((message) => {
      handleMessage(message);
    });

    const unsubscribeError = connectionManager.onError((error) => {
      onError?.(error);
    });

    return () => {
      unsubscribeState();
      unsubscribeMessage();
      unsubscribeError();
    };
  }, [connectionManager, onError]);

  // Update content when document changes
  useEffect(() => {
    if (document) {
      setContent(document.toString());
    }
  }, [document]);

  /**
   * Handle incoming messages
   */
  const handleMessage = useCallback((message: Message) => {
    switch (message.type) {
      case 'operation':
        handleOperationMessage(message);
        break;

      case 'presence_update':
        handlePresenceMessage(message);
        break;

      case 'sync':
        handleSyncMessage(message);
        break;

      case 'conflict_detected':
        handleConflictMessage(message);
        break;

      default:
        console.log('Unhandled message type:', message.type);
    }
  }, [document]);

  /**
   * Handle operation message
   */
  const handleOperationMessage = useCallback((message: Message) => {
    if (!document) return;

    const operation = message.payload as Operation;
    document.applyRemoteOperation(operation);
    setContent(document.toString());

    setSyncStatus((prev) => ({
      ...prev,
      lastSyncAt: new Date(),
    }));
  }, [document]);

  /**
   * Handle presence message
   */
  const handlePresenceMessage = useCallback((message: Message) => {
    const presence = message.payload as PresenceState;
    setPresenceStates((prev) => new Map(prev).set(presence.participantId, presence));
  }, []);

  /**
   * Handle sync message
   */
  const handleSyncMessage = useCallback((message: Message) => {
    setSyncStatus((prev) => ({
      ...prev,
      state: SyncState.SYNCED,
      lastSyncAt: new Date(),
    }));
  }, []);

  /**
   * Handle conflict message
   */
  const handleConflictMessage = useCallback((message: Message) => {
    const conflict = message.payload as Conflict;
    setConflicts((prev) => [...prev, conflict]);

    setSyncStatus((prev) => ({
      ...prev,
      state: SyncState.CONFLICT,
      unresolvedConflicts: prev.unresolvedConflicts + 1,
    }));
  }, []);

  /**
   * Connect to collaboration session
   */
  const connect = useCallback(async () => {
    try {
      await connectionManager.connect();
    } catch (error) {
      onError?.(error as Error);
      throw error;
    }
  }, [connectionManager, onError]);

  /**
   * Disconnect from collaboration session
   */
  const disconnect = useCallback(() => {
    connectionManager.disconnect();
  }, [connectionManager]);

  /**
   * Insert content
   */
  const insert = useCallback((content: string, position: number) => {
    if (!document) return;

    const operation = document.insert(content, position);
    setContent(document.toString());

    // Send to server
    if (connectionManager.isConnected()) {
      connectionManager.send({
        id: operation.id,
        type: 'operation' as any,
        payload: operation,
        senderId: participantId,
        timestamp: new Date(),
      });
    }
  }, [document, connectionManager, participantId]);

  /**
   * Delete content
   */
  const deleteContent = useCallback((position: number, length: number) => {
    if (!document) return;

    const operation = document.delete(position, length);
    setContent(document.toString());

    // Send to server
    if (connectionManager.isConnected()) {
      connectionManager.send({
        id: operation.id,
        type: 'operation' as any,
        payload: operation,
        senderId: participantId,
        timestamp: new Date(),
      });
    }
  }, [document, connectionManager, participantId]);

  /**
   * Update presence state
   */
  const updatePresence = useCallback((state: Partial<PresenceState>) => {
    const presence: PresenceState = {
      participantId,
      isTyping: false,
      lastActivity: new Date(),
      ...state,
    };

    setPresenceStates((prev) => new Map(prev).set(participantId, presence));

    // Send to server
    if (connectionManager.isConnected()) {
      connectionManager.send({
        id: `presence_${Date.now()}`,
        type: 'presence_update' as any,
        payload: presence,
        senderId: participantId,
        timestamp: new Date(),
      });
    }
  }, [connectionManager, participantId]);

  /**
   * Resolve conflict
   */
  const resolveConflict = useCallback((conflictId: string, resolution: any) => {
    setConflicts((prev) => prev.filter((c) => c.id !== conflictId));

    setSyncStatus((prev) => ({
      ...prev,
      unresolvedConflicts: Math.max(0, prev.unresolvedConflicts - 1),
      state: prev.unresolvedConflicts <= 1 ? SyncState.SYNCED : SyncState.CONFLICT,
    }));

    // Send resolution to server
    if (connectionManager.isConnected()) {
      connectionManager.send({
        id: `resolution_${Date.now()}`,
        type: 'conflict_resolved' as any,
        payload: { conflictId, resolution },
        senderId: participantId,
        timestamp: new Date(),
      });
    }
  }, [connectionManager, participantId]);

  const value: CollaborationContextValue = {
    session,
    participants,
    currentParticipant,
    connectionState,
    isConnected: connectionState === ConnectionState.CONNECTED,
    document,
    content,
    syncStatus,
    presenceStates,
    conflicts,
    insert,
    delete: deleteContent,
    updatePresence,
    resolveConflict,
    connect,
    disconnect,
  };

  return (
    <CollaborationContext.Provider value={value}>
      {children}
    </CollaborationContext.Provider>
  );
};

/**
 * Hook to access collaboration context
 */
export const useCollaborationContext = (): CollaborationContextValue => {
  const context = useContext(CollaborationContext);

  if (!context) {
    throw new Error('useCollaborationContext must be used within CollaborationProvider');
  }

  return context;
};
