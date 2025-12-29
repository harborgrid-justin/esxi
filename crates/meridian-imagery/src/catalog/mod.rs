//! Imagery catalog and STAC support

pub mod stac;
pub mod search;

pub use stac::{StacCatalog, StacItem, StacAsset};
pub use search::{ImagerySearch, SearchCriteria, SearchResult};

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Catalog entry for an imagery item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    /// Unique identifier
    pub id: String,
    /// Item title
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// Geographic bounds [min_lon, min_lat, max_lon, max_lat]
    pub bbox: [f64; 4],
    /// Acquisition datetime
    pub datetime: chrono::DateTime<chrono::Utc>,
    /// Cloud cover percentage (0-100)
    pub cloud_cover: Option<f32>,
    /// Platform/sensor
    pub platform: String,
    /// Instrument
    pub instrument: String,
    /// File path or URL
    pub path: String,
    /// Additional properties
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}
