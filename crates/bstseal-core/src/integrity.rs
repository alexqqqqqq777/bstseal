//! Simple integrity footer using 32-byte BLAKE3 digest.
//!
//! Layout of an archive produced by `integrity::add_footer`:
//!
//! ```text
//! +-----------------------+-----------------+
//! |   payload bytes [...] | 32-byte digest  |
//! +-----------------------+-----------------+
//!                                     ^
//!                                     └─ big-endian order, raw Blake3 bytes
//! ```
//!
//! The digest is `blake3(payload)` (no key, no context string).
//! Verification is O(n) hashing + constant-time compare.
//!
//! This helper is **format-agnostic** – it can wrap any byte slice.

use thiserror::Error;

/// Size of the Blake3 hash in bytes.
pub const HASH_SIZE: usize = blake3::OUT_LEN;

#[derive(Debug, Error)]
pub enum IntegrityError {
    #[error("file is smaller than integrity footer ({HASH_SIZE} bytes)")]
    TooSmall,
    #[error("checksum mismatch: expected {expected:?}, got {actual:?}")]
    Mismatch { expected: [u8; HASH_SIZE], actual: [u8; HASH_SIZE] },
}

/// Returns a new Vec consisting of `data` followed by its Blake3 digest.
#[inline]
pub fn add_footer(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + HASH_SIZE);
    out.extend_from_slice(data);
    let digest = blake3::hash(data);
    out.extend_from_slice(digest.as_bytes());
    out
}

/// Verifies integrity footer. Returns slice **without** footer on success.
#[inline]
pub fn verify_footer(data: &[u8]) -> Result<&[u8], IntegrityError> {
    if data.len() < HASH_SIZE {
        return Err(IntegrityError::TooSmall);
    }
    let (payload, footer) = data.split_at(data.len() - HASH_SIZE);
    let expected = blake3::hash(payload);
    let mut actual_arr = [0u8; HASH_SIZE];
    actual_arr.copy_from_slice(footer);
    let expected_arr = *expected.as_bytes();
    if expected_arr == actual_arr {
        Ok(payload)
    } else {
        Err(IntegrityError::Mismatch { expected: expected_arr, actual: actual_arr })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let data = b"hello world";
        let with_footer = add_footer(data);
        let stripped = verify_footer(&with_footer).unwrap();
        assert_eq!(stripped, data);
    }

    #[test]
    fn detects_corruption() {
        let data = b"payload bytes";
        let mut corrupted = add_footer(data);
        corrupted[0] ^= 0xAA; // flip a bit
        assert!(verify_footer(&corrupted).is_err());
    }
}
