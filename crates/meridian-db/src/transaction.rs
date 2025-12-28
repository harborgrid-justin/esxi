//! Transaction management with rollback support

use crate::error::{DbError, DbResult};
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};
use std::ops::{Deref, DerefMut};

/// Transaction wrapper with automatic rollback on drop
pub struct DbTransaction<'a> {
    tx: Option<Transaction<'a, Postgres>>,
    committed: bool,
}

impl<'a> DbTransaction<'a> {
    /// Create a new transaction
    pub async fn new(pool: &'a PgPool) -> DbResult<Self> {
        let tx = pool
            .begin()
            .await
            .map_err(|e| DbError::TransactionError(format!("Failed to begin transaction: {}", e)))?;

        Ok(Self {
            tx: Some(tx),
            committed: false,
        })
    }

    /// Commit the transaction
    pub async fn commit(mut self) -> DbResult<()> {
        if let Some(tx) = self.tx.take() {
            tx.commit()
                .await
                .map_err(|e| DbError::TransactionError(format!("Failed to commit: {}", e)))?;
            self.committed = true;
        }
        Ok(())
    }

    /// Rollback the transaction
    pub async fn rollback(mut self) -> DbResult<()> {
        if let Some(tx) = self.tx.take() {
            tx.rollback()
                .await
                .map_err(|e| DbError::TransactionError(format!("Failed to rollback: {}", e)))?;
        }
        Ok(())
    }

    /// Get a mutable reference to the transaction
    pub fn transaction_mut(&mut self) -> &mut Transaction<'a, Postgres> {
        self.tx.as_mut().expect("Transaction already consumed")
    }

    /// Check if transaction is still active
    pub fn is_active(&self) -> bool {
        self.tx.is_some() && !self.committed
    }
}

impl<'a> Deref for DbTransaction<'a> {
    type Target = Transaction<'a, Postgres>;

    fn deref(&self) -> &Self::Target {
        self.tx.as_ref().expect("Transaction already consumed")
    }
}

impl<'a> DerefMut for DbTransaction<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tx.as_mut().expect("Transaction already consumed")
    }
}

impl<'a> Drop for DbTransaction<'a> {
    fn drop(&mut self) {
        if self.tx.is_some() && !self.committed {
            // Transaction will be automatically rolled back by sqlx
            eprintln!("Warning: Transaction dropped without commit or explicit rollback");
        }
    }
}

/// Transaction manager for complex operations
pub struct TransactionManager {
    pool: PgPool,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Execute a function within a transaction
    pub async fn execute<F, T>(&self, f: F) -> DbResult<T>
    where
        F: for<'a> FnOnce(&'a mut Transaction<'_, Postgres>) -> std::pin::Pin<Box<dyn std::future::Future<Output = DbResult<T>> + Send + 'a>> + Send,
        T: Send,
    {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DbError::TransactionError(format!("Failed to begin transaction: {}", e)))?;

        let result = f(&mut tx).await;

        match result {
            Ok(value) => {
                tx.commit()
                    .await
                    .map_err(|e| DbError::TransactionError(format!("Failed to commit: {}", e)))?;
                Ok(value)
            }
            Err(e) => {
                tx.rollback()
                    .await
                    .map_err(|rollback_err| {
                        DbError::TransactionError(format!(
                            "Failed to rollback after error: {}. Original error: {}",
                            rollback_err, e
                        ))
                    })?;
                Err(e)
            }
        }
    }

    /// Execute with retry on transient failures
    pub async fn execute_with_retry<F, T>(&self, max_retries: u32, f: F) -> DbResult<T>
    where
        F: Fn(&mut Transaction<'_, Postgres>) -> std::pin::Pin<Box<dyn std::future::Future<Output = DbResult<T>> + Send + '_>> + Send + Sync,
        T: Send,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < max_retries {
            let mut tx = self
                .pool
                .begin()
                .await
                .map_err(|e| DbError::TransactionError(format!("Failed to begin transaction: {}", e)))?;

            match f(&mut tx).await {
                Ok(value) => {
                    match tx.commit().await {
                        Ok(_) => return Ok(value),
                        Err(e) => {
                            let err = DbError::TransactionError(format!("Failed to commit: {}", e));
                            if !err.is_retryable() {
                                return Err(err);
                            }
                            last_error = Some(err);
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.rollback().await;
                    if !e.is_retryable() {
                        return Err(e);
                    }
                    last_error = Some(e);
                }
            }

            attempts += 1;
            if attempts < max_retries {
                // Exponential backoff
                let delay = std::time::Duration::from_millis(100 * 2_u64.pow(attempts));
                tokio::time::sleep(delay).await;
            }
        }

        Err(last_error.unwrap_or_else(|| {
            DbError::TransactionError(format!("Max retries ({}) exceeded", max_retries))
        }))
    }

    /// Create a savepoint
    pub async fn savepoint<'a>(
        tx: &'a mut Transaction<'_, Postgres>,
        name: &str,
    ) -> DbResult<()> {
        sqlx::query(&format!("SAVEPOINT {}", name))
            .execute(&mut **tx)
            .await
            .map_err(|e| DbError::TransactionError(format!("Failed to create savepoint: {}", e)))?;
        Ok(())
    }

    /// Release a savepoint
    pub async fn release_savepoint<'a>(
        tx: &'a mut Transaction<'_, Postgres>,
        name: &str,
    ) -> DbResult<()> {
        sqlx::query(&format!("RELEASE SAVEPOINT {}", name))
            .execute(&mut **tx)
            .await
            .map_err(|e| DbError::TransactionError(format!("Failed to release savepoint: {}", e)))?;
        Ok(())
    }

    /// Rollback to a savepoint
    pub async fn rollback_to_savepoint<'a>(
        tx: &'a mut Transaction<'_, Postgres>,
        name: &str,
    ) -> DbResult<()> {
        sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", name))
            .execute(&mut **tx)
            .await
            .map_err(|e| DbError::TransactionError(format!("Failed to rollback to savepoint: {}", e)))?;
        Ok(())
    }
}

/// Transaction isolation level
#[derive(Debug, Clone, Copy)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl IsolationLevel {
    /// Convert to SQL string
    pub fn as_sql(&self) -> &'static str {
        match self {
            IsolationLevel::ReadUncommitted => "READ UNCOMMITTED",
            IsolationLevel::ReadCommitted => "READ COMMITTED",
            IsolationLevel::RepeatableRead => "REPEATABLE READ",
            IsolationLevel::Serializable => "SERIALIZABLE",
        }
    }

    /// Set isolation level for a transaction
    pub async fn set<'a>(
        &self,
        tx: &'a mut Transaction<'_, Postgres>,
    ) -> DbResult<()> {
        let sql = format!("SET TRANSACTION ISOLATION LEVEL {}", self.as_sql());
        sqlx::query(&sql)
            .execute(&mut **tx)
            .await
            .map_err(|e| DbError::TransactionError(format!("Failed to set isolation level: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolation_level_sql() {
        assert_eq!(IsolationLevel::ReadCommitted.as_sql(), "READ COMMITTED");
        assert_eq!(IsolationLevel::Serializable.as_sql(), "SERIALIZABLE");
    }
}
