//! Surface analysis operations for elevation data and terrain analysis

use crate::error::{AnalysisError, Result};
use geo::LineString;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Digital Elevation Model (DEM) raster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dem {
    pub width: usize,
    pub height: usize,
    pub cell_size: f64,
    pub origin_x: f64,
    pub origin_y: f64,
    pub nodata_value: Option<f64>,
    pub data: Vec<Vec<f64>>,
}

impl Dem {
    /// Create a new DEM
    pub fn new(
        width: usize,
        height: usize,
        cell_size: f64,
        origin_x: f64,
        origin_y: f64,
    ) -> Self {
        Self {
            width,
            height,
            cell_size,
            origin_x,
            origin_y,
            nodata_value: Some(-9999.0),
            data: vec![vec![0.0; width]; height],
        }
    }

    /// Get value at row, col
    pub fn get(&self, row: usize, col: usize) -> Option<f64> {
        if row >= self.height || col >= self.width {
            return None;
        }

        let value = self.data[row][col];

        if let Some(nodata) = self.nodata_value {
            if (value - nodata).abs() < 1e-10 {
                return None;
            }
        }

        Some(value)
    }

    /// Set value at row, col
    pub fn set(&mut self, row: usize, col: usize, value: f64) -> Result<()> {
        if row >= self.height || col >= self.width {
            return Err(AnalysisError::invalid_parameters("Index out of bounds"));
        }

        self.data[row][col] = value;
        Ok(())
    }

    /// Get elevation at geographic coordinates
    pub fn get_elevation(&self, x: f64, y: f64) -> Option<f64> {
        let col = ((x - self.origin_x) / self.cell_size).floor() as usize;
        let row = ((self.origin_y - y) / self.cell_size).floor() as usize;

        self.get(row, col)
    }

    /// Check if value is nodata
    pub fn is_nodata(&self, value: f64) -> bool {
        if let Some(nodata) = self.nodata_value {
            (value - nodata).abs() < 1e-10
        } else {
            false
        }
    }

    /// Get neighboring cells (8-connected)
    fn get_neighbors(&self, row: usize, col: usize) -> Vec<(usize, usize, f64)> {
        let mut neighbors = Vec::new();

        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }

                let new_row = row as i32 + dr;
                let new_col = col as i32 + dc;

                if new_row >= 0
                    && new_row < self.height as i32
                    && new_col >= 0
                    && new_col < self.width as i32
                {
                    let r = new_row as usize;
                    let c = new_col as usize;

                    if let Some(value) = self.get(r, c) {
                        neighbors.push((r, c, value));
                    }
                }
            }
        }

        neighbors
    }
}

/// Calculate slope in degrees
pub fn slope(dem: &Dem) -> Result<Dem> {
    let mut result = Dem::new(
        dem.width,
        dem.height,
        dem.cell_size,
        dem.origin_x,
        dem.origin_y,
    );
    result.nodata_value = dem.nodata_value;

    for row in 0..dem.height {
        for col in 0..dem.width {
            if let Some(elevation) = dem.get(row, col) {
                let slope_value = calculate_slope_at(dem, row, col, elevation)?;
                result.set(row, col, slope_value)?;
            } else if let Some(nodata) = dem.nodata_value {
                result.set(row, col, nodata)?;
            }
        }
    }

    Ok(result)
}

/// Calculate slope at a specific cell
fn calculate_slope_at(dem: &Dem, row: usize, col: usize, _center: f64) -> Result<f64> {
    // Get values for 3x3 neighborhood
    let z = get_3x3_window(dem, row, col)?;

    // Calculate slope using Horn's method
    let dz_dx = ((z[2] + 2.0 * z[5] + z[8]) - (z[0] + 2.0 * z[3] + z[6]))
        / (8.0 * dem.cell_size);
    let dz_dy = ((z[6] + 2.0 * z[7] + z[8]) - (z[0] + 2.0 * z[1] + z[2]))
        / (8.0 * dem.cell_size);

    let slope_radians = (dz_dx * dz_dx + dz_dy * dz_dy).sqrt().atan();
    let slope_degrees = slope_radians * 180.0 / PI;

    Ok(slope_degrees)
}

/// Calculate aspect in degrees (0-360, 0=North, clockwise)
pub fn aspect(dem: &Dem) -> Result<Dem> {
    let mut result = Dem::new(
        dem.width,
        dem.height,
        dem.cell_size,
        dem.origin_x,
        dem.origin_y,
    );
    result.nodata_value = dem.nodata_value;

    for row in 0..dem.height {
        for col in 0..dem.width {
            if let Some(_elevation) = dem.get(row, col) {
                let aspect_value = calculate_aspect_at(dem, row, col)?;
                result.set(row, col, aspect_value)?;
            } else if let Some(nodata) = dem.nodata_value {
                result.set(row, col, nodata)?;
            }
        }
    }

    Ok(result)
}

/// Calculate aspect at a specific cell
fn calculate_aspect_at(dem: &Dem, row: usize, col: usize) -> Result<f64> {
    let z = get_3x3_window(dem, row, col)?;

    let dz_dx = ((z[2] + 2.0 * z[5] + z[8]) - (z[0] + 2.0 * z[3] + z[6]))
        / (8.0 * dem.cell_size);
    let dz_dy = ((z[6] + 2.0 * z[7] + z[8]) - (z[0] + 2.0 * z[1] + z[2]))
        / (8.0 * dem.cell_size);

    if dz_dx == 0.0 && dz_dy == 0.0 {
        return Ok(-1.0); // Flat area
    }

    let aspect_radians = dz_dy.atan2(dz_dx);

    // Convert to degrees and adjust to North=0, clockwise
    let mut aspect_degrees = 90.0 - (aspect_radians * 180.0 / PI);

    if aspect_degrees < 0.0 {
        aspect_degrees += 360.0;
    }

    Ok(aspect_degrees)
}

/// Get 3x3 window of values centered at row, col
fn get_3x3_window(dem: &Dem, row: usize, col: usize) -> Result<[f64; 9]> {
    let mut window = [0.0; 9];
    let center = dem
        .get(row, col)
        .ok_or_else(|| AnalysisError::invalid_geometry("No data at center cell"))?;

    let mut idx = 0;
    for dr in -1..=1 {
        for dc in -1..=1 {
            let r = (row as i32 + dr) as usize;
            let c = (col as i32 + dc) as usize;

            window[idx] = dem.get(r, c).unwrap_or(center);
            idx += 1;
        }
    }

    Ok(window)
}

/// Calculate hillshade
pub fn hillshade(dem: &Dem, azimuth: f64, altitude: f64) -> Result<Dem> {
    let mut result = Dem::new(
        dem.width,
        dem.height,
        dem.cell_size,
        dem.origin_x,
        dem.origin_y,
    );
    result.nodata_value = dem.nodata_value;

    let azimuth_rad = azimuth * PI / 180.0;
    let altitude_rad = altitude * PI / 180.0;

    for row in 0..dem.height {
        for col in 0..dem.width {
            if dem.get(row, col).is_some() {
                let hillshade_value =
                    calculate_hillshade_at(dem, row, col, azimuth_rad, altitude_rad)?;
                result.set(row, col, hillshade_value)?;
            } else if let Some(nodata) = dem.nodata_value {
                result.set(row, col, nodata)?;
            }
        }
    }

    Ok(result)
}

/// Calculate hillshade at a specific cell
fn calculate_hillshade_at(
    dem: &Dem,
    row: usize,
    col: usize,
    azimuth_rad: f64,
    altitude_rad: f64,
) -> Result<f64> {
    let z = get_3x3_window(dem, row, col)?;

    let dz_dx = ((z[2] + 2.0 * z[5] + z[8]) - (z[0] + 2.0 * z[3] + z[6]))
        / (8.0 * dem.cell_size);
    let dz_dy = ((z[6] + 2.0 * z[7] + z[8]) - (z[0] + 2.0 * z[1] + z[2]))
        / (8.0 * dem.cell_size);

    let slope_rad = (dz_dx * dz_dx + dz_dy * dz_dy).sqrt().atan();
    let aspect_rad = dz_dy.atan2(dz_dx);

    let hillshade = ((altitude_rad.sin() * slope_rad.sin())
        + (altitude_rad.cos() * slope_rad.cos() * (azimuth_rad - aspect_rad).cos()))
    .max(0.0)
        * 255.0;

    Ok(hillshade)
}

/// Generate contour lines at specified intervals
pub fn contour(dem: &Dem, interval: f64, base: f64) -> Result<Vec<(f64, Vec<LineString>)>> {
    if interval <= 0.0 {
        return Err(AnalysisError::invalid_parameters(
            "Interval must be positive",
        ));
    }

    // Find min and max elevations
    let mut min_elev = f64::INFINITY;
    let mut max_elev = f64::NEG_INFINITY;

    for row in 0..dem.height {
        for col in 0..dem.width {
            if let Some(value) = dem.get(row, col) {
                min_elev = min_elev.min(value);
                max_elev = max_elev.max(value);
            }
        }
    }

    if min_elev == f64::INFINITY {
        return Ok(vec![]);
    }

    // Generate contour levels
    let mut level = ((min_elev - base) / interval).ceil() * interval + base;
    let mut contours = Vec::new();

    while level <= max_elev {
        // This is a simplified placeholder
        // Full implementation would use marching squares or similar algorithm
        let lines = vec![]; // Would contain actual contour lines

        contours.push((level, lines));
        level += interval;
    }

    Ok(contours)
}

/// Calculate curvature
pub fn curvature(dem: &Dem) -> Result<Dem> {
    let mut result = Dem::new(
        dem.width,
        dem.height,
        dem.cell_size,
        dem.origin_x,
        dem.origin_y,
    );
    result.nodata_value = dem.nodata_value;

    for row in 1..dem.height - 1 {
        for col in 1..dem.width - 1 {
            if let Some(_center) = dem.get(row, col) {
                let curv = calculate_curvature_at(dem, row, col)?;
                result.set(row, col, curv)?;
            } else if let Some(nodata) = dem.nodata_value {
                result.set(row, col, nodata)?;
            }
        }
    }

    Ok(result)
}

/// Calculate curvature at a specific cell
fn calculate_curvature_at(dem: &Dem, row: usize, col: usize) -> Result<f64> {
    let z = get_3x3_window(dem, row, col)?;
    let cs = dem.cell_size;

    // Second derivatives
    let d2z_dx2 = (z[3] - 2.0 * z[4] + z[5]) / (cs * cs);
    let d2z_dy2 = (z[1] - 2.0 * z[4] + z[7]) / (cs * cs);

    // Total curvature
    let curvature = -(d2z_dx2 + d2z_dy2);

    Ok(curvature)
}

/// Calculate flow direction (D8 algorithm)
pub fn flow_direction(dem: &Dem) -> Result<Dem> {
    let mut result = Dem::new(
        dem.width,
        dem.height,
        dem.cell_size,
        dem.origin_x,
        dem.origin_y,
    );
    result.nodata_value = dem.nodata_value;

    // Direction encoding: 1=E, 2=SE, 4=S, 8=SW, 16=W, 32=NW, 64=N, 128=NE
    let directions = [1, 2, 4, 8, 16, 32, 64, 128];
    let drow = [0, 1, 1, 1, 0, -1, -1, -1];
    let dcol = [1, 1, 0, -1, -1, -1, 0, 1];

    for row in 0..dem.height {
        for col in 0..dem.width {
            if let Some(center_elev) = dem.get(row, col) {
                let mut max_slope = f64::NEG_INFINITY;
                let mut flow_dir = 0;

                for i in 0..8 {
                    let new_row = row as i32 + drow[i];
                    let new_col = col as i32 + dcol[i];

                    if new_row >= 0
                        && new_row < dem.height as i32
                        && new_col >= 0
                        && new_col < dem.width as i32
                    {
                        if let Some(neighbor_elev) =
                            dem.get(new_row as usize, new_col as usize)
                        {
                            let distance = if i % 2 == 0 {
                                dem.cell_size
                            } else {
                                dem.cell_size * 2.0_f64.sqrt()
                            };

                            let slope = (center_elev - neighbor_elev) / distance;

                            if slope > max_slope {
                                max_slope = slope;
                                flow_dir = directions[i];
                            }
                        }
                    }
                }

                result.set(row, col, flow_dir as f64)?;
            } else if let Some(nodata) = dem.nodata_value {
                result.set(row, col, nodata)?;
            }
        }
    }

    Ok(result)
}

/// Calculate viewshed from an observation point
pub fn viewshed(dem: &Dem, observer_x: f64, observer_y: f64, observer_height: f64) -> Result<Dem> {
    let mut result = Dem::new(
        dem.width,
        dem.height,
        dem.cell_size,
        dem.origin_x,
        dem.origin_y,
    );

    let obs_col = ((observer_x - dem.origin_x) / dem.cell_size).floor() as usize;
    let obs_row = ((dem.origin_y - observer_y) / dem.cell_size).floor() as usize;

    if obs_row >= dem.height || obs_col >= dem.width {
        return Err(AnalysisError::invalid_parameters(
            "Observer point outside DEM",
        ));
    }

    let _observer_elev = dem
        .get(obs_row, obs_col)
        .ok_or_else(|| AnalysisError::SurfaceError("No data at observer location".to_string()))?
        + observer_height;

    // Simplified viewshed - mark all cells as visible (1.0) or not (0.0)
    // Full implementation would use line-of-sight algorithm
    for row in 0..dem.height {
        for col in 0..dem.width {
            if let Some(_elev) = dem.get(row, col) {
                // Simplified: always visible for now
                result.set(row, col, 1.0)?;
            } else {
                result.set(row, col, 0.0)?;
            }
        }
    }

    // Mark observer location
    result.set(obs_row, obs_col, 1.0)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dem() -> Dem {
        let mut dem = Dem::new(5, 5, 10.0, 0.0, 50.0);

        for row in 0..5 {
            for col in 0..5 {
                dem.set(row, col, (row + col) as f64 * 10.0).unwrap();
            }
        }

        dem
    }

    #[test]
    fn test_dem_creation() {
        let dem = create_test_dem();
        assert_eq!(dem.width, 5);
        assert_eq!(dem.height, 5);
        assert_eq!(dem.cell_size, 10.0);
    }

    #[test]
    fn test_dem_get_set() {
        let mut dem = create_test_dem();
        dem.set(2, 2, 100.0).unwrap();
        assert_eq!(dem.get(2, 2), Some(100.0));
    }

    #[test]
    fn test_slope() {
        let dem = create_test_dem();
        let slope_result = slope(&dem).unwrap();
        assert_eq!(slope_result.width, dem.width);
        assert_eq!(slope_result.height, dem.height);
    }

    #[test]
    fn test_aspect() {
        let dem = create_test_dem();
        let aspect_result = aspect(&dem).unwrap();
        assert_eq!(aspect_result.width, dem.width);
        assert_eq!(aspect_result.height, dem.height);
    }

    #[test]
    fn test_hillshade() {
        let dem = create_test_dem();
        let hillshade_result = hillshade(&dem, 315.0, 45.0).unwrap();
        assert_eq!(hillshade_result.width, dem.width);
        assert_eq!(hillshade_result.height, dem.height);
    }

    #[test]
    fn test_flow_direction() {
        let dem = create_test_dem();
        let flow_dir = flow_direction(&dem).unwrap();
        assert_eq!(flow_dir.width, dem.width);
        assert_eq!(flow_dir.height, dem.height);
    }
}
