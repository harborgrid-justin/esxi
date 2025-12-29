//! Mapbox Style Specification support

pub mod glyph;
pub mod spec;
pub mod sprite;

pub use glyph::GlyphManager;
pub use spec::{Style, StyleLayer, StyleSource};
pub use sprite::SpriteSheet;

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Style metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleMetadata {
    /// Style name
    pub name: String,
    /// Style version
    pub version: u8,
    /// Sprite URL
    pub sprite: Option<String>,
    /// Glyphs URL template
    pub glyphs: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_metadata() {
        let metadata = StyleMetadata {
            name: "Test Style".to_string(),
            version: 8,
            sprite: None,
            glyphs: None,
        };
        assert_eq!(metadata.name, "Test Style");
    }
}
