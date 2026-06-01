//! TIFF/DNG parse throughput: how fast the reader turns a serialized container
//! back into a `Dng`. Built against a realistic multi-strip 12-bit frame.
//!
//! Compare runs across changes with criterion baselines:
//!
//! ```text
//! cargo bench -p aleph-container --bench container_read -- --save-baseline before
//! # ...make changes...
//! cargo bench -p aleph-container --bench container_read -- --baseline before
//! ```

use std::hint::black_box;

use aleph_container::read;
use aleph_container::write;
use aleph_container::Dng;
use aleph_container::Endian;
use aleph_container::Ifd;
use aleph_container::Image;
use aleph_container::Layout;
use aleph_container::Value;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use criterion::Throughput;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const ROWS_PER_STRIP: u32 = 64;

// A typical uncompressed frame: standard tags plus many strips, serialized once
// so the bench measures parsing, not fixture construction.
fn sample_dng_bytes() -> Vec<u8> {
    let bytes_per_row = usize::try_from(WIDTH * 12 / 8).expect("row bytes fit usize");

    let mut segments = Vec::new();
    let mut row = 0u32;
    while row < HEIGHT {
        let rows = ROWS_PER_STRIP.min(HEIGHT - row);
        let len = usize::try_from(rows).expect("rows fit usize") * bytes_per_row;
        segments.push(vec![0xA5u8; len]);
        row += rows;
    }

    let mut ifd = Ifd::default();
    ifd.set(256, Value::Long(vec![WIDTH]));
    ifd.set(257, Value::Long(vec![HEIGHT]));
    ifd.set(258, Value::Short(vec![12]));
    ifd.set(259, Value::Short(vec![1]));
    ifd.set(271, Value::Ascii(b"Aleph\0".to_vec()));
    ifd.set(272, Value::Ascii(b"BenchCam\0".to_vec()));
    ifd.set(277, Value::Short(vec![1]));
    ifd.image = Some(Image {
        layout: Layout::Strips {
            rows_per_strip: ROWS_PER_STRIP,
        },
        segments,
    });

    let dng = Dng {
        endian: Endian::Little,
        ifds: vec![ifd],
    };
    write(&dng).expect("write sample dng")
}

fn container_read(c: &mut Criterion) {
    let bytes = sample_dng_bytes();
    let total = u64::try_from(bytes.len()).expect("len fits u64");

    let mut group = c.benchmark_group("container_read");
    group.throughput(Throughput::Bytes(total));
    group.bench_function("read", |b| {
        b.iter(|| read(black_box(&bytes)).expect("read"));
    });
    group.finish();
}

criterion_group!(benches, container_read);
criterion_main!(benches);
