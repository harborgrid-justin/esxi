//! 3D analysis tools for GIS applications

pub mod viewshed;
pub mod shadow;
pub mod flythrough;

pub use viewshed::{ViewshedAnalysis, ViewshedResult};
pub use shadow::{ShadowAnalysis, ShadowMap2D};
pub use flythrough::{FlythroughPath, FlythroughKeyframe, FlythroughAnimator};

use crate::{Camera, Scene, Error, Result};
use glam::Vec3;

/// 3D analysis system
pub struct AnalysisSystem {
    /// Viewshed analyzer
    viewshed: ViewshedAnalysis,

    /// Shadow analyzer
    shadow: ShadowAnalysis,
}

impl AnalysisSystem {
    /// Create a new analysis system
    pub fn new() -> Self {
        Self {
            viewshed: ViewshedAnalysis::new(),
            shadow: ShadowAnalysis::new(),
        }
    }

    /// Perform viewshed analysis
    pub fn analyze_viewshed(
        &mut self,
        scene: &Scene,
        observer_position: Vec3,
        max_distance: f32,
    ) -> Result<ViewshedResult> {
        self.viewshed.analyze(scene, observer_position, max_distance)
    }

    /// Perform shadow analysis
    pub fn analyze_shadows(
        &mut self,
        scene: &Scene,
        sun_direction: Vec3,
        time_start: f32,
        time_end: f32,
        time_step: f32,
    ) -> Result<ShadowMap2D> {
        self.shadow.analyze_time_range(scene, sun_direction, time_start, time_end, time_step)
    }
}

impl Default for AnalysisSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_system() {
        let system = AnalysisSystem::new();
        // Basic creation test
    }
}
