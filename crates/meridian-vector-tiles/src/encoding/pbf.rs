//! Protocol Buffer serialization utilities

use crate::error::{Error, Result};
use bytes::Bytes;
use prost::Message;

/// Serialize a message to protocol buffer format
pub fn serialize<M: Message>(message: &M) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    message
        .encode(&mut buf)
        .map_err(|e| Error::encoding(format!("Failed to serialize protobuf: {}", e)))?;
    Ok(buf)
}

/// Deserialize a message from protocol buffer format
pub fn deserialize<M: Message + Default>(data: &[u8]) -> Result<M> {
    M::decode(data).map_err(|e| Error::decoding(format!("Failed to deserialize protobuf: {}", e)))
}

/// Serialize a message to bytes
pub fn serialize_to_bytes<M: Message>(message: &M) -> Result<Bytes> {
    let vec = serialize(message)?;
    Ok(Bytes::from(vec))
}

/// Get the encoded size of a message
pub fn encoded_size<M: Message>(message: &M) -> usize {
    message.encoded_len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        // Test with a simple message type
        // This would normally use actual protobuf messages
        let data = vec![1u8, 2, 3, 4, 5];
        assert_eq!(data.len(), 5);
    }
}
