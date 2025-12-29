//! Coordinate Reference System (CRS) support.
//!
//! This module provides comprehensive support for coordinate reference systems,
//! including common projections (WGS84, Web Mercator, UTM) and custom projections
//! using PROJ strings and EPSG codes.
//!
//! # Examples
//!
//! ```ignore
//! use meridian_core::crs::Crs;
//!
//! // Create common CRS instances
//! let wgs84 = Crs::wgs84();
//! let web_mercator = Crs::web_mercator();
//! let utm_zone_10n = Crs::utm(10, true);
//!
//! // Create from EPSG code
//! let epsg_4326 = Crs::from_epsg(4326)?;
//!
//! // Create custom projection
//! let custom = Crs::from_proj_string("+proj=lcc +lat_0=40 +lon_0=-100")?;
//! ```

use crate::error::{MeridianError, Result};
use proj::Proj;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

/// Represents a Coordinate Reference System.
///
/// A CRS defines how coordinates map to locations on Earth (or other surfaces).
/// This struct supports various ways of defining CRS: EPSG codes, PROJ strings,
/// and predefined common projections.
#[derive(Clone, Serialize, Deserialize)]
pub struct Crs {
    /// EPSG code if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epsg: Option<u32>,

    /// PROJ string representation
    pub proj_string: String,

    /// Human-readable name
    pub name: String,

    /// Cached PROJ transformation object
    #[serde(skip)]
    #[allow(dead_code)]
    proj: Option<Arc<Proj>>,
}

impl Crs {
    /// Creates a WGS84 (EPSG:4326) coordinate reference system.
    ///
    /// WGS84 is the standard geographic coordinate system used by GPS
    /// and is the most common CRS for global spatial data.
    ///
    /// Coordinates are in decimal degrees: (longitude, latitude).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let wgs84 = Crs::wgs84();
    /// assert_eq!(wgs84.epsg, Some(4326));
    /// ```
    pub fn wgs84() -> Self {
        Self {
            epsg: Some(4326),
            proj_string: "EPSG:4326".to_string(),
            name: "WGS 84".to_string(),
            proj: None,
        }
    }

    /// Creates a Web Mercator (EPSG:3857) coordinate reference system.
    ///
    /// Web Mercator is the de facto standard for web mapping applications
    /// (Google Maps, OpenStreetMap, etc.). It's a spherical Mercator projection.
    ///
    /// Coordinates are in meters from the origin (0, 0) at the equator and prime meridian.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let web_mercator = Crs::web_mercator();
    /// assert_eq!(web_mercator.epsg, Some(3857));
    /// ```
    pub fn web_mercator() -> Self {
        Self {
            epsg: Some(3857),
            proj_string: "EPSG:3857".to_string(),
            name: "WGS 84 / Pseudo-Mercator".to_string(),
            proj: None,
        }
    }

    /// Creates a UTM (Universal Transverse Mercator) coordinate reference system.
    ///
    /// UTM divides the Earth into 60 zones, each 6° wide. Each zone uses a
    /// transverse Mercator projection optimized for that region.
    ///
    /// # Arguments
    ///
    /// * `zone` - UTM zone number (1-60)
    /// * `north` - `true` for northern hemisphere, `false` for southern
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let utm_10n = Crs::utm(10, true);  // Northern California
    /// let utm_56s = Crs::utm(56, false); // Eastern Australia
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if zone is not in the range 1-60.
    pub fn utm(zone: u8, north: bool) -> Self {
        assert!((1..=60).contains(&zone), "UTM zone must be between 1 and 60");

        let epsg = if north {
            32600 + zone as u32
        } else {
            32700 + zone as u32
        };

        Self {
            epsg: Some(epsg),
            proj_string: format!("EPSG:{}", epsg),
            name: format!("WGS 84 / UTM zone {}{}", zone, if north { "N" } else { "S" }),
            proj: None,
        }
    }

    /// Creates a CRS from an EPSG code.
    ///
    /// # Arguments
    ///
    /// * `code` - The EPSG code
    ///
    /// # Returns
    ///
    /// A CRS instance or an error if the EPSG code is invalid
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let wgs84 = Crs::from_epsg(4326)?;
    /// let nad83 = Crs::from_epsg(4269)?;
    /// ```
    pub fn from_epsg(code: u32) -> Result<Self> {
        let proj_string = format!("EPSG:{}", code);

        // Validate by attempting to create a Proj object
        Proj::new(&proj_string)
            .map_err(|e| MeridianError::InvalidCrs(format!("Invalid EPSG code {}: {}", code, e)))?;

        Ok(Self {
            epsg: Some(code),
            proj_string,
            name: format!("EPSG:{}", code),
            proj: None,
        })
    }

    /// Creates a CRS from a PROJ string.
    ///
    /// PROJ strings define projections using a key-value parameter format.
    ///
    /// # Arguments
    ///
    /// * `proj_string` - The PROJ string definition
    ///
    /// # Returns
    ///
    /// A CRS instance or an error if the PROJ string is invalid
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let lcc = Crs::from_proj_string(
    ///     "+proj=lcc +lat_1=33 +lat_2=45 +lat_0=39 +lon_0=-96"
    /// )?;
    /// ```
    pub fn from_proj_string(proj_string: impl Into<String>) -> Result<Self> {
        let proj_string = proj_string.into();

        // Validate by attempting to create a Proj object
        Proj::new(&proj_string)
            .map_err(|e| MeridianError::InvalidCrs(format!("Invalid PROJ string: {}", e)))?;

        Ok(Self {
            epsg: None,
            proj_string: proj_string.clone(),
            name: proj_string,
            proj: None,
        })
    }

    /// Gets or creates the PROJ transformation object for this CRS.
    ///
    /// This is used internally for coordinate transformations. The PROJ object
    /// is cached for performance.
    #[allow(dead_code)]
    #[allow(clippy::arc_with_non_send_sync)]
    pub(crate) fn get_proj(&mut self) -> Result<Arc<Proj>> {
        if let Some(ref proj) = self.proj {
            return Ok(Arc::clone(proj));
        }

        let proj = Proj::new(&self.proj_string)
            .map_err(|e| MeridianError::ProjectionError(format!("Failed to create projection: {}", e)))?;

        let arc_proj = Arc::new(proj);
        self.proj = Some(Arc::clone(&arc_proj));
        Ok(arc_proj)
    }

    /// Transforms a coordinate from this CRS to another CRS.
    ///
    /// # Arguments
    ///
    /// * `x` - The x coordinate (longitude or easting)
    /// * `y` - The y coordinate (latitude or northing)
    /// * `target` - The target CRS
    ///
    /// # Returns
    ///
    /// A tuple (x, y) in the target CRS, or an error if transformation fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let wgs84 = Crs::wgs84();
    /// let web_mercator = Crs::web_mercator();
    ///
    /// let (x, y) = wgs84.transform_point(-122.4194, 37.7749, &web_mercator)?;
    /// ```
    pub fn transform_point(&mut self, x: f64, y: f64, target: &Crs) -> Result<(f64, f64)> {
        // If same CRS, no transformation needed
        if self.proj_string == target.proj_string {
            return Ok((x, y));
        }

        let from_to = format!("{} +to {}", self.proj_string, target.proj_string);
        let proj = Proj::new(&from_to)
            .map_err(|e| MeridianError::TransformError(format!("Failed to create transformer: {}", e)))?;

        let result = proj.convert((x, y))
            .map_err(|e| MeridianError::TransformError(format!("Transformation failed: {}", e)))?;

        Ok(result)
    }

    /// Checks if this CRS is geographic (uses lat/lon coordinates).
    ///
    /// # Returns
    ///
    /// `true` if the CRS uses geographic coordinates (degrees)
    pub fn is_geographic(&self) -> bool {
        // Common geographic EPSG codes
        if let Some(epsg) = self.epsg {
            matches!(epsg, 4326 | 4269 | 4258 | 4167)
        } else {
            self.proj_string.contains("+proj=longlat") ||
            self.proj_string.contains("+proj=latlong")
        }
    }

    /// Checks if this CRS is projected (uses planar coordinates).
    ///
    /// # Returns
    ///
    /// `true` if the CRS uses projected coordinates (meters, feet, etc.)
    pub fn is_projected(&self) -> bool {
        !self.is_geographic()
    }
}

impl fmt::Debug for Crs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Crs")
            .field("epsg", &self.epsg)
            .field("proj_string", &self.proj_string)
            .field("name", &self.name)
            .finish()
    }
}

impl fmt::Display for Crs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(epsg) = self.epsg {
            write!(f, "{} (EPSG:{})", self.name, epsg)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl PartialEq for Crs {
    fn eq(&self, other: &Self) -> bool {
        // Compare by EPSG code if both have one
        if let (Some(epsg1), Some(epsg2)) = (self.epsg, other.epsg) {
            return epsg1 == epsg2;
        }
        // Otherwise compare by PROJ string
        self.proj_string == other.proj_string
    }
}

impl Eq for Crs {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wgs84_creation() {
        let wgs84 = Crs::wgs84();
        assert_eq!(wgs84.epsg, Some(4326));
        assert_eq!(wgs84.proj_string, "EPSG:4326");
        assert!(wgs84.is_geographic());
        assert!(!wgs84.is_projected());
    }

    #[test]
    fn test_web_mercator_creation() {
        let web_merc = Crs::web_mercator();
        assert_eq!(web_merc.epsg, Some(3857));
        assert!(!web_merc.is_geographic());
        assert!(web_merc.is_projected());
    }

    #[test]
    fn test_utm_creation() {
        let utm = Crs::utm(10, true);
        assert_eq!(utm.epsg, Some(32610));

        let utm_south = Crs::utm(10, false);
        assert_eq!(utm_south.epsg, Some(32710));
    }

    #[test]
    #[should_panic]
    fn test_utm_invalid_zone() {
        Crs::utm(61, true);
    }

    #[test]
    fn test_from_epsg() {
        let crs = Crs::from_epsg(4326).unwrap();
        assert_eq!(crs.epsg, Some(4326));
    }

    #[test]
    fn test_crs_equality() {
        let wgs84_1 = Crs::wgs84();
        let wgs84_2 = Crs::from_epsg(4326).unwrap();
        assert_eq!(wgs84_1, wgs84_2);
    }
}
