//! Export capabilities for 3D visualizations

pub mod screenshot;
pub mod video;

pub use screenshot::{ScreenshotExporter, ImageFormat};
pub use video::{VideoRecorder, VideoFormat, VideoCodec};

use crate::{Error, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_module() {
        // Basic test
    }
}
