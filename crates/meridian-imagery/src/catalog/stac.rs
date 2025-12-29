//! STAC (SpatioTemporal Asset Catalog) support

use crate::error::{ImageryError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// STAC Catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StacCatalog {
    /// STAC version
    #[serde(rename = "stac_version")]
    pub stac_version: String,
    /// Catalog ID
    pub id: String,
    /// Catalog title
    pub title: Option<String>,
    /// Description
    pub description: String,
    /// Links to items and sub-catalogs
    pub links: Vec<StacLink>,
}

impl StacCatalog {
    /// Create a new STAC catalog
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            stac_version: "1.0.0".to_string(),
            id: id.into(),
            title: None,
            description: description.into(),
            links: vec![],
        }
    }

    /// Add a link
    pub fn add_link(&mut self, link: StacLink) {
        self.links.push(link);
    }

    /// Load from JSON file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let catalog: StacCatalog = serde_json::from_str(&content)?;
        Ok(catalog)
    }

    /// Save to JSON file
    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

/// STAC Item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StacItem {
    /// STAC version
    #[serde(rename = "stac_version")]
    pub stac_version: String,
    /// Item type (always "Feature" for STAC items)
    #[serde(rename = "type")]
    pub type_: String,
    /// Item ID
    pub id: String,
    /// Geometry (GeoJSON)
    pub geometry: serde_json::Value,
    /// Bounding box [min_lon, min_lat, max_lon, max_lat]
    pub bbox: Vec<f64>,
    /// Properties
    pub properties: StacProperties,
    /// Assets
    pub assets: HashMap<String, StacAsset>,
    /// Links
    pub links: Vec<StacLink>,
}

impl StacItem {
    /// Create a new STAC item
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            stac_version: "1.0.0".to_string(),
            type_: "Feature".to_string(),
            id: id.into(),
            geometry: serde_json::json!(null),
            bbox: vec![],
            properties: StacProperties::default(),
            assets: HashMap::new(),
            links: vec![],
        }
    }

    /// Set geometry from bounds
    pub fn set_bounds(&mut self, min_lon: f64, min_lat: f64, max_lon: f64, max_lat: f64) {
        self.bbox = vec![min_lon, min_lat, max_lon, max_lat];

        self.geometry = serde_json::json!({
            "type": "Polygon",
            "coordinates": [[
                [min_lon, min_lat],
                [max_lon, min_lat],
                [max_lon, max_lat],
                [min_lon, max_lat],
                [min_lon, min_lat]
            ]]
        });
    }

    /// Add an asset
    pub fn add_asset(&mut self, key: impl Into<String>, asset: StacAsset) {
        self.assets.insert(key.into(), asset);
    }

    /// Load from JSON file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let item: StacItem = serde_json::from_str(&content)?;
        Ok(item)
    }

    /// Save to JSON file
    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

/// STAC Item properties
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StacProperties {
    /// Acquisition datetime
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime: Option<String>,
    /// Start datetime for ranges
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_datetime: Option<String>,
    /// End datetime for ranges
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_datetime: Option<String>,
    /// Platform
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    /// Instruments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruments: Option<Vec<String>>,
    /// Constellation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constellation: Option<String>,
    /// Ground Sample Distance (meters)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gsd: Option<f64>,
    /// Cloud cover percentage (0-100)
    #[serde(rename = "eo:cloud_cover", skip_serializing_if = "Option::is_none")]
    pub cloud_cover: Option<f32>,
    /// Additional properties
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

/// STAC Asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StacAsset {
    /// Asset href (file path or URL)
    pub href: String,
    /// Media type
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    /// Title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Roles
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    /// EO bands
    #[serde(rename = "eo:bands", skip_serializing_if = "Option::is_none")]
    pub eo_bands: Option<Vec<EoBand>>,
}

impl StacAsset {
    /// Create a new asset
    pub fn new(href: impl Into<String>) -> Self {
        Self {
            href: href.into(),
            media_type: None,
            title: None,
            description: None,
            roles: None,
            eo_bands: None,
        }
    }

    /// Set media type
    pub fn with_media_type(mut self, media_type: impl Into<String>) -> Self {
        self.media_type = Some(media_type.into());
        self
    }

    /// Set roles
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = Some(roles);
        self
    }

    /// Add EO band
    pub fn add_eo_band(&mut self, band: EoBand) {
        if let Some(ref mut bands) = self.eo_bands {
            bands.push(band);
        } else {
            self.eo_bands = Some(vec![band]);
        }
    }
}

/// EO (Electro-Optical) Band information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EoBand {
    /// Band name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Common name (e.g., "red", "green", "nir")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub common_name: Option<String>,
    /// Center wavelength (micrometers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub center_wavelength: Option<f32>,
    /// Full width at half maximum (micrometers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_width_half_max: Option<f32>,
}

/// STAC Link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StacLink {
    /// Link relation type
    pub rel: String,
    /// Link href
    pub href: String,
    /// Media type
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    /// Title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl StacLink {
    /// Create a new link
    pub fn new(rel: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            rel: rel.into(),
            href: href.into(),
            media_type: None,
            title: None,
        }
    }

    /// Create a self link
    pub fn self_link(href: impl Into<String>) -> Self {
        Self::new("self", href)
    }

    /// Create a parent link
    pub fn parent_link(href: impl Into<String>) -> Self {
        Self::new("parent", href)
    }

    /// Create a child link
    pub fn child_link(href: impl Into<String>) -> Self {
        Self::new("child", href)
    }

    /// Create an item link
    pub fn item_link(href: impl Into<String>) -> Self {
        Self::new("item", href)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stac_catalog_creation() {
        let catalog = StacCatalog::new("test-catalog", "Test imagery catalog");
        assert_eq!(catalog.id, "test-catalog");
        assert_eq!(catalog.stac_version, "1.0.0");
    }

    #[test]
    fn test_stac_item_creation() {
        let mut item = StacItem::new("test-item");
        item.set_bounds(-180.0, -90.0, 180.0, 90.0);

        assert_eq!(item.bbox.len(), 4);
        assert_eq!(item.type_, "Feature");
    }

    #[test]
    fn test_stac_asset() {
        let asset = StacAsset::new("data.tif")
            .with_media_type("image/tiff; application=geotiff")
            .with_roles(vec!["data".to_string()]);

        assert_eq!(asset.href, "data.tif");
        assert!(asset.roles.is_some());
    }
}
