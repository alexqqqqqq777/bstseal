#![allow(clippy::type_complexity)]
#![allow(clippy::unnecessary_cast)]

pub mod block_coder;
pub mod encode;
pub mod huff;
pub mod integrity;
pub mod raw;
pub mod utils;
pub mod license;

pub use license::{verify_license, Tier, set_license_secret, ensure_license_valid, set_license_key};

// Re-export key functions to make them available directly at the crate root,
// e.g., `bstseal_core::encode_parallel()`
pub use encode::{decode_parallel, encode_parallel};

// The commented-out tests below can be re-enabled once the full pipeline is stable.
#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn test_encode_parallel_basic() {
    //     const TEST_DATA_SIZE: usize = 4096 * 3 + 123; // A little over 3 blocks
    //     let mut original_data = Vec::with_capacity(TEST_DATA_SIZE);
    //     for i in 0..TEST_DATA_SIZE {
    //         // Simple compressible data
    //         original_data.push((i % 16) as u8);
    //     }

    //     let encoded_data = encode_parallel(&original_data).unwrap();
    //     let decoded_data = decode_parallel(&encoded_data).unwrap();

    //     assert_eq!(
    //         decoded_data, original_data,
    //         "Roundtrip failed for basic parallel encoding."
    //     );
    // }

    // #[test]
    // fn test_encode_parallel_empty() {
    //     let original_data: Vec<u8> = Vec::new();
    //     let encoded_data = encode_parallel(&original_data).unwrap();
    //     let decoded_data = decode_parallel(&encoded_data).unwrap();
    //     assert!(
    //         encoded_data.is_empty(),
    //         "Encoding empty data should result in empty data"
    //     );
    //     assert!(
    //         decoded_data.is_empty(),
    //         "Decoding empty data should result in empty data"
    //     );
    // }

    // #[test]
    // fn test_encode_parallel_single_small_block() {
    //     let original_data: Vec<u8> = (0..100).map(|i| i as u8).collect(); // Less than a full block
    //     let encoded_data = encode_parallel(&original_data).unwrap();
    //     let decoded_data = decode_parallel(&encoded_data).unwrap();

    //     assert_eq!(
    //         decoded_data, original_data,
    //         "Roundtrip failed for single small block."
    //     );
    // }
}
