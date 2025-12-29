//! Windowed processing for large images

use crate::error::{ImageryError, Result};
use crate::MultiBandImage;

/// Window definition
#[derive(Debug, Clone, Copy)]
pub struct Window {
    /// X offset
    pub x: u32,
    /// Y offset
    pub y: u32,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
}

impl Window {
    /// Create a new window
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    /// Get window area
    pub fn area(&self) -> usize {
        (self.width * self.height) as usize
    }

    /// Check if window is valid for given image dimensions
    pub fn is_valid(&self, image_width: u32, image_height: u32) -> bool {
        self.x + self.width <= image_width && self.y + self.height <= image_height
    }

    /// Get overlap with another window
    pub fn overlap(&self, other: &Window) -> Option<Window> {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);

        if x1 < x2 && y1 < y2 {
            Some(Window::new(x1, y1, x2 - x1, y2 - y1))
        } else {
            None
        }
    }
}

/// Windowed processor for memory-efficient operations
pub struct WindowedProcessor {
    window_width: u32,
    window_height: u32,
    overlap: u32,
}

impl WindowedProcessor {
    /// Create a new windowed processor
    pub fn new(window_width: u32, window_height: u32) -> Self {
        Self {
            window_width,
            window_height,
            overlap: 0,
        }
    }

    /// Set overlap between windows
    pub fn with_overlap(mut self, overlap: u32) -> Self {
        self.overlap = overlap;
        self
    }

    /// Generate windows for an image
    pub fn generate_windows(&self, image_width: u32, image_height: u32) -> Vec<Window> {
        let mut windows = Vec::new();

        let step_x = self.window_width - self.overlap;
        let step_y = self.window_height - self.overlap;

        let mut y = 0;
        while y < image_height {
            let mut x = 0;
            while x < image_width {
                let width = self.window_width.min(image_width - x);
                let height = self.window_height.min(image_height - y);

                windows.push(Window::new(x, y, width, height));

                x += step_x;
                if x >= image_width {
                    break;
                }
            }

            y += step_y;
            if y >= image_height {
                break;
            }
        }

        windows
    }

    /// Process image in windows
    pub fn process<F>(
        &self,
        image: &MultiBandImage,
        mut processor: F,
    ) -> Result<MultiBandImage>
    where
        F: FnMut(&MultiBandImage, &Window) -> Result<MultiBandImage>,
    {
        let windows = self.generate_windows(image.metadata.width, image.metadata.height);

        // Create output image
        let mut output = MultiBandImage::new(image.metadata.clone(), image.data_type);

        // Process each window
        for window in windows {
            // Extract window from input
            let window_image = self.extract_window(image, &window)?;

            // Process window
            let processed = processor(&window_image, &window)?;

            // Merge back into output
            self.merge_window(&mut output, &processed, &window)?;
        }

        Ok(output)
    }

    /// Extract a window from an image
    fn extract_window(
        &self,
        image: &MultiBandImage,
        window: &Window,
    ) -> Result<MultiBandImage> {
        if !window.is_valid(image.metadata.width, image.metadata.height) {
            return Err(ImageryError::InvalidDimensions(
                "Window exceeds image bounds".to_string()
            ));
        }

        let mut window_meta = image.metadata.clone();
        window_meta.width = window.width;
        window_meta.height = window.height;

        let mut window_image = MultiBandImage::new(window_meta, image.data_type);

        // Copy data for each band
        for (band_idx, band) in image.bands.iter().enumerate() {
            for wy in 0..window.height {
                for wx in 0..window.width {
                    let src_idx = ((window.y + wy) * image.metadata.width + (window.x + wx)) as usize;
                    let dst_idx = (wy * window.width + wx) as usize;

                    window_image.bands[band_idx][dst_idx] = band[src_idx];
                }
            }
        }

        Ok(window_image)
    }

    /// Merge a window back into an image
    fn merge_window(
        &self,
        output: &mut MultiBandImage,
        window_data: &MultiBandImage,
        window: &Window,
    ) -> Result<()> {
        if !window.is_valid(output.metadata.width, output.metadata.height) {
            return Err(ImageryError::InvalidDimensions(
                "Window exceeds image bounds".to_string()
            ));
        }

        // Copy data for each band
        for (band_idx, band) in window_data.bands.iter().enumerate() {
            for wy in 0..window.height {
                for wx in 0..window.width {
                    let src_idx = (wy * window.width + wx) as usize;
                    let dst_idx = ((window.y + wy) * output.metadata.width + (window.x + wx)) as usize;

                    output.bands[band_idx][dst_idx] = band[src_idx];
                }
            }
        }

        Ok(())
    }
}

impl Default for WindowedProcessor {
    fn default() -> Self {
        Self::new(512, 512)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_creation() {
        let window = Window::new(0, 0, 512, 512);
        assert_eq!(window.area(), 262144);
        assert!(window.is_valid(1024, 1024));
        assert!(!window.is_valid(256, 256));
    }

    #[test]
    fn test_window_overlap() {
        let w1 = Window::new(0, 0, 100, 100);
        let w2 = Window::new(50, 50, 100, 100);
        let w3 = Window::new(200, 200, 100, 100);

        assert!(w1.overlap(&w2).is_some());
        assert!(w1.overlap(&w3).is_none());

        let overlap = w1.overlap(&w2).unwrap();
        assert_eq!(overlap.x, 50);
        assert_eq!(overlap.y, 50);
        assert_eq!(overlap.width, 50);
        assert_eq!(overlap.height, 50);
    }

    #[test]
    fn test_window_generation() {
        let processor = WindowedProcessor::new(512, 512);
        let windows = processor.generate_windows(1024, 768);

        assert!(!windows.is_empty());
        // Should have at least 4 windows for 1024x768 with 512x512 windows
        assert!(windows.len() >= 4);
    }
}
