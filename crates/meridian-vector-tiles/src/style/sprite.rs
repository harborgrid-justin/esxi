//! Sprite sheet generation and management

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Sprite sheet containing icons and symbols
#[derive(Debug, Clone)]
pub struct SpriteSheet {
    /// Sprite metadata
    metadata: HashMap<String, SpriteMetadata>,
    /// Image data (PNG)
    image_data: Vec<u8>,
    /// Pixel ratio (1 for @1x, 2 for @2x)
    pixel_ratio: u8,
}

/// Metadata for a single sprite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteMetadata {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// X position in sprite sheet
    pub x: u32,
    /// Y position in sprite sheet
    pub y: u32,
    /// Pixel ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pixelRatio: Option<u8>,
}

impl SpriteSheet {
    /// Create a new sprite sheet
    pub fn new(pixel_ratio: u8) -> Self {
        Self {
            metadata: HashMap::new(),
            image_data: Vec::new(),
            pixel_ratio,
        }
    }

    /// Add a sprite
    pub fn add_sprite(&mut self, id: String, metadata: SpriteMetadata) {
        self.metadata.insert(id, metadata);
    }

    /// Set image data
    pub fn set_image_data(&mut self, data: Vec<u8>) {
        self.image_data = data;
    }

    /// Get metadata as JSON
    pub fn metadata_json(&self) -> Result<String> {
        serde_json::to_string(&self.metadata)
            .map_err(|e| Error::Style(format!("Failed to serialize sprite metadata: {}", e)))
    }

    /// Get image data
    pub fn image_data(&self) -> &[u8] {
        &self.image_data
    }

    /// Load sprite sheet from files
    pub async fn load<P: AsRef<Path>>(base_path: P, pixel_ratio: u8) -> Result<Self> {
        let base = base_path.as_ref();
        let suffix = if pixel_ratio == 1 {
            String::new()
        } else {
            format!("@{}x", pixel_ratio)
        };

        let json_path = base.with_extension(format!("{}.json", suffix));
        let png_path = base.with_extension(format!("{}.png", suffix));

        // Load metadata
        let json_data = tokio::fs::read_to_string(&json_path).await?;
        let metadata: HashMap<String, SpriteMetadata> = serde_json::from_str(&json_data)?;

        // Load image
        let image_data = tokio::fs::read(&png_path).await?;

        Ok(Self {
            metadata,
            image_data,
            pixel_ratio,
        })
    }

    /// Save sprite sheet to files
    pub async fn save<P: AsRef<Path>>(&self, base_path: P) -> Result<()> {
        let base = base_path.as_ref();
        let suffix = if self.pixel_ratio == 1 {
            String::new()
        } else {
            format!("@{}x", self.pixel_ratio)
        };

        let json_path = base.with_extension(format!("{}.json", suffix));
        let png_path = base.with_extension(format!("{}.png", suffix));

        // Save metadata
        let json_data = self.metadata_json()?;
        tokio::fs::write(&json_path, json_data).await?;

        // Save image
        tokio::fs::write(&png_path, &self.image_data).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_sheet() {
        let mut sheet = SpriteSheet::new(1);

        let metadata = SpriteMetadata {
            width: 32,
            height: 32,
            x: 0,
            y: 0,
            pixelRatio: Some(1),
        };

        sheet.add_sprite("icon1".to_string(), metadata);
        assert_eq!(sheet.metadata.len(), 1);
    }

    #[test]
    fn test_sprite_metadata_json() {
        let mut sheet = SpriteSheet::new(1);

        let metadata = SpriteMetadata {
            width: 32,
            height: 32,
            x: 0,
            y: 0,
            pixelRatio: None,
        };

        sheet.add_sprite("icon1".to_string(), metadata);
        let json = sheet.metadata_json().unwrap();
        assert!(json.contains("icon1"));
    }
}
