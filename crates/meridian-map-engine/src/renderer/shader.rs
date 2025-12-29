//! WGSL shader management and compilation.

use crate::error::{MapEngineError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{Device, ShaderModule};

/// Shader type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderType {
    /// Base vertex/fragment shaders.
    Base,
    /// Vector rendering shaders.
    Vector,
    /// Raster/tile rendering shaders.
    Raster,
    /// Text rendering shaders.
    Text,
    /// Marker rendering shaders.
    Marker,
    /// Feature picking shaders.
    Picking,
}

/// Manages shader compilation and caching.
pub struct ShaderManager {
    device: Arc<Device>,
    shaders: HashMap<ShaderType, ShaderModule>,
    source_cache: HashMap<ShaderType, String>,
}

impl ShaderManager {
    /// Create a new shader manager.
    pub fn new(device: Arc<Device>) -> Self {
        let mut manager = Self {
            device,
            shaders: HashMap::new(),
            source_cache: HashMap::new(),
        };

        // Load built-in shaders
        manager.load_builtin_shaders();

        manager
    }

    /// Load all built-in shader sources.
    fn load_builtin_shaders(&mut self) {
        // Base shaders are loaded from shader files
        // In production, these would be included at compile time
        let base_shader = include_str!("../shaders/base.wgsl");
        self.source_cache
            .insert(ShaderType::Base, base_shader.to_string());

        let vector_shader = include_str!("../shaders/vector.wgsl");
        self.source_cache
            .insert(ShaderType::Vector, vector_shader.to_string());

        let raster_shader = include_str!("../shaders/raster.wgsl");
        self.source_cache
            .insert(ShaderType::Raster, raster_shader.to_string());
    }

    /// Compile a shader from source.
    pub fn compile_shader(&mut self, shader_type: ShaderType, source: &str) -> Result<()> {
        let module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&format!("{:?} Shader", shader_type)),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });

        self.shaders.insert(shader_type, module);
        self.source_cache
            .insert(shader_type, source.to_string());

        Ok(())
    }

    /// Get a compiled shader module.
    pub fn get_shader(&self, shader_type: ShaderType) -> Option<&ShaderModule> {
        self.shaders.get(&shader_type)
    }

    /// Get shader source code.
    pub fn get_source(&self, shader_type: ShaderType) -> Option<&str> {
        self.source_cache.get(&shader_type).map(|s| s.as_str())
    }

    /// Reload a shader (useful for hot-reloading during development).
    #[cfg(feature = "debug-shaders")]
    pub fn reload_shader(&mut self, shader_type: ShaderType) -> Result<()> {
        if let Some(source) = self.source_cache.get(&shader_type).cloned() {
            self.compile_shader(shader_type, &source)?;
        }
        Ok(())
    }

    /// Get or compile a shader.
    pub fn get_or_compile(&mut self, shader_type: ShaderType) -> Result<&ShaderModule> {
        if !self.shaders.contains_key(&shader_type) {
            if let Some(source) = self.source_cache.get(&shader_type).cloned() {
                self.compile_shader(shader_type, &source)?;
            } else {
                return Err(MapEngineError::ShaderCompilation(format!(
                    "No source code found for shader: {:?}",
                    shader_type
                )));
            }
        }

        Ok(self
            .shaders
            .get(&shader_type)
            .expect("Shader should exist after compilation"))
    }

    /// Precompile all cached shaders.
    pub fn precompile_all(&mut self) -> Result<()> {
        let shader_types: Vec<ShaderType> = self.source_cache.keys().copied().collect();

        for shader_type in shader_types {
            if !self.shaders.contains_key(&shader_type) {
                self.get_or_compile(shader_type)?;
            }
        }

        Ok(())
    }

    /// Clear all compiled shaders (useful for memory management).
    pub fn clear_cache(&mut self) {
        self.shaders.clear();
    }

    /// Get shader statistics.
    pub fn stats(&self) -> ShaderStats {
        ShaderStats {
            total_shaders: self.shaders.len(),
            cached_sources: self.source_cache.len(),
        }
    }
}

/// Statistics about compiled shaders.
#[derive(Debug, Clone)]
pub struct ShaderStats {
    /// Total number of compiled shaders.
    pub total_shaders: usize,
    /// Number of cached source codes.
    pub cached_sources: usize,
}

/// Shader preprocessor for dynamic shader generation.
pub struct ShaderPreprocessor {
    defines: HashMap<String, String>,
}

impl ShaderPreprocessor {
    /// Create a new shader preprocessor.
    pub fn new() -> Self {
        Self {
            defines: HashMap::new(),
        }
    }

    /// Add a preprocessor define.
    pub fn define(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.defines.insert(key.into(), value.into());
    }

    /// Process shader source with defines.
    pub fn process(&self, source: &str) -> String {
        let mut output = source.to_string();

        for (key, value) in &self.defines {
            let pattern = format!("#{{{}}}", key);
            output = output.replace(&pattern, value);
        }

        output
    }
}

impl Default for ShaderPreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_preprocessor() {
        let mut preprocessor = ShaderPreprocessor::new();
        preprocessor.define("MAX_LIGHTS", "4");
        preprocessor.define("USE_TEXTURES", "true");

        let source = "const MAX_LIGHTS: u32 = #{MAX_LIGHTS};\nconst USE_TEXTURES: bool = #{USE_TEXTURES};";
        let processed = preprocessor.process(source);

        assert!(processed.contains("MAX_LIGHTS: u32 = 4"));
        assert!(processed.contains("USE_TEXTURES: bool = true"));
    }

    #[test]
    fn test_shader_stats() {
        let stats = ShaderStats {
            total_shaders: 5,
            cached_sources: 6,
        };

        assert_eq!(stats.total_shaders, 5);
        assert_eq!(stats.cached_sources, 6);
    }
}
