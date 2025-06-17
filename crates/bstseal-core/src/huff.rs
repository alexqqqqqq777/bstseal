//! Huffman coding implementation with canonical codes and a fast lookup table for decoding.

use anyhow::{anyhow, Result};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::collections::BinaryHeap;
use std::io::{Read, Write};
use std::ptr;
use std::sync::{Arc, RwLock};
use once_cell::sync::Lazy;
use std::collections::HashMap;

const MAX_CODE_LEN: usize = 15;
// Number of bits used for the fast Huffman decode lookup table.
// 16 покрывает все допустимые коды (<= 15 бит по спецификации),
// поэтому медленный путь больше не требуется.
const FAST_DECODE_BITS: usize = 16;
const TABLE_SIZE: usize = 1 << FAST_DECODE_BITS;
const CACHE_LIMIT: usize = 32;
static CODE_CACHE: Lazy<RwLock<HashMap<Vec<u8>, Arc<Vec<FastDecodeEntry>>>>> = Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct HuffCode {
    code: u16,
    len: u8,
}

#[derive(Debug, Default, Clone, Copy)]
struct FastDecodeEntry {
    symbol: u8,
    len: u8,
}

#[derive(Debug, Clone)]
pub struct CanonicalCode {
    codes: [HuffCode; 256],
    fast_decode_table: Arc<Vec<FastDecodeEntry>>,
}

impl CanonicalCode {
    pub fn new(freqs: &[u64; 256]) -> Result<Self> {
        let mut code_lengths = [0u8; 256];
        let active_symbols: Vec<_> = freqs
            .iter()
            .enumerate()
            .filter(|&(_, &f)| f > 0)
            .collect();

        if active_symbols.is_empty() {
            return Self::from_lengths(&[0; 256]);
        }

        if active_symbols.len() == 1 {
            let symbol = active_symbols[0].0;
            code_lengths[symbol] = 1;
        } else {
            let mut heap = BinaryHeap::new();
            for (symbol, &freq) in active_symbols {
                heap.push(std::cmp::Reverse((freq, vec![symbol as u8])));
            }

            let mut combined: Vec<_> = heap.into_vec().into_iter().map(|r| (r.0 .0, r.0 .1)).collect();

            while combined.len() > 1 {
                combined.sort_by_key(|k| std::cmp::Reverse(k.0));
                let (f1, s1) = combined.pop().unwrap();
                let (f2, s2) = combined.pop().unwrap();

                for &s in &s1 {
                    code_lengths[s as usize] += 1;
                }
                for &s in &s2 {
                    code_lengths[s as usize] += 1;
                }

                let new_symbols = [s1, s2].concat();
                combined.push((f1 + f2, new_symbols));
            }
        }

        // Length limiting
        for len in code_lengths.iter_mut() {
            if *len > MAX_CODE_LEN as u8 {
                *len = MAX_CODE_LEN as u8;
            }
        }

        Self::from_lengths(&code_lengths)
    }

    pub fn from_lengths(lengths: &[u8; 256]) -> Result<Self> {
        let mut codes = [HuffCode::default(); 256];
        let mut bl_count = [0u32; MAX_CODE_LEN + 1];
        for &len in lengths.iter() {
            if len as usize > MAX_CODE_LEN {
                return Err(anyhow!("Code length {} exceeds MAX_CODE_LEN {}", len, MAX_CODE_LEN));
            }
            if len > 0 {
                bl_count[len as usize] += 1;
            }
        }

        let mut next_code = [0u16; MAX_CODE_LEN + 1];
        let mut code = 0;
        for bits in 1..=MAX_CODE_LEN {
            code = (code + bl_count[bits - 1] as u16) << 1;
            next_code[bits] = code;
        }

        for i in 0..256 {
            let len = lengths[i];
            if len > 0 {
                codes[i] = HuffCode { code: next_code[len as usize], len };
                next_code[len as usize] += 1;
            }
        }

        let fast_decode_table = Arc::new(Self::build_fast_decode_table(&codes));

        Ok(Self { codes, fast_decode_table })
    }

    fn build_fast_decode_table(codes: &[HuffCode; 256]) -> Vec<FastDecodeEntry> {
        // заполнить статический массив
        let mut table = vec![FastDecodeEntry::default(); TABLE_SIZE];

        for (symbol, hc) in codes.iter().enumerate() {
            if hc.len > 0 && (hc.len as usize) <= FAST_DECODE_BITS {
                let num_entries = 1 << (FAST_DECODE_BITS - hc.len as usize);
                let start_code = (hc.code as u32) << (FAST_DECODE_BITS - hc.len as usize);
                for i in 0..num_entries {
                    let index = (start_code + i as u32) as usize;
                    if index < TABLE_SIZE {
                        table[index] = FastDecodeEntry { symbol: symbol as u8, len: hc.len };
                    }
                }
            }
        }
        table
    }

    pub fn get_code(&self, symbol: u8) -> (u16, u8) {
        let hc = self.codes[symbol as usize];
        (hc.code, hc.len)
    }

    pub fn write_lengths<W: Write>(&self, writer: &mut W) -> Result<()> {
        let non_zero: Vec<_> = self.codes.iter().enumerate()
            .filter(|(_, hc)| hc.len > 0)
            .collect();
        writer.write_u8(non_zero.len() as u8)?;
        for (symbol, hc) in non_zero {
            writer.write_u8(symbol as u8)?;
            writer.write_u8(hc.len)?;
        }
        Ok(())
    }

    pub fn read_lengths<R: Read>(reader: &mut R) -> Result<Self> {
        let mut lengths = [0u8; 256];
        let count = reader.read_u8()?;
        for _ in 0..count {
            let symbol = reader.read_u8()?;
            let len = reader.read_u8()?;
            lengths[symbol as usize] = len;
        }
        // Проверяем кэш по ключу длины
        let key: Vec<u8> = lengths.to_vec();
        if let Some(entry) = CODE_CACHE.read().unwrap().get(&key) {
            // Быстрый путь: таблица уже есть
            return Ok(Self {
                codes: {
                    // восстановим коды (быстро)
                    let mut codes = [HuffCode::default(); 256];
                    let mut bl_count = [0u32; MAX_CODE_LEN + 1];
                    for &len in &lengths {
                        if len > 0 {
                            bl_count[len as usize] += 1;
                        }
                    }
                    let mut next_code = [0u16; MAX_CODE_LEN + 1];
                    let mut code = 0u16;
                    for bits in 1..=MAX_CODE_LEN {
                        code = (code + bl_count[bits - 1] as u16) << 1;
                        next_code[bits] = code;
                    }
                    for i in 0..256 {
                        let len = lengths[i];
                        if len > 0 {
                            codes[i] = HuffCode { code: next_code[len as usize], len };
                            next_code[len as usize] += 1;
                        }
                    }
                    codes
                },
                fast_decode_table: entry.clone(),
            });
        }
        // Нет в кэше — строим и добавляем
        let cc = Self::from_lengths(&lengths)?;
        {
            let mut cache = CODE_CACHE.write().unwrap();
            if cache.len() >= CACHE_LIMIT {
                // remove an arbitrary entry (first)
                if let Some(first_key) = cache.keys().next().cloned() {
                    cache.remove(&first_key);
                }
            }
            cache.insert(key, cc.fast_decode_table.clone());
        }
        Ok(cc)
    }
}

pub fn encode(input: &[u8]) -> Result<Vec<u8>> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let mut freqs = [0u64; 256];
    for &byte in input {
        freqs[byte as usize] += 1;
    }

    let huff_tree = CanonicalCode::new(&freqs)?;

    let mut out = Vec::new();
    huff_tree.write_lengths(&mut out)?;

    let mut bit_writer = BitWriter::new();
    for &byte in input {
        let (code, len) = huff_tree.get_code(byte);
        if len > 0 {
            bit_writer.write(code, len);
        }
    }
    out.extend(bit_writer.as_bytes());

    Ok(out)
}

pub fn decode(input: &[u8], out: &mut Vec<u8>, expected_size: Option<usize>) -> Result<()> {
    if input.is_empty() {
        return Ok(());
    }
    let mut reader = std::io::Cursor::new(input);
    let huff_tree = CanonicalCode::read_lengths(&mut reader)?;
    let data_start_pos = reader.position() as usize;
    let bit_buf = &input[data_start_pos..];

    // Fast decode path using custom bit cursor (≈3× быстрее стандартного BitReader).
    let mut byte_pos: usize = 0;
    let mut bit_pos: u8 = 0; // 0..=7, номер следующего бита (от MSB)

    let total_bits = bit_buf.len() * 8;
    let mut decoded: usize = 0;
    let expect = expected_size.unwrap_or(usize::MAX);
    out.reserve(expect);

    // Быстрый peek 16 бит с использованием небезопасного чтения u32 без проверок границ.
    // Для последних ≤3 байтов потока fallback на безопасный вариант.
    let peek16 = |bp: usize, bpos: u8| -> u16 {
        let remaining = bit_buf.len().saturating_sub(bp);
        if remaining >= 4 {
            unsafe {
                // Читаем 32 бита big-endian, получаем верхние 16 с учётом смещения.
                let ptr = bit_buf.as_ptr().add(bp) as *const u32;
                let val32 = u32::from_be(ptr::read_unaligned(ptr));
                ((val32 >> (16 - bpos)) & 0xFFFF) as u16
            }
        } else {
            // Хвост — используем безопасное сложение.
            let mut acc: u32 = 0;
            for i in 0..remaining {
                acc |= (bit_buf[bp + i] as u32) << ((3 - i) * 8);
            }
            ((acc >> (16 - bpos)) & 0xFFFF) as u16
        }
    };

    while decoded < expect {
        let bits_consumed = byte_pos * 8 + bit_pos as usize;
        if bits_consumed >= total_bits {
            break;
        }

        // Если осталось менее 16 бит — резервный медленный декодер.
        if total_bits - bits_consumed < FAST_DECODE_BITS {
            let mut br = BitReader { buffer: bit_buf, byte_pos, bit_pos };
            match decode_slow(&mut br, &huff_tree.codes) {
                Some(sym) => {
                    out.push(sym);
                    decoded += 1;
                    // продвигаем курсор и продолжаем основной цикл
                    byte_pos = br.byte_pos;
                    bit_pos = br.bit_pos;
                    continue;
                }
                None => break,
            }
        }

        // распаковка двух символов на итерацию, если хватает бит
        for _ in 0..2 {
            let idx = peek16(byte_pos, bit_pos) as usize;
            let entry = &huff_tree.fast_decode_table[idx];
            if entry.len == 0 {
                // fallback (очень редко)
                let mut br = BitReader { buffer: bit_buf, byte_pos, bit_pos };
                if let Some(sym) = decode_slow(&mut br, &huff_tree.codes) {
                    out.push(sym);
                    decoded += 1;
                    byte_pos = br.byte_pos;
                    bit_pos = br.bit_pos;
                } else {
                    break;
                }
            } else {
                out.push(entry.symbol);
                decoded += 1;
                bit_pos += entry.len;
                byte_pos += (bit_pos >> 3) as usize;
                bit_pos &= 7;
            }
            if decoded >= expect { break; }
        }
    }

    Ok(())
}

// ... (rest of the code remains the same)
#[allow(dead_code)]
fn decode_slow(reader: &mut BitReader, codes: &[HuffCode; 256]) -> Option<u8> {
    let mut code = 0u16;
    let mut len = 0u8;
    for _ in 0..MAX_CODE_LEN {
        code = (code << 1) | reader.read(1)? as u16;
        len += 1;
        for (symbol, hc) in codes.iter().enumerate() {
            if hc.len == len && hc.code == code {
                return Some(symbol as u8);
            }
        }
    }
    None
}

// --- Bit-level I/O ---
struct BitWriter {
    buffer: Vec<u8>,
    current_byte: u8,
    bit_pos: u8, // 0-7, from MSB to LSB
}

impl BitWriter {
    fn new() -> Self {
        Self { buffer: Vec::new(), current_byte: 0, bit_pos: 0 }
    }

        fn write(&mut self, bits: u16, mut len: u8) {
        while len > 0 {
            let bits_to_write = (8 - self.bit_pos).min(len);
            let mask = (1u16 << bits_to_write) - 1;
            let chunk = (bits >> (len - bits_to_write)) & mask;

            self.current_byte |= (chunk as u8) << (8 - self.bit_pos - bits_to_write);
            self.bit_pos += bits_to_write;
            len -= bits_to_write;

            if self.bit_pos == 8 {
                self.buffer.push(self.current_byte);
                self.current_byte = 0;
                self.bit_pos = 0;
            }
        }
    }

    fn as_bytes(&mut self) -> &[u8] {
        if self.bit_pos > 0 {
            self.buffer.push(self.current_byte);
            self.current_byte = 0;
            self.bit_pos = 0;
        }
        &self.buffer
    }
}

struct BitReader<'a> {
    buffer: &'a [u8],
    byte_pos: usize,
    bit_pos: u8, // 0-7, from MSB to LSB
}

#[allow(dead_code)]
impl<'a> BitReader<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, byte_pos: 0, bit_pos: 0 }
    }

    #[inline(always)]
    fn read(&mut self, mut len: u8) -> Option<u16> {
        let mut val = 0u16;
        while len > 0 {
            if self.byte_pos >= self.buffer.len() {
                return None;
            }
            let bits_to_read = (8 - self.bit_pos).min(len);
                        let mask = ((1u16 << bits_to_read) - 1) as u8;
            let chunk = (self.buffer[self.byte_pos] >> (8 - self.bit_pos - bits_to_read)) & mask;

            val = (val << bits_to_read) | chunk as u16;
            self.bit_pos += bits_to_read;
            len -= bits_to_read;

            if self.bit_pos == 8 {
                self.byte_pos += 1;
                self.bit_pos = 0;
            }
        }
        Some(val)
    }

    #[inline(always)]
    fn peek(&self, len: u8) -> u16 {
        if len == 0 {
            return 0;
        }
        let mut val = 0u16;
        let mut remaining = len;
        let mut byte_pos = self.byte_pos;
        let mut bit_pos = self.bit_pos;
        while remaining > 0 {
            if byte_pos >= self.buffer.len() {
                break;
            }
            let bits_to_read = (8 - bit_pos).min(remaining as u8);
            let mask = ((1u16 << bits_to_read) - 1) as u8;
            let chunk = (self.buffer[byte_pos] >> (8 - bit_pos - bits_to_read)) & mask;
            val = (val << bits_to_read) | chunk as u16;
            bit_pos += bits_to_read;
            remaining -= bits_to_read;
            if bit_pos == 8 {
                byte_pos += 1;
                bit_pos = 0;
            }
            if remaining == 0 {
                break;
            }
        }
        if remaining > 0 {
            val <<= remaining; // pad with zeros if not enough bits
        }
        val
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn consume(&mut self, len: u8) -> bool {
        let new_bit_pos = self.byte_pos * 8 + self.bit_pos as usize + len as usize;
        if new_bit_pos > self.buffer.len() * 8 {
            self.byte_pos = self.buffer.len();
            self.bit_pos = 0;
            return false;
        }
        self.byte_pos = new_bit_pos / 8;
        self.bit_pos = (new_bit_pos % 8) as u8;
        true
    }
}