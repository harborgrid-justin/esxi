//! Vertex and index buffer management for efficient GPU memory usage.

use crate::error::{MapEngineError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{Buffer, BufferUsages, Device};

/// Buffer handle for referencing GPU buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferHandle(u64);

impl BufferHandle {
    /// Create a new buffer handle.
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the internal ID.
    pub fn id(&self) -> u64 {
        self.0
    }
}

/// Type of buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    /// Vertex buffer.
    Vertex,
    /// Index buffer.
    Index,
    /// Uniform buffer.
    Uniform,
    /// Instance buffer.
    Instance,
}

/// Metadata about a buffer.
#[derive(Debug, Clone)]
pub struct BufferInfo {
    /// Buffer type.
    pub buffer_type: BufferType,
    /// Size in bytes.
    pub size: u64,
    /// Number of elements.
    pub element_count: u32,
    /// Element size in bytes.
    pub element_size: u32,
    /// Whether the buffer is dynamic (can be updated).
    pub dynamic: bool,
}

impl BufferInfo {
    /// Create new buffer info.
    pub fn new(
        buffer_type: BufferType,
        size: u64,
        element_count: u32,
        element_size: u32,
        dynamic: bool,
    ) -> Self {
        Self {
            buffer_type,
            size,
            element_count,
            element_size,
            dynamic,
        }
    }
}

/// Manages GPU buffers for vertex data, indices, and uniforms.
pub struct BufferManager {
    device: Arc<Device>,
    buffers: HashMap<BufferHandle, Buffer>,
    buffer_info: HashMap<BufferHandle, BufferInfo>,
    next_handle_id: u64,
}

impl BufferManager {
    /// Create a new buffer manager.
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            device,
            buffers: HashMap::new(),
            buffer_info: HashMap::new(),
            next_handle_id: 0,
        }
    }

    /// Create a vertex buffer.
    pub fn create_vertex_buffer(
        &mut self,
        data: &[u8],
        element_size: u32,
        dynamic: bool,
    ) -> Result<BufferHandle> {
        let handle = self.generate_handle();
        let element_count = (data.len() / element_size as usize) as u32;

        let usage = if dynamic {
            BufferUsages::VERTEX | BufferUsages::COPY_DST
        } else {
            BufferUsages::VERTEX | BufferUsages::COPY_DST
        };

        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Vertex Buffer {}", handle.id())),
            size: data.len() as u64,
            usage,
            mapped_at_creation: false,
        });

        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Buffer Upload"),
            });

        let info = BufferInfo::new(
            BufferType::Vertex,
            data.len() as u64,
            element_count,
            element_size,
            dynamic,
        );

        self.buffers.insert(handle, buffer);
        self.buffer_info.insert(handle, info);

        Ok(handle)
    }

    /// Create an index buffer.
    pub fn create_index_buffer(&mut self, indices: &[u32], dynamic: bool) -> Result<BufferHandle> {
        let handle = self.generate_handle();
        let data = bytemuck::cast_slice(indices);

        let usage = if dynamic {
            BufferUsages::INDEX | BufferUsages::COPY_DST
        } else {
            BufferUsages::INDEX | BufferUsages::COPY_DST
        };

        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Index Buffer {}", handle.id())),
            size: data.len() as u64,
            usage,
            mapped_at_creation: false,
        });

        let info = BufferInfo::new(
            BufferType::Index,
            data.len() as u64,
            indices.len() as u32,
            std::mem::size_of::<u32>() as u32,
            dynamic,
        );

        self.buffers.insert(handle, buffer);
        self.buffer_info.insert(handle, info);

        Ok(handle)
    }

    /// Create a uniform buffer.
    pub fn create_uniform_buffer(&mut self, size: u64, label: &str) -> Result<BufferHandle> {
        let handle = self.generate_handle();

        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let info = BufferInfo::new(BufferType::Uniform, size, 1, size as u32, true);

        self.buffers.insert(handle, buffer);
        self.buffer_info.insert(handle, info);

        Ok(handle)
    }

    /// Create an instance buffer.
    pub fn create_instance_buffer(
        &mut self,
        data: &[u8],
        element_size: u32,
        dynamic: bool,
    ) -> Result<BufferHandle> {
        let handle = self.generate_handle();
        let element_count = (data.len() / element_size as usize) as u32;

        let usage = if dynamic {
            BufferUsages::VERTEX | BufferUsages::COPY_DST
        } else {
            BufferUsages::VERTEX | BufferUsages::COPY_DST
        };

        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Instance Buffer {}", handle.id())),
            size: data.len() as u64,
            usage,
            mapped_at_creation: false,
        });

        let info = BufferInfo::new(
            BufferType::Instance,
            data.len() as u64,
            element_count,
            element_size,
            dynamic,
        );

        self.buffers.insert(handle, buffer);
        self.buffer_info.insert(handle, info);

        Ok(handle)
    }

    /// Get a buffer by handle.
    pub fn get_buffer(&self, handle: BufferHandle) -> Option<&Buffer> {
        self.buffers.get(&handle)
    }

    /// Get buffer info.
    pub fn get_buffer_info(&self, handle: BufferHandle) -> Option<&BufferInfo> {
        self.buffer_info.get(&handle)
    }

    /// Update buffer data (for dynamic buffers).
    pub fn update_buffer(
        &self,
        handle: BufferHandle,
        queue: &wgpu::Queue,
        data: &[u8],
        offset: u64,
    ) -> Result<()> {
        let buffer = self
            .buffers
            .get(&handle)
            .ok_or_else(|| MapEngineError::Buffer("Buffer not found".to_string()))?;

        let info = self
            .buffer_info
            .get(&handle)
            .ok_or_else(|| MapEngineError::Buffer("Buffer info not found".to_string()))?;

        if !info.dynamic {
            return Err(MapEngineError::Buffer(
                "Cannot update static buffer".to_string(),
            ));
        }

        queue.write_buffer(buffer, offset, data);

        Ok(())
    }

    /// Delete a buffer.
    pub fn delete_buffer(&mut self, handle: BufferHandle) {
        self.buffers.remove(&handle);
        self.buffer_info.remove(&handle);
    }

    /// Generate a unique buffer handle.
    fn generate_handle(&mut self) -> BufferHandle {
        let handle = BufferHandle::new(self.next_handle_id);
        self.next_handle_id += 1;
        handle
    }

    /// Get buffer manager statistics.
    pub fn stats(&self) -> BufferStats {
        let mut stats = BufferStats::default();

        for info in self.buffer_info.values() {
            match info.buffer_type {
                BufferType::Vertex => {
                    stats.vertex_buffer_count += 1;
                    stats.vertex_buffer_size += info.size;
                }
                BufferType::Index => {
                    stats.index_buffer_count += 1;
                    stats.index_buffer_size += info.size;
                }
                BufferType::Uniform => {
                    stats.uniform_buffer_count += 1;
                    stats.uniform_buffer_size += info.size;
                }
                BufferType::Instance => {
                    stats.instance_buffer_count += 1;
                    stats.instance_buffer_size += info.size;
                }
            }
        }

        stats
    }

    /// Clear all buffers (useful for cleanup).
    pub fn clear(&mut self) {
        self.buffers.clear();
        self.buffer_info.clear();
    }
}

/// Statistics about buffer memory usage.
#[derive(Debug, Clone, Default)]
pub struct BufferStats {
    /// Number of vertex buffers.
    pub vertex_buffer_count: usize,
    /// Total size of vertex buffers in bytes.
    pub vertex_buffer_size: u64,
    /// Number of index buffers.
    pub index_buffer_count: usize,
    /// Total size of index buffers in bytes.
    pub index_buffer_size: u64,
    /// Number of uniform buffers.
    pub uniform_buffer_count: usize,
    /// Total size of uniform buffers in bytes.
    pub uniform_buffer_size: u64,
    /// Number of instance buffers.
    pub instance_buffer_count: usize,
    /// Total size of instance buffers in bytes.
    pub instance_buffer_size: u64,
}

impl BufferStats {
    /// Get total buffer count.
    pub fn total_count(&self) -> usize {
        self.vertex_buffer_count
            + self.index_buffer_count
            + self.uniform_buffer_count
            + self.instance_buffer_count
    }

    /// Get total buffer size in bytes.
    pub fn total_size(&self) -> u64 {
        self.vertex_buffer_size
            + self.index_buffer_size
            + self.uniform_buffer_size
            + self.instance_buffer_size
    }

    /// Get total buffer size in megabytes.
    pub fn total_size_mb(&self) -> f32 {
        self.total_size() as f32 / (1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_handle() {
        let handle1 = BufferHandle::new(0);
        let handle2 = BufferHandle::new(1);
        assert_ne!(handle1, handle2);
        assert_eq!(handle1.id(), 0);
        assert_eq!(handle2.id(), 1);
    }

    #[test]
    fn test_buffer_info() {
        let info = BufferInfo::new(BufferType::Vertex, 1024, 16, 64, true);
        assert_eq!(info.buffer_type, BufferType::Vertex);
        assert_eq!(info.size, 1024);
        assert_eq!(info.element_count, 16);
        assert_eq!(info.element_size, 64);
        assert!(info.dynamic);
    }

    #[test]
    fn test_buffer_stats() {
        let mut stats = BufferStats::default();
        stats.vertex_buffer_count = 2;
        stats.vertex_buffer_size = 2048;
        stats.index_buffer_count = 1;
        stats.index_buffer_size = 512;

        assert_eq!(stats.total_count(), 3);
        assert_eq!(stats.total_size(), 2560);
    }
}
