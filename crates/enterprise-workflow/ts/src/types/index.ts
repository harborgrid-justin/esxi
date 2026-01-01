/**
 * Enterprise Workflow Automation - Core Type Definitions
 * Comprehensive type system for workflow orchestration and automation
 */

// ============================================================================
// Base Types
// ============================================================================

export type WorkflowId = string;
export type StepId = string;
export type ExecutionId = string;
export type VariableId = string;
export type ActionId = string;
export type TriggerId = string;
export type ConditionId = string;

export enum WorkflowStatus {
  DRAFT = 'draft',
  ACTIVE = 'active',
  PAUSED = 'paused',
  ARCHIVED = 'archived',
  DEPRECATED = 'deprecated'
}

export enum ExecutionStatus {
  PENDING = 'pending',
  RUNNING = 'running',
  WAITING = 'waiting',
  SUCCESS = 'success',
  FAILED = 'failed',
  CANCELLED = 'cancelled',
  TIMEOUT = 'timeout'
}

export enum StepStatus {
  PENDING = 'pending',
  RUNNING = 'running',
  SUCCESS = 'success',
  FAILED = 'failed',
  SKIPPED = 'skipped',
  RETRY = 'retry'
}

export enum TriggerType {
  WEBHOOK = 'webhook',
  SCHEDULE = 'schedule',
  EVENT = 'event',
  MANUAL = 'manual',
  CONDITIONAL = 'conditional'
}

export enum ActionType {
  HTTP = 'http',
  EMAIL = 'email',
  NOTIFICATION = 'notification',
  DATABASE = 'database',
  TRANSFORM = 'transform',
  APPROVAL = 'approval',
  SCRIPT = 'script',
  CONDITION = 'condition'
}

export enum ConditionOperator {
  EQUALS = 'equals',
  NOT_EQUALS = 'not_equals',
  GREATER_THAN = 'greater_than',
  LESS_THAN = 'less_than',
  GREATER_THAN_OR_EQUAL = 'greater_than_or_equal',
  LESS_THAN_OR_EQUAL = 'less_than_or_equal',
  CONTAINS = 'contains',
  NOT_CONTAINS = 'not_contains',
  STARTS_WITH = 'starts_with',
  ENDS_WITH = 'ends_with',
  MATCHES_REGEX = 'matches_regex',
  IN = 'in',
  NOT_IN = 'not_in',
  IS_NULL = 'is_null',
  IS_NOT_NULL = 'is_not_null'
}

export enum LogicalOperator {
  AND = 'and',
  OR = 'or',
  NOT = 'not'
}

// ============================================================================
// Variable System
// ============================================================================

export type VariableType = 'string' | 'number' | 'boolean' | 'object' | 'array' | 'date' | 'null';

export interface Variable {
  id: VariableId;
  name: string;
  type: VariableType;
  value: any;
  description?: string;
  required?: boolean;
  defaultValue?: any;
  validation?: VariableValidation;
  metadata?: Record<string, any>;
}

export interface VariableValidation {
  pattern?: string;
  minLength?: number;
  maxLength?: number;
  min?: number;
  max?: number;
  enum?: any[];
  custom?: (value: any) => boolean | Promise<boolean>;
}

export interface Context {
  workflowId: WorkflowId;
  executionId: ExecutionId;
  variables: Map<string, any>;
  metadata: Record<string, any>;
  timestamp: Date;
  userId?: string;
  tenantId?: string;
  environment: 'development' | 'staging' | 'production';
}

// ============================================================================
// Conditions
// ============================================================================

export interface Condition {
  id: ConditionId;
  type: 'simple' | 'composite';
  operator?: ConditionOperator;
  logicalOperator?: LogicalOperator;
  left?: string | number | boolean | Variable;
  right?: string | number | boolean | Variable;
  conditions?: Condition[];
  expression?: string;
  metadata?: Record<string, any>;
}

export interface ConditionResult {
  conditionId: ConditionId;
  result: boolean;
  evaluatedAt: Date;
  context: Record<string, any>;
  error?: string;
}

// ============================================================================
// Actions
// ============================================================================

export interface Action {
  id: ActionId;
  name: string;
  type: ActionType;
  config: ActionConfig;
  retryPolicy?: RetryPolicy;
  timeout?: number;
  onSuccess?: Transition[];
  onFailure?: Transition[];
  metadata?: Record<string, any>;
}

export interface ActionConfig {
  [key: string]: any;
}

export interface HTTPActionConfig extends ActionConfig {
  method: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
  url: string;
  headers?: Record<string, string>;
  body?: any;
  queryParams?: Record<string, string>;
  auth?: {
    type: 'basic' | 'bearer' | 'oauth2' | 'apikey';
    credentials: Record<string, string>;
  };
}

export interface EmailActionConfig extends ActionConfig {
  to: string | string[];
  cc?: string | string[];
  bcc?: string | string[];
  subject: string;
  body: string;
  html?: string;
  attachments?: {
    filename: string;
    content: string | Buffer;
    contentType?: string;
  }[];
}

export interface NotificationActionConfig extends ActionConfig {
  channel: 'push' | 'sms' | 'slack' | 'teams' | 'discord';
  recipients: string[];
  title: string;
  message: string;
  priority?: 'low' | 'medium' | 'high' | 'critical';
  data?: Record<string, any>;
}

export interface DatabaseActionConfig extends ActionConfig {
  operation: 'query' | 'insert' | 'update' | 'delete';
  connection: string;
  table?: string;
  query?: string;
  parameters?: Record<string, any>;
  transaction?: boolean;
}

export interface TransformActionConfig extends ActionConfig {
  inputVariable: string;
  outputVariable: string;
  transformType: 'map' | 'filter' | 'reduce' | 'custom';
  transformation: string | ((input: any) => any);
}

export interface ApprovalActionConfig extends ActionConfig {
  approvers: string[];
  approvalType: 'any' | 'all' | 'majority';
  deadline?: Date;
  message: string;
  metadata?: Record<string, any>;
}

export interface RetryPolicy {
  maxAttempts: number;
  backoffType: 'fixed' | 'exponential' | 'linear';
  initialDelay: number;
  maxDelay?: number;
  multiplier?: number;
}

// ============================================================================
// Triggers
// ============================================================================

export interface Trigger {
  id: TriggerId;
  type: TriggerType;
  config: TriggerConfig;
  enabled: boolean;
  metadata?: Record<string, any>;
}

export interface TriggerConfig {
  [key: string]: any;
}

export interface WebhookTriggerConfig extends TriggerConfig {
  url: string;
  method: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
  secret?: string;
  headers?: Record<string, string>;
  validation?: (payload: any) => boolean;
}

export interface ScheduleTriggerConfig extends TriggerConfig {
  cronExpression: string;
  timezone?: string;
  startDate?: Date;
  endDate?: Date;
  maxRuns?: number;
}

export interface EventTriggerConfig extends TriggerConfig {
  eventName: string;
  source?: string;
  filter?: Condition;
  debounce?: number;
}

export interface ManualTriggerConfig extends TriggerConfig {
  requiredRoles?: string[];
  confirmationRequired?: boolean;
  customInputs?: Variable[];
}

export interface ConditionalTriggerConfig extends TriggerConfig {
  condition: Condition;
  evaluationInterval?: number;
  watchVariables?: string[];
}

// ============================================================================
// Workflow Steps & Transitions
// ============================================================================

export interface WorkflowStep {
  id: StepId;
  name: string;
  description?: string;
  type: 'action' | 'condition' | 'parallel' | 'loop' | 'wait' | 'subworkflow';
  action?: Action;
  condition?: Condition;
  parallelBranches?: WorkflowStep[][];
  loopConfig?: LoopConfig;
  waitConfig?: WaitConfig;
  subWorkflowId?: WorkflowId;
  position: Position;
  transitions: Transition[];
  metadata?: Record<string, any>;
}

export interface Transition {
  id: string;
  from: StepId;
  to: StepId;
  condition?: Condition;
  label?: string;
  metadata?: Record<string, any>;
}

export interface LoopConfig {
  type: 'for' | 'while' | 'forEach';
  condition?: Condition;
  iterator?: string;
  maxIterations?: number;
  breakOn?: Condition;
}

export interface WaitConfig {
  type: 'duration' | 'until' | 'event';
  duration?: number;
  until?: Date;
  eventName?: string;
  timeout?: number;
}

export interface Position {
  x: number;
  y: number;
}

// ============================================================================
// Workflow Definition
// ============================================================================

export interface Workflow {
  id: WorkflowId;
  name: string;
  description?: string;
  version: string;
  status: WorkflowStatus;
  triggers: Trigger[];
  steps: WorkflowStep[];
  variables: Variable[];
  startStepId: StepId;
  endStepIds: StepId[];
  tags?: string[];
  category?: string;
  createdBy: string;
  createdAt: Date;
  updatedAt: Date;
  settings: WorkflowSettings;
  metadata?: Record<string, any>;
}

export interface WorkflowSettings {
  timeout?: number;
  maxConcurrentExecutions?: number;
  retryPolicy?: RetryPolicy;
  errorHandling?: 'fail' | 'continue' | 'rollback';
  logging?: {
    level: 'debug' | 'info' | 'warn' | 'error';
    retention?: number;
  };
  notifications?: {
    onStart?: boolean;
    onSuccess?: boolean;
    onFailure?: boolean;
    recipients?: string[];
  };
}

// ============================================================================
// Execution & Monitoring
// ============================================================================

export interface Execution {
  id: ExecutionId;
  workflowId: WorkflowId;
  workflowVersion: string;
  status: ExecutionStatus;
  triggeredBy: string;
  triggerId?: TriggerId;
  context: Context;
  currentStepId?: StepId;
  stepExecutions: StepExecution[];
  startedAt: Date;
  completedAt?: Date;
  duration?: number;
  error?: ExecutionError;
  metrics: ExecutionMetrics;
  metadata?: Record<string, any>;
}

export interface StepExecution {
  stepId: StepId;
  status: StepStatus;
  startedAt: Date;
  completedAt?: Date;
  duration?: number;
  attempt: number;
  input?: any;
  output?: any;
  error?: ExecutionError;
  logs: ExecutionLog[];
}

export interface ExecutionError {
  code: string;
  message: string;
  stepId?: StepId;
  timestamp: Date;
  stackTrace?: string;
  recoverable: boolean;
  context?: Record<string, any>;
}

export interface ExecutionLog {
  id: string;
  executionId: ExecutionId;
  stepId?: StepId;
  level: 'debug' | 'info' | 'warn' | 'error';
  message: string;
  timestamp: Date;
  data?: Record<string, any>;
}

export interface ExecutionMetrics {
  totalSteps: number;
  completedSteps: number;
  failedSteps: number;
  skippedSteps: number;
  totalDuration?: number;
  avgStepDuration?: number;
  retryCount: number;
}

// ============================================================================
// State Management
// ============================================================================

export interface State {
  executionId: ExecutionId;
  currentStep: StepId;
  visitedSteps: Set<StepId>;
  variables: Map<string, any>;
  status: ExecutionStatus;
  history: StateTransition[];
  checkpoints: Checkpoint[];
}

export interface StateTransition {
  from: StepId;
  to: StepId;
  timestamp: Date;
  reason?: string;
  metadata?: Record<string, any>;
}

export interface Checkpoint {
  id: string;
  timestamp: Date;
  stepId: StepId;
  variables: Map<string, any>;
  metadata?: Record<string, any>;
}

// ============================================================================
// Templates & Integration
// ============================================================================

export interface WorkflowTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  workflow: Omit<Workflow, 'id' | 'createdAt' | 'updatedAt' | 'createdBy'>;
  parameters: Variable[];
  tags: string[];
  popularity?: number;
  version: string;
}

export interface Integration {
  id: string;
  name: string;
  type: string;
  provider: string;
  config: Record<string, any>;
  credentials: Record<string, any>;
  enabled: boolean;
  metadata?: Record<string, any>;
}

// ============================================================================
// Events
// ============================================================================

export interface WorkflowEvent {
  id: string;
  type: WorkflowEventType;
  workflowId: WorkflowId;
  executionId?: ExecutionId;
  timestamp: Date;
  payload: any;
  metadata?: Record<string, any>;
}

export enum WorkflowEventType {
  WORKFLOW_CREATED = 'workflow.created',
  WORKFLOW_UPDATED = 'workflow.updated',
  WORKFLOW_DELETED = 'workflow.deleted',
  WORKFLOW_TRIGGERED = 'workflow.triggered',
  EXECUTION_STARTED = 'execution.started',
  EXECUTION_COMPLETED = 'execution.completed',
  EXECUTION_FAILED = 'execution.failed',
  STEP_STARTED = 'step.started',
  STEP_COMPLETED = 'step.completed',
  STEP_FAILED = 'step.failed',
  APPROVAL_REQUESTED = 'approval.requested',
  APPROVAL_GRANTED = 'approval.granted',
  APPROVAL_REJECTED = 'approval.rejected'
}

// ============================================================================
// API Responses
// ============================================================================

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: any;
  };
  metadata?: {
    timestamp: Date;
    requestId: string;
  };
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
}

// ============================================================================
// Exports
// ============================================================================

export * from './index';
