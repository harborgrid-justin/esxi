//! Parallel processing for multi-core systems

use crate::error::Result;
use crate::MultiBandImage;
use crate::streaming::window::{Window, WindowedProcessor};
use rayon::prelude::*;

/// Parallel processor
pub struct ParallelProcessor {
    window_processor: WindowedProcessor,
    num_threads: Option<usize>,
}

impl ParallelProcessor {
    /// Create a new parallel processor
    pub fn new(window_width: u32, window_height: u32) -> Self {
        Self {
            window_processor: WindowedProcessor::new(window_width, window_height),
            num_threads: None,
        }
    }

    /// Set number of threads
    pub fn with_threads(mut self, threads: usize) -> Self {
        self.num_threads = Some(threads);
        self
    }

    /// Set window overlap
    pub fn with_overlap(mut self, overlap: u32) -> Self {
        self.window_processor = self.window_processor.with_overlap(overlap);
        self
    }

    /// Process image in parallel
    pub fn process<F>(
        &self,
        image: &MultiBandImage,
        processor: F,
    ) -> Result<MultiBandImage>
    where
        F: Fn(&MultiBandImage, &Window) -> Result<MultiBandImage> + Sync + Send,
    {
        // Generate windows
        let windows = self.window_processor.generate_windows(
            image.metadata.width,
            image.metadata.height,
        );

        // Create output image
        let mut output = MultiBandImage::new(image.metadata.clone(), image.data_type);

        // Process windows in parallel
        let results: Vec<(Window, MultiBandImage)> = windows
            .par_iter()
            .map(|window| {
                // Extract window
                let window_image = self.extract_window(image, window)?;

                // Process window
                let processed = processor(&window_image, window)?;

                Ok::<_, crate::error::ImageryError>((*window, processed))
            })
            .collect::<Result<Vec<_>>>()?;

        // Merge results
        for (window, window_data) in results {
            self.merge_window(&mut output, &window_data, &window)?;
        }

        Ok(output)
    }

    /// Process each band in parallel
    pub fn process_bands<F>(
        image: &MultiBandImage,
        processor: F,
    ) -> Result<MultiBandImage>
    where
        F: Fn(&[f32]) -> Vec<f32> + Sync + Send,
    {
        let processed_bands: Vec<Vec<f32>> = image.bands
            .par_iter()
            .map(|band| processor(band))
            .collect();

        let mut output = MultiBandImage::new(image.metadata.clone(), image.data_type);
        output.bands = processed_bands;

        Ok(output)
    }

    /// Process pixels in parallel (across all bands)
    pub fn process_pixels<F>(
        image: &MultiBandImage,
        processor: F,
    ) -> Result<MultiBandImage>
    where
        F: Fn(&[f32]) -> Vec<f32> + Sync + Send,
    {
        let size = (image.metadata.width * image.metadata.height) as usize;
        let num_bands = image.bands.len();

        // Process pixels in parallel
        let processed_pixels: Vec<Vec<f32>> = (0..size)
            .into_par_iter()
            .map(|pixel_idx| {
                let mut pixel_values = Vec::with_capacity(num_bands);
                for band in &image.bands {
                    pixel_values.push(band[pixel_idx]);
                }
                processor(&pixel_values)
            })
            .collect();

        // Reorganize into bands
        let mut output = MultiBandImage::new(image.metadata.clone(), image.data_type);

        for pixel_idx in 0..size {
            for (band_idx, value) in processed_pixels[pixel_idx].iter().enumerate() {
                if band_idx < output.bands.len() {
                    output.bands[band_idx][pixel_idx] = *value;
                }
            }
        }

        Ok(output)
    }

    /// Extract window (same as WindowedProcessor)
    fn extract_window(
        &self,
        image: &MultiBandImage,
        window: &Window,
    ) -> Result<MultiBandImage> {
        use crate::error::ImageryError;

        if !window.is_valid(image.metadata.width, image.metadata.height) {
            return Err(ImageryError::InvalidDimensions(
                "Window exceeds image bounds".to_string()
            ));
        }

        let mut window_meta = image.metadata.clone();
        window_meta.width = window.width;
        window_meta.height = window.height;

        let mut window_image = MultiBandImage::new(window_meta, image.data_type);

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

    /// Merge window (same as WindowedProcessor)
    fn merge_window(
        &self,
        output: &mut MultiBandImage,
        window_data: &MultiBandImage,
        window: &Window,
    ) -> Result<()> {
        use crate::error::ImageryError;

        if !window.is_valid(output.metadata.width, output.metadata.height) {
            return Err(ImageryError::InvalidDimensions(
                "Window exceeds image bounds".to_string()
            ));
        }

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

impl Default for ParallelProcessor {
    fn default() -> Self {
        Self::new(512, 512)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ImageMetadata, DataType};

    #[test]
    fn test_parallel_band_processing() {
        let metadata = ImageMetadata {
            width: 100,
            height: 100,
            bands: 3,
            bits_per_sample: 8,
            geo_transform: None,
            crs: None,
            no_data: None,
            band_names: vec!["R".to_string(), "G".to_string(), "B".to_string()],
        };

        let mut image = MultiBandImage::new(metadata, DataType::UInt8);
        for band in &mut image.bands {
            band.fill(100.0);
        }

        let result = ParallelProcessor::process_bands(&image, |band| {
            band.iter().map(|v| v * 2.0).collect()
        });

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.bands[0][0], 200.0);
    }
}
