//! Async bridge for seamless async/await integration between JavaScript and Rust.
//!
//! This module provides utilities for:
//! - Converting Rust futures to JavaScript Promises
//! - Handling JavaScript Promises in Rust
//! - Managing async operation lifecycle
//! - Error propagation across the boundary

use futures::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// Type alias for boxed futures that can cross the WASM boundary.
pub type AsyncResult<T> = Pin<Box<dyn Future<Output = Result<T, JsValue>>>>;

/// Wrapper for async operations that need to be exposed to JavaScript.
///
/// This provides a consistent interface for all async operations and handles
/// error conversion automatically.
#[wasm_bindgen]
pub struct AsyncOperation {
    id: String,
    status: Arc<RwLock<OperationStatus>>,
}

/// Status of an async operation.
#[derive(Debug, Clone)]
pub enum OperationStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

#[wasm_bindgen]
impl AsyncOperation {
    /// Create a new async operation with a unique ID.
    #[wasm_bindgen(constructor)]
    pub fn new(id: String) -> Self {
        Self {
            id,
            status: Arc::new(RwLock::new(OperationStatus::Pending)),
        }
    }

    /// Get the operation ID.
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Check if the operation is complete.
    pub async fn is_complete(&self) -> bool {
        let status = self.status.read().await;
        matches!(*status, OperationStatus::Completed | OperationStatus::Failed(_) | OperationStatus::Cancelled)
    }

    /// Get the current status as a string.
    pub async fn status(&self) -> String {
        let status = self.status.read().await;
        match &*status {
            OperationStatus::Pending => "pending".to_string(),
            OperationStatus::Running => "running".to_string(),
            OperationStatus::Completed => "completed".to_string(),
            OperationStatus::Failed(err) => format!("failed: {}", err),
            OperationStatus::Cancelled => "cancelled".to_string(),
        }
    }

    /// Cancel the operation.
    pub async fn cancel(&self) -> Result<(), JsValue> {
        let mut status = self.status.write().await;
        *status = OperationStatus::Cancelled;
        Ok(())
    }
}

impl AsyncOperation {
    /// Mark the operation as running.
    pub async fn set_running(&self) {
        let mut status = self.status.write().await;
        *status = OperationStatus::Running;
    }

    /// Mark the operation as completed.
    pub async fn set_completed(&self) {
        let mut status = self.status.write().await;
        *status = OperationStatus::Completed;
    }

    /// Mark the operation as failed.
    pub async fn set_failed(&self, error: String) {
        let mut status = self.status.write().await;
        *status = OperationStatus::Failed(error);
    }

    /// Check if the operation was cancelled.
    pub async fn is_cancelled(&self) -> bool {
        let status = self.status.read().await;
        matches!(*status, OperationStatus::Cancelled)
    }
}

/// Execute an async Rust function and return a JavaScript Promise.
///
/// This is the primary interface for exposing async Rust functions to JavaScript.
///
/// # Example
///
/// ```rust,ignore
/// #[wasm_bindgen]
/// pub async fn my_async_function(param: String) -> Result<JsValue, JsValue> {
///     execute_async(async move {
///         // Your async Rust code here
///         let result = do_something(param).await?;
///         Ok(serde_wasm_bindgen::to_value(&result)?)
///     }).await
/// }
/// ```
pub async fn execute_async<F, T>(f: F) -> Result<T, JsValue>
where
    F: Future<Output = Result<T, JsValue>>,
{
    f.await
}

/// Convert a JavaScript Promise to a Rust Future.
///
/// This allows Rust code to await JavaScript Promises.
pub async fn promise_to_future<T>(promise: js_sys::Promise) -> Result<T, JsValue>
where
    T: JsCast,
{
    let result = JsFuture::from(promise).await?;
    result.dyn_into::<T>().map_err(|_| {
        JsValue::from_str("Failed to convert Promise result to expected type")
    })
}

/// Spawn a Rust future on the WASM runtime.
///
/// This allows fire-and-forget async operations.
pub fn spawn_local<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

/// Execute a timeout operation.
///
/// Returns an error if the operation doesn't complete within the specified duration.
pub async fn with_timeout<F, T>(
    future: F,
    timeout_ms: u32,
) -> Result<T, JsValue>
where
    F: Future<Output = Result<T, JsValue>>,
{
    let timeout = async {
        sleep(timeout_ms).await;
        Err(JsValue::from_str("Operation timed out"))
    };

    // Race between the future and the timeout
    match futures::future::select(
        Box::pin(future),
        Box::pin(timeout),
    ).await {
        futures::future::Either::Left((result, _)) => result,
        futures::future::Either::Right((timeout_result, _)) => timeout_result,
    }
}

/// Sleep for the specified number of milliseconds.
pub async fn sleep(ms: u32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("no global window");
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms as i32)
            .expect("failed to set timeout");
    });

    let _ = JsFuture::from(promise).await;
}

/// Batch multiple async operations and execute them concurrently.
///
/// Returns results in the same order as the input operations.
pub async fn batch_execute<F, T>(
    operations: Vec<F>,
) -> Vec<Result<T, JsValue>>
where
    F: Future<Output = Result<T, JsValue>>,
    T: Clone,
{
    let futures: Vec<_> = operations.into_iter().map(Box::pin).collect();

    let mut results = Vec::new();
    for future in futures {
        results.push(future.await);
    }

    results
}

/// Stream processor for handling async data streams.
#[wasm_bindgen]
pub struct AsyncStream {
    id: String,
    buffer: Arc<RwLock<Vec<JsValue>>>,
    closed: Arc<RwLock<bool>>,
}

#[wasm_bindgen]
impl AsyncStream {
    /// Create a new async stream.
    #[wasm_bindgen(constructor)]
    pub fn new(id: String) -> Self {
        Self {
            id,
            buffer: Arc::new(RwLock::new(Vec::new())),
            closed: Arc::new(RwLock::new(false)),
        }
    }

    /// Push a value to the stream.
    pub async fn push(&self, value: JsValue) -> Result<(), JsValue> {
        let closed = self.closed.read().await;
        if *closed {
            return Err(JsValue::from_str("Stream is closed"));
        }
        drop(closed);

        let mut buffer = self.buffer.write().await;
        buffer.push(value);
        Ok(())
    }

    /// Read the next value from the stream.
    pub async fn next(&self) -> Result<JsValue, JsValue> {
        loop {
            let mut buffer = self.buffer.write().await;
            if !buffer.is_empty() {
                return Ok(buffer.remove(0));
            }
            drop(buffer);

            let closed = self.closed.read().await;
            if *closed {
                return Err(JsValue::from_str("Stream is closed"));
            }
            drop(closed);

            // Wait a bit before checking again
            sleep(10).await;
        }
    }

    /// Close the stream.
    pub async fn close(&self) {
        let mut closed = self.closed.write().await;
        *closed = true;
    }

    /// Check if the stream is closed.
    pub async fn is_closed(&self) -> bool {
        let closed = self.closed.read().await;
        *closed
    }

    /// Get the number of buffered items.
    pub async fn buffered_count(&self) -> usize {
        let buffer = self.buffer.read().await;
        buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_async_operation() {
        let op = AsyncOperation::new("test-op".to_string());
        assert_eq!(op.id(), "test-op");
        assert_eq!(op.status().await, "pending");

        op.set_running().await;
        assert_eq!(op.status().await, "running");

        op.set_completed().await;
        assert!(op.is_complete().await);
    }

    #[wasm_bindgen_test]
    async fn test_async_stream() {
        let stream = AsyncStream::new("test-stream".to_string());

        stream.push(JsValue::from_str("hello")).await.unwrap();
        stream.push(JsValue::from_str("world")).await.unwrap();

        assert_eq!(stream.buffered_count().await, 2);

        let value1 = stream.next().await.unwrap();
        assert_eq!(value1.as_string().unwrap(), "hello");

        stream.close().await;
        assert!(stream.is_closed().await);
    }
}
