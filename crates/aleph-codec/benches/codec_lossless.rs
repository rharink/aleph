//! Encode/decode throughput for the lossless-JPEG codec — the make-or-break hot
//! path. Measured against a representative single-component 12-bit CFA frame.
//!
//! Compare runs across changes with criterion baselines:
//!
//! ```text
//! cargo bench -p aleph-codec --bench codec_lossless -- --save-baseline before
//! # ...make changes...
//! cargo bench -p aleph-codec --bench codec_lossless -- --baseline before
//! ```

use std::hint::black_box;

use aleph_codec::decode;
use aleph_codec::encode;
use aleph_codec::Frame;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use criterion::Throughput;

const WIDTH: u16 = 512;
const HEIGHT: u16 = 512;
const COMPONENTS: u8 = 1;
const PRECISION: u8 = 12;

// A gradient with cheap pseudo-noise: compresses like real footage rather than a
// degenerate all-zeros stream that flatters the entropy coder.
fn sample_samples() -> Vec<u16> {
    let max = (1u32 << PRECISION) - 1;
    let count = usize::from(WIDTH) * usize::from(HEIGHT) * usize::from(COMPONENTS);
    (0..count)
        .map(|i| {
            let i = u32::try_from(i).expect("count fits u32");
            let noise = i.wrapping_mul(2_654_435_761) ^ (i >> 3);
            u16::try_from(noise & max).expect("masked to precision bits")
        })
        .collect()
}

fn codec_lossless(c: &mut Criterion) {
    let samples = sample_samples();
    let frame = Frame {
        width: WIDTH,
        height: HEIGHT,
        components: COMPONENTS,
        precision: PRECISION,
        samples: &samples,
    };
    let stream = encode(&frame).expect("encode sample frame");

    // Throughput is reported over the raw pixel payload (2 bytes per sample).
    let pixel_bytes = u64::try_from(samples.len() * 2).expect("payload fits u64");

    let mut group = c.benchmark_group("codec_lossless");
    group.throughput(Throughput::Bytes(pixel_bytes));
    group.bench_function("encode", |b| {
        b.iter(|| encode(black_box(&frame)).expect("encode"));
    });
    group.bench_function("decode", |b| {
        b.iter(|| decode(black_box(&stream)).expect("decode"));
    });
    group.finish();
}

criterion_group!(benches, codec_lossless);
criterion_main!(benches);
