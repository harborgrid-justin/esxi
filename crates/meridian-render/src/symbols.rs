//! Symbol and icon management for map rendering

use crate::error::{RenderError, RenderResult};
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Symbol type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolType {
    /// Raster icon (PNG, JPEG, etc.)
    Icon,
    /// Vector symbol (SVG)
    Vector,
    /// Text label
    Text,
    /// Marker symbol
    Marker,
}

/// Symbol definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    /// Unique identifier
    pub id: String,
    /// Symbol type
    pub symbol_type: SymbolType,
    /// Image data (for raster symbols)
    pub data: Option<Vec<u8>>,
    /// SVG content (for vector symbols)
    pub svg: Option<String>,
    /// Symbol width in pixels
    pub width: u32,
    /// Symbol height in pixels
    pub height: u32,
    /// Anchor point X (0.0-1.0, 0.5 = center)
    pub anchor_x: f32,
    /// Anchor point Y (0.0-1.0, 0.5 = center)
    pub anchor_y: f32,
    /// Scale factor
    pub scale: f32,
}

impl Symbol {
    /// Create a new icon symbol from image data
    pub fn icon(id: String, data: Vec<u8>) -> RenderResult<Self> {
        // Decode image to get dimensions
        let img = image::load_from_memory(&data)
            .map_err(|e| RenderError::SymbolNotFound(format!("Invalid image: {}", e)))?;

        Ok(Symbol {
            id,
            symbol_type: SymbolType::Icon,
            data: Some(data),
            svg: None,
            width: img.width(),
            height: img.height(),
            anchor_x: 0.5,
            anchor_y: 0.5,
            scale: 1.0,
        })
    }

    /// Create a new SVG symbol
    pub fn svg(id: String, svg: String, width: u32, height: u32) -> Self {
        Symbol {
            id,
            symbol_type: SymbolType::Vector,
            data: None,
            svg: Some(svg),
            width,
            height,
            anchor_x: 0.5,
            anchor_y: 0.5,
            scale: 1.0,
        }
    }

    /// Set anchor point
    pub fn with_anchor(mut self, x: f32, y: f32) -> Self {
        self.anchor_x = x.clamp(0.0, 1.0);
        self.anchor_y = y.clamp(0.0, 1.0);
        self
    }

    /// Set scale
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.1);
        self
    }

    /// Get scaled dimensions
    pub fn scaled_dimensions(&self) -> (u32, u32) {
        let width = (self.width as f32 * self.scale).round() as u32;
        let height = (self.height as f32 * self.scale).round() as u32;
        (width, height)
    }

    /// Get anchor offset in pixels
    pub fn anchor_offset(&self) -> (i32, i32) {
        let (width, height) = self.scaled_dimensions();
        let offset_x = (width as f32 * self.anchor_x).round() as i32;
        let offset_y = (height as f32 * self.anchor_y).round() as i32;
        (offset_x, offset_y)
    }

    /// Render symbol as image
    pub fn render(&self) -> RenderResult<DynamicImage> {
        match self.symbol_type {
            SymbolType::Icon => {
                if let Some(data) = &self.data {
                    let img = image::load_from_memory(data)?;

                    // Apply scaling if needed
                    if (self.scale - 1.0).abs() > 0.01 {
                        let (width, height) = self.scaled_dimensions();
                        Ok(img.resize(width, height, image::imageops::FilterType::Lanczos3))
                    } else {
                        Ok(img)
                    }
                } else {
                    Err(RenderError::SymbolNotFound(format!(
                        "No data for icon symbol '{}'",
                        self.id
                    )))
                }
            }
            SymbolType::Vector => {
                if let Some(svg) = &self.svg {
                    render_svg_to_image(svg, self.width, self.height, self.scale)
                } else {
                    Err(RenderError::SymbolNotFound(format!(
                        "No SVG data for vector symbol '{}'",
                        self.id
                    )))
                }
            }
            _ => Err(RenderError::SymbolNotFound(format!(
                "Cannot render symbol type {:?}",
                self.symbol_type
            ))),
        }
    }
}

/// Symbol registry for managing all available symbols
pub struct SymbolRegistry {
    symbols: Arc<RwLock<HashMap<String, Symbol>>>,
    search_paths: Vec<PathBuf>,
}

impl SymbolRegistry {
    /// Create a new symbol registry
    pub fn new() -> Self {
        SymbolRegistry {
            symbols: Arc::new(RwLock::new(HashMap::new())),
            search_paths: Vec::new(),
        }
    }

    /// Add a search path for symbol files
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.search_paths.push(path.as_ref().to_path_buf());
    }

    /// Register a symbol
    pub fn register(&self, symbol: Symbol) {
        let mut symbols = self.symbols.write().unwrap();
        symbols.insert(symbol.id.clone(), symbol);
    }

    /// Get a symbol by ID
    pub fn get(&self, id: &str) -> RenderResult<Symbol> {
        let symbols = self.symbols.read().unwrap();
        symbols
            .get(id)
            .cloned()
            .ok_or_else(|| RenderError::SymbolNotFound(id.to_string()))
    }

    /// Check if a symbol exists
    pub fn contains(&self, id: &str) -> bool {
        let symbols = self.symbols.read().unwrap();
        symbols.contains_key(id)
    }

    /// Load an icon from file
    pub fn load_icon<P: AsRef<Path>>(&self, id: String, path: P) -> RenderResult<()> {
        let data = std::fs::read(path)?;
        let symbol = Symbol::icon(id, data)?;
        self.register(symbol);
        Ok(())
    }

    /// Load an SVG from file
    pub fn load_svg<P: AsRef<Path>>(
        &self,
        id: String,
        path: P,
        width: u32,
        height: u32,
    ) -> RenderResult<()> {
        let svg = std::fs::read_to_string(path)?;
        let symbol = Symbol::svg(id, svg, width, height);
        self.register(symbol);
        Ok(())
    }

    /// Load all symbols from a directory
    pub fn load_directory<P: AsRef<Path>>(&self, path: P) -> RenderResult<usize> {
        let path = path.as_ref();
        let mut count = 0;

        if !path.is_dir() {
            return Err(RenderError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Not a directory: {}", path.display()),
            )));
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    let id = entry_path
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();

                    match ext.as_str() {
                        "png" | "jpg" | "jpeg" | "gif" | "webp" => {
                            if self.load_icon(id, &entry_path).is_ok() {
                                count += 1;
                            }
                        }
                        "svg" => {
                            // Default SVG size
                            if self.load_svg(id, &entry_path, 32, 32).is_ok() {
                                count += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(count)
    }

    /// Get all symbol IDs
    pub fn list(&self) -> Vec<String> {
        let symbols = self.symbols.read().unwrap();
        symbols.keys().cloned().collect()
    }

    /// Clear all symbols
    pub fn clear(&self) {
        let mut symbols = self.symbols.write().unwrap();
        symbols.clear();
    }

    /// Get number of registered symbols
    pub fn count(&self) -> usize {
        let symbols = self.symbols.read().unwrap();
        symbols.len()
    }
}

impl Default for SymbolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Render SVG to raster image
fn render_svg_to_image(
    svg_data: &str,
    width: u32,
    height: u32,
    scale: f32,
) -> RenderResult<DynamicImage> {
    // Parse SVG
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg_data, &opt)
        .map_err(|e| RenderError::SvgError(format!("Failed to parse SVG: {}", e)))?;

    // Calculate scaled dimensions
    let scaled_width = (width as f32 * scale).round() as u32;
    let scaled_height = (height as f32 * scale).round() as u32;

    // Create pixmap
    let mut pixmap = tiny_skia::Pixmap::new(scaled_width, scaled_height)
        .ok_or_else(|| RenderError::SvgError("Failed to create pixmap".to_string()))?;

    // Render SVG
    let transform = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Convert to image
    let img_buffer = image::RgbaImage::from_raw(scaled_width, scaled_height, pixmap.take())
        .ok_or_else(|| RenderError::SvgError("Failed to create image buffer".to_string()))?;

    Ok(DynamicImage::ImageRgba8(img_buffer))
}

/// Sprite sheet for efficient symbol storage
pub struct SpriteSheet {
    /// Sprite sheet image
    pub image: DynamicImage,
    /// Symbol positions and sizes
    pub positions: HashMap<String, SpritePosition>,
}

/// Position of a symbol in a sprite sheet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpritePosition {
    /// X coordinate
    pub x: u32,
    /// Y coordinate
    pub y: u32,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
    /// Pixel ratio
    pub pixel_ratio: f32,
}

impl SpriteSheet {
    /// Create a sprite sheet from symbols
    pub fn from_symbols(symbols: &[Symbol]) -> RenderResult<Self> {
        if symbols.is_empty() {
            return Err(RenderError::Other("No symbols to pack".to_string()));
        }

        // Simple horizontal packing for now
        let total_width: u32 = symbols.iter().map(|s| s.width).sum();
        let max_height: u32 = symbols.iter().map(|s| s.height).max().unwrap_or(0);

        let mut sprite_image = image::DynamicImage::new_rgba8(total_width, max_height);
        let mut positions = HashMap::new();
        let mut x_offset = 0;

        for symbol in symbols {
            let img = symbol.render()?;
            image::imageops::overlay(&mut sprite_image, &img, x_offset.into(), 0);

            positions.insert(
                symbol.id.clone(),
                SpritePosition {
                    x: x_offset,
                    y: 0,
                    width: symbol.width,
                    height: symbol.height,
                    pixel_ratio: symbol.scale,
                },
            );

            x_offset += symbol.width;
        }

        Ok(SpriteSheet {
            image: sprite_image,
            positions,
        })
    }

    /// Save sprite sheet to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> RenderResult<()> {
        self.image.save(path)?;
        Ok(())
    }

    /// Save sprite sheet metadata (JSON)
    pub fn save_metadata<P: AsRef<Path>>(&self, path: P) -> RenderResult<()> {
        let json = serde_json::to_string_pretty(&self.positions)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_creation() {
        // Create a simple 1x1 pixel PNG
        let mut data = Vec::new();
        let img = DynamicImage::new_rgba8(1, 1);
        img.write_to(&mut Cursor::new(&mut data), ImageFormat::Png)
            .unwrap();

        let symbol = Symbol::icon("test".to_string(), data).unwrap();
        assert_eq!(symbol.id, "test");
        assert_eq!(symbol.symbol_type, SymbolType::Icon);
        assert_eq!(symbol.width, 1);
        assert_eq!(symbol.height, 1);
    }

    #[test]
    fn test_symbol_anchor() {
        let symbol = Symbol::svg("test".to_string(), "".to_string(), 100, 100)
            .with_anchor(0.0, 0.0);

        let (offset_x, offset_y) = symbol.anchor_offset();
        assert_eq!(offset_x, 0);
        assert_eq!(offset_y, 0);
    }

    #[test]
    fn test_symbol_registry() {
        let registry = SymbolRegistry::new();

        let symbol = Symbol::svg("marker".to_string(), "".to_string(), 32, 32);
        registry.register(symbol);

        assert!(registry.contains("marker"));
        assert_eq!(registry.count(), 1);

        let retrieved = registry.get("marker").unwrap();
        assert_eq!(retrieved.id, "marker");
    }

    #[test]
    fn test_scaled_dimensions() {
        let symbol = Symbol::svg("test".to_string(), "".to_string(), 100, 100)
            .with_scale(2.0);

        let (width, height) = symbol.scaled_dimensions();
        assert_eq!(width, 200);
        assert_eq!(height, 200);
    }
}
