//! High-resolution screenshot export

use crate::{Camera, Scene, RenderContext, Error, Result};
use image::{DynamicImage, ImageBuffer, RgbaImage};
use std::path::Path;
use wgpu::{Device, Queue, Texture};

/// Image format for export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// PNG format
    Png,
    /// JPEG format
    Jpeg,
    /// TIFF format
    Tiff,
    /// BMP format
    Bmp,
}

impl ImageFormat {
    /// Get file extension
    pub fn extension(&self) -> &str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Tiff => "tif",
            Self::Bmp => "bmp",
        }
    }
}

/// Screenshot exporter
pub struct ScreenshotExporter {
    /// Default format
    default_format: ImageFormat,

    /// Default quality (for JPEG)
    jpeg_quality: u8,
}

impl ScreenshotExporter {
    /// Create a new screenshot exporter
    pub fn new() -> Self {
        Self {
            default_format: ImageFormat::Png,
            jpeg_quality: 95,
        }
    }

    /// Capture a screenshot at current resolution
    pub async fn capture(
        &self,
        device: &Device,
        queue: &Queue,
        scene: &Scene,
        camera: &Camera,
        width: u32,
        height: u32,
    ) -> Result<RgbaImage> {
        // Create offscreen texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Screenshot Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        // Render scene to texture
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Screenshot Encoder"),
        });

        scene.render(device, queue, &mut encoder, &view)?;

        queue.submit(std::iter::once(encoder.finish()));

        // Copy texture to buffer
        let buffer = self.copy_texture_to_buffer(device, queue, &texture, width, height).await?;

        // Convert buffer to image
        let image = RgbaImage::from_raw(width, height, buffer)
            .ok_or_else(|| Error::export("Failed to create image from buffer"))?;

        Ok(image)
    }

    /// Capture high-resolution screenshot
    pub async fn capture_high_res(
        &self,
        device: &Device,
        queue: &Queue,
        scene: &Scene,
        camera: &Camera,
        width: u32,
        height: u32,
        scale: u32,
    ) -> Result<RgbaImage> {
        self.capture(device, queue, scene, camera, width * scale, height * scale).await
    }

    /// Save screenshot to file
    pub async fn save(
        &self,
        device: &Device,
        queue: &Queue,
        scene: &Scene,
        camera: &Camera,
        width: u32,
        height: u32,
        path: impl AsRef<Path>,
        format: Option<ImageFormat>,
    ) -> Result<()> {
        let image = self.capture(device, queue, scene, camera, width, height).await?;

        let format = format.unwrap_or(self.default_format);

        match format {
            ImageFormat::Png => {
                image.save_with_format(path, image::ImageFormat::Png)?;
            }
            ImageFormat::Jpeg => {
                let mut output = std::fs::File::create(path)?;
                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                    &mut output,
                    self.jpeg_quality,
                );
                encoder.encode(
                    &image,
                    width,
                    height,
                    image::ColorType::Rgba8,
                )?;
            }
            ImageFormat::Tiff => {
                image.save_with_format(path, image::ImageFormat::Tiff)?;
            }
            ImageFormat::Bmp => {
                image.save_with_format(path, image::ImageFormat::Bmp)?;
            }
        }

        Ok(())
    }

    /// Copy texture to CPU buffer
    async fn copy_texture_to_buffer(
        &self,
        device: &Device,
        queue: &Queue,
        texture: &Texture,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>> {
        let bytes_per_pixel = 4; // RGBA8
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = ((unpadded_bytes_per_row + align - 1) / align) * align;

        let buffer_size = (padded_bytes_per_row * height) as u64;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Screenshot Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Screenshot Copy Encoder"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(std::iter::once(encoder.finish()));

        // Map buffer and read data
        let buffer_slice = buffer.slice(..);
        let (tx, rx) = futures::channel::oneshot::channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        device.poll(wgpu::Maintain::Wait);
        rx.await.unwrap()
            .map_err(|e| crate::Error::CaptureError(format!("Buffer mapping failed: {:?}", e)))?;

        let data = buffer_slice.get_mapped_range();

        // Remove padding if necessary
        let mut result = Vec::with_capacity((unpadded_bytes_per_row * height) as usize);

        for row in 0..height {
            let start = (row * padded_bytes_per_row) as usize;
            let end = start + unpadded_bytes_per_row as usize;
            result.extend_from_slice(&data[start..end]);
        }

        drop(data);
        buffer.unmap();

        Ok(result)
    }

    /// Set default format
    pub fn set_default_format(&mut self, format: ImageFormat) {
        self.default_format = format;
    }

    /// Set JPEG quality
    pub fn set_jpeg_quality(&mut self, quality: u8) {
        self.jpeg_quality = quality.min(100);
    }
}

impl Default for ScreenshotExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screenshot_exporter() {
        let exporter = ScreenshotExporter::new();
        assert_eq!(exporter.default_format, ImageFormat::Png);
    }

    #[test]
    fn test_image_format_extension() {
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
        assert_eq!(ImageFormat::Tiff.extension(), "tif");
        assert_eq!(ImageFormat::Bmp.extension(), "bmp");
    }
}
