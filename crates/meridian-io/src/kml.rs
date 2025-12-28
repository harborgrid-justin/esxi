//! KML and KMZ file support

use crate::error::{IoError, Result};
use crate::traits::{Feature, FeatureCollection, FeatureStream, Metadata, Reader, Writer};
use futures::stream;
use geo_types::{Coord, Geometry, LineString, Point, Polygon};
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader as XmlReader, Writer as XmlWriter};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use zip::ZipArchive;

/// KML reader
pub struct KmlReader;

impl KmlReader {
    /// Create a new KML reader
    pub fn new() -> Self {
        Self
    }

    /// Parse KML content
    fn parse_kml(content: &str) -> Result<FeatureCollection> {
        let mut reader = XmlReader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut features = Vec::new();
        let mut buf = Vec::new();

        let mut in_placemark = false;
        let mut current_name = None;
        let mut current_description = None;
        let mut current_geometry = None;
        let mut current_properties = HashMap::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"Placemark" => {
                            in_placemark = true;
                            current_name = None;
                            current_description = None;
                            current_geometry = None;
                            current_properties = HashMap::new();
                        }
                        b"Point" if in_placemark => {
                            current_geometry = Some(Self::parse_point(&mut reader)?);
                        }
                        b"LineString" if in_placemark => {
                            current_geometry = Some(Self::parse_linestring(&mut reader)?);
                        }
                        b"Polygon" if in_placemark => {
                            current_geometry = Some(Self::parse_polygon(&mut reader)?);
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.name().as_ref() == b"Placemark" && in_placemark {
                        // Create feature from collected data
                        let mut props = current_properties.clone();
                        if let Some(name) = current_name.take() {
                            props.insert("name".to_string(), Value::String(name));
                        }
                        if let Some(desc) = current_description.take() {
                            props.insert("description".to_string(), Value::String(desc));
                        }

                        features.push(Feature {
                            id: None,
                            geometry: current_geometry.take(),
                            properties: props,
                            crs: None,
                        });

                        in_placemark = false;
                    }
                }
                Ok(Event::Text(_e)) if in_placemark => {
                    // This would need more sophisticated parsing to extract name/description
                    // For now, we'll use a simplified approach
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(IoError::Xml(format!("XML parse error: {}", e))),
                _ => {}
            }

            buf.clear();
        }

        Ok(FeatureCollection::from_features(features))
    }

    /// Parse KML Point
    fn parse_point(reader: &mut XmlReader<&[u8]>) -> Result<Geometry<f64>> {
        let mut buf = Vec::new();
        let mut coords_text = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"coordinates" => {
                    // Next event should be text
                }
                Ok(Event::Text(e)) => {
                    coords_text.push_str(&e.unescape().unwrap_or_default());
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"Point" => break,
                Ok(Event::Eof) => break,
                Err(e) => return Err(IoError::Xml(format!("XML parse error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        let coords = Self::parse_coordinates(&coords_text)?;
        if let Some(coord) = coords.first() {
            Ok(Geometry::Point(Point::new(coord.x, coord.y)))
        } else {
            Err(IoError::InvalidGeometry("Empty point coordinates".to_string()))
        }
    }

    /// Parse KML LineString
    fn parse_linestring(reader: &mut XmlReader<&[u8]>) -> Result<Geometry<f64>> {
        let mut buf = Vec::new();
        let mut coords_text = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Text(e)) => {
                    coords_text.push_str(&e.unescape().unwrap_or_default());
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"LineString" => break,
                Ok(Event::Eof) => break,
                Err(e) => return Err(IoError::Xml(format!("XML parse error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        let coords = Self::parse_coordinates(&coords_text)?;
        Ok(Geometry::LineString(LineString::from(coords)))
    }

    /// Parse KML Polygon
    fn parse_polygon(reader: &mut XmlReader<&[u8]>) -> Result<Geometry<f64>> {
        let mut buf = Vec::new();
        let mut coords_text = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Text(e)) => {
                    coords_text.push_str(&e.unescape().unwrap_or_default());
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"Polygon" => break,
                Ok(Event::Eof) => break,
                Err(e) => return Err(IoError::Xml(format!("XML parse error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        let coords = Self::parse_coordinates(&coords_text)?;
        let exterior = LineString::from(coords);
        Ok(Geometry::Polygon(Polygon::new(exterior, vec![])))
    }

    /// Parse KML coordinate string (lon,lat,alt format)
    fn parse_coordinates(coords_str: &str) -> Result<Vec<Coord<f64>>> {
        let mut coords = Vec::new();

        for point_str in coords_str.split_whitespace() {
            let parts: Vec<&str> = point_str.split(',').collect();
            if parts.len() >= 2 {
                let lon = parts[0].trim().parse::<f64>()
                    .map_err(|_| IoError::Kml(format!("Invalid longitude: {}", parts[0])))?;
                let lat = parts[1].trim().parse::<f64>()
                    .map_err(|_| IoError::Kml(format!("Invalid latitude: {}", parts[1])))?;

                coords.push(Coord { x: lon, y: lat });
            }
        }

        Ok(coords)
    }
}

impl Default for KmlReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader for KmlReader {
    fn read(&self, path: &Path) -> Result<FeatureCollection> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        Self::parse_kml(&content)
    }

    fn read_stream(&self, path: &Path) -> Result<FeatureStream> {
        let collection = self.read(path)?;
        let stream = stream::iter(collection.features.into_iter().map(Ok));
        Ok(Box::pin(stream))
    }

    fn read_crs(&self, _path: &Path) -> Result<Option<String>> {
        // KML always uses WGS84
        Ok(Some("EPSG:4326".to_string()))
    }

    fn read_metadata(&self, path: &Path) -> Result<Metadata> {
        let collection = self.read(path)?;

        let mut metadata = Metadata::new();
        metadata.layers.push("default".to_string());
        metadata.feature_counts.insert("default".to_string(), collection.features.len());
        metadata.crs = Some("EPSG:4326".to_string()); // KML always uses WGS84

        Ok(metadata)
    }
}

/// KMZ (compressed KML) reader
pub struct KmzReader;

impl KmzReader {
    /// Create a new KMZ reader
    pub fn new() -> Self {
        Self
    }

    /// Extract and parse KML from KMZ
    fn extract_kml(path: &Path) -> Result<String> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file)?;

        // Find the main KML file (usually doc.kml or *.kml)
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_lowercase();

            if name.ends_with(".kml") {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                return Ok(content);
            }
        }

        Err(IoError::Kml("No KML file found in KMZ archive".to_string()))
    }
}

impl Default for KmzReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader for KmzReader {
    fn read(&self, path: &Path) -> Result<FeatureCollection> {
        let content = Self::extract_kml(path)?;
        KmlReader::parse_kml(&content)
    }

    fn read_stream(&self, path: &Path) -> Result<FeatureStream> {
        let collection = self.read(path)?;
        let stream = stream::iter(collection.features.into_iter().map(Ok));
        Ok(Box::pin(stream))
    }

    fn read_crs(&self, _path: &Path) -> Result<Option<String>> {
        Ok(Some("EPSG:4326".to_string()))
    }

    fn read_metadata(&self, path: &Path) -> Result<Metadata> {
        let collection = self.read(path)?;

        let mut metadata = Metadata::new();
        metadata.layers.push("default".to_string());
        metadata.feature_counts.insert("default".to_string(), collection.features.len());
        metadata.crs = Some("EPSG:4326".to_string());

        Ok(metadata)
    }
}

/// KML writer
pub struct KmlWriter {
    /// Pretty print XML
    pub pretty: bool,
}

impl KmlWriter {
    /// Create a new KML writer
    pub fn new() -> Self {
        Self { pretty: true }
    }

    /// Write geometry as KML
    fn write_geometry<W: Write>(
        writer: &mut XmlWriter<W>,
        geometry: &Geometry<f64>,
    ) -> Result<()> {
        match geometry {
            Geometry::Point(pt) => {
                writer.write_event(Event::Start(BytesStart::new("Point")))?;
                writer.write_event(Event::Start(BytesStart::new("coordinates")))?;
                let coords = format!("{},{}", pt.x(), pt.y());
                writer.write_event(Event::Text(quick_xml::events::BytesText::new(&coords)))?;
                writer.write_event(Event::End(quick_xml::events::BytesEnd::new("coordinates")))?;
                writer.write_event(Event::End(quick_xml::events::BytesEnd::new("Point")))?;
            }
            Geometry::LineString(ls) => {
                writer.write_event(Event::Start(BytesStart::new("LineString")))?;
                writer.write_event(Event::Start(BytesStart::new("coordinates")))?;
                let coords: Vec<String> = ls.coords()
                    .map(|c| format!("{},{}", c.x, c.y))
                    .collect();
                writer.write_event(Event::Text(quick_xml::events::BytesText::new(&coords.join(" "))))?;
                writer.write_event(Event::End(quick_xml::events::BytesEnd::new("coordinates")))?;
                writer.write_event(Event::End(quick_xml::events::BytesEnd::new("LineString")))?;
            }
            Geometry::Polygon(pg) => {
                writer.write_event(Event::Start(BytesStart::new("Polygon")))?;
                writer.write_event(Event::Start(BytesStart::new("outerBoundaryIs")))?;
                writer.write_event(Event::Start(BytesStart::new("LinearRing")))?;
                writer.write_event(Event::Start(BytesStart::new("coordinates")))?;
                let coords: Vec<String> = pg.exterior().coords()
                    .map(|c| format!("{},{}", c.x, c.y))
                    .collect();
                writer.write_event(Event::Text(quick_xml::events::BytesText::new(&coords.join(" "))))?;
                writer.write_event(Event::End(quick_xml::events::BytesEnd::new("coordinates")))?;
                writer.write_event(Event::End(quick_xml::events::BytesEnd::new("LinearRing")))?;
                writer.write_event(Event::End(quick_xml::events::BytesEnd::new("outerBoundaryIs")))?;
                writer.write_event(Event::End(quick_xml::events::BytesEnd::new("Polygon")))?;
            }
            _ => {
                return Err(IoError::UnsupportedFormat(
                    "Geometry type not supported for KML export".to_string()
                ));
            }
        }

        Ok(())
    }
}

impl Default for KmlWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer for KmlWriter {
    fn write(&self, path: &Path, collection: &FeatureCollection) -> Result<()> {
        let file = File::create(path)?;
        let buf_writer = BufWriter::new(file);
        let mut writer = XmlWriter::new(buf_writer);

        // Write KML header
        writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new("1.0", Some("UTF-8"), None)))?;

        let mut kml_start = BytesStart::new("kml");
        kml_start.push_attribute(("xmlns", "http://www.opengis.net/kml/2.2"));
        writer.write_event(Event::Start(kml_start))?;

        writer.write_event(Event::Start(BytesStart::new("Document")))?;

        // Write features as Placemarks
        for feature in &collection.features {
            writer.write_event(Event::Start(BytesStart::new("Placemark")))?;

            // Write name if available
            if let Some(name) = feature.get_string("name") {
                writer.write_event(Event::Start(BytesStart::new("name")))?;
                writer.write_event(Event::Text(quick_xml::events::BytesText::new(name)))?;
                writer.write_event(Event::End(quick_xml::events::BytesEnd::new("name")))?;
            }

            // Write description if available
            if let Some(desc) = feature.get_string("description") {
                writer.write_event(Event::Start(BytesStart::new("description")))?;
                writer.write_event(Event::Text(quick_xml::events::BytesText::new(desc)))?;
                writer.write_event(Event::End(quick_xml::events::BytesEnd::new("description")))?;
            }

            // Write geometry
            if let Some(ref geom) = feature.geometry {
                Self::write_geometry(&mut writer, geom)?;
            }

            writer.write_event(Event::End(quick_xml::events::BytesEnd::new("Placemark")))?;
        }

        writer.write_event(Event::End(quick_xml::events::BytesEnd::new("Document")))?;
        writer.write_event(Event::End(quick_xml::events::BytesEnd::new("kml")))?;

        Ok(())
    }

    fn write_stream<S>(&self, _path: &Path, _stream: S) -> Result<()>
    where
        S: futures::Stream<Item = Result<Feature>> + Send + 'static,
    {
        Err(IoError::UnsupportedFormat("KML streaming write not yet implemented".to_string()))
    }

    fn append(&self, _path: &Path, _collection: &FeatureCollection) -> Result<()> {
        Err(IoError::UnsupportedFormat("KML append not yet implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kml_reader_creation() {
        let _reader = KmlReader::new();
    }

    #[test]
    fn test_parse_coordinates() {
        let coords_str = "-122.0,37.0 -122.1,37.1";
        let coords = KmlReader::parse_coordinates(coords_str).unwrap();
        assert_eq!(coords.len(), 2);
        assert_eq!(coords[0].x, -122.0);
        assert_eq!(coords[0].y, 37.0);
    }
}
