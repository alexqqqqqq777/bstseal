fn main() {
    let data = b"hello hello hello, this is a test of the huffman coding system".repeat(10);
    let enc = bstseal_core::block_coder::encode_block(&data).unwrap();
    println!("encoded len {}", enc.len());
    let dec = bstseal_core::block_coder::decode_block(&enc).unwrap();
    println!("decoded len {} orig len {}", dec.len(), data.len());
    if dec != data {
        println!("Mismatch at pos {:?}", dec.iter().zip(&data).position(|(a,b)| a!=b));
    }
}
