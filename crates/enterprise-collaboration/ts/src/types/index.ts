/**
 * Enterprise Real-time Collaboration Engine - Core Types
 * @module types
 */

// ============================================================================
// Session & Participant Types
// ============================================================================

export enum ParticipantRole {
  OWNER = 'owner',
  EDITOR = 'editor',
  VIEWER = 'viewer',
  COMMENTER = 'commenter',
}

export enum ParticipantStatus {
  ACTIVE = 'active',
  IDLE = 'idle',
  AWAY = 'away',
  OFFLINE = 'offline',
}

export interface Participant {
  id: string;
  sessionId: string;
  userId: string;
  displayName: string;
  avatarUrl?: string;
  role: ParticipantRole;
  status: ParticipantStatus;
  color: string;
  joinedAt: Date;
  lastSeenAt: Date;
  metadata?: Record<string, unknown>;
}

export interface CollaborationSession {
  id: string;
  documentId: string;
  participants: Map<string, Participant>;
  createdAt: Date;
  updatedAt: Date;
  metadata?: Record<string, unknown>;
}

// ============================================================================
// Cursor & Selection Types
// ============================================================================

export interface Position {
  line: number;
  column: number;
  offset?: number;
}

export interface Range {
  start: Position;
  end: Position;
}

export interface Cursor {
  participantId: string;
  position: Position;
  timestamp: Date;
}

export interface Selection {
  participantId: string;
  ranges: Range[];
  timestamp: Date;
  primary?: Range;
}

// ============================================================================
// Presence State Types
// ============================================================================

export interface PresenceState {
  participantId: string;
  cursor?: Cursor;
  selection?: Selection;
  viewport?: {
    top: number;
    bottom: number;
    left: number;
    right: number;
  };
  activeElement?: string;
  isTyping: boolean;
  lastActivity: Date;
  customData?: Record<string, unknown>;
}

// ============================================================================
// Permission Types
// ============================================================================

export enum PermissionAction {
  READ = 'read',
  WRITE = 'write',
  DELETE = 'delete',
  COMMENT = 'comment',
  SHARE = 'share',
  ADMIN = 'admin',
}

export interface Permission {
  action: PermissionAction;
  resource: string;
  granted: boolean;
  conditions?: Record<string, unknown>;
}

export interface PermissionSet {
  participantId: string;
  permissions: Permission[];
  inheritedFrom?: string[];
}

// ============================================================================
// Document State Types
// ============================================================================

export enum DocumentType {
  TEXT = 'text',
  JSON = 'json',
  RICH_TEXT = 'rich_text',
  SPREADSHEET = 'spreadsheet',
  CUSTOM = 'custom',
}

export interface DocumentMetadata {
  id: string;
  type: DocumentType;
  version: number;
  createdAt: Date;
  updatedAt: Date;
  createdBy: string;
  lastModifiedBy: string;
  title?: string;
  tags?: string[];
  customData?: Record<string, unknown>;
}

export interface DocumentState {
  metadata: DocumentMetadata;
  content: unknown;
  checksum: string;
  vectorClock: Record<string, number>;
}

// ============================================================================
// Operation Types (CRDT/OT)
// ============================================================================

export enum OperationType {
  INSERT = 'insert',
  DELETE = 'delete',
  REPLACE = 'replace',
  MOVE = 'move',
  FORMAT = 'format',
  CUSTOM = 'custom',
}

export interface Operation {
  id: string;
  type: OperationType;
  position: Position;
  content?: unknown;
  length?: number;
  participantId: string;
  timestamp: Date;
  vectorClock: Record<string, number>;
  metadata?: Record<string, unknown>;
}

export interface OperationLog {
  operations: Operation[];
  lastSequenceNumber: number;
  checkpoints: Map<number, DocumentState>;
}

// ============================================================================
// Conflict Resolution Types
// ============================================================================

export enum ConflictType {
  CONCURRENT_EDIT = 'concurrent_edit',
  ORDERING = 'ordering',
  CAUSALITY = 'causality',
  SEMANTIC = 'semantic',
}

export enum ConflictResolutionStrategy {
  LAST_WRITE_WINS = 'last_write_wins',
  FIRST_WRITE_WINS = 'first_write_wins',
  MERGE = 'merge',
  MANUAL = 'manual',
  CUSTOM = 'custom',
}

export interface Conflict {
  id: string;
  type: ConflictType;
  operations: Operation[];
  affectedRanges: Range[];
  detectedAt: Date;
  resolved: boolean;
  resolution?: ConflictResolution;
}

export interface ConflictResolution {
  conflictId: string;
  strategy: ConflictResolutionStrategy;
  selectedOperation?: string;
  mergedResult?: unknown;
  resolvedBy: string;
  resolvedAt: Date;
  metadata?: Record<string, unknown>;
}

// ============================================================================
// Synchronization Types
// ============================================================================

export enum SyncState {
  SYNCED = 'synced',
  SYNCING = 'syncing',
  OUT_OF_SYNC = 'out_of_sync',
  CONFLICT = 'conflict',
  ERROR = 'error',
}

export interface SyncStatus {
  state: SyncState;
  lastSyncAt?: Date;
  pendingOperations: number;
  unresolvedConflicts: number;
  error?: Error;
}

export interface SyncMessage {
  type: 'sync' | 'ack' | 'nack' | 'checkpoint';
  operations?: Operation[];
  vectorClock: Record<string, number>;
  timestamp: Date;
  senderId: string;
  sequenceNumber?: number;
}

// ============================================================================
// WebSocket Message Types
// ============================================================================

export enum MessageType {
  // Connection lifecycle
  CONNECT = 'connect',
  DISCONNECT = 'disconnect',
  HEARTBEAT = 'heartbeat',

  // Collaboration
  OPERATION = 'operation',
  SYNC = 'sync',
  CHECKPOINT = 'checkpoint',

  // Presence
  PRESENCE_UPDATE = 'presence_update',
  CURSOR_MOVE = 'cursor_move',
  SELECTION_CHANGE = 'selection_change',

  // Comments
  COMMENT_ADD = 'comment_add',
  COMMENT_UPDATE = 'comment_update',
  COMMENT_DELETE = 'comment_delete',
  COMMENT_RESOLVE = 'comment_resolve',

  // Conflicts
  CONFLICT_DETECTED = 'conflict_detected',
  CONFLICT_RESOLVED = 'conflict_resolved',

  // System
  ERROR = 'error',
  ACK = 'ack',
  NACK = 'nack',
}

export interface Message<T = unknown> {
  id: string;
  type: MessageType;
  payload: T;
  senderId: string;
  timestamp: Date;
  sequenceNumber?: number;
  requiresAck?: boolean;
}

// ============================================================================
// Comment Types
// ============================================================================

export enum CommentStatus {
  OPEN = 'open',
  RESOLVED = 'resolved',
  DELETED = 'deleted',
}

export interface Comment {
  id: string;
  threadId: string;
  documentId: string;
  range: Range;
  content: string;
  authorId: string;
  authorName: string;
  createdAt: Date;
  updatedAt: Date;
  status: CommentStatus;
  reactions?: Map<string, string[]>; // emoji -> participant IDs
  metadata?: Record<string, unknown>;
}

export interface CommentThread {
  id: string;
  documentId: string;
  range: Range;
  comments: Comment[];
  status: CommentStatus;
  createdAt: Date;
  updatedAt: Date;
  participants: string[];
}

// ============================================================================
// Version History Types
// ============================================================================

export interface Version {
  id: string;
  documentId: string;
  number: number;
  state: DocumentState;
  operations: Operation[];
  createdBy: string;
  createdAt: Date;
  label?: string;
  description?: string;
  tags?: string[];
}

export interface VersionDiff {
  fromVersion: number;
  toVersion: number;
  additions: number;
  deletions: number;
  modifications: number;
  operations: Operation[];
}

// ============================================================================
// Audit Types
// ============================================================================

export enum AuditEventType {
  DOCUMENT_CREATED = 'document_created',
  DOCUMENT_UPDATED = 'document_updated',
  DOCUMENT_DELETED = 'document_deleted',
  PARTICIPANT_JOINED = 'participant_joined',
  PARTICIPANT_LEFT = 'participant_left',
  PERMISSION_GRANTED = 'permission_granted',
  PERMISSION_REVOKED = 'permission_revoked',
  CONFLICT_DETECTED = 'conflict_detected',
  CONFLICT_RESOLVED = 'conflict_resolved',
  VERSION_CREATED = 'version_created',
  COMMENT_ADDED = 'comment_added',
}

export interface AuditEvent {
  id: string;
  type: AuditEventType;
  documentId: string;
  participantId: string;
  timestamp: Date;
  details: Record<string, unknown>;
  ipAddress?: string;
  userAgent?: string;
}

// ============================================================================
// WebSocket Connection Types
// ============================================================================

export enum ConnectionState {
  CONNECTING = 'connecting',
  CONNECTED = 'connected',
  RECONNECTING = 'reconnecting',
  DISCONNECTED = 'disconnected',
  FAILED = 'failed',
}

export interface ConnectionConfig {
  url: string;
  protocols?: string[];
  reconnect?: boolean;
  reconnectAttempts?: number;
  reconnectInterval?: number;
  reconnectBackoff?: boolean;
  heartbeatInterval?: number;
  timeout?: number;
  headers?: Record<string, string>;
}

export interface ConnectionMetrics {
  latency: number;
  messagesSent: number;
  messagesReceived: number;
  byteseSent: number;
  bytesReceived: number;
  reconnections: number;
  errors: number;
  lastError?: Error;
}

// ============================================================================
// Error Types
// ============================================================================

export enum ErrorCode {
  UNKNOWN = 'UNKNOWN',
  CONNECTION_FAILED = 'CONNECTION_FAILED',
  AUTHENTICATION_FAILED = 'AUTHENTICATION_FAILED',
  PERMISSION_DENIED = 'PERMISSION_DENIED',
  OPERATION_FAILED = 'OPERATION_FAILED',
  SYNC_FAILED = 'SYNC_FAILED',
  CONFLICT_RESOLUTION_FAILED = 'CONFLICT_RESOLUTION_FAILED',
  INVALID_STATE = 'INVALID_STATE',
  TIMEOUT = 'TIMEOUT',
}

export class CollaborationError extends Error {
  constructor(
    public code: ErrorCode,
    message: string,
    public details?: Record<string, unknown>
  ) {
    super(message);
    this.name = 'CollaborationError';
    Object.setPrototypeOf(this, CollaborationError.prototype);
  }
}
