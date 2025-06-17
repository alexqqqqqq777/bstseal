use bstseal_core::block_coder::{encode_block, decode_block};
use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn bench_decode(c: &mut Criterion) {
    // Prepare a representative 4 KB compressible block
    let data = b"hello hello hello, this is a test of the huffman coding system".repeat(64);
    assert!(data.len() >= 4096);
    let data = &data[..4096];
    let encoded = encode_block(data).expect("encode");

    c.bench_function("decode 4KB block", |b| {
        b.iter(|| {
            let out = decode_block(black_box(&encoded)).expect("decode");
            assert_eq!(out.len(), data.len());
            black_box(out);
        })
    });
}

criterion_group!(benches, bench_decode);
criterion_main!(benches);
