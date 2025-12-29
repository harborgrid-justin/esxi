//! Raster tile layer rendering for map tiles.

use super::{Layer, LayerProperties, LayerType, Visibility};
use crate::camera::Camera;
use crate::error::Result;
use crate::renderer::{buffer::BufferHandle, texture::TextureHandle, Renderer, Vertex};
use crate::tile::{TileCache, TileCoord};
use glam::Vec2;
use std::collections::HashSet;

/// Raster layer for rendering map tiles.
pub struct RasterLayer {
    /// Layer properties.
    properties: LayerProperties,
    /// Tile source URL template (e.g., "https://tile.server/{z}/{x}/{y}.png").
    tile_url_template: String,
    /// Tile size in pixels.
    tile_size: u32,
    /// Currently visible tiles.
    visible_tiles: HashSet<TileCoord>,
    /// Vertex buffer for tile quads.
    vertex_buffer: Option<BufferHandle>,
    /// Index buffer for tile quads.
    index_buffer: Option<BufferHandle>,
    /// Texture handles for loaded tiles.
    tile_textures: std::collections::HashMap<TileCoord, TextureHandle>,
    /// Whether buffers need to be rebuilt.
    dirty: bool,
    /// Tile cache reference.
    cache: Option<TileCache>,
}

impl RasterLayer {
    /// Create a new raster layer.
    pub fn new(name: impl Into<String>, tile_url_template: impl Into<String>) -> Self {
        Self {
            properties: LayerProperties::new(name, LayerType::Raster),
            tile_url_template: tile_url_template.into(),
            tile_size: 256,
            visible_tiles: HashSet::new(),
            vertex_buffer: None,
            index_buffer: None,
            tile_textures: std::collections::HashMap::new(),
            dirty: true,
            cache: None,
        }
    }

    /// Set the tile URL template.
    pub fn set_tile_url_template(&mut self, template: impl Into<String>) {
        self.tile_url_template = template.into();
    }

    /// Set the tile size.
    pub fn set_tile_size(&mut self, size: u32) {
        self.tile_size = size;
    }

    /// Get tile URL for specific coordinates.
    pub fn get_tile_url(&self, coord: &TileCoord) -> String {
        self.tile_url_template
            .replace("{z}", &coord.z.to_string())
            .replace("{x}", &coord.x.to_string())
            .replace("{y}", &coord.y.to_string())
    }

    /// Calculate visible tiles based on camera position.
    fn calculate_visible_tiles(&self, camera: &Camera) -> HashSet<TileCoord> {
        let mut tiles = HashSet::new();
        let zoom = camera.zoom.floor() as u32;
        let center = camera.center();

        // Convert center to tile coordinates
        let scale = 2_f32.powi(zoom as i32);
        let center_tile_x = ((center.x + 180.0) / 360.0 * scale) as i32;
        let center_tile_y =
            ((1.0 - (center.y.to_radians().tan() + 1.0 / center.y.to_radians().cos()).ln()
                / std::f32::consts::PI)
                / 2.0
                * scale) as i32;

        // Calculate tile range based on viewport
        let tile_range = 2; // Number of tiles to load around center

        for dx in -tile_range..=tile_range {
            for dy in -tile_range..=tile_range {
                let x = center_tile_x + dx;
                let y = center_tile_y + dy;

                // Check tile bounds
                let max_tile = 2_i32.pow(zoom);
                if x >= 0 && x < max_tile && y >= 0 && y < max_tile {
                    tiles.insert(TileCoord {
                        x: x as u32,
                        y: y as u32,
                        z: zoom,
                    });
                }
            }
        }

        tiles
    }

    /// Build vertex and index buffers for visible tiles.
    fn build_tile_buffers(&mut self, renderer: &mut Renderer, camera: &Camera) -> Result<()> {
        let visible_tiles = self.calculate_visible_tiles(camera);
        self.visible_tiles = visible_tiles;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for tile_coord in &self.visible_tiles {
            // Calculate tile bounds in world coordinates
            let tile_bounds = self.tile_to_world_bounds(tile_coord);

            let base_index = vertices.len() as u32;

            // Create quad for this tile
            vertices.push(Vertex {
                position: [tile_bounds.0.x, tile_bounds.0.y],
                tex_coords: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, self.properties.opacity],
            });
            vertices.push(Vertex {
                position: [tile_bounds.1.x, tile_bounds.0.y],
                tex_coords: [1.0, 0.0],
                color: [1.0, 1.0, 1.0, self.properties.opacity],
            });
            vertices.push(Vertex {
                position: [tile_bounds.1.x, tile_bounds.1.y],
                tex_coords: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, self.properties.opacity],
            });
            vertices.push(Vertex {
                position: [tile_bounds.0.x, tile_bounds.1.y],
                tex_coords: [0.0, 1.0],
                color: [1.0, 1.0, 1.0, self.properties.opacity],
            });

            // Create quad indices
            indices.extend_from_slice(&[
                base_index,
                base_index + 1,
                base_index + 2,
                base_index,
                base_index + 2,
                base_index + 3,
            ]);
        }

        // Create GPU buffers
        if !vertices.is_empty() {
            let vertex_data = bytemuck::cast_slice(&vertices);
            self.vertex_buffer = Some(renderer.buffer_manager_mut().create_vertex_buffer(
                vertex_data,
                std::mem::size_of::<Vertex>() as u32,
                false,
            )?);

            self.index_buffer = Some(
                renderer
                    .buffer_manager_mut()
                    .create_index_buffer(&indices, false)?,
            );
        }

        self.dirty = false;
        Ok(())
    }

    /// Convert tile coordinates to world bounds.
    fn tile_to_world_bounds(&self, coord: &TileCoord) -> (Vec2, Vec2) {
        let scale = 2_f32.powi(coord.z as i32);
        let tile_size = 360.0 / scale;

        let min_lon = coord.x as f32 * tile_size - 180.0;
        let max_lon = (coord.x + 1) as f32 * tile_size - 180.0;

        // Simplified lat calculation (proper Web Mercator conversion needed for production)
        let min_lat = 85.0511 - (coord.y as f32 * 170.1022 / scale);
        let max_lat = 85.0511 - ((coord.y + 1) as f32 * 170.1022 / scale);

        (Vec2::new(min_lon, min_lat), Vec2::new(max_lon, max_lat))
    }

    /// Get visible tile count.
    pub fn visible_tile_count(&self) -> usize {
        self.visible_tiles.len()
    }
}

impl Layer for RasterLayer {
    fn properties(&self) -> &LayerProperties {
        &self.properties
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.properties
    }

    fn update(&mut self, camera: &Camera, _delta_time: f32) -> Result<()> {
        // Check if visible tiles have changed
        let new_visible_tiles = self.calculate_visible_tiles(camera);
        if new_visible_tiles != self.visible_tiles {
            self.dirty = true;
        }

        Ok(())
    }

    fn render(&self, renderer: &mut Renderer, _camera: &Camera) -> Result<()> {
        // Note: In a complete implementation, this would:
        // 1. Bind the raster rendering pipeline
        // 2. For each visible tile, bind its texture and draw the quad
        // 3. Handle tile loading states (show placeholder while loading)

        if let (Some(_vertex_buffer), Some(_index_buffer)) =
            (self.vertex_buffer, self.index_buffer)
        {
            // Render each tile with its texture
            // for (coord, texture) in &self.tile_textures {
            //     if self.visible_tiles.contains(coord) {
            //         render_pass.set_bind_group(1, texture_bind_group, &[]);
            //         render_pass.draw_indexed(...);
            //     }
            // }
        }

        Ok(())
    }
}

/// Tile loading state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileLoadState {
    /// Tile is not loaded.
    NotLoaded,
    /// Tile is currently loading.
    Loading,
    /// Tile is loaded and ready.
    Loaded,
    /// Tile failed to load.
    Error,
}

/// Tile with loading state.
pub struct TileEntry {
    /// Tile coordinates.
    pub coord: TileCoord,
    /// Texture handle (if loaded).
    pub texture: Option<TextureHandle>,
    /// Loading state.
    pub state: TileLoadState,
    /// Last access timestamp.
    pub last_access: std::time::Instant,
}

impl TileEntry {
    /// Create a new tile entry.
    pub fn new(coord: TileCoord) -> Self {
        Self {
            coord,
            texture: None,
            state: TileLoadState::NotLoaded,
            last_access: std::time::Instant::now(),
        }
    }

    /// Mark as accessed.
    pub fn touch(&mut self) {
        self.last_access = std::time::Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raster_layer_creation() {
        let layer = RasterLayer::new("test_raster", "https://tile.server/{z}/{x}/{y}.png");
        assert_eq!(layer.properties.name, "test_raster");
        assert_eq!(layer.tile_size, 256);
    }

    #[test]
    fn test_tile_url_generation() {
        let layer = RasterLayer::new("test", "https://tile.server/{z}/{x}/{y}.png");
        let coord = TileCoord { x: 1, y: 2, z: 3 };
        let url = layer.get_tile_url(&coord);
        assert_eq!(url, "https://tile.server/3/1/2.png");
    }

    #[test]
    fn test_tile_entry() {
        let mut entry = TileEntry::new(TileCoord { x: 0, y: 0, z: 0 });
        assert_eq!(entry.state, TileLoadState::NotLoaded);
        assert!(entry.texture.is_none());

        entry.touch();
        // Last access should be updated
    }
}
