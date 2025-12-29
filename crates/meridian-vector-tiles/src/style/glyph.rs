//! Glyph (font) handling for map labels

use crate::error::{Error, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Glyph manager for font handling
pub struct GlyphManager {
    /// Base path for glyph PBF files
    base_path: PathBuf,
    /// Cached glyph ranges
    cache: HashMap<String, Vec<u8>>,
}

impl GlyphManager {
    /// Create a new glyph manager
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            cache: HashMap::new(),
        }
    }

    /// Get glyphs for a font and range
    ///
    /// Font stack format: "Font Name"
    /// Range format: "0-255", "256-511", etc.
    pub async fn get_glyphs(&mut self, font_stack: &str, range: &str) -> Result<Vec<u8>> {
        let key = format!("{}:{}", font_stack, range);

        // Check cache
        if let Some(data) = self.cache.get(&key) {
            return Ok(data.clone());
        }

        // Load from file
        let path = self.glyph_path(font_stack, range);
        let data = tokio::fs::read(&path).await.map_err(|e| {
            Error::Style(format!(
                "Failed to load glyphs for {} range {}: {}",
                font_stack, range, e
            ))
        })?;

        // Cache the result
        self.cache.insert(key, data.clone());

        Ok(data)
    }

    /// Get the file path for a glyph range
    fn glyph_path(&self, font_stack: &str, range: &str) -> PathBuf {
        // Standard glyph path format: {fontstack}/{range}.pbf
        let mut path = self.base_path.clone();
        path.push(font_stack);
        path.push(format!("{}.pbf", range));
        path
    }

    /// Clear the glyph cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

/// Glyph range utilities
pub struct GlyphRange;

impl GlyphRange {
    /// Parse a glyph range string (e.g., "0-255")
    pub fn parse(range: &str) -> Result<(u32, u32)> {
        let parts: Vec<&str> = range.split('-').collect();
        if parts.len() != 2 {
            return Err(Error::Style(format!("Invalid glyph range: {}", range)));
        }

        let start = parts[0]
            .parse()
            .map_err(|_| Error::Style(format!("Invalid range start: {}", parts[0])))?;
        let end = parts[1]
            .parse()
            .map_err(|_| Error::Style(format!("Invalid range end: {}", parts[1])))?;

        if start > end {
            return Err(Error::Style(format!(
                "Invalid range: start {} > end {}",
                start, end
            )));
        }

        Ok((start, end))
    }

    /// Get the range for a character code
    pub fn for_char(char_code: u32) -> String {
        let start = (char_code / 256) * 256;
        let end = start + 255;
        format!("{}-{}", start, end)
    }

    /// Get all standard glyph ranges (0-65535)
    pub fn standard_ranges() -> Vec<String> {
        (0..256).map(|i| format!("{}-{}", i * 256, i * 256 + 255)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glyph_range_parse() {
        let (start, end) = GlyphRange::parse("0-255").unwrap();
        assert_eq!(start, 0);
        assert_eq!(end, 255);

        let (start, end) = GlyphRange::parse("256-511").unwrap();
        assert_eq!(start, 256);
        assert_eq!(end, 511);

        assert!(GlyphRange::parse("invalid").is_err());
        assert!(GlyphRange::parse("100-50").is_err());
    }

    #[test]
    fn test_glyph_range_for_char() {
        assert_eq!(GlyphRange::for_char(0), "0-255");
        assert_eq!(GlyphRange::for_char(256), "256-511");
        assert_eq!(GlyphRange::for_char(300), "256-511");
    }

    #[test]
    fn test_standard_ranges() {
        let ranges = GlyphRange::standard_ranges();
        assert_eq!(ranges.len(), 256);
        assert_eq!(ranges[0], "0-255");
        assert_eq!(ranges[1], "256-511");
    }

    #[test]
    fn test_glyph_manager() {
        let manager = GlyphManager::new("/tmp/glyphs");
        assert_eq!(manager.cache_size(), 0);
    }
}
