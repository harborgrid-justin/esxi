//! Binary protocol using MessagePack

use bytes::{Buf, BufMut, BytesMut};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::protocol::{Message, FrameHeader, PROTOCOL_MAGIC, PROTOCOL_VERSION};

/// Binary protocol encoder/decoder
pub struct BinaryProtocol;

impl BinaryProtocol {
    /// Encode message to binary format
    pub fn encode(message: &Message) -> Result<Vec<u8>> {
        // Serialize message with MessagePack
        let payload = rmp_serde::to_vec(message)?;

        // Calculate checksum (CRC32)
        let checksum = crc32fast::hash(&payload);

        // Create frame header
        let header = FrameHeader::new(payload.len() as u32).with_checksum(checksum);

        // Serialize header
        let header_bytes = rmp_serde::to_vec(&header)?;

        // Combine header and payload
        let mut buffer = BytesMut::with_capacity(header_bytes.len() + payload.len());
        buffer.put_slice(&header_bytes);
        buffer.put_slice(&payload);

        Ok(buffer.to_vec())
    }

    /// Decode binary format to message
    pub fn decode(data: &[u8]) -> Result<Message> {
        let mut cursor = std::io::Cursor::new(data);

        // Deserialize header
        let header: FrameHeader = rmp_serde::from_read(&mut cursor)
            .map_err(|e| Error::Protocol(format!("Failed to decode header: {}", e)))?;

        // Validate header
        if !header.validate() {
            return Err(Error::Protocol(format!(
                "Invalid protocol header: magic={:x}, version={}",
                header.magic, header.version
            )));
        }

        // Get remaining data (payload)
        let position = cursor.position() as usize;
        let payload = &data[position..];

        // Validate length
        if payload.len() != header.length as usize {
            return Err(Error::Protocol(format!(
                "Payload length mismatch: expected {}, got {}",
                header.length,
                payload.len()
            )));
        }

        // Validate checksum if present
        if let Some(expected_checksum) = header.checksum {
            let actual_checksum = crc32fast::hash(payload);
            if actual_checksum != expected_checksum {
                return Err(Error::Protocol(format!(
                    "Checksum mismatch: expected {}, got {}",
                    expected_checksum, actual_checksum
                )));
            }
        }

        // Deserialize message
        let message: Message = rmp_serde::from_slice(payload)
            .map_err(|e| Error::Protocol(format!("Failed to decode message: {}", e)))?;

        Ok(message)
    }

    /// Get encoded size estimate
    pub fn estimate_size(message: &Message) -> usize {
        // Rough estimate: header (32 bytes) + payload
        32 + message.payload.len()
    }
}

/// Message encoder
pub struct Encoder {
    /// Compression enabled
    compress: bool,

    /// Compression level (0-9)
    compression_level: u32,
}

impl Encoder {
    /// Create new encoder
    pub fn new() -> Self {
        Self {
            compress: false,
            compression_level: 6,
        }
    }

    /// Enable compression
    pub fn with_compression(mut self, level: u32) -> Self {
        self.compress = true;
        self.compression_level = level.min(9);
        self
    }

    /// Encode message
    pub fn encode(&self, message: &Message) -> Result<Vec<u8>> {
        let data = BinaryProtocol::encode(message)?;

        if self.compress {
            self.compress_data(&data)
        } else {
            Ok(data)
        }
    }

    /// Compress data using gzip
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(
            Vec::new(),
            Compression::new(self.compression_level),
        );
        encoder
            .write_all(data)
            .map_err(|e| Error::Internal(format!("Compression failed: {}", e)))?;

        encoder
            .finish()
            .map_err(|e| Error::Internal(format!("Compression failed: {}", e)))
    }
}

impl Default for Encoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Message decoder
pub struct Decoder {
    /// Decompress enabled
    decompress: bool,
}

impl Decoder {
    /// Create new decoder
    pub fn new() -> Self {
        Self {
            decompress: false,
        }
    }

    /// Enable decompression
    pub fn with_decompression(mut self) -> Self {
        self.decompress = true;
        self
    }

    /// Decode message
    pub fn decode(&self, data: &[u8]) -> Result<Message> {
        let data = if self.decompress {
            self.decompress_data(data)?
        } else {
            data.to_vec()
        };

        BinaryProtocol::decode(&data)
    }

    /// Decompress data using gzip
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();

        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| Error::Internal(format!("Decompression failed: {}", e)))?;

        Ok(decompressed)
    }
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Streaming decoder for parsing messages from a byte stream
pub struct StreamDecoder {
    /// Internal buffer
    buffer: BytesMut,

    /// Maximum message size
    max_size: usize,
}

impl StreamDecoder {
    /// Create new stream decoder
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(8192),
            max_size,
        }
    }

    /// Feed data into decoder
    pub fn feed(&mut self, data: &[u8]) -> Result<()> {
        if self.buffer.len() + data.len() > self.max_size {
            return Err(Error::Protocol(format!(
                "Message too large: {} bytes (max: {})",
                self.buffer.len() + data.len(),
                self.max_size
            )));
        }

        self.buffer.extend_from_slice(data);
        Ok(())
    }

    /// Try to decode next message
    pub fn next_message(&mut self) -> Result<Option<Message>> {
        if self.buffer.is_empty() {
            return Ok(None);
        }

        // Try to decode message
        match BinaryProtocol::decode(&self.buffer) {
            Ok(message) => {
                // Calculate how many bytes were consumed
                // For now, clear the buffer (simplified)
                self.buffer.clear();
                Ok(Some(message))
            }
            Err(Error::Protocol(_)) => {
                // Not enough data or invalid, wait for more
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Clear buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get buffer length
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{Message, MessageType};

    #[test]
    fn test_binary_protocol_encode_decode() {
        let msg = Message::new(MessageType::Data, vec![1, 2, 3, 4, 5]);

        let encoded = BinaryProtocol::encode(&msg).unwrap();
        assert!(!encoded.is_empty());

        let decoded = BinaryProtocol::decode(&encoded).unwrap();
        assert_eq!(decoded.msg_type, msg.msg_type);
        assert_eq!(decoded.payload, msg.payload);
    }

    #[test]
    fn test_encoder_decoder() {
        let msg = Message::new(MessageType::Data, vec![1, 2, 3]);

        let encoder = Encoder::new();
        let encoded = encoder.encode(&msg).unwrap();

        let decoder = Decoder::new();
        let decoded = decoder.decode(&encoded).unwrap();

        assert_eq!(decoded.msg_type, msg.msg_type);
        assert_eq!(decoded.payload, msg.payload);
    }

    #[test]
    fn test_compression() {
        let large_payload = vec![42u8; 10000];
        let msg = Message::new(MessageType::Data, large_payload);

        let encoder_no_compress = Encoder::new();
        let encoded_no_compress = encoder_no_compress.encode(&msg).unwrap();

        let encoder_compress = Encoder::new().with_compression(6);
        let encoded_compress = encoder_compress.encode(&msg).unwrap();

        // Compressed should be smaller
        assert!(encoded_compress.len() < encoded_no_compress.len());

        // Should decode correctly
        let decoder = Decoder::new().with_decompression();
        let decoded = decoder.decode(&encoded_compress).unwrap();
        assert_eq!(decoded.payload, msg.payload);
    }

    #[test]
    fn test_stream_decoder() {
        let msg = Message::new(MessageType::Data, vec![1, 2, 3]);
        let encoded = BinaryProtocol::encode(&msg).unwrap();

        let mut decoder = StreamDecoder::new(1024 * 1024);

        // Feed data in chunks
        let chunk_size = encoded.len() / 2;
        decoder.feed(&encoded[..chunk_size]).unwrap();

        // Not enough data yet
        assert!(decoder.next_message().unwrap().is_none());

        // Feed rest
        decoder.feed(&encoded[chunk_size..]).unwrap();

        // Should have complete message now
        let decoded = decoder.next_message().unwrap().unwrap();
        assert_eq!(decoded.msg_type, msg.msg_type);
    }

    #[test]
    fn test_invalid_protocol() {
        let invalid_data = vec![0xFF; 100];
        let result = BinaryProtocol::decode(&invalid_data);

        assert!(result.is_err());
    }
}
