//! Wire protocol definitions for real-time communication

pub mod message;
pub mod binary;

pub use message::{Message, MessageType, MessagePriority};
pub use binary::{BinaryProtocol, Encoder, Decoder};

use serde::{Deserialize, Serialize};

/// Protocol version
pub const PROTOCOL_VERSION: u16 = 1;

/// Magic number for protocol identification
pub const PROTOCOL_MAGIC: u32 = 0x4D455249; // "MERI"

/// Maximum protocol message size (10MB)
pub const MAX_PROTOCOL_MESSAGE_SIZE: usize = 10 * 1024 * 1024;

/// Protocol frame header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameHeader {
    /// Magic number
    pub magic: u32,

    /// Protocol version
    pub version: u16,

    /// Message length
    pub length: u32,

    /// Checksum (CRC32)
    pub checksum: Option<u32>,
}

impl FrameHeader {
    /// Create new frame header
    pub fn new(length: u32) -> Self {
        Self {
            magic: PROTOCOL_MAGIC,
            version: PROTOCOL_VERSION,
            length,
            checksum: None,
        }
    }

    /// With checksum
    pub fn with_checksum(mut self, checksum: u32) -> Self {
        self.checksum = Some(checksum);
        self
    }

    /// Validate header
    pub fn validate(&self) -> bool {
        self.magic == PROTOCOL_MAGIC && self.version == PROTOCOL_VERSION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_header() {
        let header = FrameHeader::new(1024).with_checksum(12345);

        assert_eq!(header.magic, PROTOCOL_MAGIC);
        assert_eq!(header.version, PROTOCOL_VERSION);
        assert_eq!(header.length, 1024);
        assert_eq!(header.checksum, Some(12345));
        assert!(header.validate());
    }
}
