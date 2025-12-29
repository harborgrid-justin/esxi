//! Procedural building generation

use crate::{Error, Result, Vertex};
use super::{BuildingModel, ExtrusionParams, FootprintExtruder, RoofType};
use glam::{Vec2, Vec3};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Building architectural style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingStyle {
    /// Modern glass and steel
    Modern,
    /// Classical architecture
    Classical,
    /// Industrial warehouse style
    Industrial,
    /// Residential house
    Residential,
    /// Art Deco
    ArtDeco,
}

/// Procedurally generated building
pub struct ProceduralBuilding {
    /// Building geometry
    model: BuildingModel,

    /// Building style
    style: BuildingStyle,

    /// Building parameters
    params: BuildingParams,
}

impl ProceduralBuilding {
    /// Get the building model
    pub fn model(&self) -> &BuildingModel {
        &self.model
    }

    /// Get building style
    pub fn style(&self) -> BuildingStyle {
        self.style
    }

    /// Get building parameters
    pub fn params(&self) -> &BuildingParams {
        &self.params
    }
}

impl From<ProceduralBuilding> for BuildingModel {
    fn from(building: ProceduralBuilding) -> Self {
        building.model
    }
}

/// Building generation parameters
#[derive(Debug, Clone)]
pub struct BuildingParams {
    /// Base footprint
    pub footprint: Vec<Vec2>,

    /// Total height
    pub height: f32,

    /// Number of floors
    pub floors: u32,

    /// Floor height
    pub floor_height: f32,

    /// Window configuration
    pub windows: WindowConfig,

    /// Roof configuration
    pub roof: RoofConfig,

    /// Random seed
    pub seed: u64,
}

impl Default for BuildingParams {
    fn default() -> Self {
        Self {
            footprint: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(10.0, 0.0),
                Vec2::new(10.0, 10.0),
                Vec2::new(0.0, 10.0),
            ],
            height: 30.0,
            floors: 10,
            floor_height: 3.0,
            windows: WindowConfig::default(),
            roof: RoofConfig::default(),
            seed: 0,
        }
    }
}

/// Window configuration
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Window width
    pub width: f32,

    /// Window height
    pub height: f32,

    /// Horizontal spacing
    pub spacing_h: f32,

    /// Vertical spacing (between floors)
    pub spacing_v: f32,

    /// Window style
    pub style: WindowStyle,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1.5,
            height: 2.0,
            spacing_h: 0.5,
            spacing_v: 1.0,
            style: WindowStyle::Rectangular,
        }
    }
}

/// Window style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowStyle {
    /// Simple rectangular windows
    Rectangular,
    /// Arched windows
    Arched,
    /// Bay windows
    Bay,
    /// Floor-to-ceiling glass
    FloorToCeiling,
}

/// Roof configuration
#[derive(Debug, Clone)]
pub struct RoofConfig {
    /// Roof type
    pub roof_type: RoofType,

    /// Roof height (for pitched roofs)
    pub height: f32,

    /// Has parapet
    pub parapet: bool,

    /// Parapet height
    pub parapet_height: f32,
}

impl Default for RoofConfig {
    fn default() -> Self {
        Self {
            roof_type: RoofType::Flat,
            height: 0.0,
            parapet: true,
            parapet_height: 1.0,
        }
    }
}

/// Procedural building generator
pub struct BuildingGenerator;

impl BuildingGenerator {
    /// Generate a procedural building
    pub fn generate(
        footprint: &[Vec2],
        height: f32,
        style: BuildingStyle,
    ) -> Result<ProceduralBuilding> {
        let params = BuildingParams {
            footprint: footprint.to_vec(),
            height,
            floors: (height / 3.0) as u32,
            floor_height: 3.0,
            windows: Self::windows_for_style(style),
            roof: Self::roof_for_style(style),
            seed: 0,
        };

        Self::generate_with_params(params, style)
    }

    /// Generate with detailed parameters
    pub fn generate_with_params(
        params: BuildingParams,
        style: BuildingStyle,
    ) -> Result<ProceduralBuilding> {
        // Create basic extrusion
        let extrusion_params = ExtrusionParams {
            height: params.height,
            floor_height: params.floor_height,
            roof_type: params.roof.roof_type,
            roof_height: params.roof.height,
            generate_uvs: true,
            wall_inset: 0.0,
        };

        let extruder = FootprintExtruder::new();
        let mut model = extruder.extrude(&params.footprint, extrusion_params)?;

        // Add style-specific details
        match style {
            BuildingStyle::Modern => {
                // Add glass facades, clean lines
            }
            BuildingStyle::Classical => {
                // Add columns, cornices
            }
            BuildingStyle::Industrial => {
                // Add industrial details
            }
            BuildingStyle::Residential => {
                // Add residential features
            }
            BuildingStyle::ArtDeco => {
                // Add Art Deco ornamentation
            }
        }

        Ok(ProceduralBuilding {
            model,
            style,
            params,
        })
    }

    /// Generate a random building
    pub fn random(seed: u64, min_height: f32, max_height: f32) -> Result<ProceduralBuilding> {
        let mut rng = StdRng::seed_from_u64(seed);

        // Random footprint
        let width = rng.gen_range(10.0..30.0);
        let depth = rng.gen_range(10.0..30.0);

        let footprint = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(width, 0.0),
            Vec2::new(width, depth),
            Vec2::new(0.0, depth),
        ];

        // Random height
        let height = rng.gen_range(min_height..max_height);

        // Random style
        let styles = [
            BuildingStyle::Modern,
            BuildingStyle::Classical,
            BuildingStyle::Industrial,
            BuildingStyle::Residential,
            BuildingStyle::ArtDeco,
        ];
        let style = styles[rng.gen_range(0..styles.len())];

        Self::generate(&footprint, height, style)
    }

    /// Get window configuration for a building style
    fn windows_for_style(style: BuildingStyle) -> WindowConfig {
        match style {
            BuildingStyle::Modern => WindowConfig {
                width: 2.0,
                height: 2.5,
                spacing_h: 0.3,
                spacing_v: 0.5,
                style: WindowStyle::FloorToCeiling,
            },
            BuildingStyle::Classical => WindowConfig {
                width: 1.2,
                height: 2.0,
                spacing_h: 0.8,
                spacing_v: 1.0,
                style: WindowStyle::Arched,
            },
            BuildingStyle::Industrial => WindowConfig {
                width: 1.0,
                height: 1.5,
                spacing_h: 1.0,
                spacing_v: 1.5,
                style: WindowStyle::Rectangular,
            },
            BuildingStyle::Residential => WindowConfig {
                width: 1.2,
                height: 1.5,
                spacing_h: 0.5,
                spacing_v: 1.0,
                style: WindowStyle::Bay,
            },
            BuildingStyle::ArtDeco => WindowConfig {
                width: 1.5,
                height: 2.0,
                spacing_h: 0.5,
                spacing_v: 0.8,
                style: WindowStyle::Rectangular,
            },
        }
    }

    /// Get roof configuration for a building style
    fn roof_for_style(style: BuildingStyle) -> RoofConfig {
        match style {
            BuildingStyle::Modern => RoofConfig {
                roof_type: RoofType::Flat,
                height: 0.0,
                parapet: true,
                parapet_height: 1.2,
            },
            BuildingStyle::Classical => RoofConfig {
                roof_type: RoofType::Hipped,
                height: 4.0,
                parapet: false,
                parapet_height: 0.0,
            },
            BuildingStyle::Industrial => RoofConfig {
                roof_type: RoofType::Flat,
                height: 0.0,
                parapet: false,
                parapet_height: 0.0,
            },
            BuildingStyle::Residential => RoofConfig {
                roof_type: RoofType::Gabled,
                height: 3.0,
                parapet: false,
                parapet_height: 0.0,
            },
            BuildingStyle::ArtDeco => RoofConfig {
                roof_type: RoofType::Flat,
                height: 0.0,
                parapet: true,
                parapet_height: 2.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_procedural_generation() {
        let footprint = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(15.0, 0.0),
            Vec2::new(15.0, 15.0),
            Vec2::new(0.0, 15.0),
        ];

        let building = BuildingGenerator::generate(
            &footprint,
            30.0,
            BuildingStyle::Modern,
        ).unwrap();

        assert_eq!(building.style(), BuildingStyle::Modern);
        assert!(building.model().vertex_count() > 0);
    }

    #[test]
    fn test_random_generation() {
        let building = BuildingGenerator::random(42, 20.0, 50.0).unwrap();

        assert!(building.model().vertex_count() > 0);
        assert!(building.params().height >= 20.0 && building.params().height <= 50.0);
    }

    #[test]
    fn test_all_styles() {
        let footprint = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];

        for style in [
            BuildingStyle::Modern,
            BuildingStyle::Classical,
            BuildingStyle::Industrial,
            BuildingStyle::Residential,
            BuildingStyle::ArtDeco,
        ] {
            let building = BuildingGenerator::generate(&footprint, 25.0, style).unwrap();
            assert_eq!(building.style(), style);
        }
    }
}
