//! Video export for animations

use crate::{Camera, Scene, Error, Result};
use image::RgbaImage;
use std::path::Path;
use wgpu::{Device, Queue};

/// Video format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFormat {
    /// MP4 container
    Mp4,
    /// WebM container
    WebM,
    /// AVI container
    Avi,
}

/// Video codec
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodec {
    /// H.264
    H264,
    /// H.265 (HEVC)
    H265,
    /// VP8
    VP8,
    /// VP9
    VP9,
}

/// Video recorder settings
#[derive(Debug, Clone)]
pub struct VideoSettings {
    /// Output width
    pub width: u32,

    /// Output height
    pub height: u32,

    /// Frames per second
    pub fps: u32,

    /// Video format
    pub format: VideoFormat,

    /// Video codec
    pub codec: VideoCodec,

    /// Bitrate (bits per second)
    pub bitrate: u32,
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 30,
            format: VideoFormat::Mp4,
            codec: VideoCodec::H264,
            bitrate: 5_000_000, // 5 Mbps
        }
    }
}

/// Video recorder
pub struct VideoRecorder {
    /// Video settings
    settings: VideoSettings,

    /// Recording state
    recording: bool,

    /// Recorded frames
    frames: Vec<RgbaImage>,

    /// Current frame index
    frame_index: usize,
}

impl VideoRecorder {
    /// Create a new video recorder
    pub fn new(settings: VideoSettings) -> Self {
        Self {
            settings,
            recording: false,
            frames: Vec::new(),
            frame_index: 0,
        }
    }

    /// Start recording
    pub fn start(&mut self) {
        self.recording = true;
        self.frames.clear();
        self.frame_index = 0;
    }

    /// Stop recording
    pub fn stop(&mut self) {
        self.recording = false;
    }

    /// Check if recording
    pub fn is_recording(&self) -> bool {
        self.recording
    }

    /// Add a frame
    pub fn add_frame(&mut self, frame: RgbaImage) -> Result<()> {
        if !self.recording {
            return Err(Error::export("Not currently recording"));
        }

        if frame.width() != self.settings.width || frame.height() != self.settings.height {
            return Err(Error::export("Frame size doesn't match video settings"));
        }

        self.frames.push(frame);
        self.frame_index += 1;

        Ok(())
    }

    /// Save recorded video to file
    #[cfg(feature = "video-export")]
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        if self.frames.is_empty() {
            return Err(Error::export("No frames to export"));
        }

        // Video encoding using ffmpeg-next would go here
        // This is a placeholder implementation

        // For now, just indicate that the feature is available
        tracing::info!("Video export feature is available but not fully implemented");
        tracing::info!("Would export {} frames to {:?}", self.frames.len(), path.as_ref());

        Ok(())
    }

    /// Save recorded video to file (stub when feature is disabled)
    #[cfg(not(feature = "video-export"))]
    pub fn save(&self, _path: impl AsRef<Path>) -> Result<()> {
        Err(Error::export(
            "Video export feature not enabled. Enable with 'video-export' feature flag.",
        ))
    }

    /// Get frame count
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Get duration in seconds
    pub fn duration(&self) -> f32 {
        self.frames.len() as f32 / self.settings.fps as f32
    }

    /// Clear recorded frames
    pub fn clear(&mut self) {
        self.frames.clear();
        self.frame_index = 0;
    }

    /// Get settings
    pub fn settings(&self) -> &VideoSettings {
        &self.settings
    }

    /// Update settings (only when not recording)
    pub fn set_settings(&mut self, settings: VideoSettings) -> Result<()> {
        if self.recording {
            return Err(Error::export("Cannot change settings while recording"));
        }

        self.settings = settings;
        Ok(())
    }
}

impl Default for VideoRecorder {
    fn default() -> Self {
        Self::new(VideoSettings::default())
    }
}

/// Video frame capturer helper
pub struct VideoFrameCapturer {
    /// Screenshot exporter
    screenshot_exporter: crate::export::ScreenshotExporter,
}

impl VideoFrameCapturer {
    /// Create a new frame capturer
    pub fn new() -> Self {
        Self {
            screenshot_exporter: crate::export::ScreenshotExporter::new(),
        }
    }

    /// Capture a frame for video
    pub async fn capture_frame(
        &self,
        device: &Device,
        queue: &Queue,
        scene: &Scene,
        camera: &Camera,
        width: u32,
        height: u32,
    ) -> Result<RgbaImage> {
        self.screenshot_exporter
            .capture(device, queue, scene, camera, width, height)
            .await
    }
}

impl Default for VideoFrameCapturer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_recorder() {
        let recorder = VideoRecorder::new(VideoSettings::default());
        assert!(!recorder.is_recording());
        assert_eq!(recorder.frame_count(), 0);
    }

    #[test]
    fn test_recording_state() {
        let mut recorder = VideoRecorder::new(VideoSettings::default());

        recorder.start();
        assert!(recorder.is_recording());

        recorder.stop();
        assert!(!recorder.is_recording());
    }

    #[test]
    fn test_video_settings() {
        let settings = VideoSettings::default();
        assert_eq!(settings.width, 1920);
        assert_eq!(settings.height, 1080);
        assert_eq!(settings.fps, 30);
    }

    #[test]
    fn test_duration_calculation() {
        let mut recorder = VideoRecorder::new(VideoSettings {
            fps: 30,
            ..Default::default()
        });

        // Add 90 frames = 3 seconds at 30fps
        for _ in 0..90 {
            recorder.frames.push(RgbaImage::new(1920, 1080));
        }

        assert_eq!(recorder.duration(), 3.0);
    }
}
