//! ETag generation for HTTP caching

use crate::tile::coordinate::TileCoordinate;
use md5::{Md5, Digest};
use sha2::Sha256;

/// ETag generation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ETagStrategy {
    /// MD5 hash of content
    MD5,
    /// SHA256 hash of content
    SHA256,
    /// Tile coordinate-based (z/x/y)
    Coordinate,
}

/// ETag generator
pub struct ETagGenerator {
    strategy: ETagStrategy,
}

impl ETagGenerator {
    /// Create a new ETag generator
    pub fn new(strategy: ETagStrategy) -> Self {
        Self { strategy }
    }

    /// Generate ETag for tile data
    pub fn generate(&self, tile: TileCoordinate, data: &[u8]) -> String {
        match self.strategy {
            ETagStrategy::MD5 => self.generate_md5(data),
            ETagStrategy::SHA256 => self.generate_sha256(data),
            ETagStrategy::Coordinate => self.generate_coordinate(tile),
        }
    }

    /// Generate MD5-based ETag
    fn generate_md5(&self, data: &[u8]) -> String {
        let mut hasher = Md5::new();
        hasher.update(data);
        let result = hasher.finalize();
        format!("\"{:x}\"", result)
    }

    /// Generate SHA256-based ETag
    fn generate_sha256(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        format!("\"{:x}\"", result)
    }

    /// Generate coordinate-based ETag
    fn generate_coordinate(&self, tile: TileCoordinate) -> String {
        format!("\"{}-{}-{}\"", tile.z, tile.x, tile.y)
    }

    /// Validate if ETag matches
    pub fn validate(&self, tile: TileCoordinate, data: &[u8], etag: &str) -> bool {
        let generated = self.generate(tile, data);
        generated == etag
    }
}

impl Default for ETagGenerator {
    fn default() -> Self {
        Self::new(ETagStrategy::MD5)
    }
}

/// Extract ETag from If-None-Match header
pub fn parse_if_none_match(header: &str) -> Option<String> {
    // Simple parser - just extract the first quoted value
    if header.starts_with('"') && header.ends_with('"') {
        Some(header.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5_etag() {
        let generator = ETagGenerator::new(ETagStrategy::MD5);
        let tile = TileCoordinate::new(10, 512, 384);
        let data = b"test data";

        let etag = generator.generate(tile, data);
        assert!(etag.starts_with('"'));
        assert!(etag.ends_with('"'));

        // Same data should produce same ETag
        let etag2 = generator.generate(tile, data);
        assert_eq!(etag, etag2);
    }

    #[test]
    fn test_coordinate_etag() {
        let generator = ETagGenerator::new(ETagStrategy::Coordinate);
        let tile = TileCoordinate::new(10, 512, 384);
        let data = b"test data";

        let etag = generator.generate(tile, data);
        assert_eq!(etag, "\"10-512-384\"");
    }

    #[test]
    fn test_etag_validation() {
        let generator = ETagGenerator::new(ETagStrategy::MD5);
        let tile = TileCoordinate::new(10, 512, 384);
        let data = b"test data";

        let etag = generator.generate(tile, data);
        assert!(generator.validate(tile, data, &etag));

        let other_data = b"other data";
        assert!(!generator.validate(tile, other_data, &etag));
    }

    #[test]
    fn test_parse_if_none_match() {
        let etag = "\"abc123\"";
        let parsed = parse_if_none_match(etag);
        assert_eq!(parsed, Some(etag.to_string()));

        let invalid = "abc123";
        assert!(parse_if_none_match(invalid).is_none());
    }
}
