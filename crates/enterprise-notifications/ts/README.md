# @harborgrid/enterprise-notifications

Enterprise-grade Notification & Alerting System with multi-channel delivery, advanced routing, and comprehensive alerting capabilities.

## Features

### Core Notification System
- **Multi-Channel Delivery**: Email, SMS, Push, Slack, Teams, Webhooks, In-App
- **Template Engine**: Handlebars/Mustache templates with localization
- **Priority Queue**: Priority-based notification processing
- **Batch Processing**: Efficient bulk notification handling
- **Deduplication**: Prevent duplicate notifications
- **Rate Limiting**: Control notification flow

### Alerting System
- **Alert Engine**: Comprehensive alert lifecycle management
- **Rule Evaluation**: Flexible condition and threshold evaluation
- **Escalation Policies**: Multi-level escalation with customizable actions
- **Incident Management**: Track and manage incidents
- **On-Call Scheduling**: Rotation management and overrides
- **Alert Correlation**: Group and correlate related alerts

### User Management
- **Preferences**: Per-user notification preferences
- **Quiet Hours**: Do Not Disturb scheduling
- **Subscriptions**: Subscribe to entities and events
- **Channel Selection**: Choose preferred notification channels

### Analytics & Reporting
- **Delivery Tracking**: Monitor notification delivery
- **Engagement Metrics**: Track reads, clicks, and interactions
- **Channel Analytics**: Per-channel performance metrics
- **Time Series Data**: Historical analytics and trends

## Installation

```bash
npm install @harborgrid/enterprise-notifications
```

## Quick Start

### Basic Notification

```typescript
import { NotificationEngine, NotificationPriority } from '@harborgrid/enterprise-notifications';

const engine = new NotificationEngine();

await engine.send({
  tenantId: 'tenant-123',
  userId: 'user-456',
  title: 'Welcome!',
  message: 'Thanks for signing up',
  priority: NotificationPriority.NORMAL,
  channels: ['email', 'push'],
  recipients: [{
    id: 'user-456',
    type: 'user',
    identifier: 'user-456',
    email: 'user@example.com',
  }],
});
```

### Using Templates

```typescript
import { TemplateEngine, NotificationService } from '@harborgrid/enterprise-notifications';

const templateEngine = new TemplateEngine();
const notificationService = new NotificationService(
  engine,
  templateEngine,
  deliveryEngine
);

// Create template
await notificationService.saveTemplate({
  id: 'welcome-email',
  name: 'Welcome Email',
  type: 'welcome',
  channels: [{
    type: 'email',
    subject: 'Welcome {{userName}}!',
    body: 'Hi {{userName}}, welcome to {{appName}}!',
    html: '<h1>Welcome {{userName}}!</h1><p>Thanks for joining {{appName}}</p>',
  }],
  defaultPriority: NotificationPriority.NORMAL,
  defaultChannels: ['email'],
  variables: [],
  enabled: true,
  tenantId: 'tenant-123',
  version: 1,
  createdAt: new Date(),
  updatedAt: new Date(),
});

// Send from template
await notificationService.sendFromTemplate(
  'welcome-email',
  { userName: 'John', appName: 'MyApp' },
  recipients
);
```

### Alert Rules

```typescript
import { AlertEngine, RuleEvaluator, AlertSeverity } from '@harborgrid/enterprise-notifications/alerting';

const alertEngine = new AlertEngine();
const ruleEvaluator = new RuleEvaluator();

// Register alert rule
ruleEvaluator.registerRule({
  id: 'cpu-high',
  tenantId: 'tenant-123',
  name: 'High CPU Usage',
  description: 'Alert when CPU usage exceeds 80%',
  enabled: true,
  conditions: [{
    id: 'cond-1',
    field: 'cpu.usage',
    operator: 'greater_than',
    value: 80,
    valueType: 'static',
  }],
  conditionOperator: 'AND',
  thresholds: [{
    id: 'thresh-1',
    name: 'CPU Threshold',
    type: 'static',
    metric: 'cpu.usage',
    operator: 'greater_than',
    value: 80,
  }],
  severity: AlertSeverity.WARNING,
  evaluationInterval: 60,
  evaluationWindow: 300,
  channels: ['email', 'slack'],
  recipients: [],
  autoResolve: true,
  autoResolveAfter: 300,
  deduplicationStrategy: 'fingerprint',
  tags: ['infrastructure', 'cpu'],
  triggerCount: 0,
  createdAt: new Date(),
  updatedAt: new Date(),
});

// Evaluate rules
const results = ruleEvaluator.evaluateAll({
  data: { cpu: { usage: 85 } },
  metrics: { 'cpu.usage': 85 },
});

// Create alert if rules match
for (const result of results) {
  if (result.matched) {
    await alertEngine.createAlert({
      tenantId: 'tenant-123',
      name: 'High CPU Usage',
      message: 'CPU usage is at 85%',
      severity: AlertSeverity.WARNING,
      source: 'monitoring',
    });
  }
}
```

### Email Channel

```typescript
import { EmailChannel } from '@harborgrid/enterprise-notifications/channels';

const emailChannel = new EmailChannel({
  type: 'email',
  provider: 'smtp',
  from: 'noreply@example.com',
  fromName: 'MyApp Notifications',
  host: 'smtp.example.com',
  port: 587,
  secure: false,
  auth: {
    user: 'smtp-user',
    pass: 'smtp-password',
  },
});

await emailChannel.initialize();
await emailChannel.send(notification, {
  cc: ['manager@example.com'],
  attachments: [{
    filename: 'report.pdf',
    path: '/path/to/report.pdf',
  }],
});
```

### React Components

```typescript
import { NotificationCenter, NotificationBell } from '@harborgrid/enterprise-notifications/components';

// Notification Bell
<NotificationBell
  notifications={notifications}
  unreadCount={unreadCount}
  onOpen={() => console.log('Opened')}
/>

// Notification Center
<NotificationCenter
  userId="user-123"
  tenantId="tenant-123"
  notifications={notifications}
  onMarkAsRead={handleMarkAsRead}
  onMarkAllAsRead={handleMarkAllAsRead}
  onDelete={handleDelete}
/>
```

## Architecture

### Notification Flow
1. **Notification Creation**: Create notification with recipients and channels
2. **Template Rendering**: Apply templates and render dynamic content
3. **Preference Check**: Apply user preferences and quiet hours
4. **Deduplication**: Check for duplicate notifications
5. **Priority Queue**: Add to priority-based queue
6. **Channel Delivery**: Send through selected channels
7. **Tracking**: Record delivery attempts and receipts

### Alert Flow
1. **Metric Collection**: Collect metrics and events
2. **Rule Evaluation**: Evaluate alert rules against metrics
3. **Alert Creation**: Create alerts when rules match
4. **Deduplication**: Group similar alerts
5. **Escalation**: Escalate alerts based on policies
6. **Notification**: Send notifications to on-call users
7. **Resolution**: Track alert lifecycle to resolution

## API Reference

### NotificationEngine
- `send(notification)` - Send a notification
- `sendBatch(notifications)` - Send multiple notifications
- `cancel(notificationId)` - Cancel pending notification
- `getStats()` - Get queue statistics

### AlertEngine
- `createAlert(alert)` - Create new alert
- `acknowledgeAlert(alertId, userId)` - Acknowledge alert
- `resolveAlert(alertId, userId)` - Resolve alert
- `assignAlert(alertId, userId)` - Assign alert to user

### Channels
- **EmailChannel**: SMTP/SES email delivery
- **SMSChannel**: Twilio/AWS SNS SMS delivery
- **PushChannel**: Web/mobile push notifications
- **SlackChannel**: Slack message delivery
- **TeamsChannel**: Microsoft Teams delivery
- **WebhookChannel**: Custom webhook delivery
- **InAppChannel**: In-app notification storage

## Configuration

```typescript
const engine = new NotificationEngine({
  maxConcurrent: 10,
  retryAttempts: 3,
  retryDelay: 1000,
  enableDeduplication: true,
  deduplicationWindow: 300000,
  enableBatching: true,
  batchSize: 100,
});
```

## License

MIT

## Support

For issues and questions, please visit our [GitHub repository](https://github.com/harborgrid/enterprise-notifications).
