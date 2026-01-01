/**
 * Enterprise Notification & Alerting System - Engine Module
 * Export all engine components
 */

export { NotificationEngine } from './NotificationEngine';
export type { NotificationEngineConfig, ChannelHandler } from './NotificationEngine';

export { TemplateEngine } from './TemplateEngine';
export type {
  TemplateEngineConfig,
  RenderContext,
  RenderedTemplate,
} from './TemplateEngine';

export { DeliveryEngine } from './DeliveryEngine';
export type { DeliveryEngineConfig, ChannelDeliveryHandler } from './DeliveryEngine';

export { BatchProcessor } from './BatchProcessor';
export type { BatchProcessorConfig, BatchJob } from './BatchProcessor';

export { PriorityQueue } from './PriorityQueue';
export type { PriorityQueueConfig } from './PriorityQueue';

export { DeduplicationEngine } from './DeduplicationEngine';
export type { DeduplicationEngineConfig, DeduplicationEntry } from './DeduplicationEngine';
