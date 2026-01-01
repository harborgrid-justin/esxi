/**
 * Enterprise Real-time Collaboration Engine
 * @module @esxi/enterprise-collaboration
 */

// ============================================================================
// Core Types
// ============================================================================
export * from './types';

// ============================================================================
// CRDT Implementation
// ============================================================================
export { CRDTDocument } from './crdt/CRDTDocument';
export { VectorClock, ClockComparison } from './crdt/VectorClock';
export { OperationalTransform } from './crdt/OperationalTransform';
export { MergeEngine } from './crdt/MergeEngine';

// ============================================================================
// WebSocket Management
// ============================================================================
export { ConnectionManager } from './websocket/ConnectionManager';
export { MessageProtocol, ProtocolVersion } from './websocket/MessageProtocol';
export { ReconnectionStrategy } from './websocket/ReconnectionStrategy';

// ============================================================================
// React Components
// ============================================================================
export {
  CollaborationProvider,
  useCollaborationContext,
} from './components/CollaborationProvider';
export { PresenceIndicator } from './components/PresenceIndicator';
export { CursorOverlay } from './components/CursorOverlay';
export { ConflictResolver } from './components/ConflictResolver';
export { VersionHistory } from './components/VersionHistory';
export { CommentThread } from './components/CommentThread';

// ============================================================================
// Custom Hooks
// ============================================================================
export { useCollaboration } from './hooks/useCollaboration';
export { usePresence } from './hooks/usePresence';
export { useCursors } from './hooks/useCursors';
export { useConflictResolution } from './hooks/useConflictResolution';

// ============================================================================
// Services
// ============================================================================
export { SyncService } from './services/SyncService';
export { PermissionService } from './services/PermissionService';
export { AuditService } from './services/AuditService';

// ============================================================================
// Re-export common types for convenience
// ============================================================================
export type {
  CollaborationSession,
  Participant,
  Cursor,
  Selection,
  PresenceState,
  Operation,
  Conflict,
  ConflictResolution,
  DocumentState,
  Version,
  Comment,
  CommentThread as CommentThreadType,
  Message,
  SyncStatus,
  ConnectionState,
  Permission,
  PermissionSet,
  AuditEvent,
} from './types';
