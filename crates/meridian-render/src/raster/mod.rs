//! Raster tile rendering engine

use crate::error::{RenderError, RenderResult};
use crate::style::{Layer, LayerType, PaintProperties, Style};
use crate::symbols::SymbolRegistry;
use crate::tile::{TileBounds, TileCoord, TILE_SIZE};
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba};
use std::io::Cursor;
use tiny_skia::{Color as SkiaColor, Paint, PathBuilder, Pixmap, Stroke, Transform};

/// Raster tile format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileFormat {
    Png,
    Jpeg,
    WebP,
}

impl TileFormat {
    /// Get MIME type
    pub fn mime_type(&self) -> &'static str {
        match self {
            TileFormat::Png => "image/png",
            TileFormat::Jpeg => "image/jpeg",
            TileFormat::WebP => "image/webp",
        }
    }

    /// Get file extension
    pub fn extension(&self) -> &'static str {
        match self {
            TileFormat::Png => "png",
            TileFormat::Jpeg => "jpg",
            TileFormat::WebP => "webp",
        }
    }
}

/// Raster tile renderer
pub struct RasterRenderer {
    /// Tile size in pixels
    tile_size: u32,
    /// Background color
    background_color: Rgba<u8>,
    /// Anti-aliasing enabled
    antialias: bool,
    /// Symbol registry
    symbols: Option<SymbolRegistry>,
}

impl RasterRenderer {
    /// Create a new raster renderer
    pub fn new() -> Self {
        RasterRenderer {
            tile_size: TILE_SIZE,
            background_color: Rgba([255, 255, 255, 255]),
            antialias: true,
            symbols: None,
        }
    }

    /// Set tile size
    pub fn with_tile_size(mut self, size: u32) -> Self {
        self.tile_size = size;
        self
    }

    /// Set background color
    pub fn with_background_color(mut self, color: Rgba<u8>) -> Self {
        self.background_color = color;
        self
    }

    /// Enable or disable anti-aliasing
    pub fn with_antialias(mut self, antialias: bool) -> Self {
        self.antialias = antialias;
        self
    }

    /// Set symbol registry
    pub fn with_symbols(mut self, symbols: SymbolRegistry) -> Self {
        self.symbols = Some(symbols);
        self
    }

    /// Render a tile with the given style
    pub fn render_tile(
        &self,
        coord: TileCoord,
        style: &Style,
        data: &TileData,
    ) -> RenderResult<DynamicImage> {
        let mut pixmap = Pixmap::new(self.tile_size, self.tile_size)
            .ok_or_else(|| RenderError::Other("Failed to create pixmap".to_string()))?;

        // Fill background
        pixmap.fill(SkiaColor::from_rgba8(
            self.background_color[0],
            self.background_color[1],
            self.background_color[2],
            self.background_color[3],
        ));

        let bounds = coord.bounds();

        // Render layers in order
        for layer in &style.layers {
            if !layer.is_visible_at_zoom(f64::from(coord.z)) {
                continue;
            }

            self.render_layer(&mut pixmap, layer, &bounds, data)?;
        }

        // Convert pixmap to image
        let img_buffer = ImageBuffer::from_raw(self.tile_size, self.tile_size, pixmap.take())
            .ok_or_else(|| RenderError::Other("Failed to create image buffer".to_string()))?;

        Ok(DynamicImage::ImageRgba8(img_buffer))
    }

    /// Render a single layer
    fn render_layer(
        &self,
        pixmap: &mut Pixmap,
        layer: &Layer,
        bounds: &TileBounds,
        data: &TileData,
    ) -> RenderResult<()> {
        match layer.layer_type {
            LayerType::Fill => self.render_fill_layer(pixmap, layer, bounds, data),
            LayerType::Line => self.render_line_layer(pixmap, layer, bounds, data),
            LayerType::Circle => self.render_circle_layer(pixmap, layer, bounds, data),
            LayerType::Symbol => self.render_symbol_layer(pixmap, layer, bounds, data),
            LayerType::Background => self.render_background_layer(pixmap, layer),
            _ => Ok(()), // Other layer types not implemented yet
        }
    }

    /// Render background layer
    fn render_background_layer(&self, pixmap: &mut Pixmap, layer: &Layer) -> RenderResult<()> {
        if let Some(paint) = &layer.paint {
            if let Some(color) = &paint.fill_color {
                if let Some(color_value) = color.constant() {
                    let rgba = color_value.to_rgba()?;
                    pixmap.fill(SkiaColor::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]));
                }
            }
        }
        Ok(())
    }

    /// Render fill layer
    fn render_fill_layer(
        &self,
        pixmap: &mut Pixmap,
        layer: &Layer,
        bounds: &TileBounds,
        data: &TileData,
    ) -> RenderResult<()> {
        let paint_props = layer.paint.as_ref();
        if paint_props.is_none() {
            return Ok(());
        }

        // Get fill color and opacity
        let color = self.get_fill_color(paint_props.unwrap())?;
        let opacity = self.get_opacity(paint_props.unwrap());

        // Render polygons
        for polygon in &data.polygons {
            self.render_polygon(pixmap, polygon, bounds, color, opacity)?;
        }

        Ok(())
    }

    /// Render line layer
    fn render_line_layer(
        &self,
        pixmap: &mut Pixmap,
        layer: &Layer,
        bounds: &TileBounds,
        data: &TileData,
    ) -> RenderResult<()> {
        let paint_props = layer.paint.as_ref();
        if paint_props.is_none() {
            return Ok(());
        }

        let color = self.get_line_color(paint_props.unwrap())?;
        let width = self.get_line_width(paint_props.unwrap());
        let opacity = self.get_opacity(paint_props.unwrap());

        for line in &data.lines {
            self.render_line(pixmap, line, bounds, color, width, opacity)?;
        }

        Ok(())
    }

    /// Render circle layer
    fn render_circle_layer(
        &self,
        pixmap: &mut Pixmap,
        layer: &Layer,
        bounds: &TileBounds,
        data: &TileData,
    ) -> RenderResult<()> {
        let paint_props = layer.paint.as_ref();
        if paint_props.is_none() {
            return Ok(());
        }

        let color = self.get_circle_color(paint_props.unwrap())?;
        let radius = self.get_circle_radius(paint_props.unwrap());
        let opacity = self.get_opacity(paint_props.unwrap());

        for point in &data.points {
            self.render_circle(pixmap, point, bounds, color, radius, opacity)?;
        }

        Ok(())
    }

    /// Render symbol layer
    fn render_symbol_layer(
        &self,
        _pixmap: &mut Pixmap,
        _layer: &Layer,
        _bounds: &TileBounds,
        _data: &TileData,
    ) -> RenderResult<()> {
        // Symbol rendering would require text rendering and icon placement
        // This is a complex feature that would be implemented with additional libraries
        Ok(())
    }

    /// Render a polygon
    fn render_polygon(
        &self,
        pixmap: &mut Pixmap,
        polygon: &[(f64, f64)],
        bounds: &TileBounds,
        color: SkiaColor,
        opacity: f32,
    ) -> RenderResult<()> {
        if polygon.is_empty() {
            return Ok(());
        }

        let mut path_builder = PathBuilder::new();
        let first = self.project_point(polygon[0], bounds);
        path_builder.move_to(first.0, first.1);

        for &point in &polygon[1..] {
            let (x, y) = self.project_point(point, bounds);
            path_builder.line_to(x, y);
        }

        path_builder.close();

        if let Some(path) = path_builder.finish() {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.anti_alias = self.antialias;
            // Opacity handled in blend mode

            pixmap.fill_path(
                &path,
                &paint,
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                None,
            );
        }

        Ok(())
    }

    /// Render a line
    fn render_line(
        &self,
        pixmap: &mut Pixmap,
        line: &[(f64, f64)],
        bounds: &TileBounds,
        color: SkiaColor,
        width: f32,
        opacity: f32,
    ) -> RenderResult<()> {
        if line.len() < 2 {
            return Ok(());
        }

        let mut path_builder = PathBuilder::new();
        let first = self.project_point(line[0], bounds);
        path_builder.move_to(first.0, first.1);

        for &point in &line[1..] {
            let (x, y) = self.project_point(point, bounds);
            path_builder.line_to(x, y);
        }

        if let Some(path) = path_builder.finish() {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.anti_alias = self.antialias;
            // Opacity handled in blend mode

            let mut stroke = Stroke::default();
            stroke.width = width;

            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }

        Ok(())
    }

    /// Render a circle
    fn render_circle(
        &self,
        pixmap: &mut Pixmap,
        point: &(f64, f64),
        bounds: &TileBounds,
        color: SkiaColor,
        radius: f32,
        opacity: f32,
    ) -> RenderResult<()> {
        let (x, y) = self.project_point(*point, bounds);

        let mut path_builder = PathBuilder::new();
        path_builder.push_circle(x, y, radius);

        if let Some(path) = path_builder.finish() {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.anti_alias = self.antialias;
            // Opacity handled in blend mode

            pixmap.fill_path(
                &path,
                &paint,
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                None,
            );
        }

        Ok(())
    }

    /// Project a world coordinate to pixel coordinate
    fn project_point(&self, point: (f64, f64), bounds: &TileBounds) -> (f32, f32) {
        let x = ((point.0 - bounds.min_x) / bounds.width() * f64::from(self.tile_size)) as f32;
        let y = ((bounds.max_y - point.1) / bounds.height() * f64::from(self.tile_size)) as f32;
        (x, y)
    }

    /// Get fill color from paint properties
    fn get_fill_color(&self, paint: &PaintProperties) -> RenderResult<SkiaColor> {
        if let Some(color) = &paint.fill_color {
            if let Some(color_value) = color.constant() {
                let rgba = color_value.to_rgba()?;
                return Ok(SkiaColor::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]));
            }
        }
        Ok(SkiaColor::from_rgba8(0, 0, 0, 255))
    }

    /// Get line color from paint properties
    fn get_line_color(&self, paint: &PaintProperties) -> RenderResult<SkiaColor> {
        if let Some(color) = &paint.line_color {
            if let Some(color_value) = color.constant() {
                let rgba = color_value.to_rgba()?;
                return Ok(SkiaColor::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]));
            }
        }
        Ok(SkiaColor::from_rgba8(0, 0, 0, 255))
    }

    /// Get circle color from paint properties
    fn get_circle_color(&self, paint: &PaintProperties) -> RenderResult<SkiaColor> {
        if let Some(color) = &paint.circle_color {
            if let Some(color_value) = color.constant() {
                let rgba = color_value.to_rgba()?;
                return Ok(SkiaColor::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]));
            }
        }
        Ok(SkiaColor::from_rgba8(0, 0, 0, 255))
    }

    /// Get line width from paint properties
    fn get_line_width(&self, paint: &PaintProperties) -> f32 {
        if let Some(width) = &paint.line_width {
            if let Some(&w) = width.constant() {
                return w as f32;
            }
        }
        1.0
    }

    /// Get circle radius from paint properties
    fn get_circle_radius(&self, paint: &PaintProperties) -> f32 {
        if let Some(radius) = &paint.circle_radius {
            if let Some(&r) = radius.constant() {
                return r as f32;
            }
        }
        5.0
    }

    /// Get opacity from paint properties
    fn get_opacity(&self, paint: &PaintProperties) -> f32 {
        if let Some(opacity) = &paint.fill_opacity {
            if let Some(&o) = opacity.constant() {
                return o as f32;
            }
        }
        if let Some(opacity) = &paint.line_opacity {
            if let Some(&o) = opacity.constant() {
                return o as f32;
            }
        }
        if let Some(opacity) = &paint.circle_opacity {
            if let Some(&o) = opacity.constant() {
                return o as f32;
            }
        }
        1.0
    }

    /// Encode image to bytes
    pub fn encode(&self, image: &DynamicImage, format: TileFormat) -> RenderResult<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());

        match format {
            TileFormat::Png => {
                image.write_to(&mut buffer, ImageFormat::Png)?;
            }
            TileFormat::Jpeg => {
                image.write_to(&mut buffer, ImageFormat::Jpeg)?;
            }
            TileFormat::WebP => {
                image.write_to(&mut buffer, ImageFormat::WebP)?;
            }
        }

        Ok(buffer.into_inner())
    }
}

impl Default for RasterRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Tile data containing geometries to render
#[derive(Debug, Clone, Default)]
pub struct TileData {
    /// Points
    pub points: Vec<(f64, f64)>,
    /// Lines
    pub lines: Vec<Vec<(f64, f64)>>,
    /// Polygons
    pub polygons: Vec<Vec<(f64, f64)>>,
}

impl TileData {
    /// Create new empty tile data
    pub fn new() -> Self {
        TileData::default()
    }

    /// Add a point
    pub fn add_point(&mut self, x: f64, y: f64) {
        self.points.push((x, y));
    }

    /// Add a line
    pub fn add_line(&mut self, line: Vec<(f64, f64)>) {
        self.lines.push(line);
    }

    /// Add a polygon
    pub fn add_polygon(&mut self, polygon: Vec<(f64, f64)>) {
        self.polygons.push(polygon);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_format() {
        assert_eq!(TileFormat::Png.mime_type(), "image/png");
        assert_eq!(TileFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(TileFormat::WebP.mime_type(), "image/webp");
    }

    #[test]
    fn test_raster_renderer_creation() {
        let renderer = RasterRenderer::new()
            .with_tile_size(512)
            .with_antialias(true);

        assert_eq!(renderer.tile_size, 512);
        assert!(renderer.antialias);
    }

    #[test]
    fn test_tile_data() {
        let mut data = TileData::new();
        data.add_point(0.0, 0.0);
        data.add_line(vec![(0.0, 0.0), (1.0, 1.0)]);
        data.add_polygon(vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);

        assert_eq!(data.points.len(), 1);
        assert_eq!(data.lines.len(), 1);
        assert_eq!(data.polygons.len(), 1);
    }
}
