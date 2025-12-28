//! Distributed task queue with priority levels.

use crate::error::{WorkflowError, WorkflowResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, Semaphore};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// A unique identifier for a queued task.
pub type QueuedTaskId = Uuid;

/// Priority level for tasks in the queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    /// Low priority (value: 0).
    Low = 0,
    /// Normal priority (value: 1).
    Normal = 1,
    /// High priority (value: 2).
    High = 2,
    /// Critical priority (value: 3).
    Critical = 3,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

/// A task in the queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedTask<T> {
    /// Unique task identifier.
    pub id: QueuedTaskId,

    /// Task payload.
    pub payload: T,

    /// Task priority.
    pub priority: Priority,

    /// Timestamp when the task was queued.
    pub queued_at: DateTime<Utc>,

    /// Optional execution delay (task won't be processed before this time).
    pub execute_after: Option<DateTime<Utc>>,

    /// Number of times this task has been attempted.
    pub attempt_count: u32,

    /// Custom metadata.
    pub metadata: HashMap<String, String>,
}

impl<T> QueuedTask<T> {
    /// Creates a new queued task.
    pub fn new(payload: T) -> Self {
        Self {
            id: Uuid::new_v4(),
            payload,
            priority: Priority::default(),
            queued_at: Utc::now(),
            execute_after: None,
            attempt_count: 0,
            metadata: HashMap::new(),
        }
    }

    /// Sets the task priority.
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Sets a delay before execution.
    pub fn with_delay(mut self, delay: chrono::Duration) -> Self {
        self.execute_after = Some(Utc::now() + delay);
        self
    }

    /// Checks if the task is ready for execution.
    pub fn is_ready(&self) -> bool {
        match self.execute_after {
            Some(execute_after) => Utc::now() >= execute_after,
            None => true,
        }
    }
}

/// Internal wrapper for priority queue ordering.
#[derive(Debug, Clone)]
struct PriorityWrapper<T> {
    task: QueuedTask<T>,
}

impl<T> PartialEq for PriorityWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.task.priority == other.task.priority && self.task.queued_at == other.task.queued_at
    }
}

impl<T> Eq for PriorityWrapper<T> {}

impl<T> PartialOrd for PriorityWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for PriorityWrapper<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by priority (higher priority first)
        match self.task.priority.cmp(&other.task.priority) {
            Ordering::Equal => {
                // Then by queued time (earlier first - reverse order for min-heap behavior)
                other.task.queued_at.cmp(&self.task.queued_at)
            }
            ordering => ordering,
        }
    }
}

/// Statistics for the task queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    /// Total number of tasks in the queue.
    pub total_tasks: usize,

    /// Number of tasks by priority.
    pub tasks_by_priority: HashMap<String, usize>,

    /// Number of tasks currently being processed.
    pub in_flight: usize,

    /// Maximum concurrent tasks allowed.
    pub max_concurrent: usize,
}

/// Distributed task queue with priority support.
pub struct TaskQueue<T: Clone + Send + Sync> {
    /// Priority queue for tasks.
    queue: Arc<Mutex<BinaryHeap<PriorityWrapper<T>>>>,

    /// In-flight tasks (being processed).
    in_flight: Arc<RwLock<HashMap<QueuedTaskId, QueuedTask<T>>>>,

    /// Semaphore for controlling concurrent execution.
    semaphore: Arc<Semaphore>,

    /// Maximum queue size (0 = unlimited).
    max_queue_size: usize,

    /// Queue name for identification.
    name: String,
}

impl<T: Clone + Send + Sync> TaskQueue<T> {
    /// Creates a new task queue.
    pub fn new(name: impl Into<String>, max_concurrent: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(BinaryHeap::new())),
            in_flight: Arc::new(RwLock::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_queue_size: 0,
            name: name.into(),
        }
    }

    /// Sets the maximum queue size.
    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_queue_size = max_size;
        self
    }

    /// Enqueues a task.
    pub async fn enqueue(&self, task: QueuedTask<T>) -> WorkflowResult<QueuedTaskId> {
        let mut queue = self.queue.lock().await;

        // Check queue size limit
        if self.max_queue_size > 0 && queue.len() >= self.max_queue_size {
            return Err(WorkflowError::QueueError(format!(
                "Queue {} is full (max size: {})",
                self.name, self.max_queue_size
            )));
        }

        let task_id = task.id;
        info!(
            "Enqueuing task {} with priority {:?} in queue {}",
            task_id, task.priority, self.name
        );

        queue.push(PriorityWrapper { task });
        Ok(task_id)
    }

    /// Enqueues multiple tasks.
    pub async fn enqueue_batch(&self, tasks: Vec<QueuedTask<T>>) -> WorkflowResult<Vec<QueuedTaskId>> {
        let mut queue = self.queue.lock().await;

        // Check queue size limit
        if self.max_queue_size > 0 && queue.len() + tasks.len() > self.max_queue_size {
            return Err(WorkflowError::QueueError(format!(
                "Batch would exceed queue {} size limit (max size: {})",
                self.name, self.max_queue_size
            )));
        }

        let mut task_ids = Vec::new();
        for task in tasks {
            let task_id = task.id;
            task_ids.push(task_id);
            queue.push(PriorityWrapper { task });
        }

        info!("Enqueued batch of {} tasks in queue {}", task_ids.len(), self.name);
        Ok(task_ids)
    }

    /// Dequeues the next ready task (blocks until a permit is available).
    pub async fn dequeue(&self) -> WorkflowResult<Option<QueuedTask<T>>> {
        // Acquire semaphore permit
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            WorkflowError::QueueError(format!("Failed to acquire semaphore: {}", e))
        })?;

        let mut queue = self.queue.lock().await;

        // Find the first ready task
        let mut temp_queue = BinaryHeap::new();
        let mut result = None;

        while let Some(wrapper) = queue.pop() {
            if wrapper.task.is_ready() {
                result = Some(wrapper.task);
                break;
            } else {
                temp_queue.push(wrapper);
            }
        }

        // Put non-ready tasks back
        while let Some(wrapper) = temp_queue.pop() {
            queue.push(wrapper);
        }

        if let Some(ref task) = result {
            // Add to in-flight
            let mut in_flight = self.in_flight.write().await;
            in_flight.insert(task.id, task.clone());

            debug!("Dequeued task {} from queue {}", task.id, self.name);
        }

        // Note: The permit is intentionally forgotten here, as it will be released
        // when the task is acknowledged or nacked
        std::mem::forget(_permit);

        Ok(result)
    }

    /// Tries to dequeue without blocking (returns None if no permit available).
    pub async fn try_dequeue(&self) -> WorkflowResult<Option<QueuedTask<T>>> {
        // Try to acquire semaphore permit without blocking
        let permit = match self.semaphore.try_acquire() {
            Ok(permit) => permit,
            Err(_) => return Ok(None),
        };

        let mut queue = self.queue.lock().await;

        // Find the first ready task
        let mut temp_queue = BinaryHeap::new();
        let mut result = None;

        while let Some(wrapper) = queue.pop() {
            if wrapper.task.is_ready() {
                result = Some(wrapper.task);
                break;
            } else {
                temp_queue.push(wrapper);
            }
        }

        // Put non-ready tasks back
        while let Some(wrapper) = temp_queue.pop() {
            queue.push(wrapper);
        }

        if let Some(ref task) = result {
            // Add to in-flight
            let mut in_flight = self.in_flight.write().await;
            in_flight.insert(task.id, task.clone());

            debug!("Dequeued task {} from queue {}", task.id, self.name);
            std::mem::forget(permit);
        } else {
            // Release the permit if no task was dequeued
            drop(permit);
        }

        Ok(result)
    }

    /// Acknowledges a task (removes from in-flight and releases permit).
    pub async fn ack(&self, task_id: QueuedTaskId) -> WorkflowResult<()> {
        let mut in_flight = self.in_flight.write().await;
        in_flight
            .remove(&task_id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Task {} not in flight", task_id)))?;

        self.semaphore.add_permits(1);
        debug!("Acknowledged task {} in queue {}", task_id, self.name);
        Ok(())
    }

    /// Negative acknowledges a task (requeues it and releases permit).
    pub async fn nack(&self, task_id: QueuedTaskId, requeue: bool) -> WorkflowResult<()> {
        let mut in_flight = self.in_flight.write().await;
        let mut task = in_flight
            .remove(&task_id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Task {} not in flight", task_id)))?;

        if requeue {
            task.attempt_count += 1;
            drop(in_flight); // Release lock before enqueuing
            self.enqueue(task).await?;
            warn!("Negative acknowledged and requeued task {} in queue {}", task_id, self.name);
        } else {
            warn!("Negative acknowledged task {} (not requeued) in queue {}", task_id, self.name);
        }

        self.semaphore.add_permits(1);
        Ok(())
    }

    /// Gets the current queue statistics.
    pub async fn stats(&self) -> QueueStats {
        let queue = self.queue.lock().await;
        let in_flight = self.in_flight.read().await;

        let mut tasks_by_priority: HashMap<String, usize> = HashMap::new();
        for wrapper in queue.iter() {
            let priority_name = format!("{:?}", wrapper.task.priority);
            *tasks_by_priority.entry(priority_name).or_insert(0) += 1;
        }

        QueueStats {
            total_tasks: queue.len(),
            tasks_by_priority,
            in_flight: in_flight.len(),
            max_concurrent: self.semaphore.available_permits(),
        }
    }

    /// Gets the number of tasks in the queue.
    pub async fn len(&self) -> usize {
        let queue = self.queue.lock().await;
        queue.len()
    }

    /// Checks if the queue is empty.
    pub async fn is_empty(&self) -> bool {
        let queue = self.queue.lock().await;
        queue.is_empty()
    }

    /// Clears all tasks from the queue (does not affect in-flight tasks).
    pub async fn clear(&self) {
        let mut queue = self.queue.lock().await;
        queue.clear();
        info!("Cleared queue {}", self.name);
    }

    /// Gets all in-flight tasks.
    pub async fn in_flight_tasks(&self) -> Vec<QueuedTask<T>> {
        let in_flight = self.in_flight.read().await;
        in_flight.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_queue_enqueue_dequeue() {
        let queue = TaskQueue::new("test", 10);

        let task1 = QueuedTask::new("task1".to_string()).with_priority(Priority::Normal);
        let task2 = QueuedTask::new("task2".to_string()).with_priority(Priority::High);

        queue.enqueue(task1).await.unwrap();
        queue.enqueue(task2).await.unwrap();

        // High priority task should be dequeued first
        let dequeued = queue.dequeue().await.unwrap().unwrap();
        assert_eq!(dequeued.payload, "task2");
        assert_eq!(dequeued.priority, Priority::High);
    }

    #[tokio::test]
    async fn test_queue_priority_ordering() {
        let queue = TaskQueue::new("test", 10);

        let low = QueuedTask::new("low".to_string()).with_priority(Priority::Low);
        let normal = QueuedTask::new("normal".to_string()).with_priority(Priority::Normal);
        let high = QueuedTask::new("high".to_string()).with_priority(Priority::High);
        let critical = QueuedTask::new("critical".to_string()).with_priority(Priority::Critical);

        queue.enqueue(low).await.unwrap();
        queue.enqueue(normal).await.unwrap();
        queue.enqueue(critical).await.unwrap();
        queue.enqueue(high).await.unwrap();

        assert_eq!(queue.dequeue().await.unwrap().unwrap().payload, "critical");
        assert_eq!(queue.dequeue().await.unwrap().unwrap().payload, "high");
        assert_eq!(queue.dequeue().await.unwrap().unwrap().payload, "normal");
        assert_eq!(queue.dequeue().await.unwrap().unwrap().payload, "low");
    }

    #[tokio::test]
    async fn test_queue_ack_nack() {
        let queue = TaskQueue::new("test", 10);

        let task = QueuedTask::new("task".to_string());
        let task_id = task.id;

        queue.enqueue(task).await.unwrap();

        let dequeued = queue.dequeue().await.unwrap().unwrap();
        assert_eq!(dequeued.id, task_id);

        // Test ack
        queue.ack(task_id).await.unwrap();

        let in_flight = queue.in_flight_tasks().await;
        assert!(in_flight.is_empty());
    }

    #[tokio::test]
    async fn test_queue_stats() {
        let queue = TaskQueue::new("test", 10);

        let task1 = QueuedTask::new("task1".to_string()).with_priority(Priority::High);
        let task2 = QueuedTask::new("task2".to_string()).with_priority(Priority::Normal);

        queue.enqueue(task1).await.unwrap();
        queue.enqueue(task2).await.unwrap();

        let stats = queue.stats().await;
        assert_eq!(stats.total_tasks, 2);
    }
}
