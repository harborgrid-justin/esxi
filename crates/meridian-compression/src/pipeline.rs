//! Multi-stage compression pipeline
//!
//! Chain multiple compression algorithms for specialized use cases.

use crate::error::{CompressionError, Result};
use crate::stats::{CompressionStats, Timer};
use crate::{CompressionPipeline, Compressor};
use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Pipeline stage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    /// Stage name/identifier
    pub name: String,
    /// Algorithm used
    pub algorithm: String,
    /// Stage index
    pub index: usize,
}

/// Pipeline execution result
#[derive(Debug, Clone)]
pub struct PipelineResult {
    /// Final compressed data
    pub data: Vec<u8>,
    /// Statistics for each stage
    pub stage_stats: Vec<(String, CompressionStats)>,
    /// Total pipeline time
    pub total_time: std::time::Duration,
}

impl PipelineResult {
    /// Get overall compression ratio
    pub fn overall_ratio(&self) -> f64 {
        if let (Some(first), Some(last)) = (self.stage_stats.first(), self.stage_stats.last()) {
            first.1.original_size as f64 / last.1.compressed_size as f64
        } else {
            1.0
        }
    }

    /// Get total throughput
    pub fn throughput_mbps(&self) -> f64 {
        if let Some(first) = self.stage_stats.first() {
            if self.total_time.as_secs_f64() > 0.0 {
                (first.1.original_size as f64 / 1_000_000.0) / self.total_time.as_secs_f64()
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}

/// Multi-stage compression pipeline implementation
pub struct CompressionPipelineImpl {
    /// Ordered stages
    stages: Vec<(String, Box<dyn Compressor>)>,
    /// Enable statistics collection
    collect_stats: bool,
}

impl Default for CompressionPipelineImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl CompressionPipelineImpl {
    /// Create a new empty pipeline
    pub fn new() -> Self {
        Self {
            stages: Vec::new(),
            collect_stats: true,
        }
    }

    /// Create with statistics collection setting
    pub fn with_stats(collect_stats: bool) -> Self {
        Self {
            stages: Vec::new(),
            collect_stats,
        }
    }

    /// Add a named stage to the pipeline
    pub fn add_named_stage(
        &mut self,
        name: impl Into<String>,
        compressor: Box<dyn Compressor>,
    ) -> Result<()> {
        self.stages.push((name.into(), compressor));
        Ok(())
    }

    /// Execute pipeline with detailed results
    pub async fn execute_with_stats(&self, data: Bytes) -> Result<PipelineResult> {
        if self.stages.is_empty() {
            return Err(CompressionError::Pipeline {
                stage: "initialization".to_string(),
                error: "Pipeline has no stages".to_string(),
            });
        }

        let total_timer = Timer::start("pipeline_total");
        let mut current_data = data.to_vec();
        let mut stage_stats = Vec::new();

        for (index, (name, compressor)) in self.stages.iter().enumerate() {
            let stage_timer = Timer::start(format!("stage_{}", name));

            let compressed = compressor.compress(&current_data).map_err(|e| {
                CompressionError::Pipeline {
                    stage: name.clone(),
                    error: e.to_string(),
                }
            })?;

            let elapsed = stage_timer.stop();

            if self.collect_stats {
                let stats = CompressionStats::new(
                    current_data.len(),
                    compressed.len(),
                    elapsed,
                    name.clone(),
                );
                stage_stats.push((name.clone(), stats));
            }

            current_data = compressed;
        }

        let total_time = total_timer.stop();

        Ok(PipelineResult {
            data: current_data,
            stage_stats,
            total_time,
        })
    }

    /// Execute pipeline in reverse (decompression)
    pub async fn execute_reverse_with_stats(&self, data: Bytes) -> Result<PipelineResult> {
        if self.stages.is_empty() {
            return Err(CompressionError::Pipeline {
                stage: "initialization".to_string(),
                error: "Pipeline has no stages".to_string(),
            });
        }

        let total_timer = Timer::start("pipeline_reverse_total");
        let mut current_data = data.to_vec();
        let mut stage_stats = Vec::new();

        // Execute stages in reverse order
        for (index, (name, compressor)) in self.stages.iter().enumerate().rev() {
            let stage_timer = Timer::start(format!("stage_{}_decompress", name));

            let decompressed = compressor.decompress(&current_data).map_err(|e| {
                CompressionError::Pipeline {
                    stage: format!("{}_decompress", name),
                    error: e.to_string(),
                }
            })?;

            let elapsed = stage_timer.stop();

            if self.collect_stats {
                let stats = CompressionStats::new(
                    decompressed.len(),
                    current_data.len(),
                    elapsed,
                    format!("{}_decompress", name),
                );
                stage_stats.push((format!("{}_decompress", name), stats));
            }

            current_data = decompressed;
        }

        let total_time = total_timer.stop();

        Ok(PipelineResult {
            data: current_data,
            stage_stats,
            total_time,
        })
    }

    /// Get pipeline stage information
    pub fn get_stages(&self) -> Vec<PipelineStage> {
        self.stages
            .iter()
            .enumerate()
            .map(|(index, (name, compressor))| PipelineStage {
                name: name.clone(),
                algorithm: compressor.algorithm().to_string(),
                index,
            })
            .collect()
    }
}

#[async_trait]
impl CompressionPipeline for CompressionPipelineImpl {
    fn add_stage(&mut self, compressor: Box<dyn Compressor>) -> Result<()> {
        let name = format!("stage_{}", self.stages.len());
        self.stages.push((name, compressor));
        Ok(())
    }

    async fn execute(&self, data: Bytes) -> Result<Bytes> {
        let result = self.execute_with_stats(data).await?;
        Ok(Bytes::from(result.data))
    }

    async fn execute_reverse(&self, data: Bytes) -> Result<Bytes> {
        let result = self.execute_reverse_with_stats(data).await?;
        Ok(Bytes::from(result.data))
    }

    fn stage_count(&self) -> usize {
        self.stages.len()
    }

    fn clear(&mut self) {
        self.stages.clear();
    }
}

/// Pipeline builder for fluent API
pub struct PipelineBuilder {
    pipeline: CompressionPipelineImpl,
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineBuilder {
    /// Create a new pipeline builder
    pub fn new() -> Self {
        Self {
            pipeline: CompressionPipelineImpl::new(),
        }
    }

    /// Add a stage with name
    pub fn add_stage(
        mut self,
        name: impl Into<String>,
        compressor: Box<dyn Compressor>,
    ) -> Self {
        let _ = self.pipeline.add_named_stage(name, compressor);
        self
    }

    /// Enable or disable statistics collection
    pub fn with_stats(mut self, enable: bool) -> Self {
        self.pipeline.collect_stats = enable;
        self
    }

    /// Build the pipeline
    pub fn build(self) -> CompressionPipelineImpl {
        self.pipeline
    }
}

/// Pre-configured pipeline templates
pub struct PipelineTemplates;

impl PipelineTemplates {
    /// Delta + Zstandard pipeline for versioned data
    pub fn delta_zstd() -> CompressionPipelineImpl {
        use crate::delta::DeltaCompressor;
        use crate::zstd::ZstdCompressor;

        let mut pipeline = CompressionPipelineImpl::new();
        // Note: Delta compression requires source data, so this is simplified
        pipeline
            .add_named_stage("zstd", Box::new(ZstdCompressor::with_level(3)))
            .unwrap();
        pipeline
    }

    /// LZ4 + Zstandard pipeline for balanced compression
    pub fn lz4_zstd() -> CompressionPipelineImpl {
        use crate::lz4::Lz4Compressor;
        use crate::zstd::ZstdCompressor;

        let mut pipeline = CompressionPipelineImpl::new();
        pipeline
            .add_named_stage("lz4", Box::new(Lz4Compressor::with_level(4)))
            .unwrap();
        pipeline
            .add_named_stage("zstd", Box::new(ZstdCompressor::with_level(1)))
            .unwrap();
        pipeline
    }

    /// Maximum compression pipeline
    pub fn maximum_compression() -> CompressionPipelineImpl {
        use crate::zstd::ZstdCompressor;
        use crate::brotli::BrotliCompressor;

        let mut pipeline = CompressionPipelineImpl::new();
        pipeline
            .add_named_stage("zstd", Box::new(ZstdCompressor::with_level(19)))
            .unwrap();
        pipeline
            .add_named_stage("brotli", Box::new(BrotliCompressor::with_level(11)))
            .unwrap();
        pipeline
    }

    /// Fast compression pipeline
    pub fn fast_compression() -> CompressionPipelineImpl {
        use crate::snappy::SnappyCompressor;
        use crate::lz4::Lz4Compressor;

        let mut pipeline = CompressionPipelineImpl::new();
        pipeline
            .add_named_stage("snappy", Box::new(SnappyCompressor::new()))
            .unwrap();
        pipeline
            .add_named_stage("lz4", Box::new(Lz4Compressor::with_level(1)))
            .unwrap();
        pipeline
    }
}

/// Parallel pipeline executor
pub struct ParallelPipeline {
    pipelines: Vec<Arc<CompressionPipelineImpl>>,
}

impl ParallelPipeline {
    /// Create a new parallel pipeline
    pub fn new() -> Self {
        Self {
            pipelines: Vec::new(),
        }
    }

    /// Add a pipeline variant
    pub fn add_variant(&mut self, pipeline: CompressionPipelineImpl) {
        self.pipelines.push(Arc::new(pipeline));
    }

    /// Execute all pipelines in parallel and return best result
    pub async fn execute_best(&self, data: Bytes) -> Result<PipelineResult> {
        use rayon::prelude::*;

        if self.pipelines.is_empty() {
            return Err(CompressionError::Pipeline {
                stage: "initialization".to_string(),
                error: "No pipeline variants".to_string(),
            });
        }

        let data_arc = Arc::new(data);

        // Execute all pipelines in parallel
        let results: Vec<Result<PipelineResult>> = self
            .pipelines
            .par_iter()
            .map(|pipeline| {
                let data_clone = Bytes::from(data_arc.as_ref().clone());
                let pipeline_clone = Arc::clone(pipeline);

                // Use tokio runtime for async execution
                let runtime = tokio::runtime::Handle::current();
                runtime.block_on(async move {
                    pipeline_clone.execute_with_stats(data_clone).await
                })
            })
            .collect();

        // Find best result (smallest compressed size)
        let mut best: Option<PipelineResult> = None;
        let mut best_size = usize::MAX;

        for result in results {
            if let Ok(r) = result {
                if r.data.len() < best_size {
                    best_size = r.data.len();
                    best = Some(r);
                }
            }
        }

        best.ok_or_else(|| {
            CompressionError::Pipeline {
                stage: "parallel_execution".to_string(),
                error: "All pipelines failed".to_string(),
            }
        })
    }
}

impl Default for ParallelPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lz4::Lz4Compressor;
    use crate::zstd::ZstdCompressor;

    #[tokio::test]
    async fn test_pipeline_execution() {
        let mut pipeline = CompressionPipelineImpl::new();

        pipeline
            .add_named_stage("lz4", Box::new(Lz4Compressor::with_level(4)))
            .unwrap();

        pipeline
            .add_named_stage("zstd", Box::new(ZstdCompressor::with_level(3)))
            .unwrap();

        let data = Bytes::from("Test data for pipeline compression");
        let result = pipeline.execute_with_stats(data.clone()).await.unwrap();

        assert!(result.data.len() < data.len());
        assert_eq!(result.stage_stats.len(), 2);
    }

    #[tokio::test]
    async fn test_pipeline_roundtrip() {
        let mut pipeline = CompressionPipelineImpl::new();

        pipeline
            .add_stage(Box::new(Lz4Compressor::with_level(4)))
            .unwrap();

        pipeline
            .add_stage(Box::new(ZstdCompressor::with_level(3)))
            .unwrap();

        let original = Bytes::from("Roundtrip test data for pipeline");
        let compressed = pipeline.execute(original.clone()).await.unwrap();
        let decompressed = pipeline.execute_reverse(compressed).await.unwrap();

        assert_eq!(original, decompressed);
    }

    #[tokio::test]
    async fn test_pipeline_builder() {
        let pipeline = PipelineBuilder::new()
            .add_stage("lz4", Box::new(Lz4Compressor::new()))
            .add_stage("zstd", Box::new(ZstdCompressor::new()))
            .with_stats(true)
            .build();

        assert_eq!(pipeline.stage_count(), 2);

        let stages = pipeline.get_stages();
        assert_eq!(stages[0].name, "lz4");
        assert_eq!(stages[1].name, "zstd");
    }

    #[tokio::test]
    async fn test_template_pipelines() {
        let data = Bytes::from("Test data" as &[u8]).repeat(100);

        // Test LZ4 + Zstd template
        let pipeline = PipelineTemplates::lz4_zstd();
        let result = pipeline.execute(data.clone()).await.unwrap();
        assert!(result.len() < data.len());

        // Test fast compression template
        let pipeline = PipelineTemplates::fast_compression();
        let result = pipeline.execute(data.clone()).await.unwrap();
        assert!(result.len() < data.len());
    }

    #[tokio::test]
    async fn test_parallel_pipeline() {
        let mut parallel = ParallelPipeline::new();

        parallel.add_variant(PipelineTemplates::fast_compression());
        parallel.add_variant(PipelineTemplates::lz4_zstd());

        let data = Bytes::from("Test data for parallel pipeline" as &[u8]).repeat(100);
        let result = parallel.execute_best(data.clone()).await.unwrap();

        assert!(result.data.len() < data.len());
        assert!(result.overall_ratio() > 1.0);
    }

    #[tokio::test]
    async fn test_empty_pipeline() {
        let pipeline = CompressionPipelineImpl::new();
        let data = Bytes::from("Test");

        let result = pipeline.execute(data).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pipeline_stats() {
        let mut pipeline = CompressionPipelineImpl::new();

        pipeline
            .add_named_stage("stage1", Box::new(Lz4Compressor::new()))
            .unwrap();

        let data = Bytes::from("Test data" as &[u8]).repeat(100);
        let result = pipeline.execute_with_stats(data).await.unwrap();

        assert!(result.throughput_mbps() > 0.0);
        assert!(result.overall_ratio() > 1.0);
    }
}
