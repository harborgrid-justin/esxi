//! Image processing operations
//!
//! Advanced processing for satellite and aerial imagery including:
//! - Radiometric corrections
//! - Atmospheric corrections
//! - Pan-sharpening
//! - Orthorectification
//! - Mosaicking

pub mod radiometric;
pub mod atmospheric;
pub mod pansharpening;
pub mod orthorectification;
pub mod mosaic;

pub use radiometric::RadiometricCorrection;
pub use atmospheric::AtmosphericCorrection;
pub use pansharpening::Pansharpening;
pub use orthorectification::Orthorectification;
pub use mosaic::MosaicBuilder;

use crate::error::Result;
use crate::MultiBandImage;

/// Resampling methods for geometric transformations
#[derive(Debug, Clone, Copy)]
pub enum ResamplingMethod {
    /// Nearest neighbor (fastest, no smoothing)
    NearestNeighbor,
    /// Bilinear interpolation
    Bilinear,
    /// Cubic convolution (highest quality)
    Cubic,
    /// Lanczos resampling
    Lanczos,
}

/// Image processing pipeline
pub struct ProcessingPipeline {
    steps: Vec<Box<dyn ProcessingStep>>,
}

impl ProcessingPipeline {
    /// Create a new processing pipeline
    pub fn new() -> Self {
        Self { steps: vec![] }
    }

    /// Add a processing step
    pub fn add_step(&mut self, step: Box<dyn ProcessingStep>) -> &mut Self {
        self.steps.push(step);
        self
    }

    /// Execute the pipeline
    pub fn execute(&self, image: &mut MultiBandImage) -> Result<()> {
        for step in &self.steps {
            step.process(image)?;
        }
        Ok(())
    }
}

impl Default for ProcessingPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for processing steps
pub trait ProcessingStep: Send + Sync {
    /// Process an image in-place
    fn process(&self, image: &mut MultiBandImage) -> Result<()>;

    /// Get step name
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = ProcessingPipeline::new();
        assert_eq!(pipeline.steps.len(), 0);
    }
}
