/**
 * useCollaboration Hook
 * Main hook for collaboration features
 */

import { useCollaborationContext } from '../components/CollaborationProvider';
import type {
  CollaborationSession,
  Participant,
  SyncStatus,
  Operation,
  Conflict,
  ConnectionState,
} from '../types';

export interface UseCollaborationResult {
  // Session
  session: CollaborationSession | null;
  participants: Map<string, Participant>;
  currentParticipant: Participant | null;

  // Connection
  connectionState: ConnectionState;
  isConnected: boolean;

  // Document
  content: string;
  syncStatus: SyncStatus;

  // Conflicts
  conflicts: Conflict[];
  hasConflicts: boolean;

  // Operations
  insert: (content: string, position: number) => void;
  delete: (position: number, length: number) => void;
  resolveConflict: (conflictId: string, resolution: any) => void;

  // Connection
  connect: () => Promise<void>;
  disconnect: () => void;
}

/**
 * Main collaboration hook
 */
export const useCollaboration = (): UseCollaborationResult => {
  const context = useCollaborationContext();

  return {
    session: context.session,
    participants: context.participants,
    currentParticipant: context.currentParticipant,
    connectionState: context.connectionState,
    isConnected: context.isConnected,
    content: context.content,
    syncStatus: context.syncStatus,
    conflicts: context.conflicts,
    hasConflicts: context.conflicts.length > 0,
    insert: context.insert,
    delete: context.delete,
    resolveConflict: context.resolveConflict,
    connect: context.connect,
    disconnect: context.disconnect,
  };
};
