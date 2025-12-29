//! Object detection and change analysis

pub mod change;
pub mod segmentation;

pub use change::ChangeDetection;
pub use segmentation::ImageSegmentation;

use crate::error::Result;
use crate::MultiBandImage;

/// Detection result
#[derive(Debug, Clone)]
pub struct DetectionResult {
    /// Detected objects or changes
    pub mask: Vec<u8>,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
    /// Detection metadata
    pub metadata: DetectionMetadata,
}

/// Detection metadata
#[derive(Debug, Clone, Default)]
pub struct DetectionMetadata {
    /// Algorithm used
    pub algorithm: String,
    /// Confidence threshold
    pub threshold: f32,
    /// Number of detections
    pub count: usize,
}

/// Object detector trait
pub trait ObjectDetector {
    /// Detect objects in an image
    fn detect(&self, image: &MultiBandImage) -> Result<DetectionResult>;
}
