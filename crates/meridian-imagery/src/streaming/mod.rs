//! Streaming and parallel processing

pub mod window;
pub mod parallel;

pub use window::{WindowedProcessor, Window};
pub use parallel::ParallelProcessor;

use crate::error::Result;
use crate::MultiBandImage;

/// Streaming reader interface
pub trait StreamingReader {
    /// Read a window from the image
    fn read_window(&mut self, window: &Window) -> Result<MultiBandImage>;

    /// Get image dimensions
    fn dimensions(&self) -> (u32, u32);

    /// Get number of bands
    fn bands(&self) -> usize;
}

/// Streaming writer interface
pub trait StreamingWriter {
    /// Write a window to the image
    fn write_window(&mut self, window: &Window, data: &MultiBandImage) -> Result<()>;

    /// Finalize writing
    fn finalize(&mut self) -> Result<()>;
}
