/**
 * Event Bus Type Definitions for Enterprise SaaS Platform
 * @module @harborgrid/enterprise-shared/types/events
 */

import { z } from 'zod';

// ============================================================================
// Event Base Types
// ============================================================================

export interface EnterpriseEvent<T = unknown> {
  id: string;
  type: EventType;
  version: string;
  timestamp: Date;
  source: EventSource;
  tenantId?: string;
  userId?: string;
  data: T;
  metadata: EventMetadata;
}

export interface EventMetadata {
  correlationId?: string;
  causationId?: string;
  requestId?: string;
  sessionId?: string;
  environment?: string;
  tags?: string[];
  custom?: Record<string, unknown>;
}

export const EnterpriseEventSchema = z.object({
  id: z.string().uuid(),
  type: z.string(),
  version: z.string().regex(/^\d+\.\d+\.\d+$/),
  timestamp: z.date(),
  source: z.string(),
  tenantId: z.string().uuid().optional(),
  userId: z.string().uuid().optional(),
  data: z.unknown(),
  metadata: z.object({
    correlationId: z.string().uuid().optional(),
    causationId: z.string().uuid().optional(),
    requestId: z.string().uuid().optional(),
    sessionId: z.string().uuid().optional(),
    environment: z.string().optional(),
    tags: z.array(z.string()).optional(),
    custom: z.record(z.unknown()).optional(),
  }),
});

// ============================================================================
// Event Sources
// ============================================================================

export enum EventSource {
  ANALYTICS = 'analytics',
  BILLING = 'billing',
  CAD_EDITOR = 'cad-editor',
  COLLABORATION = 'collaboration',
  COMPRESSION = 'compression',
  GATEWAY = 'gateway',
  NOTIFICATIONS = 'notifications',
  SECURITY = 'security',
  SPATIAL = 'spatial',
  WORKFLOW = 'workflow',
  SYSTEM = 'system',
}

// ============================================================================
// Event Types
// ============================================================================

export type EventType =
  | AnalyticsEventType
  | BillingEventType
  | CADEventType
  | CollaborationEventType
  | CompressionEventType
  | GatewayEventType
  | NotificationEventType
  | SecurityEventType
  | SpatialEventType
  | WorkflowEventType
  | SystemEventType;

// Analytics Events
export enum AnalyticsEventType {
  QUERY_EXECUTED = 'analytics.query.executed',
  DASHBOARD_CREATED = 'analytics.dashboard.created',
  DASHBOARD_UPDATED = 'analytics.dashboard.updated',
  DASHBOARD_DELETED = 'analytics.dashboard.deleted',
  REPORT_GENERATED = 'analytics.report.generated',
  THRESHOLD_EXCEEDED = 'analytics.threshold.exceeded',
  DATA_SOURCE_CONNECTED = 'analytics.data_source.connected',
  DATA_SOURCE_FAILED = 'analytics.data_source.failed',
}

// Billing Events
export enum BillingEventType {
  SUBSCRIPTION_CREATED = 'billing.subscription.created',
  SUBSCRIPTION_UPDATED = 'billing.subscription.updated',
  SUBSCRIPTION_CANCELED = 'billing.subscription.canceled',
  INVOICE_CREATED = 'billing.invoice.created',
  INVOICE_PAID = 'billing.invoice.paid',
  INVOICE_FAILED = 'billing.invoice.failed',
  PAYMENT_SUCCEEDED = 'billing.payment.succeeded',
  PAYMENT_FAILED = 'billing.payment.failed',
  USAGE_THRESHOLD = 'billing.usage.threshold',
  PLAN_CHANGED = 'billing.plan.changed',
}

// CAD Editor Events
export enum CADEventType {
  SHAPE_ADDED = 'cad.shape.added',
  SHAPE_MODIFIED = 'cad.shape.modified',
  SHAPE_REMOVED = 'cad.shape.removed',
  LAYER_ADDED = 'cad.layer.added',
  LAYER_MODIFIED = 'cad.layer.modified',
  DOCUMENT_SAVED = 'cad.document.saved',
  DOCUMENT_LOADED = 'cad.document.loaded',
  EXPORT_COMPLETED = 'cad.export.completed',
}

// Collaboration Events
export enum CollaborationEventType {
  PARTICIPANT_JOINED = 'collab.participant.joined',
  PARTICIPANT_LEFT = 'collab.participant.left',
  OPERATION_APPLIED = 'collab.operation.applied',
  CONFLICT_DETECTED = 'collab.conflict.detected',
  CONFLICT_RESOLVED = 'collab.conflict.resolved',
  COMMENT_ADDED = 'collab.comment.added',
  COMMENT_UPDATED = 'collab.comment.updated',
  COMMENT_RESOLVED = 'collab.comment.resolved',
}

// Compression Events
export enum CompressionEventType {
  COMPRESSION_STARTED = 'compression.started',
  COMPRESSION_COMPLETED = 'compression.completed',
  COMPRESSION_FAILED = 'compression.failed',
  CACHE_HIT = 'compression.cache.hit',
  CACHE_MISS = 'compression.cache.miss',
  CACHE_CLEARED = 'compression.cache.cleared',
}

// Gateway Events
export enum GatewayEventType {
  REQUEST_RECEIVED = 'gateway.request.received',
  REQUEST_COMPLETED = 'gateway.request.completed',
  REQUEST_FAILED = 'gateway.request.failed',
  RATE_LIMIT_EXCEEDED = 'gateway.rate_limit.exceeded',
  CIRCUIT_BREAKER_OPENED = 'gateway.circuit_breaker.opened',
  CIRCUIT_BREAKER_CLOSED = 'gateway.circuit_breaker.closed',
  UPSTREAM_FAILURE = 'gateway.upstream.failure',
  ROUTE_ADDED = 'gateway.route.added',
}

// Notification Events
export enum NotificationEventType {
  NOTIFICATION_SENT = 'notification.sent',
  NOTIFICATION_DELIVERED = 'notification.delivered',
  NOTIFICATION_FAILED = 'notification.failed',
  NOTIFICATION_READ = 'notification.read',
  ALERT_TRIGGERED = 'alert.triggered',
  ALERT_ACKNOWLEDGED = 'alert.acknowledged',
  ALERT_ESCALATED = 'alert.escalated',
  ALERT_RESOLVED = 'alert.resolved',
  INCIDENT_CREATED = 'incident.created',
  INCIDENT_UPDATED = 'incident.updated',
}

// Security Events
export enum SecurityEventType {
  LOGIN_SUCCESS = 'security.login.success',
  LOGIN_FAILURE = 'security.login.failure',
  LOGOUT = 'security.logout',
  PASSWORD_CHANGED = 'security.password.changed',
  MFA_ENABLED = 'security.mfa.enabled',
  ACCESS_GRANTED = 'security.access.granted',
  ACCESS_DENIED = 'security.access.denied',
  PERMISSION_CHANGED = 'security.permission.changed',
  THREAT_DETECTED = 'security.threat.detected',
  POLICY_VIOLATED = 'security.policy.violated',
  AUDIT_LOGGED = 'security.audit.logged',
}

// Spatial Events
export enum SpatialEventType {
  ANALYSIS_STARTED = 'spatial.analysis.started',
  ANALYSIS_COMPLETED = 'spatial.analysis.completed',
  ANALYSIS_FAILED = 'spatial.analysis.failed',
  TILE_GENERATED = 'spatial.tile.generated',
  GEOCODE_COMPLETED = 'spatial.geocode.completed',
  PROJECTION_TRANSFORMED = 'spatial.projection.transformed',
}

// Workflow Events
export enum WorkflowEventType {
  WORKFLOW_CREATED = 'workflow.created',
  WORKFLOW_UPDATED = 'workflow.updated',
  WORKFLOW_TRIGGERED = 'workflow.triggered',
  EXECUTION_STARTED = 'workflow.execution.started',
  EXECUTION_COMPLETED = 'workflow.execution.completed',
  EXECUTION_FAILED = 'workflow.execution.failed',
  STEP_STARTED = 'workflow.step.started',
  STEP_COMPLETED = 'workflow.step.completed',
  APPROVAL_REQUESTED = 'workflow.approval.requested',
  APPROVAL_GRANTED = 'workflow.approval.granted',
}

// System Events
export enum SystemEventType {
  TENANT_CREATED = 'system.tenant.created',
  TENANT_SUSPENDED = 'system.tenant.suspended',
  USER_CREATED = 'system.user.created',
  USER_UPDATED = 'system.user.updated',
  FEATURE_FLAG_CHANGED = 'system.feature_flag.changed',
  MAINTENANCE_STARTED = 'system.maintenance.started',
  MAINTENANCE_COMPLETED = 'system.maintenance.completed',
  SERVICE_STARTED = 'system.service.started',
  SERVICE_STOPPED = 'system.service.stopped',
}

// ============================================================================
// Event Payloads
// ============================================================================

export interface AnalyticsQueryExecutedPayload {
  queryId: string;
  dashboardId?: string;
  dataSourceId: string;
  executionTime: number;
  rowCount: number;
  cached: boolean;
}

export interface BillingSubscriptionCreatedPayload {
  subscriptionId: string;
  planId: string;
  status: string;
  amount: number;
  currency: string;
}

export interface CADShapeModifiedPayload {
  documentId: string;
  shapeId: string;
  operation: string;
  properties: Record<string, unknown>;
}

export interface CollaborationParticipantJoinedPayload {
  sessionId: string;
  participantId: string;
  documentId: string;
  role: string;
}

export interface GatewayRateLimitExceededPayload {
  consumerId?: string;
  route: string;
  limit: number;
  remaining: number;
  resetAt: number;
}

export interface NotificationAlertTriggeredPayload {
  alertId: string;
  severity: string;
  message: string;
  source: string;
  ruleId?: string;
}

export interface SecurityThreatDetectedPayload {
  threatId: string;
  type: string;
  severity: string;
  source: string;
  target: string;
  description: string;
}

export interface WorkflowExecutionStartedPayload {
  executionId: string;
  workflowId: string;
  triggeredBy: string;
  variables: Record<string, unknown>;
}

// ============================================================================
// Event Bus Interface
// ============================================================================

export interface EventBus {
  publish<T = unknown>(event: EnterpriseEvent<T>): Promise<void>;
  subscribe<T = unknown>(
    eventType: EventType | EventType[],
    handler: EventHandler<T>
  ): EventSubscription;
  unsubscribe(subscription: EventSubscription): void;
}

export type EventHandler<T = unknown> = (
  event: EnterpriseEvent<T>
) => Promise<void> | void;

export interface EventSubscription {
  id: string;
  eventTypes: EventType[];
  handler: EventHandler;
  unsubscribe: () => void;
}

// ============================================================================
// Event Store Interface
// ============================================================================

export interface EventStore {
  save<T = unknown>(event: EnterpriseEvent<T>): Promise<void>;
  load(options: EventLoadOptions): Promise<EnterpriseEvent[]>;
  replay(options: EventReplayOptions): AsyncIterableIterator<EnterpriseEvent>;
}

export interface EventLoadOptions {
  eventTypes?: EventType[];
  tenantId?: string;
  userId?: string;
  startTime?: Date;
  endTime?: Date;
  limit?: number;
  offset?: number;
}

export interface EventReplayOptions extends EventLoadOptions {
  batchSize?: number;
  speed?: number; // Playback speed multiplier
}

// ============================================================================
// Event Filtering
// ============================================================================

export interface EventFilter {
  eventTypes?: EventType[];
  sources?: EventSource[];
  tenantIds?: string[];
  userIds?: string[];
  startTime?: Date;
  endTime?: Date;
  tags?: string[];
  custom?: (event: EnterpriseEvent) => boolean;
}

export function filterEvent(event: EnterpriseEvent, filter: EventFilter): boolean {
  if (filter.eventTypes && !filter.eventTypes.includes(event.type)) {
    return false;
  }
  if (filter.sources && !filter.sources.includes(event.source)) {
    return false;
  }
  if (filter.tenantIds && event.tenantId && !filter.tenantIds.includes(event.tenantId)) {
    return false;
  }
  if (filter.userIds && event.userId && !filter.userIds.includes(event.userId)) {
    return false;
  }
  if (filter.startTime && event.timestamp < filter.startTime) {
    return false;
  }
  if (filter.endTime && event.timestamp > filter.endTime) {
    return false;
  }
  if (filter.tags && filter.tags.length > 0) {
    const eventTags = event.metadata.tags || [];
    if (!filter.tags.some((tag) => eventTags.includes(tag))) {
      return false;
    }
  }
  if (filter.custom && !filter.custom(event)) {
    return false;
  }
  return true;
}

// ============================================================================
// Event Builder
// ============================================================================

export class EventBuilder<T = unknown> {
  private event: Partial<EnterpriseEvent<T>> = {
    metadata: {},
  };

  constructor(type: EventType, source: EventSource) {
    this.event.id = crypto.randomUUID();
    this.event.type = type;
    this.event.source = source;
    this.event.version = '1.0.0';
    this.event.timestamp = new Date();
  }

  withTenant(tenantId: string): this {
    this.event.tenantId = tenantId;
    return this;
  }

  withUser(userId: string): this {
    this.event.userId = userId;
    return this;
  }

  withData(data: T): this {
    this.event.data = data;
    return this;
  }

  withCorrelationId(correlationId: string): this {
    this.event.metadata!.correlationId = correlationId;
    return this;
  }

  withCausationId(causationId: string): this {
    this.event.metadata!.causationId = causationId;
    return this;
  }

  withRequestId(requestId: string): this {
    this.event.metadata!.requestId = requestId;
    return this;
  }

  withTags(tags: string[]): this {
    this.event.metadata!.tags = tags;
    return this;
  }

  withCustomMetadata(key: string, value: unknown): this {
    if (!this.event.metadata!.custom) {
      this.event.metadata!.custom = {};
    }
    this.event.metadata!.custom[key] = value;
    return this;
  }

  build(): EnterpriseEvent<T> {
    if (!this.event.data) {
      throw new Error('Event data is required');
    }
    return this.event as EnterpriseEvent<T>;
  }
}

// ============================================================================
// Export all types
// ============================================================================

export type {
  EnterpriseEvent,
  EventMetadata,
  EventHandler,
  EventSubscription,
  EventBus,
  EventStore,
  EventLoadOptions,
  EventReplayOptions,
  EventFilter,
};
