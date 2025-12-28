//! Automatic format detection for geospatial files

use crate::error::{IoError, Result};
use crate::traits::Format;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// Detect the format of a geospatial file
pub fn detect_format(path: &Path) -> Result<Format> {
    // First try extension-based detection
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if let Some(format) = Format::from_extension(ext) {
            // Verify the format by checking file contents
            if verify_format(path, format)? {
                return Ok(format);
            }
        }
    }

    // Fall back to content-based detection
    detect_by_content(path)
}

/// Verify that a file matches the expected format
fn verify_format(path: &Path, format: Format) -> Result<bool> {
    let mut file = File::open(path)?;
    let mut buffer = vec![0u8; 512];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    match format {
        Format::GeoJson => {
            // Check for JSON structure
            let content = String::from_utf8_lossy(&buffer);
            Ok(content.trim_start().starts_with('{') || content.trim_start().starts_with('['))
        }
        Format::Shapefile => {
            // Check for shapefile magic number (9994 in big-endian)
            Ok(buffer.len() >= 4 && buffer[0..4] == [0x00, 0x00, 0x27, 0x0a])
        }
        Format::Kml => {
            // Check for XML and KML namespace
            let content = String::from_utf8_lossy(&buffer);
            Ok(content.contains("<?xml") && content.contains("kml"))
        }
        Format::Kmz => {
            // Check for ZIP magic number
            Ok(buffer.len() >= 4 && buffer[0..4] == [0x50, 0x4b, 0x03, 0x04])
        }
        Format::GeoTiff => {
            // Check for TIFF magic number (II or MM)
            Ok(buffer.len() >= 4 &&
               (buffer[0..4] == [0x49, 0x49, 0x2a, 0x00] || // Little-endian
                buffer[0..4] == [0x4d, 0x4d, 0x00, 0x2a]))  // Big-endian
        }
        Format::GeoPackage => {
            // Check for SQLite magic string
            Ok(buffer.len() >= 16 && buffer[0..16] == *b"SQLite format 3\0")
        }
        Format::Wkt => {
            // Check for WKT geometry types
            let content = String::from_utf8_lossy(&buffer).to_uppercase();
            Ok(content.contains("POINT") || content.contains("LINESTRING") ||
               content.contains("POLYGON") || content.contains("MULTIPOINT") ||
               content.contains("MULTILINESTRING") || content.contains("MULTIPOLYGON") ||
               content.contains("GEOMETRYCOLLECTION"))
        }
        Format::Wkb => {
            // WKB starts with byte order marker (0 or 1) followed by geometry type
            Ok(buffer.len() >= 5 && (buffer[0] == 0x00 || buffer[0] == 0x01))
        }
        Format::Csv => {
            // Check for CSV structure (headers with lat/lon/wkt)
            let content = String::from_utf8_lossy(&buffer).to_lowercase();
            Ok(content.contains("lat") || content.contains("lon") ||
               content.contains("wkt") || content.contains("x,y"))
        }
        Format::Gml => {
            // Check for XML and GML namespace
            let content = String::from_utf8_lossy(&buffer);
            Ok(content.contains("<?xml") && content.contains("gml"))
        }
    }
}

/// Detect format by analyzing file content
fn detect_by_content(path: &Path) -> Result<Format> {
    let mut file = File::open(path)?;
    let mut buffer = vec![0u8; 1024];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    // Check binary formats first (they have magic numbers)

    // Shapefile magic number
    if buffer.len() >= 4 && buffer[0..4] == [0x00, 0x00, 0x27, 0x0a] {
        return Ok(Format::Shapefile);
    }

    // TIFF magic number
    if buffer.len() >= 4 &&
       (buffer[0..4] == [0x49, 0x49, 0x2a, 0x00] ||
        buffer[0..4] == [0x4d, 0x4d, 0x00, 0x2a]) {
        return Ok(Format::GeoTiff);
    }

    // ZIP magic number (could be KMZ)
    if buffer.len() >= 4 && buffer[0..4] == [0x50, 0x4b, 0x03, 0x04] {
        return detect_zip_format(path);
    }

    // SQLite magic string (GeoPackage)
    if buffer.len() >= 16 && buffer[0..16] == *b"SQLite format 3\0" {
        return Ok(Format::GeoPackage);
    }

    // WKB byte order marker
    if buffer.len() >= 5 && (buffer[0] == 0x00 || buffer[0] == 0x01) {
        // Further validation needed
        return Ok(Format::Wkb);
    }

    // Check text-based formats
    let content = String::from_utf8_lossy(&buffer);
    let content_lower = content.to_lowercase();
    let content_trimmed = content.trim_start();

    // JSON/GeoJSON
    if content_trimmed.starts_with('{') || content_trimmed.starts_with('[') {
        // Try to parse as JSON
        if content.contains("\"type\"") &&
           (content.contains("\"Feature\"") || content.contains("\"FeatureCollection\"") ||
            content.contains("\"Point\"") || content.contains("\"LineString\"") ||
            content.contains("\"Polygon\"")) {
            return Ok(Format::GeoJson);
        }
    }

    // XML-based formats (KML, GML)
    if content_trimmed.starts_with("<?xml") || content_trimmed.starts_with("<") {
        if content_lower.contains("kml") || content_lower.contains("google.com/kml") {
            return Ok(Format::Kml);
        }
        if content_lower.contains("gml") || content_lower.contains("opengis.net/gml") {
            return Ok(Format::Gml);
        }
    }

    // WKT text
    if content_lower.contains("point") || content_lower.contains("linestring") ||
       content_lower.contains("polygon") || content_lower.contains("multipoint") {
        return Ok(Format::Wkt);
    }

    // CSV with spatial columns
    if content_lower.contains("lat") || content_lower.contains("lon") ||
       content_lower.contains("wkt") || content_lower.contains("x,y") {
        return Ok(Format::Csv);
    }

    Err(IoError::UnknownFormat(path.to_path_buf()))
}

/// Detect format for ZIP files (KMZ detection)
fn detect_zip_format(path: &Path) -> Result<Format> {
    use zip::ZipArchive;

    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    // Check if it contains a KML file (KMZ is ZIP with KML inside)
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let name = file.name().to_lowercase();
        if name.ends_with(".kml") {
            return Ok(Format::Kmz);
        }
    }

    // If no KML found, it's just a generic ZIP (not supported)
    Err(IoError::UnsupportedFormat("ZIP archive without KML".to_string()))
}

/// Read the first few lines of a text file
pub fn read_header_lines(path: &Path, num_lines: usize) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut lines = Vec::new();
    for line in reader.lines().take(num_lines) {
        lines.push(line?);
    }

    Ok(lines)
}

/// Peek at the beginning of a file without consuming it
pub fn peek_file(path: &Path, num_bytes: usize) -> Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = vec![0u8; num_bytes];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);
    Ok(buffer)
}

/// Check if a file is likely a text file
pub fn is_text_file(path: &Path) -> Result<bool> {
    let buffer = peek_file(path, 512)?;

    // Check for null bytes (indicates binary)
    if buffer.contains(&0) {
        return Ok(false);
    }

    // Check for high percentage of printable ASCII
    let printable_count = buffer.iter()
        .filter(|&&b| (b >= 32 && b <= 126) || b == b'\n' || b == b'\r' || b == b'\t')
        .count();

    let ratio = printable_count as f64 / buffer.len() as f64;
    Ok(ratio > 0.85)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_extension() {
        assert_eq!(Format::from_extension("geojson"), Some(Format::GeoJson));
        assert_eq!(Format::from_extension("shp"), Some(Format::Shapefile));
        assert_eq!(Format::from_extension("kml"), Some(Format::Kml));
        assert_eq!(Format::from_extension("tif"), Some(Format::GeoTiff));
        assert_eq!(Format::from_extension("gpkg"), Some(Format::GeoPackage));
    }

    #[test]
    fn test_mime_types() {
        assert_eq!(Format::GeoJson.mime_type(), "application/geo+json");
        assert_eq!(Format::Kml.mime_type(), "application/vnd.google-earth.kml+xml");
        assert_eq!(Format::GeoTiff.mime_type(), "image/tiff");
    }
}
