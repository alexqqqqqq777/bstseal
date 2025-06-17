use bstseal_core::block_coder::{encode_block, decode_block};
use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn bench_decode(c: &mut Criterion) {
    // 4 KB сжимаемый буфер (динамически гарантируем размер ≥ 4096)
    let mut data = Vec::new();
    while data.len() < 4096 {
        data.extend_from_slice(b"hello hello hello, this is a test of the huffman coding system");
    }
    let sample = &data[..4096];
    let encoded = encode_block(sample).expect("encode");

    c.bench_function("decode 4KB block", |b| {
        b.iter(|| {
            let out = decode_block(black_box(&encoded)).expect("decode");
            black_box(out);
        })
    });
}

criterion_group!(benches, bench_decode);
criterion_main!(benches);
