//! Streaming compression for large files
//!
//! Efficient handling of large files with memory-conscious streaming operations.

use crate::error::{CompressionError, Result};
use crate::{CompressionAlgorithm, Compressor};
use crate::{lz4::Lz4Compressor, zstd::ZstdCompressor, gzip::GzipCompressor};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use std::path::Path;
use std::pin::Pin;
use tokio::fs::File;

/// Streaming compression configuration
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Algorithm to use
    pub algorithm: CompressionAlgorithm,
    /// Compression level
    pub level: i32,
    /// Buffer size for streaming
    pub buffer_size: usize,
    /// Enable progress reporting
    pub report_progress: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Zstd,
            level: 3,
            buffer_size: 1024 * 1024, // 1 MB
            report_progress: false,
        }
    }
}

/// Progress information for streaming operations
#[derive(Debug, Clone)]
pub struct StreamProgress {
    /// Bytes processed so far
    pub bytes_processed: u64,
    /// Total bytes (if known)
    pub total_bytes: Option<u64>,
    /// Progress percentage (0-100)
    pub percentage: Option<f64>,
}

impl StreamProgress {
    /// Create new progress
    pub fn new(bytes_processed: u64, total_bytes: Option<u64>) -> Self {
        let percentage = total_bytes.map(|total| {
            if total > 0 {
                (bytes_processed as f64 / total as f64) * 100.0
            } else {
                0.0
            }
        });

        Self {
            bytes_processed,
            total_bytes,
            percentage,
        }
    }
}

/// Streaming compressor for large files
pub struct StreamingCompressor {
    config: StreamConfig,
}

impl StreamingCompressor {
    /// Create a new streaming compressor
    pub fn new(config: StreamConfig) -> Self {
        Self { config }
    }

    /// Create with algorithm
    pub fn with_algorithm(algorithm: CompressionAlgorithm) -> Self {
        Self::new(StreamConfig {
            algorithm,
            ..Default::default()
        })
    }

    /// Compress file to file
    pub async fn compress_file(
        &self,
        input_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
    ) -> Result<u64> {
        let mut input = File::open(input_path)
            .await
            .map_err(CompressionError::from)?;

        let mut output = File::create(output_path)
            .await
            .map_err(CompressionError::from)?;

        self.compress_stream(&mut input, &mut output).await
    }

    /// Decompress file to file
    pub async fn decompress_file(
        &self,
        input_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
    ) -> Result<u64> {
        let mut input = File::open(input_path)
            .await
            .map_err(CompressionError::from)?;

        let mut output = File::create(output_path)
            .await
            .map_err(CompressionError::from)?;

        self.decompress_stream(&mut input, &mut output).await
    }

    /// Compress from async reader to async writer
    pub async fn compress_stream<R, W>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<u64>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let mut buffer = vec![0u8; self.config.buffer_size];
        let mut total_bytes = 0u64;
        let mut all_data = Vec::new();

        // Read all data (chunked)
        loop {
            let bytes_read = reader
                .read(&mut buffer)
                .await
                .map_err(CompressionError::from)?;

            if bytes_read == 0 {
                break;
            }

            all_data.extend_from_slice(&buffer[..bytes_read]);
            total_bytes += bytes_read as u64;
        }

        // Compress the data
        let compressed = self.compress_data(&all_data)?;

        // Write compressed data
        writer
            .write_all(&compressed)
            .await
            .map_err(CompressionError::from)?;

        writer.flush().await.map_err(CompressionError::from)?;

        Ok(total_bytes)
    }

    /// Decompress from async reader to async writer
    pub async fn decompress_stream<R, W>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<u64>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let mut buffer = vec![0u8; self.config.buffer_size];
        let mut all_data = Vec::new();

        // Read all compressed data
        loop {
            let bytes_read = reader
                .read(&mut buffer)
                .await
                .map_err(CompressionError::from)?;

            if bytes_read == 0 {
                break;
            }

            all_data.extend_from_slice(&buffer[..bytes_read]);
        }

        // Decompress the data
        let decompressed = self.decompress_data(&all_data)?;

        // Write decompressed data
        writer
            .write_all(&decompressed)
            .await
            .map_err(CompressionError::from)?;

        writer.flush().await.map_err(CompressionError::from)?;

        Ok(decompressed.len() as u64)
    }

    /// Compress data using configured algorithm
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        match self.config.algorithm {
            CompressionAlgorithm::Lz4 => {
                Lz4Compressor::with_level(self.config.level).compress(data)
            }
            CompressionAlgorithm::Zstd => {
                ZstdCompressor::with_level(self.config.level).compress(data)
            }
            CompressionAlgorithm::Gzip => {
                GzipCompressor::with_level(self.config.level as u32).compress(data)
            }
            _ => Err(CompressionError::Streaming(
                format!("Algorithm {:?} not supported for streaming", self.config.algorithm),
            )),
        }
    }

    /// Decompress data using configured algorithm
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        match self.config.algorithm {
            CompressionAlgorithm::Lz4 => {
                Lz4Compressor::default().decompress(data)
            }
            CompressionAlgorithm::Zstd => {
                ZstdCompressor::default().decompress(data)
            }
            CompressionAlgorithm::Gzip => {
                GzipCompressor::default().decompress(data)
            }
            _ => Err(CompressionError::Streaming(
                format!("Algorithm {:?} not supported for streaming", self.config.algorithm),
            )),
        }
    }
}

/// Chunked streaming compressor for very large files
pub struct ChunkedStreamCompressor {
    config: StreamConfig,
    chunk_size: usize,
}

impl ChunkedStreamCompressor {
    /// Create a new chunked compressor
    pub fn new(config: StreamConfig, chunk_size: usize) -> Self {
        Self {
            config,
            chunk_size,
        }
    }

    /// Compress file in chunks
    pub async fn compress_file_chunked(
        &self,
        input_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
    ) -> Result<ChunkedCompressionResult> {
        let mut input = File::open(input_path)
            .await
            .map_err(CompressionError::from)?;

        let mut output = File::create(output_path)
            .await
            .map_err(CompressionError::from)?;

        let metadata = input
            .metadata()
            .await
            .map_err(CompressionError::from)?;

        let total_size = metadata.len();
        let mut chunks_compressed = 0u64;
        let mut total_compressed_size = 0u64;
        let mut buffer = vec![0u8; self.chunk_size];

        loop {
            let bytes_read = input
                .read(&mut buffer)
                .await
                .map_err(CompressionError::from)?;

            if bytes_read == 0 {
                break;
            }

            // Compress chunk
            let chunk_data = &buffer[..bytes_read];
            let compressed_chunk = self.compress_chunk(chunk_data)?;

            // Write chunk header (size)
            let chunk_size = compressed_chunk.len() as u64;
            output
                .write_all(&chunk_size.to_le_bytes())
                .await
                .map_err(CompressionError::from)?;

            // Write compressed chunk
            output
                .write_all(&compressed_chunk)
                .await
                .map_err(CompressionError::from)?;

            chunks_compressed += 1;
            total_compressed_size += chunk_size + 8; // +8 for header
        }

        output.flush().await.map_err(CompressionError::from)?;

        Ok(ChunkedCompressionResult {
            total_size,
            compressed_size: total_compressed_size,
            chunks_count: chunks_compressed,
        })
    }

    /// Decompress chunked file
    pub async fn decompress_file_chunked(
        &self,
        input_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
    ) -> Result<u64> {
        let mut input = File::open(input_path)
            .await
            .map_err(CompressionError::from)?;

        let mut output = File::create(output_path)
            .await
            .map_err(CompressionError::from)?;

        let mut total_decompressed = 0u64;

        loop {
            // Read chunk size
            let mut size_buf = [0u8; 8];
            match input.read_exact(&mut size_buf).await {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(CompressionError::from(e)),
            }

            let chunk_size = u64::from_le_bytes(size_buf) as usize;

            // Read compressed chunk
            let mut chunk_data = vec![0u8; chunk_size];
            input
                .read_exact(&mut chunk_data)
                .await
                .map_err(CompressionError::from)?;

            // Decompress chunk
            let decompressed = self.decompress_chunk(&chunk_data)?;

            // Write decompressed chunk
            output
                .write_all(&decompressed)
                .await
                .map_err(CompressionError::from)?;

            total_decompressed += decompressed.len() as u64;
        }

        output.flush().await.map_err(CompressionError::from)?;

        Ok(total_decompressed)
    }

    /// Compress a single chunk
    fn compress_chunk(&self, data: &[u8]) -> Result<Vec<u8>> {
        match self.config.algorithm {
            CompressionAlgorithm::Lz4 => {
                Lz4Compressor::with_level(self.config.level).compress(data)
            }
            CompressionAlgorithm::Zstd => {
                ZstdCompressor::with_level(self.config.level).compress(data)
            }
            CompressionAlgorithm::Gzip => {
                GzipCompressor::with_level(self.config.level as u32).compress(data)
            }
            _ => Err(CompressionError::Streaming(
                format!("Algorithm {:?} not supported", self.config.algorithm),
            )),
        }
    }

    /// Decompress a single chunk
    fn decompress_chunk(&self, data: &[u8]) -> Result<Vec<u8>> {
        match self.config.algorithm {
            CompressionAlgorithm::Lz4 => Lz4Compressor::default().decompress(data),
            CompressionAlgorithm::Zstd => ZstdCompressor::default().decompress(data),
            CompressionAlgorithm::Gzip => GzipCompressor::default().decompress(data),
            _ => Err(CompressionError::Streaming(
                format!("Algorithm {:?} not supported", self.config.algorithm),
            )),
        }
    }
}

/// Result of chunked compression
#[derive(Debug, Clone)]
pub struct ChunkedCompressionResult {
    pub total_size: u64,
    pub compressed_size: u64,
    pub chunks_count: u64,
}

impl ChunkedCompressionResult {
    /// Get compression ratio
    pub fn compression_ratio(&self) -> f64 {
        if self.compressed_size > 0 {
            self.total_size as f64 / self.compressed_size as f64
        } else {
            0.0
        }
    }

    /// Get space savings percentage
    pub fn space_savings_percent(&self) -> f64 {
        if self.total_size > 0 {
            ((self.total_size - self.compressed_size) as f64 / self.total_size as f64) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_streaming_compression() {
        let config = StreamConfig {
            algorithm: CompressionAlgorithm::Zstd,
            level: 3,
            ..Default::default()
        };

        let compressor = StreamingCompressor::new(config);

        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        // Write test data
        let test_data = b"Test data for streaming compression" as &[u8];
        let test_data = test_data.repeat(1000);
        tokio::fs::write(input_file.path(), &test_data)
            .await
            .unwrap();

        // Compress
        let bytes_processed = compressor
            .compress_file(input_file.path(), output_file.path())
            .await
            .unwrap();

        assert_eq!(bytes_processed, test_data.len() as u64);

        // Verify compressed file exists and is smaller
        let compressed_size = tokio::fs::metadata(output_file.path())
            .await
            .unwrap()
            .len();

        assert!(compressed_size < test_data.len() as u64);
    }

    #[tokio::test]
    async fn test_streaming_roundtrip() {
        let config = StreamConfig {
            algorithm: CompressionAlgorithm::Lz4,
            ..Default::default()
        };

        let compressor = StreamingCompressor::new(config);

        let input_file = NamedTempFile::new().unwrap();
        let compressed_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        let test_data = b"Roundtrip test data";
        tokio::fs::write(input_file.path(), test_data)
            .await
            .unwrap();

        // Compress
        compressor
            .compress_file(input_file.path(), compressed_file.path())
            .await
            .unwrap();

        // Decompress
        compressor
            .decompress_file(compressed_file.path(), output_file.path())
            .await
            .unwrap();

        // Verify
        let result = tokio::fs::read(output_file.path()).await.unwrap();
        assert_eq!(result, test_data);
    }

    #[tokio::test]
    async fn test_chunked_compression() {
        let config = StreamConfig {
            algorithm: CompressionAlgorithm::Zstd,
            level: 3,
            ..Default::default()
        };

        let compressor = ChunkedStreamCompressor::new(config, 1024);

        let input_file = NamedTempFile::new().unwrap();
        let compressed_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        let test_data = b"x".repeat(10000);
        tokio::fs::write(input_file.path(), &test_data)
            .await
            .unwrap();

        // Compress
        let result = compressor
            .compress_file_chunked(input_file.path(), compressed_file.path())
            .await
            .unwrap();

        assert!(result.chunks_count > 0);
        assert!(result.compression_ratio() > 1.0);

        // Decompress
        let decompressed_size = compressor
            .decompress_file_chunked(compressed_file.path(), output_file.path())
            .await
            .unwrap();

        assert_eq!(decompressed_size, test_data.len() as u64);

        // Verify
        let result_data = tokio::fs::read(output_file.path()).await.unwrap();
        assert_eq!(result_data, test_data);
    }
}
