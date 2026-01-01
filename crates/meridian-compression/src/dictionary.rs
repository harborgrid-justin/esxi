//! Dictionary-based compression with training
//!
//! Advanced dictionary management for optimized compression of similar data.

use crate::error::{CompressionError, Result};
use crate::zstd::ZstdCompressor;
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;

/// Dictionary metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryMetadata {
    /// Unique dictionary ID
    pub id: String,
    /// Dictionary hash for validation
    pub hash: String,
    /// Size of the dictionary in bytes
    pub size: usize,
    /// Number of samples used for training
    pub sample_count: usize,
    /// Total size of training data
    pub training_data_size: usize,
    /// Creation timestamp
    pub created_at: std::time::SystemTime,
    /// Dictionary version
    pub version: u32,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Description
    pub description: Option<String>,
}

impl DictionaryMetadata {
    /// Create new metadata
    pub fn new(id: String, dictionary_data: &[u8], sample_count: usize, training_data_size: usize) -> Self {
        let hash = Self::compute_hash(dictionary_data);

        Self {
            id,
            hash,
            size: dictionary_data.len(),
            sample_count,
            training_data_size,
            created_at: std::time::SystemTime::now(),
            version: 1,
            tags: Vec::new(),
            description: None,
        }
    }

    /// Compute hash of dictionary data
    fn compute_hash(data: &[u8]) -> String {
        let mut hasher = Hasher::new();
        hasher.update(data);
        let hash = hasher.finalize();
        hex::encode(hash.as_bytes())
    }

    /// Validate dictionary against metadata
    pub fn validate(&self, dictionary_data: &[u8]) -> bool {
        let computed_hash = Self::compute_hash(dictionary_data);
        computed_hash == self.hash && dictionary_data.len() == self.size
    }
}

/// Dictionary entry with data and metadata
#[derive(Debug, Clone)]
pub struct Dictionary {
    /// Dictionary metadata
    pub metadata: DictionaryMetadata,
    /// Dictionary data
    pub data: Arc<Vec<u8>>,
}

impl Dictionary {
    /// Create a new dictionary
    pub fn new(id: String, data: Vec<u8>, sample_count: usize, training_data_size: usize) -> Self {
        let metadata = DictionaryMetadata::new(id, &data, sample_count, training_data_size);

        Self {
            metadata,
            data: Arc::new(data),
        }
    }

    /// Validate dictionary integrity
    pub fn validate(&self) -> bool {
        self.metadata.validate(&self.data)
    }

    /// Get dictionary size
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// Dictionary trainer for creating optimized dictionaries
pub struct DictionaryTrainer {
    /// Target dictionary size
    target_size: usize,
    /// Maximum number of samples to use
    max_samples: usize,
    /// Minimum sample size
    min_sample_size: usize,
}

impl Default for DictionaryTrainer {
    fn default() -> Self {
        Self {
            target_size: 110 * 1024, // 110 KB (recommended for Zstd)
            max_samples: 1000,
            min_sample_size: 128,
        }
    }
}

impl DictionaryTrainer {
    /// Create a new trainer
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with target dictionary size
    pub fn with_target_size(target_size: usize) -> Self {
        Self {
            target_size,
            ..Default::default()
        }
    }

    /// Train dictionary from samples
    pub fn train(&self, id: String, samples: &[Vec<u8>]) -> Result<Dictionary> {
        if samples.is_empty() {
            return Err(CompressionError::DictionaryTraining(
                "No samples provided".to_string(),
            ));
        }

        // Filter out small samples
        let valid_samples: Vec<&Vec<u8>> = samples
            .iter()
            .filter(|s| s.len() >= self.min_sample_size)
            .collect();

        if valid_samples.is_empty() {
            return Err(CompressionError::DictionaryTraining(
                format!("No samples larger than {} bytes", self.min_sample_size),
            ));
        }

        // Limit number of samples
        let selected_samples: Vec<Vec<u8>> = valid_samples
            .into_iter()
            .take(self.max_samples)
            .cloned()
            .collect();

        let training_data_size: usize = selected_samples.iter().map(|s| s.len()).sum();

        // Train using Zstd dictionary training
        let dictionary_data = ZstdCompressor::train_dictionary(
            &selected_samples,
            self.target_size,
        )?;

        Ok(Dictionary::new(
            id,
            dictionary_data,
            selected_samples.len(),
            training_data_size,
        ))
    }

    /// Train from files
    pub async fn train_from_files(
        &self,
        id: String,
        file_paths: &[PathBuf],
    ) -> Result<Dictionary> {
        let mut samples = Vec::new();

        for path in file_paths {
            let data = tokio::fs::read(path)
                .await
                .map_err(|e| CompressionError::custom(
                    "Failed to read training file",
                    e.to_string(),
                ))?;

            samples.push(data);
        }

        self.train(id, &samples)
    }

    /// Incremental training - add samples to existing dictionary
    pub fn train_incremental(
        &self,
        existing: &Dictionary,
        new_samples: &[Vec<u8>],
    ) -> Result<Dictionary> {
        // Combine existing dictionary data with new samples
        let mut all_samples = vec![existing.data.as_ref().clone()];
        all_samples.extend_from_slice(new_samples);

        let id = format!("{}_v{}", existing.metadata.id, existing.metadata.version + 1);
        self.train(id, &all_samples)
    }
}

/// Dictionary store for managing multiple dictionaries
pub struct DictionaryStore {
    dictionaries: Arc<RwLock<HashMap<String, Dictionary>>>,
    storage_path: Option<PathBuf>,
}

impl Default for DictionaryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl DictionaryStore {
    /// Create a new dictionary store
    pub fn new() -> Self {
        Self {
            dictionaries: Arc::new(RwLock::new(HashMap::new())),
            storage_path: None,
        }
    }

    /// Create with persistent storage
    pub fn with_storage(path: PathBuf) -> Self {
        Self {
            dictionaries: Arc::new(RwLock::new(HashMap::new())),
            storage_path: Some(path),
        }
    }

    /// Add dictionary to store
    pub fn add(&self, dictionary: Dictionary) -> Result<()> {
        if !dictionary.validate() {
            return Err(CompressionError::Dictionary(
                "Dictionary validation failed".to_string(),
            ));
        }

        let id = dictionary.metadata.id.clone();
        let mut dicts = self.dictionaries.write();
        dicts.insert(id, dictionary);

        Ok(())
    }

    /// Get dictionary by ID
    pub fn get(&self, id: &str) -> Option<Dictionary> {
        let dicts = self.dictionaries.read();
        dicts.get(id).cloned()
    }

    /// Remove dictionary
    pub fn remove(&self, id: &str) -> bool {
        let mut dicts = self.dictionaries.write();
        dicts.remove(id).is_some()
    }

    /// List all dictionary IDs
    pub fn list_ids(&self) -> Vec<String> {
        let dicts = self.dictionaries.read();
        dicts.keys().cloned().collect()
    }

    /// List all dictionaries with metadata
    pub fn list_all(&self) -> Vec<DictionaryMetadata> {
        let dicts = self.dictionaries.read();
        dicts.values().map(|d| d.metadata.clone()).collect()
    }

    /// Get dictionary count
    pub fn count(&self) -> usize {
        let dicts = self.dictionaries.read();
        dicts.len()
    }

    /// Clear all dictionaries
    pub fn clear(&self) {
        let mut dicts = self.dictionaries.write();
        dicts.clear();
    }

    /// Save dictionary to disk
    pub async fn save_to_disk(&self, id: &str) -> Result<PathBuf> {
        let storage_path = self.storage_path.as_ref().ok_or_else(|| {
            CompressionError::Dictionary("No storage path configured".to_string())
        })?;

        let dictionary = self.get(id).ok_or_else(|| {
            CompressionError::Dictionary(format!("Dictionary {} not found", id))
        })?;

        // Create storage directory if it doesn't exist
        tokio::fs::create_dir_all(storage_path)
            .await
            .map_err(|e| CompressionError::custom("Failed to create storage directory", e.to_string()))?;

        // Save dictionary data
        let dict_path = storage_path.join(format!("{}.dict", id));
        tokio::fs::write(&dict_path, dictionary.data.as_ref())
            .await
            .map_err(|e| CompressionError::custom("Failed to write dictionary", e.to_string()))?;

        // Save metadata
        let meta_path = storage_path.join(format!("{}.meta.json", id));
        let meta_json = serde_json::to_string_pretty(&dictionary.metadata)
            .map_err(|e| CompressionError::custom("Failed to serialize metadata", e.to_string()))?;

        tokio::fs::write(&meta_path, meta_json)
            .await
            .map_err(|e| CompressionError::custom("Failed to write metadata", e.to_string()))?;

        Ok(dict_path)
    }

    /// Load dictionary from disk
    pub async fn load_from_disk(&self, id: &str) -> Result<()> {
        let storage_path = self.storage_path.as_ref().ok_or_else(|| {
            CompressionError::Dictionary("No storage path configured".to_string())
        })?;

        // Load dictionary data
        let dict_path = storage_path.join(format!("{}.dict", id));
        let data = tokio::fs::read(&dict_path)
            .await
            .map_err(|e| CompressionError::custom("Failed to read dictionary", e.to_string()))?;

        // Load metadata
        let meta_path = storage_path.join(format!("{}.meta.json", id));
        let meta_json = tokio::fs::read_to_string(&meta_path)
            .await
            .map_err(|e| CompressionError::custom("Failed to read metadata", e.to_string()))?;

        let metadata: DictionaryMetadata = serde_json::from_str(&meta_json)
            .map_err(|e| CompressionError::custom("Failed to parse metadata", e.to_string()))?;

        // Validate
        if !metadata.validate(&data) {
            return Err(CompressionError::Dictionary(
                "Dictionary integrity check failed".to_string(),
            ));
        }

        let dictionary = Dictionary {
            metadata,
            data: Arc::new(data),
        };

        self.add(dictionary)?;

        Ok(())
    }

    /// Load all dictionaries from storage directory
    pub async fn load_all_from_disk(&self) -> Result<usize> {
        let storage_path = self.storage_path.as_ref().ok_or_else(|| {
            CompressionError::Dictionary("No storage path configured".to_string())
        })?;

        let mut count = 0;
        let mut entries = tokio::fs::read_dir(storage_path)
            .await
            .map_err(|e| CompressionError::custom("Failed to read storage directory", e.to_string()))?;

        while let Some(entry) = entries.next_entry()
            .await
            .map_err(|e| CompressionError::custom("Failed to read directory entry", e.to_string()))?
        {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "dict" {
                    if let Some(stem) = path.file_stem() {
                        if let Some(id) = stem.to_str() {
                            if self.load_from_disk(id).await.is_ok() {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(count)
    }
}

// Add hex dependency to Cargo.toml would be needed for this
// For now, let's implement a simple hex encoder
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_creation() {
        let data = vec![1, 2, 3, 4, 5];
        let dict = Dictionary::new(
            "test_dict".to_string(),
            data.clone(),
            10,
            1000,
        );

        assert_eq!(dict.metadata.id, "test_dict");
        assert_eq!(dict.metadata.size, data.len());
        assert!(dict.validate());
    }

    #[test]
    fn test_dictionary_trainer() {
        let trainer = DictionaryTrainer::new();
        let samples = vec![
            b"Hello, World!".to_vec(),
            b"Hello, Dictionary!".to_vec(),
            b"Hello, Compression!".to_vec(),
        ];

        let dict = trainer.train("test".to_string(), &samples).unwrap();
        assert!(dict.validate());
        assert_eq!(dict.metadata.sample_count, 3);
    }

    #[test]
    fn test_dictionary_store() {
        let store = DictionaryStore::new();

        let data = vec![1, 2, 3, 4, 5];
        let dict = Dictionary::new("test".to_string(), data, 5, 100);

        store.add(dict).unwrap();
        assert_eq!(store.count(), 1);

        let retrieved = store.get("test").unwrap();
        assert_eq!(retrieved.metadata.id, "test");

        assert!(store.remove("test"));
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_metadata_validation() {
        let data = vec![1, 2, 3, 4, 5];
        let metadata = DictionaryMetadata::new(
            "test".to_string(),
            &data,
            5,
            100,
        );

        assert!(metadata.validate(&data));

        let wrong_data = vec![1, 2, 3];
        assert!(!metadata.validate(&wrong_data));
    }

    #[tokio::test]
    async fn test_persistent_storage() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let store = DictionaryStore::with_storage(temp_dir.path().to_path_buf());

        let data = vec![1, 2, 3, 4, 5];
        let dict = Dictionary::new("test_persist".to_string(), data, 5, 100);

        store.add(dict).unwrap();
        store.save_to_disk("test_persist").await.unwrap();

        // Create new store and load
        let store2 = DictionaryStore::with_storage(temp_dir.path().to_path_buf());
        store2.load_from_disk("test_persist").await.unwrap();

        assert_eq!(store2.count(), 1);
        let loaded = store2.get("test_persist").unwrap();
        assert!(loaded.validate());
    }
}
