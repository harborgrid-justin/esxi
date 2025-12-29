//! Real-time index updates and document management.

use crate::client::SearchClient;
use crate::error::{SearchError, SearchResult};
use elasticsearch::{DeleteParts, GetParts, IndexParts, UpdateParts};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, info};

/// Real-time document manager.
pub struct RealtimeIndexer {
    client: SearchClient,
}

impl RealtimeIndexer {
    /// Create a new real-time indexer.
    pub fn new(client: SearchClient) -> Self {
        Self { client }
    }

    /// Index a single document with immediate availability.
    pub async fn index_document<T: Serialize>(
        &self,
        index: &str,
        id: Option<&str>,
        document: T,
        refresh: RefreshPolicy,
    ) -> SearchResult<IndexResponse> {
        info!(
            "Indexing document to '{}' with refresh policy: {:?}",
            index, refresh
        );

        let doc = serde_json::to_value(document)?;
        debug!("Document: {}", serde_json::to_string_pretty(&doc).unwrap());

        let request = if let Some(id) = id {
            self.client
                .client()
                .index(IndexParts::IndexId(index, id))
                .body(doc)
        } else {
            self.client.client().index(IndexParts::Index(index))
                .body(doc)
        };

        let request = match refresh {
            RefreshPolicy::True => request.refresh(elasticsearch::params::Refresh::True),
            RefreshPolicy::WaitFor => {
                request.refresh(elasticsearch::params::Refresh::WaitFor)
            }
            RefreshPolicy::False => request,
        };

        let response = request.send().await?;

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::IndexError(format!(
                "Failed to index document: {}",
                error_text
            )));
        }

        let body: Value = response.json().await?;

        Ok(IndexResponse {
            id: body["_id"].as_str().unwrap_or("").to_string(),
            version: body["_version"].as_u64().unwrap_or(1),
            result: body["result"].as_str().unwrap_or("").to_string(),
            created: body["result"].as_str() == Some("created"),
        })
    }

    /// Get a document by ID.
    pub async fn get_document<T: for<'de> Deserialize<'de>>(
        &self,
        index: &str,
        id: &str,
    ) -> SearchResult<Option<DocumentResponse<T>>> {
        debug!("Getting document from '{}' with id: {}", index, id);

        let response = self
            .client
            .client()
            .get(GetParts::IndexId(index, id))
            .send()
            .await?;

        if response.status_code().as_u16() == 404 {
            return Ok(None);
        }

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::ElasticsearchError(format!(
                "Failed to get document: {}",
                error_text
            )));
        }

        let body: Value = response.json().await?;

        let found = body["found"].as_bool().unwrap_or(false);
        if !found {
            return Ok(None);
        }

        let source: T = serde_json::from_value(body["_source"].clone())?;

        Ok(Some(DocumentResponse {
            id: body["_id"].as_str().unwrap_or("").to_string(),
            version: body["_version"].as_u64().unwrap_or(1),
            source,
        }))
    }

    /// Update a document.
    pub async fn update_document<T: Serialize>(
        &self,
        index: &str,
        id: &str,
        document: T,
        refresh: RefreshPolicy,
        upsert: bool,
    ) -> SearchResult<UpdateResponse> {
        info!(
            "Updating document in '{}' with id: {}, upsert: {}",
            index, id, upsert
        );

        let doc = serde_json::to_value(document)?;

        let body = if upsert {
            json!({
                "doc": doc,
                "doc_as_upsert": true
            })
        } else {
            json!({
                "doc": doc
            })
        };

        let mut request = self
            .client
            .client()
            .update(UpdateParts::IndexId(index, id))
            .body(body);

        match refresh {
            RefreshPolicy::True => {
                request = request.refresh(elasticsearch::params::Refresh::True)
            }
            RefreshPolicy::WaitFor => {
                request = request.refresh(elasticsearch::params::Refresh::WaitFor)
            }
            RefreshPolicy::False => {}
        }

        let response = request.send().await?;

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::IndexError(format!(
                "Failed to update document: {}",
                error_text
            )));
        }

        let result: Value = response.json().await?;

        Ok(UpdateResponse {
            id: result["_id"].as_str().unwrap_or("").to_string(),
            version: result["_version"].as_u64().unwrap_or(1),
            result: result["result"].as_str().unwrap_or("").to_string(),
        })
    }

    /// Delete a document.
    pub async fn delete_document(
        &self,
        index: &str,
        id: &str,
        refresh: RefreshPolicy,
    ) -> SearchResult<DeleteResponse> {
        info!("Deleting document from '{}' with id: {}", index, id);

        let mut request = self.client.client().delete(DeleteParts::IndexId(index, id));

        match refresh {
            RefreshPolicy::True => {
                request = request.refresh(elasticsearch::params::Refresh::True)
            }
            RefreshPolicy::WaitFor => {
                request = request.refresh(elasticsearch::params::Refresh::WaitFor)
            }
            RefreshPolicy::False => {}
        }

        let response = request.send().await?;

        if response.status_code().as_u16() == 404 {
            return Err(SearchError::DocumentNotFound(id.to_string()));
        }

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::IndexError(format!(
                "Failed to delete document: {}",
                error_text
            )));
        }

        let result: Value = response.json().await?;

        Ok(DeleteResponse {
            id: result["_id"].as_str().unwrap_or("").to_string(),
            version: result["_version"].as_u64().unwrap_or(1),
            result: result["result"].as_str().unwrap_or("").to_string(),
            deleted: result["result"].as_str() == Some("deleted"),
        })
    }

    /// Check if a document exists.
    pub async fn document_exists(&self, index: &str, id: &str) -> SearchResult<bool> {
        debug!("Checking if document exists in '{}' with id: {}", index, id);

        let response = self
            .client
            .client()
            .exists(elasticsearch::ExistsParts::IndexId(index, id))
            .send()
            .await?;

        Ok(response.status_code().is_success())
    }

    /// Perform a partial update using a script.
    pub async fn script_update(
        &self,
        index: &str,
        id: &str,
        script: ScriptUpdate,
        refresh: RefreshPolicy,
    ) -> SearchResult<UpdateResponse> {
        info!(
            "Updating document in '{}' with id: {} using script",
            index, id
        );

        let body = script.to_json();

        let mut request = self
            .client
            .client()
            .update(UpdateParts::IndexId(index, id))
            .body(body);

        match refresh {
            RefreshPolicy::True => {
                request = request.refresh(elasticsearch::params::Refresh::True)
            }
            RefreshPolicy::WaitFor => {
                request = request.refresh(elasticsearch::params::Refresh::WaitFor)
            }
            RefreshPolicy::False => {}
        }

        let response = request.send().await?;

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::IndexError(format!(
                "Failed to update document with script: {}",
                error_text
            )));
        }

        let result: Value = response.json().await?;

        Ok(UpdateResponse {
            id: result["_id"].as_str().unwrap_or("").to_string(),
            version: result["_version"].as_u64().unwrap_or(1),
            result: result["result"].as_str().unwrap_or("").to_string(),
        })
    }
}

/// Refresh policy for index operations.
#[derive(Debug, Clone, Copy)]
pub enum RefreshPolicy {
    /// Do not refresh (default, fastest)
    False,
    /// Refresh immediately (slowest, but makes changes visible immediately)
    True,
    /// Wait for refresh (balanced)
    WaitFor,
}

/// Response from indexing a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexResponse {
    pub id: String,
    pub version: u64,
    pub result: String,
    pub created: bool,
}

/// Response from getting a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentResponse<T> {
    pub id: String,
    pub version: u64,
    pub source: T,
}

/// Response from updating a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResponse {
    pub id: String,
    pub version: u64,
    pub result: String,
}

/// Response from deleting a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteResponse {
    pub id: String,
    pub version: u64,
    pub result: String,
    pub deleted: bool,
}

/// Script update for partial document updates.
#[derive(Debug, Clone)]
pub struct ScriptUpdate {
    pub script: String,
    pub lang: String,
    pub params: serde_json::Map<String, Value>,
}

impl ScriptUpdate {
    /// Create a new script update.
    pub fn new(script: impl Into<String>) -> Self {
        Self {
            script: script.into(),
            lang: "painless".to_string(),
            params: serde_json::Map::new(),
        }
    }

    /// Add a parameter to the script.
    pub fn param(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    fn to_json(&self) -> Value {
        json!({
            "script": {
                "source": self.script,
                "lang": self.lang,
                "params": self.params
            }
        })
    }
}

/// Document change tracker for optimistic locking.
pub struct ChangeTracker {
    client: SearchClient,
}

impl ChangeTracker {
    /// Create a new change tracker.
    pub fn new(client: SearchClient) -> Self {
        Self { client }
    }

    /// Update a document with version checking (optimistic locking).
    pub async fn update_with_version<T: Serialize>(
        &self,
        index: &str,
        id: &str,
        document: T,
        expected_version: u64,
    ) -> SearchResult<UpdateResponse> {
        info!(
            "Updating document in '{}' with id: {} and expected version: {}",
            index, id, expected_version
        );

        let doc = serde_json::to_value(document)?;

        let body = json!({
            "doc": doc
        });

        let response = self
            .client
            .client()
            .update(UpdateParts::IndexId(index, id))
            .body(body)
            .if_seq_no(expected_version as i64)
            .send()
            .await?;

        if response.status_code().as_u16() == 409 {
            return Err(SearchError::Internal(
                "Version conflict - document was modified".to_string(),
            ));
        }

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::IndexError(format!(
                "Failed to update document: {}",
                error_text
            )));
        }

        let result: Value = response.json().await?;

        Ok(UpdateResponse {
            id: result["_id"].as_str().unwrap_or("").to_string(),
            version: result["_version"].as_u64().unwrap_or(1),
            result: result["result"].as_str().unwrap_or("").to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_update_builder() {
        let script = ScriptUpdate::new("ctx._source.count += params.increment")
            .param("increment", 1);

        assert_eq!(script.lang, "painless");
        assert_eq!(script.params.len(), 1);
        assert!(script.params.contains_key("increment"));
    }

    #[test]
    fn test_script_update_json() {
        let script = ScriptUpdate::new("ctx._source.count += 1");
        let json = script.to_json();

        assert!(json["script"]["source"].is_string());
        assert_eq!(json["script"]["lang"], "painless");
    }

    #[test]
    fn test_refresh_policy() {
        let policy = RefreshPolicy::True;
        // Just ensure it compiles and can be used
        match policy {
            RefreshPolicy::True => assert!(true),
            _ => assert!(false),
        }
    }
}
