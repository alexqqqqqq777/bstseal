// src/block_coder.rs
//! Dispatches between different block-level compression algorithms.

use crate::{huff, raw};
use anyhow::{anyhow, Result};

pub const BLOCK_SIZE: usize = 4096;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockType {
    Raw = 0,
    Huffman = 1,
}

impl TryFrom<u8> for BlockType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(BlockType::Raw),
            1 => Ok(BlockType::Huffman),
            _ => Err(anyhow!("Unknown block type: {}", value)),
        }
    }
}

/// Encodes a single block of data.
///
/// It attempts to compress the data using Huffman coding. If the compressed
/// data is not smaller than the raw (uncompressed) representation, it will
/// use the raw format as a fallback. This prevents data inflation for
/// incompressible data.
pub fn encode_block(input: &[u8]) -> Result<Vec<u8>> {
    if input.is_empty() {
        let raw_encoded = raw::encode(input)?;
        let mut final_block = Vec::with_capacity(1 + raw_encoded.len());
        final_block.push(BlockType::Raw as u8);
        final_block.extend_from_slice(&raw_encoded);
        return Ok(final_block);
    }

    // Attempt Huffman encoding.
    let huff_encoded = huff::encode(input)?;

    // The raw encoder just prepends the length. Inflation is minimal.
    // We compare the total size of the Huffman-encoded payload vs the raw input size.
    // The threshold of 1.03 is implicitly handled by this comparison, as raw encoding
    // adds only a few bytes for the length, which is far less than a 3% increase.
    if huff_encoded.len() < input.len() {
        // Huffman was successful and produced a smaller output.
        let mut final_block = Vec::with_capacity(1 + 10 + huff_encoded.len()); // +10 for varint
        final_block.push(BlockType::Huffman as u8);
        
        // Добавляем размер исходных данных как VarInt
        crate::utils::write_varint_u64(&mut final_block, input.len() as u64)?;
        
        final_block.extend_from_slice(&huff_encoded);
        Ok(final_block)
    } else {
        // Huffman did not provide a benefit, or inflated the data. Use raw.
        let raw_encoded = raw::encode(input)?;
        let mut final_block = Vec::with_capacity(1 + raw_encoded.len());
        final_block.push(BlockType::Raw as u8);
        final_block.extend_from_slice(&raw_encoded);
        Ok(final_block)
    }
}

/// Decodes a single block of data.
///
/// It reads a `BlockType` byte to determine the encoding format (Huffman or raw)
/// and dispatches to the appropriate decoder.
pub fn decode_block(input: &[u8]) -> Result<Vec<u8>> {
    if input.is_empty() {
        return Err(anyhow!("Input to decode_block cannot be empty."));
    }

    let block_type = BlockType::try_from(input[0])?;
    let payload = &input[1..];

    match block_type {
        BlockType::Raw => raw::decode(payload),
        BlockType::Huffman => {
            // Извлекаем размер из VarInt в начале данных
            let (expected_size, bytes_read) = crate::utils::read_varint_u64(payload)
                .ok_or_else(|| anyhow!("Failed to read varint for expected size"))?;

            let mut out = Vec::with_capacity(expected_size as usize);
            huff::decode(&payload[bytes_read..], &mut out, Some(expected_size as usize))?;
            Ok(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip_compressible() {
        let data = b"hello hello hello, this is a test of the huffman coding system".repeat(10);
        let encoded = encode_block(&data).unwrap();
        let decoded = decode_block(&encoded).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
        // Check that it chose Huffman
        assert_eq!(encoded[0], BlockType::Huffman as u8);
    }

    #[test]
    fn test_encode_decode_roundtrip_incompressible() {
        // This data is random and should not be compressible
        let data: Vec<u8> = (0..1024).map(|i| (i * 13 % 256) as u8).collect();
        let encoded = encode_block(&data).unwrap();
        let decoded = decode_block(&encoded).unwrap();
        assert_eq!(data, decoded);
        // Check that it chose Raw
        assert_eq!(encoded[0], BlockType::Raw as u8);
    }

    #[test]
    fn test_empty_block() {
        let data = b"";
        let encoded = encode_block(data).unwrap();
        let decoded = decode_block(&encoded).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
        assert_eq!(encoded[0], BlockType::Raw as u8);
    }

    #[test]
    fn test_single_byte_block() {
        let data = b"a";
        let encoded = encode_block(data).unwrap();
        let decoded = decode_block(&encoded).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
        // For a single byte, huffman might be larger due to header
        // The logic should fall back to raw.
        assert_eq!(encoded[0], BlockType::Raw as u8);
    }
}
