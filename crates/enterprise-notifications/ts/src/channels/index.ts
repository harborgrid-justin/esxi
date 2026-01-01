/**
 * Enterprise Notification & Alerting System - Channels Module
 * Export all notification channel implementations
 */

export { EmailChannel } from './EmailChannel';
export type { EmailAttachment, EmailOptions } from './EmailChannel';

export { SMSChannel } from './SMSChannel';
export type { SMSOptions } from './SMSChannel';

export { PushChannel } from './PushChannel';
export type { PushSubscription, PushPayload } from './PushChannel';

export { SlackChannel } from './SlackChannel';
export type { SlackMessageOptions, SlackBlock } from './SlackChannel';

export { TeamsChannel } from './TeamsChannel';
export type { TeamsMessageOptions, TeamsAdaptiveCard } from './TeamsChannel';

export { WebhookChannel } from './WebhookChannel';
export type { WebhookPayload, WebhookResponse } from './WebhookChannel';

export { InAppChannel } from './InAppChannel';
export type { StoredNotification, NotificationFilter } from './InAppChannel';
