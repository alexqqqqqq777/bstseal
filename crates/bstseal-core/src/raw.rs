//! Handles raw (uncompressed) data blocks.

use anyhow::Result;

/// Returns the input data as a Vec. Length is handled by the caller.
pub fn encode(input: &[u8]) -> Result<Vec<u8>> {
    Ok(input.to_vec())
}

/// Returns the decoded data as is. Length is handled by the caller.
pub fn decode(input: &[u8]) -> Result<Vec<u8>> {
    Ok(input.to_vec())
}
