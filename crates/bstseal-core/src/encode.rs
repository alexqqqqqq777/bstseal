use crate::block_coder::{self, BLOCK_SIZE};
use crate::utils;
use anyhow::{anyhow, Result};
use rayon::prelude::*;

/// Encodes input data by splitting it into blocks and processing them in parallel.
///
/// Each encoded block is prefixed with a varint indicating its size.
pub fn encode_parallel(input: &[u8]) -> Result<Vec<u8>> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let results: Vec<Result<Vec<u8>>> = input
        .par_chunks(BLOCK_SIZE)
        .map(|chunk| block_coder::encode_block(chunk))
        .collect();

    let mut final_data = Vec::new();
    for result in results {
        let encoded_block = result?;
        utils::write_varint_u64(&mut final_data, encoded_block.len() as u64)?;
        final_data.extend(&encoded_block);
    }

    Ok(final_data)
}

/// Decodes data that was previously encoded with `encode_parallel`.
///
/// It reads a sequence of blocks, each prefixed with a varint length header,
/// and decodes them, reassembling the original data.
pub fn decode_parallel(encoded_data: &[u8]) -> Result<Vec<u8>> {
    if encoded_data.is_empty() {
        return Ok(Vec::new());
    }

    // 1. Собираем границы всех блоков.
    let mut boundaries = Vec::<(usize, usize)>::new(); // (start, end)
    let mut pos = 0;
    while pos < encoded_data.len() {
        let (block_len, varint_len) = utils::read_varint_u64(&encoded_data[pos..])
            .ok_or_else(|| anyhow!("Failed to read block length varint"))?;
        let start = pos + varint_len;
        let end = start + block_len as usize;
        if end > encoded_data.len() {
            return Err(anyhow!("Incomplete block data"));
        }
        boundaries.push((start, end));
        pos = end;
    }

    // 2. Декодируем блоки параллельно. Сохраняем порядок с индексом.
    let mut decoded_parts: Vec<(usize, Vec<u8>)> = boundaries
        .par_iter()
        .enumerate()
        .map(|(idx, &(s, e))| Ok((idx, block_coder::decode_block(&encoded_data[s..e])?)))
        .collect::<Result<Vec<_>>>()?;

    decoded_parts.sort_by_key(|&(idx, _)| idx);
    let total_len: usize = decoded_parts.iter().map(|(_, v)| v.len()).sum();
    let mut out = Vec::with_capacity(total_len);
    for (_, mut part) in decoded_parts {
        out.extend(part.drain(..));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_roundtrip_test(original_data: &[u8]) {
        let encoded_data = encode_parallel(original_data).expect("Encoding failed");
        let decoded_data = decode_parallel(&encoded_data).expect("Decoding failed");
        assert_eq!(original_data, decoded_data, "Roundtrip failed!");
    }

    #[test]
    fn test_parallel_roundtrip_empty() {
        run_roundtrip_test(&[]);
    }

    #[test]
    fn test_parallel_roundtrip_small() {
        run_roundtrip_test(b"hello world");
    }

    #[test]
    fn test_parallel_roundtrip_one_block() {
        let data: Vec<u8> = (0..BLOCK_SIZE).map(|i| (i % 256) as u8).collect();
        run_roundtrip_test(&data);
    }

    #[test]
    fn test_parallel_roundtrip_multiple_blocks() {
        let data: Vec<u8> = (0..BLOCK_SIZE * 3 + 123).map(|i| (i % 256) as u8).collect();
        run_roundtrip_test(&data);
    }

    #[test]
    fn test_parallel_roundtrip_incompressible() {
        // Use random data which is unlikely to be compressible
        let data: Vec<u8> = (0..BLOCK_SIZE * 2).map(|_| rand::random::<u8>()).collect();
        run_roundtrip_test(&data);
    }

    #[test]
    fn test_parallel_roundtrip_compressible() {
        let data = vec![b'a'; BLOCK_SIZE * 2];
        run_roundtrip_test(&data);
    }
}
