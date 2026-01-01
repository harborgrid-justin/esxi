# Enterprise Workflow Automation & Pipeline System

A comprehensive, production-ready workflow orchestration platform for enterprise applications. Build, execute, and monitor complex workflows with a visual drag-and-drop interface.

## Features

### Core Engine
- **Powerful Workflow Engine** - Execute complex workflows with state management
- **State Machine** - Advanced state tracking with checkpointing and rollback
- **Transition Engine** - Smart routing between workflow steps
- **Condition Evaluator** - Boolean logic evaluation with composite conditions
- **Action Executor** - Execute actions with retry policies and timeouts
- **Parallel Execution** - Run multiple branches concurrently

### Triggers
- **Webhook Trigger** - HTTP webhooks with signature validation
- **Schedule Trigger** - Cron-based scheduling with timezone support
- **Event Trigger** - Event-driven workflows with filtering and debouncing
- **Manual Trigger** - Human-initiated workflows with role-based access
- **Conditional Trigger** - Trigger based on condition evaluation

### Actions
- **HTTP Action** - REST API calls with authentication
- **Email Action** - Send emails via SMTP or email services
- **Notification Action** - Push notifications, SMS, Slack, Teams, Discord
- **Database Action** - Query, insert, update, delete operations
- **Transform Action** - Data transformation (map, filter, reduce)
- **Approval Action** - Human approval workflows

### Visual Builder
- **Workflow Canvas** - Drag-and-drop workflow designer
- **Node Palette** - Library of available workflow nodes
- **Property Panel** - Configure node properties
- **Variable Editor** - Manage workflow variables
- **Condition Builder** - Visual condition editor

### Execution Monitoring
- **Execution Monitor** - Live workflow execution monitoring
- **Execution History** - View past executions
- **Step Debugger** - Debug individual steps
- **Log Viewer** - Real-time execution logs
- **Retry Manager** - Retry failed steps

## Installation

```bash
npm install @enterprise/workflow
```

## Quick Start

### 1. Create a Workflow

```typescript
import { WorkflowService, Workflow, WorkflowStatus } from '@enterprise/workflow';

const workflowService = new WorkflowService();

const workflow: Omit<Workflow, 'id' | 'createdAt' | 'updatedAt'> = {
  name: 'User Onboarding',
  version: '1.0.0',
  status: WorkflowStatus.ACTIVE,
  triggers: [
    {
      id: 'webhook-trigger',
      type: 'webhook',
      enabled: true,
      config: {
        url: '/api/webhooks/onboarding',
        method: 'POST'
      }
    }
  ],
  steps: [
    {
      id: 'send-welcome-email',
      name: 'Send Welcome Email',
      type: 'action',
      action: {
        id: 'email-action',
        name: 'Send Email',
        type: 'email',
        config: {
          to: '${user_email}',
          subject: 'Welcome to Our Platform!',
          body: 'Hello ${user_name}, welcome aboard!'
        }
      },
      position: { x: 100, y: 100 },
      transitions: [
        {
          id: 'trans-1',
          from: 'send-welcome-email',
          to: 'create-account'
        }
      ]
    },
    {
      id: 'create-account',
      name: 'Create Account',
      type: 'action',
      action: {
        id: 'db-action',
        name: 'Create User',
        type: 'database',
        config: {
          operation: 'insert',
          connection: 'main-db',
          table: 'users',
          parameters: {
            name: '${user_name}',
            email: '${user_email}'
          }
        }
      },
      position: { x: 100, y: 200 },
      transitions: []
    }
  ],
  variables: [
    {
      id: 'var-1',
      name: 'user_name',
      type: 'string',
      value: '',
      required: true
    },
    {
      id: 'var-2',
      name: 'user_email',
      type: 'string',
      value: '',
      required: true
    }
  ],
  startStepId: 'send-welcome-email',
  endStepIds: ['create-account'],
  createdBy: 'admin',
  settings: {
    timeout: 300000,
    errorHandling: 'fail'
  }
};

const result = await workflowService.create(workflow);
```

### 2. Execute a Workflow

```typescript
import { ExecutionService } from '@enterprise/workflow';

const executionService = new ExecutionService();

const execution = await executionService.execute(
  workflow,
  {
    variables: new Map([
      ['user_name', 'John Doe'],
      ['user_email', 'john@example.com']
    ])
  },
  'webhook'
);

console.log('Execution ID:', execution.data?.id);
console.log('Status:', execution.data?.status);
```

### 3. Monitor Execution

```typescript
import { ExecutionMonitor } from '@enterprise/workflow';
import React from 'react';

function MyWorkflowMonitor() {
  const [execution, setExecution] = useState(null);

  const handleRefresh = async () => {
    const result = await executionService.getById(executionId);
    setExecution(result.data);
  };

  return (
    <ExecutionMonitor
      execution={execution}
      onRefresh={handleRefresh}
    />
  );
}
```

### 4. Build Workflows Visually

```typescript
import { WorkflowCanvasWithProvider } from '@enterprise/workflow';
import React from 'react';

function WorkflowBuilder() {
  const [workflow, setWorkflow] = useState(null);

  return (
    <div style={{ height: '800px' }}>
      <WorkflowCanvasWithProvider
        workflow={workflow}
        onWorkflowChange={setWorkflow}
      />
    </div>
  );
}
```

## Architecture

### Workflow Engine

The workflow engine is the core component that executes workflows. It manages:
- State transitions
- Step execution
- Error handling
- Retry logic
- Parallel execution
- Checkpointing

```typescript
import { WorkflowEngine } from '@enterprise/workflow';

const engine = new WorkflowEngine({
  maxConcurrentExecutions: 100,
  defaultTimeout: 3600000,
  enableCheckpoints: true,
  logLevel: 'info'
});

// Listen to events
engine.on('event', (event) => {
  console.log('Workflow event:', event.type);
});

engine.on('log', (log) => {
  console.log('Log:', log.message);
});
```

### Triggers

Triggers start workflow executions based on various events:

```typescript
import {
  WebhookTrigger,
  ScheduleTrigger,
  EventTrigger
} from '@enterprise/workflow';

// Webhook trigger
const webhookTrigger = new WebhookTrigger();
webhookTrigger.register(trigger);

// Schedule trigger
const scheduleTrigger = new ScheduleTrigger();
scheduleTrigger.register(trigger);

// Event trigger
const eventTrigger = new EventTrigger();
eventTrigger.register(trigger);
```

### Actions

Actions perform operations during workflow execution:

```typescript
import {
  HTTPAction,
  EmailAction,
  DatabaseAction
} from '@enterprise/workflow';

// HTTP Action
const httpAction = new HTTPAction();
await httpAction.execute({
  method: 'POST',
  url: 'https://api.example.com/users',
  body: { name: 'John' }
}, context);

// Email Action
const emailAction = new EmailAction();
await emailAction.execute({
  to: 'user@example.com',
  subject: 'Hello',
  body: 'Welcome!'
}, context);

// Database Action
const dbAction = new DatabaseAction();
await dbAction.execute({
  operation: 'insert',
  connection: 'main-db',
  table: 'users',
  parameters: { name: 'John' }
}, context);
```

## API Reference

### WorkflowService

```typescript
class WorkflowService {
  create(workflow: Omit<Workflow, 'id' | 'createdAt' | 'updatedAt'>): Promise<ApiResponse<Workflow>>;
  getById(id: WorkflowId): Promise<ApiResponse<Workflow>>;
  list(page?: number, pageSize?: number, filters?: object): Promise<ApiResponse<PaginatedResponse<Workflow>>>;
  update(id: WorkflowId, updates: Partial<Workflow>): Promise<ApiResponse<Workflow>>;
  delete(id: WorkflowId): Promise<ApiResponse<void>>;
  publish(id: WorkflowId): Promise<ApiResponse<Workflow>>;
  archive(id: WorkflowId): Promise<ApiResponse<Workflow>>;
  clone(id: WorkflowId, name?: string): Promise<ApiResponse<Workflow>>;
  validate(workflow: Workflow): Promise<ApiResponse<{ valid: boolean; errors: string[] }>>;
}
```

### ExecutionService

```typescript
class ExecutionService {
  execute(workflow: Workflow, context?: Partial<Context>, triggeredBy?: string): Promise<ApiResponse<Execution>>;
  getById(id: ExecutionId): Promise<ApiResponse<Execution>>;
  list(workflowId?: string, page?: number, pageSize?: number): Promise<ApiResponse<PaginatedResponse<Execution>>>;
  cancel(id: ExecutionId): Promise<ApiResponse<Execution>>;
  retry(id: ExecutionId): Promise<ApiResponse<Execution>>;
  getStatistics(workflowId?: string): Promise<ApiResponse<Statistics>>;
}
```

## Configuration

### Workflow Settings

```typescript
{
  timeout: 3600000,              // Maximum execution time (ms)
  maxConcurrentExecutions: 100,  // Max concurrent executions
  retryPolicy: {
    maxAttempts: 3,
    backoffType: 'exponential',
    initialDelay: 1000,
    maxDelay: 60000,
    multiplier: 2
  },
  errorHandling: 'fail',         // 'fail' | 'continue' | 'rollback'
  logging: {
    level: 'info',               // 'debug' | 'info' | 'warn' | 'error'
    retention: 2592000000        // Log retention period (ms)
  },
  notifications: {
    onStart: true,
    onSuccess: true,
    onFailure: true,
    recipients: ['admin@example.com']
  }
}
```

## Best Practices

1. **Use Variables** - Define variables for dynamic values
2. **Enable Checkpoints** - For long-running workflows
3. **Set Timeouts** - Prevent indefinite execution
4. **Implement Retry Logic** - For transient failures
5. **Log Extensively** - For debugging and monitoring
6. **Validate Workflows** - Before publishing
7. **Use Templates** - For common workflow patterns
8. **Monitor Executions** - Track performance and failures

## Testing

```bash
npm test
npm run test:watch
npm run test:coverage
```

## Building

```bash
npm run build
```

## License

MIT

## Support

For issues and questions, please open an issue on GitHub.
