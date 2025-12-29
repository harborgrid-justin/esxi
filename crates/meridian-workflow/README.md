# Meridian Workflow Engine

[![Version](https://img.shields.io/badge/version-0.1.5-blue.svg)](https://github.com/meridian-gis/meridian)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

A comprehensive enterprise workflow engine for the Meridian GIS Platform, providing robust DAG-based workflow orchestration with advanced scheduling, retry policies, and event-driven execution.

## Features

### Core Capabilities

- **DAG-based Workflow Definition** - Define complex workflows using directed acyclic graphs with full cycle detection
- **Cron-based Job Scheduling** - Schedule workflows with cron expressions and full timezone support
- **Distributed Task Queue** - Priority-based task queue with configurable concurrency control
- **Workflow State Machine** - Comprehensive state management with persistence support
- **Retry Policies** - Exponential backoff, linear backoff, fixed delay, and custom retry strategies
- **Dead Letter Queue** - Automatic handling of failed tasks with detailed error tracking
- **Workflow Versioning** - Version control with migration strategies (blue-green, gradual rollout)
- **Parallel & Sequential Execution** - Execute tasks in parallel or sequence with dependency management
- **Conditional Branching** - Support for conditional task execution based on runtime conditions
- **Event-triggered Workflows** - Execute workflows based on external events with flexible trigger conditions
- **Workflow Templates** - Reusable workflow templates with parameter substitution
- **Progress Tracking** - Real-time progress monitoring with ETA estimation

### Enterprise Features

- **Multi-tenant Support** - Isolate workflows across different tenants
- **Audit Logging** - Comprehensive execution history and audit trails
- **Metrics & Monitoring** - Built-in progress tracking and performance metrics
- **Resource Management** - Control concurrency and resource allocation
- **Error Recovery** - Automatic retry with dead letter queue for failed tasks
- **High Availability** - Designed for distributed deployment

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Workflow Engine                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   Scheduler  │  │   Triggers   │  │   Templates  │    │
│  │   (Cron)     │  │   (Events)   │  │  (Reusable)  │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Workflow Executor                        │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐    │  │
│  │  │ DAG Engine │  │    Queue   │  │   Retry    │    │  │
│  │  └────────────┘  └────────────┘  └────────────┘    │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │    State     │  │     DLQ      │  │  Versioning  │    │
│  │   Manager    │  │ (Dead Letter)│  │  & Migration │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### Basic Workflow Creation

```rust
use meridian_workflow::prelude::*;

#[tokio::main]
async fn main() -> WorkflowResult<()> {
    // Create a workflow
    let mut workflow = WorkflowDag::new("data_pipeline");

    // Define tasks
    let extract = Task::new("extract", "data_extract")
        .with_config(serde_json::json!({"source": "database"}))
        .with_timeout(300)
        .with_retries(3);

    let transform = Task::new("transform", "data_transform")
        .with_config(serde_json::json!({"rules": ["normalize", "validate"]}));

    let load = Task::new("load", "data_load")
        .with_config(serde_json::json!({"target": "warehouse"}));

    // Add tasks to workflow
    let id1 = workflow.add_task(extract);
    let id2 = workflow.add_task(transform);
    let id3 = workflow.add_task(load);

    // Define dependencies
    workflow.add_dependency(id1, id2, DependencyType::Sequential)?;
    workflow.add_dependency(id2, id3, DependencyType::Sequential)?;

    // Validate workflow
    workflow.validate()?;

    // Execute workflow
    let engine = WorkflowEngine::new();
    let execution_id = engine.execute(&workflow, ExecutionOptions::default()).await?;

    println!("Workflow executing with ID: {}", execution_id);

    Ok(())
}
```

### Scheduled Workflows

```rust
use meridian_workflow::prelude::*;
use std::sync::Arc;

let callback = Arc::new(|workflow_id: String| {
    tokio::spawn(async move {
        // Execute your workflow here
        println!("Executing workflow: {}", workflow_id);
        Ok(())
    })
});

let mut scheduler = CronScheduler::new(callback);

// Schedule daily at midnight UTC
let config = ScheduleConfig::new("0 0 * * *", "UTC");
scheduler.add_schedule(config, "my_workflow_id".to_string()).await?;

scheduler.start();
```

### Event-triggered Workflows

```rust
use meridian_workflow::prelude::*;
use std::sync::Arc;

// Define trigger callback
struct MyCallback;

#[async_trait]
impl TriggerCallback for MyCallback {
    async fn on_trigger(
        &self,
        workflow_id: WorkflowId,
        inputs: serde_json::Value,
        event: &Event,
    ) -> WorkflowResult<String> {
        // Execute workflow with inputs
        Ok("execution-id".to_string())
    }
}

let manager = TriggerManager::new(Arc::new(MyCallback));

// Register a trigger
let trigger = Trigger::new(
    "user_created_trigger",
    workflow_id,
    TriggerCondition::EventType {
        event_type: "user.created".to_string(),
    },
);

manager.register_trigger(trigger).await?;

// Process events
let event = Event::new("user.created", serde_json::json!({"user_id": 123}));
let executions = manager.process_event(event).await?;
```

### Using Workflow Templates

```rust
use meridian_workflow::prelude::*;

let registry = TemplateRegistry::new();

// Use a standard template
let template = StandardTemplates::sequential_pipeline();
let template_id = registry.register_template(template).await?;

// Render workflow with parameters
let mut params = HashMap::new();
params.insert("task_type_1".to_string(), serde_json::json!("extract"));
params.insert("task_type_2".to_string(), serde_json::json!("transform"));
params.insert("task_type_3".to_string(), serde_json::json!("load"));

let workflow = registry.render_workflow(template_id, params).await?;
```

## Module Overview

### DAG (`dag.rs`)
- Workflow and task definition
- Directed acyclic graph structure
- Cycle detection and topological sorting
- Dependency management

### Scheduler (`scheduler.rs`)
- Cron-based job scheduling
- Timezone support with chrono-tz
- Concurrent execution limits
- Schedule enable/disable controls

### Queue (`queue.rs`)
- Priority-based task queue
- Configurable concurrency
- Task acknowledgment (ack/nack)
- Queue statistics

### Executor (`executor.rs`)
- Workflow execution engine
- Task handler registration
- Parallel and sequential execution
- Timeout management

### State (`state.rs`)
- Workflow state machine
- Task execution tracking
- State transitions and validation
- Execution metadata

### Retry (`retry.rs`)
- Exponential backoff strategy
- Linear backoff strategy
- Fixed delay strategy
- Custom retry policies

### DLQ (`dlq.rs`)
- Dead letter queue for failed tasks
- Failure reason tracking
- Retry from DLQ support
- TTL and capacity management

### Versioning (`versioning.rs`)
- Workflow version control
- Migration strategies
- Version history tracking
- Backward compatibility

### Triggers (`triggers.rs`)
- Event-driven execution
- Flexible trigger conditions
- Input mapping from events
- Cooldown periods

### Templates (`templates.rs`)
- Reusable workflow definitions
- Parameter validation
- Template rendering
- Standard template library

### Progress (`progress.rs`)
- Real-time progress tracking
- ETA estimation
- Progress rate calculation
- Trend analysis

## Configuration

### Retry Policies

```rust
// Exponential backoff
let policy = RetryPolicy::new(
    RetryStrategy::exponential(1000, 2.0, 60000, 5)
);

// Linear backoff
let policy = RetryPolicy::new(
    RetryStrategy::linear(1000, 500, 10000, 10)
);

// Fixed delay
let policy = RetryPolicy::new(
    RetryStrategy::fixed(2000, 3)
);

// Custom delays
let policy = RetryPolicy::new(
    RetryStrategy::custom(vec![1000, 2000, 5000, 10000])
);
```

### Execution Options

```rust
let options = ExecutionOptions {
    max_concurrent: 10,
    default_retry_policy: RetryPolicy::default(),
    fail_fast: false,
    timeout: Some(Duration::from_secs(3600)),
    worker_id: Some("worker-1".to_string()),
};
```

## Testing

Run tests:

```bash
cargo test -p meridian-workflow
```

Run specific module tests:

```bash
cargo test -p meridian-workflow --lib dag
cargo test -p meridian-workflow --lib scheduler
```

## Performance

- Supports thousands of concurrent workflows
- Efficient DAG traversal with topological sorting
- Lock-free task queue operations where possible
- Optimized state transitions

## Examples

See the `examples/` directory for complete examples:

- `basic_workflow.rs` - Simple sequential workflow
- `parallel_workflow.rs` - Parallel task execution
- `scheduled_workflow.rs` - Cron-based scheduling
- `event_triggered.rs` - Event-driven workflows
- `template_usage.rs` - Using workflow templates
- `retry_policies.rs` - Configuring retry behavior

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass
2. Code is formatted with `cargo fmt`
3. No clippy warnings: `cargo clippy`
4. Documentation is updated

## License

MIT License - see LICENSE file for details

## Version History

### v0.1.5 (Current)
- Initial release of enterprise workflow engine
- Complete DAG-based workflow support
- Cron scheduling with timezone support
- Priority-based distributed task queue
- Comprehensive retry policies
- Dead letter queue for failed tasks
- Workflow versioning and migration
- Event-triggered workflows
- Reusable workflow templates
- Real-time progress tracking with ETA

## Architecture Decisions

### Why DAG-based?
- Ensures no circular dependencies
- Clear execution order
- Easy to visualize and debug
- Efficient parallel execution

### Why Async/Await?
- Non-blocking I/O operations
- Efficient resource utilization
- Scalable to thousands of workflows
- Better error handling

### Why Separate Queue?
- Decouples workflow definition from execution
- Enables distributed processing
- Priority-based execution
- Rate limiting and backpressure

## Roadmap

- [ ] Workflow visualization API
- [ ] GraphQL API for workflow management
- [ ] Workflow debugging tools
- [ ] Performance benchmarks
- [ ] Distributed execution coordinator
- [ ] Workflow composition (sub-workflows)
- [ ] Advanced conditional logic (if/else, loops)
- [ ] Workflow pause/resume
- [ ] Manual approval steps
- [ ] Integration with external systems
