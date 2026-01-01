/**
 * Enterprise Notification & Alerting System - Core Types
 * Comprehensive type definitions for notification, alerting, and delivery management
 */

// ============================================================================
// Enums
// ============================================================================

export enum NotificationPriority {
  LOW = 'low',
  NORMAL = 'normal',
  HIGH = 'high',
  URGENT = 'urgent',
  CRITICAL = 'critical',
}

export enum NotificationStatus {
  PENDING = 'pending',
  QUEUED = 'queued',
  PROCESSING = 'processing',
  SENT = 'sent',
  DELIVERED = 'delivered',
  FAILED = 'failed',
  BOUNCED = 'bounced',
  REJECTED = 'rejected',
  READ = 'read',
  CLICKED = 'clicked',
}

export enum NotificationChannelType {
  EMAIL = 'email',
  SMS = 'sms',
  PUSH = 'push',
  SLACK = 'slack',
  TEAMS = 'teams',
  WEBHOOK = 'webhook',
  IN_APP = 'in_app',
}

export enum AlertSeverity {
  INFO = 'info',
  WARNING = 'warning',
  ERROR = 'error',
  CRITICAL = 'critical',
  FATAL = 'fatal',
}

export enum AlertStatus {
  OPEN = 'open',
  ACKNOWLEDGED = 'acknowledged',
  IN_PROGRESS = 'in_progress',
  RESOLVED = 'resolved',
  CLOSED = 'closed',
  SUPPRESSED = 'suppressed',
}

export enum RuleConditionOperator {
  EQUALS = 'equals',
  NOT_EQUALS = 'not_equals',
  GREATER_THAN = 'greater_than',
  GREATER_THAN_OR_EQUAL = 'greater_than_or_equal',
  LESS_THAN = 'less_than',
  LESS_THAN_OR_EQUAL = 'less_than_or_equal',
  CONTAINS = 'contains',
  NOT_CONTAINS = 'not_contains',
  MATCHES = 'matches',
  IN = 'in',
  NOT_IN = 'not_in',
}

export enum ThresholdType {
  STATIC = 'static',
  DYNAMIC = 'dynamic',
  PERCENTAGE = 'percentage',
  BASELINE = 'baseline',
}

export enum EscalationAction {
  NOTIFY = 'notify',
  REASSIGN = 'reassign',
  ESCALATE = 'escalate',
  CREATE_INCIDENT = 'create_incident',
  EXECUTE_WEBHOOK = 'execute_webhook',
  RUN_AUTOMATION = 'run_automation',
}

// ============================================================================
// Core Notification Types
// ============================================================================

export interface Notification {
  id: string;
  tenantId: string;
  userId?: string;
  type: string;
  category?: string;
  priority: NotificationPriority;
  status: NotificationStatus;

  // Content
  title: string;
  message: string;
  html?: string;
  data?: Record<string, unknown>;
  metadata?: Record<string, unknown>;

  // Delivery
  channels: NotificationChannelType[];
  recipients: NotificationRecipient[];
  scheduledFor?: Date;
  expiresAt?: Date;

  // Template
  templateId?: string;
  templateData?: Record<string, unknown>;

  // Tracking
  attempts: number;
  maxAttempts: number;
  lastAttemptAt?: Date;
  sentAt?: Date;
  deliveredAt?: Date;
  readAt?: Date;
  clickedAt?: Date;

  // Links
  actionUrl?: string;
  actionLabel?: string;
  links?: NotificationLink[];

  // Settings
  silent?: boolean;
  badge?: number;
  sound?: string;
  icon?: string;
  image?: string;

  // Deduplication
  deduplicationKey?: string;
  groupKey?: string;

  createdAt: Date;
  updatedAt: Date;
}

export interface NotificationRecipient {
  id: string;
  type: 'user' | 'group' | 'role' | 'email' | 'phone';
  identifier: string;
  name?: string;
  email?: string;
  phone?: string;
  locale?: string;
  timezone?: string;
  channels?: NotificationChannelType[];
  preferences?: NotificationPreference;
}

export interface NotificationLink {
  label: string;
  url: string;
  action?: string;
}

// ============================================================================
// Channel Types
// ============================================================================

export interface NotificationChannel {
  id: string;
  tenantId: string;
  type: NotificationChannelType;
  name: string;
  description?: string;
  enabled: boolean;
  priority: number;

  // Configuration
  config: ChannelConfig;

  // Limits
  rateLimit?: RateLimit;
  dailyQuota?: number;
  monthlyQuota?: number;

  // Tracking
  totalSent: number;
  totalFailed: number;
  lastUsedAt?: Date;

  createdAt: Date;
  updatedAt: Date;
}

export type ChannelConfig =
  | EmailChannelConfig
  | SMSChannelConfig
  | PushChannelConfig
  | SlackChannelConfig
  | TeamsChannelConfig
  | WebhookChannelConfig
  | InAppChannelConfig;

export interface EmailChannelConfig {
  type: 'email';
  provider: 'smtp' | 'ses' | 'sendgrid' | 'mailgun';
  from: string;
  fromName?: string;
  replyTo?: string;

  // SMTP
  host?: string;
  port?: number;
  secure?: boolean;
  auth?: {
    user: string;
    pass: string;
  };

  // AWS SES
  region?: string;
  accessKeyId?: string;
  secretAccessKey?: string;

  // SendGrid/Mailgun
  apiKey?: string;
  domain?: string;
}

export interface SMSChannelConfig {
  type: 'sms';
  provider: 'twilio' | 'aws-sns' | 'nexmo' | 'messagebird';
  from: string;

  // Twilio
  accountSid?: string;
  authToken?: string;

  // Other providers
  apiKey?: string;
  apiSecret?: string;
}

export interface PushChannelConfig {
  type: 'push';
  platform: 'web' | 'ios' | 'android' | 'all';

  // Web Push
  vapidPublicKey?: string;
  vapidPrivateKey?: string;
  vapidSubject?: string;

  // FCM (Firebase Cloud Messaging)
  fcmServerKey?: string;
  fcmProjectId?: string;

  // APNS (Apple Push Notification Service)
  apnsKeyId?: string;
  apnsTeamId?: string;
  apnsPrivateKey?: string;
  apnsTopic?: string;
}

export interface SlackChannelConfig {
  type: 'slack';
  webhookUrl?: string;
  token?: string;
  defaultChannel?: string;
  botToken?: string;
  signingSecret?: string;
}

export interface TeamsChannelConfig {
  type: 'teams';
  webhookUrl?: string;
  tenantId?: string;
  clientId?: string;
  clientSecret?: string;
  defaultChannel?: string;
}

export interface WebhookChannelConfig {
  type: 'webhook';
  url: string;
  method: 'POST' | 'PUT' | 'PATCH';
  headers?: Record<string, string>;
  auth?: {
    type: 'basic' | 'bearer' | 'api-key';
    username?: string;
    password?: string;
    token?: string;
    apiKey?: string;
    apiKeyHeader?: string;
  };
  retryConfig?: {
    maxRetries: number;
    retryDelay: number;
    backoffMultiplier: number;
  };
}

export interface InAppChannelConfig {
  type: 'in_app';
  persistDays: number;
  showBadge: boolean;
  playSound: boolean;
  enablePersistence: boolean;
}

export interface RateLimit {
  maxRequests: number;
  windowMs: number;
  burstLimit?: number;
}

// ============================================================================
// Template Types
// ============================================================================

export interface NotificationTemplate {
  id: string;
  tenantId: string;
  name: string;
  description?: string;
  type: string;
  category?: string;

  // Channels
  channels: TemplateChannel[];

  // Default settings
  defaultPriority: NotificationPriority;
  defaultChannels: NotificationChannelType[];

  // Variables
  variables: TemplateVariable[];
  schema?: Record<string, unknown>;

  // Metadata
  tags?: string[];
  locale?: string;
  version: number;

  enabled: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface TemplateChannel {
  type: NotificationChannelType;
  subject?: string;
  body: string;
  html?: string;
  template?: string;
  layout?: string;
}

export interface TemplateVariable {
  name: string;
  type: 'string' | 'number' | 'boolean' | 'date' | 'object' | 'array';
  required: boolean;
  default?: unknown;
  description?: string;
}

// ============================================================================
// Alert Types
// ============================================================================

export interface Alert {
  id: string;
  tenantId: string;
  ruleId?: string;

  // Identification
  name: string;
  description?: string;
  severity: AlertSeverity;
  status: AlertStatus;

  // Source
  source: string;
  sourceId?: string;
  sourceType?: string;

  // Content
  message: string;
  details?: Record<string, unknown>;
  metrics?: AlertMetric[];

  // Assignment
  assignedTo?: string;
  assignedAt?: Date;
  acknowledgedBy?: string;
  acknowledgedAt?: Date;
  resolvedBy?: string;
  resolvedAt?: Date;

  // Incident
  incidentId?: string;

  // Notifications
  notificationIds: string[];
  escalationLevel: number;
  lastEscalatedAt?: Date;

  // Deduplication
  fingerprint?: string;
  count: number;
  firstOccurrenceAt: Date;
  lastOccurrenceAt: Date;

  // Suppression
  suppressedUntil?: Date;
  suppressionReason?: string;

  createdAt: Date;
  updatedAt: Date;
}

export interface AlertMetric {
  name: string;
  value: number;
  unit?: string;
  threshold?: number;
  operator?: RuleConditionOperator;
}

// ============================================================================
// Alert Rule Types
// ============================================================================

export interface AlertRule {
  id: string;
  tenantId: string;
  name: string;
  description?: string;
  enabled: boolean;

  // Conditions
  conditions: RuleCondition[];
  conditionOperator: 'AND' | 'OR';

  // Thresholds
  thresholds: Threshold[];

  // Timing
  evaluationInterval: number; // seconds
  evaluationWindow: number; // seconds
  cooldownPeriod?: number; // seconds

  // Actions
  severity: AlertSeverity;
  notificationTemplateId?: string;
  channels: NotificationChannelType[];
  recipients: NotificationRecipient[];

  // Escalation
  escalationPolicy?: EscalationPolicy;
  autoResolve: boolean;
  autoResolveAfter?: number; // seconds

  // Deduplication
  deduplicationStrategy: 'fingerprint' | 'time-window' | 'count';
  deduplicationWindow?: number; // seconds

  // Metadata
  tags?: string[];
  metadata?: Record<string, unknown>;

  // Tracking
  lastTriggeredAt?: Date;
  triggerCount: number;

  createdAt: Date;
  updatedAt: Date;
}

export interface RuleCondition {
  id: string;
  field: string;
  operator: RuleConditionOperator;
  value: unknown;
  valueType: 'static' | 'dynamic' | 'reference';
}

export interface Threshold {
  id: string;
  name: string;
  type: ThresholdType;
  metric: string;
  operator: RuleConditionOperator;
  value: number;

  // Dynamic thresholds
  baselineWindow?: number; // seconds
  deviationMultiplier?: number;

  // Percentage thresholds
  percentageOf?: string;

  // Time-based
  duration?: number; // seconds
  consecutiveCount?: number;
}

// ============================================================================
// Escalation Types
// ============================================================================

export interface EscalationPolicy {
  id: string;
  tenantId: string;
  name: string;
  description?: string;

  // Levels
  levels: EscalationLevel[];

  // Settings
  repeatInterval?: number; // seconds
  maxRepeats?: number;

  enabled: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface EscalationLevel {
  level: number;
  delayMinutes: number;
  actions: EscalationActionConfig[];
  recipients: NotificationRecipient[];
  channels: NotificationChannelType[];
  notificationTemplateId?: string;
}

export interface EscalationActionConfig {
  type: EscalationAction;
  config: Record<string, unknown>;
}

// ============================================================================
// Incident Types
// ============================================================================

export interface Incident {
  id: string;
  tenantId: string;

  // Identification
  title: string;
  description?: string;
  severity: AlertSeverity;
  status: 'open' | 'investigating' | 'identified' | 'monitoring' | 'resolved' | 'closed';

  // Assignment
  assignedTo?: string;
  team?: string;

  // Alerts
  alertIds: string[];
  primaryAlertId?: string;

  // Timeline
  detectedAt: Date;
  acknowledgedAt?: Date;
  resolvedAt?: Date;
  closedAt?: Date;

  // Impact
  affectedServices?: string[];
  affectedUsers?: number;
  impactDescription?: string;

  // Response
  responders: IncidentResponder[];
  timeline: IncidentTimelineEvent[];

  // Metadata
  tags?: string[];
  metadata?: Record<string, unknown>;

  createdAt: Date;
  updatedAt: Date;
}

export interface IncidentResponder {
  userId: string;
  name: string;
  role: string;
  joinedAt: Date;
  status: 'active' | 'inactive';
}

export interface IncidentTimelineEvent {
  id: string;
  type: 'created' | 'acknowledged' | 'escalated' | 'note' | 'status_change' | 'resolved' | 'closed';
  description: string;
  userId?: string;
  userName?: string;
  timestamp: Date;
  metadata?: Record<string, unknown>;
}

// ============================================================================
// On-Call Types
// ============================================================================

export interface OnCallSchedule {
  id: string;
  tenantId: string;
  name: string;
  description?: string;
  timezone: string;

  // Rotation
  rotationType: 'daily' | 'weekly' | 'custom';
  rotationStartDate: Date;
  rotations: OnCallRotation[];

  // Overrides
  overrides: OnCallOverride[];

  // Current
  currentOnCall?: OnCallAssignment[];

  enabled: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface OnCallRotation {
  id: string;
  name: string;
  users: string[];
  handoffTime: string; // HH:MM format
  restrictionDays?: number[]; // 0-6, Sunday-Saturday
}

export interface OnCallOverride {
  id: string;
  userId: string;
  startDate: Date;
  endDate: Date;
  reason?: string;
}

export interface OnCallAssignment {
  userId: string;
  userName: string;
  startTime: Date;
  endTime: Date;
  rotationId: string;
}

// ============================================================================
// Preference Types
// ============================================================================

export interface NotificationPreference {
  userId: string;
  tenantId: string;

  // Global settings
  enabled: boolean;
  mutedUntil?: Date;

  // Channel preferences
  channelPreferences: ChannelPreference[];

  // Quiet hours
  quietHours?: QuietHours;

  // Category preferences
  categoryPreferences: CategoryPreference[];

  // Digest settings
  digestEnabled: boolean;
  digestFrequency?: 'hourly' | 'daily' | 'weekly';
  digestTime?: string; // HH:MM format

  createdAt: Date;
  updatedAt: Date;
}

export interface ChannelPreference {
  channel: NotificationChannelType;
  enabled: boolean;
  priority?: NotificationPriority[];
  categories?: string[];
}

export interface QuietHours {
  enabled: boolean;
  startTime: string; // HH:MM format
  endTime: string; // HH:MM format
  days?: number[]; // 0-6, Sunday-Saturday
  timezone?: string;
  allowUrgent?: boolean;
  allowCritical?: boolean;
}

export interface CategoryPreference {
  category: string;
  enabled: boolean;
  channels?: NotificationChannelType[];
  minimumPriority?: NotificationPriority;
}

// ============================================================================
// Subscription Types
// ============================================================================

export interface Subscription {
  id: string;
  userId: string;
  tenantId: string;

  // Target
  entityType: string; // e.g., 'project', 'task', 'document'
  entityId: string;

  // Events
  events: string[]; // e.g., ['created', 'updated', 'deleted']

  // Delivery
  channels: NotificationChannelType[];
  frequency: 'instant' | 'digest' | 'manual';

  // Filters
  filters?: Record<string, unknown>;

  enabled: boolean;
  createdAt: Date;
  updatedAt: Date;
}

// ============================================================================
// Delivery Types
// ============================================================================

export interface DeliveryAttempt {
  id: string;
  notificationId: string;
  channel: NotificationChannelType;
  recipientId: string;

  // Status
  status: 'pending' | 'sent' | 'delivered' | 'failed' | 'bounced';

  // Timing
  attemptNumber: number;
  scheduledFor: Date;
  sentAt?: Date;
  deliveredAt?: Date;
  failedAt?: Date;

  // Result
  response?: Record<string, unknown>;
  error?: string;
  errorCode?: string;

  // Tracking
  externalId?: string; // Provider's message ID
  trackingData?: Record<string, unknown>;

  createdAt: Date;
  updatedAt: Date;
}

export interface DeliveryReceipt {
  id: string;
  notificationId: string;
  deliveryAttemptId: string;
  channel: NotificationChannelType;

  // Status
  event: 'sent' | 'delivered' | 'opened' | 'clicked' | 'bounced' | 'complained' | 'failed';

  // Details
  timestamp: Date;
  details?: Record<string, unknown>;

  // Tracking
  ipAddress?: string;
  userAgent?: string;
  location?: {
    country?: string;
    city?: string;
  };

  createdAt: Date;
}

// ============================================================================
// Analytics Types
// ============================================================================

export interface NotificationAnalytics {
  tenantId: string;
  period: {
    start: Date;
    end: Date;
  };

  // Volume metrics
  totalSent: number;
  totalDelivered: number;
  totalFailed: number;
  totalRead: number;
  totalClicked: number;

  // Rate metrics
  deliveryRate: number;
  readRate: number;
  clickRate: number;
  failureRate: number;

  // Channel breakdown
  byChannel: Record<NotificationChannelType, ChannelAnalytics>;

  // Priority breakdown
  byPriority: Record<NotificationPriority, number>;

  // Time series
  timeSeries: TimeSeriesPoint[];
}

export interface ChannelAnalytics {
  sent: number;
  delivered: number;
  failed: number;
  read: number;
  clicked: number;
  deliveryRate: number;
  avgDeliveryTime: number; // milliseconds
}

export interface TimeSeriesPoint {
  timestamp: Date;
  sent: number;
  delivered: number;
  failed: number;
  read: number;
  clicked: number;
}

// ============================================================================
// Batch Processing Types
// ============================================================================

export interface BatchNotification {
  id: string;
  tenantId: string;
  name: string;

  // Notifications
  notifications: Notification[];
  totalCount: number;

  // Processing
  status: 'pending' | 'processing' | 'completed' | 'failed' | 'cancelled';
  processedCount: number;
  successCount: number;
  failureCount: number;

  // Settings
  priority: NotificationPriority;
  rateLimit?: number; // per second

  // Timing
  scheduledFor?: Date;
  startedAt?: Date;
  completedAt?: Date;

  createdAt: Date;
  updatedAt: Date;
}

// ============================================================================
// Queue Types
// ============================================================================

export interface QueuedNotification {
  id: string;
  notificationId: string;
  priority: NotificationPriority;
  scheduledFor: Date;
  attempts: number;
  maxAttempts: number;
  nextRetryAt?: Date;
  data: Notification;
  createdAt: Date;
}

// ============================================================================
// Export all types
// ============================================================================

export type {
  Notification,
  NotificationRecipient,
  NotificationLink,
  NotificationChannel,
  ChannelConfig,
  NotificationTemplate,
  TemplateChannel,
  TemplateVariable,
  Alert,
  AlertMetric,
  AlertRule,
  RuleCondition,
  Threshold,
  EscalationPolicy,
  EscalationLevel,
  EscalationActionConfig,
  Incident,
  IncidentResponder,
  IncidentTimelineEvent,
  OnCallSchedule,
  OnCallRotation,
  OnCallOverride,
  OnCallAssignment,
  NotificationPreference,
  ChannelPreference,
  QuietHours,
  CategoryPreference,
  Subscription,
  DeliveryAttempt,
  DeliveryReceipt,
  NotificationAnalytics,
  ChannelAnalytics,
  TimeSeriesPoint,
  BatchNotification,
  QueuedNotification,
};
