//! Parallel multi-frame compression throughput — the metric that matters for the
//! on-set use case (a card of frames compressed across all cores). Mirrors the
//! pipeline's rayon fan-out over a batch of synthetic 16-bit frames.
//!
//! Compare runs across changes with criterion baselines:
//!
//! ```text
//! cargo bench -p aleph-orchestration --bench orchestration_parallel -- --save-baseline before
//! # ...make changes...
//! cargo bench -p aleph-orchestration --bench orchestration_parallel -- --baseline before
//! ```

use std::hint::black_box;

use aleph_container::Dng;
use aleph_container::Endian;
use aleph_container::Ifd;
use aleph_container::Image;
use aleph_container::Layout;
use aleph_container::Value;
use aleph_orchestration::compress_dng;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use criterion::Throughput;
use rayon::iter::IntoParallelRefIterator as _;
use rayon::iter::ParallelIterator as _;

const FRAMES: usize = 16;
const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;

// One uncompressed single-strip 16-bit CFA frame with gradient + pseudo-noise.
fn raw_frame() -> Dng {
    let count = usize::try_from(WIDTH * HEIGHT).expect("count fits usize");
    let mut segment = Vec::with_capacity(count * 2);
    for i in 0..count {
        let i = u32::try_from(i).expect("count fits u32");
        let noise = i.wrapping_mul(2_654_435_761) ^ (i >> 3);
        let sample = u16::try_from(noise & 0xFFFF).expect("masked to 16 bits");
        segment.extend_from_slice(&sample.to_le_bytes());
    }

    let mut ifd = Ifd::default();
    ifd.set(256, Value::Long(vec![WIDTH]));
    ifd.set(257, Value::Long(vec![HEIGHT]));
    ifd.set(258, Value::Short(vec![16]));
    ifd.set(259, Value::Short(vec![1]));
    ifd.set(277, Value::Short(vec![1]));
    ifd.image = Some(Image {
        layout: Layout::Strips {
            rows_per_strip: HEIGHT,
        },
        segments: vec![segment],
    });

    Dng {
        endian: Endian::Little,
        ifds: vec![ifd],
    }
}

fn orchestration_parallel(c: &mut Criterion) {
    let frames: Vec<Dng> = (0..FRAMES).map(|_| raw_frame()).collect();

    let pixel_bytes =
        u64::try_from(FRAMES).expect("fits u64") * u64::from(WIDTH) * u64::from(HEIGHT) * 2;

    let mut group = c.benchmark_group("orchestration_parallel");
    group.throughput(Throughput::Bytes(pixel_bytes));
    group.sample_size(30);
    group.bench_function("compress_batch", |b| {
        b.iter(|| {
            let out: Vec<Dng> = black_box(&frames)
                .par_iter()
                .map(|frame| compress_dng(frame).expect("compress"))
                .collect();
            black_box(out)
        });
    });
    group.finish();
}

criterion_group!(benches, orchestration_parallel);
criterion_main!(benches);
