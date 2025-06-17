//! Varint encoding and decoding utilities.

/// Writes a u64 as a varint to a writer.
pub fn write_varint_u64<W: std::io::Write>(w: &mut W, mut value: u64) -> std::io::Result<usize> {
    let mut bytes_written = 0;
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value == 0 {
            w.write_all(&[byte])?;
            bytes_written += 1;
            return Ok(bytes_written);
        } else {
            byte |= 0x80;
            w.write_all(&[byte])?;
            bytes_written += 1;
        }
    }
}

/// Reads a varint-encoded u64 from a slice.
/// Returns the value and the number of bytes read.
pub fn read_varint_u64(r: &[u8]) -> Option<(u64, usize)> {
    let mut value = 0u64;
    let mut shift = 0;
    for (i, &byte) in r.iter().enumerate() {
        if i >= 10 {
            // Max 10 bytes for u64
            return None;
        }
        let val_part = (byte & 0x7F) as u64;
        value |= val_part << shift;

        if byte & 0x80 == 0 {
            return Some((value, i + 1));
        }
        shift += 7;
    }
    None
}
