//! End-to-end tests driving the compiled `aleph` binary.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use aleph_container::Dng;
use aleph_container::Endian;
use aleph_container::Ifd;
use aleph_container::Image;
use aleph_container::Layout;
use aleph_container::Value;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_aleph")
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut dir = std::env::temp_dir();
    dir.push(format!("aleph-cli-{tag}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn gradient_dng() -> Dng {
    let mut samples = Vec::with_capacity(64);
    for y in 0..8u16 {
        for x in 0..8u16 {
            samples.push(x * 5 + y * 2);
        }
    }
    let segment: Vec<u8> = samples.iter().flat_map(|&v| v.to_le_bytes()).collect();
    let mut ifd = Ifd::default();
    ifd.set(256, Value::Long(vec![8]));
    ifd.set(257, Value::Long(vec![8]));
    ifd.set(258, Value::Short(vec![16]));
    ifd.set(259, Value::Short(vec![1]));
    ifd.set(277, Value::Short(vec![1]));
    ifd.set(0x9000, Value::Ascii(b"cli-meta\0".to_vec()));
    ifd.image = Some(Image {
        layout: Layout::Strips { rows_per_strip: 8 },
        segments: vec![segment],
    });
    Dng {
        endian: Endian::Little,
        ifds: vec![ifd],
    }
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("binary runs")
}

fn s(path: &Path) -> String {
    path.to_str().unwrap().to_owned()
}

#[test]
fn compress_then_decompress_recovers_original_bytes() {
    let in_dir = unique_dir("in");
    let out_dir = unique_dir("out");
    let back_dir = unique_dir("back");

    let in_path = in_dir.join("A001.dng");
    aleph_container::write_file(&in_path, &gradient_dng()).unwrap();
    let original = std::fs::read(&in_path).unwrap();

    let out = run(&["compress", &s(&in_path), "--out", &s(&out_dir)]);
    assert!(out.status.success(), "compress failed: {out:?}");

    let compressed_path = out_dir.join("A001.dng");
    assert!(compressed_path.exists());
    let compressed = std::fs::read(&compressed_path).unwrap();
    assert!(
        compressed.len() < original.len(),
        "gradient should compress: {} -> {}",
        original.len(),
        compressed.len()
    );

    let out = run(&["decompress", &s(&compressed_path), "--out", &s(&back_dir)]);
    assert!(out.status.success(), "decompress failed: {out:?}");

    let restored = std::fs::read(back_dir.join("A001.dng")).unwrap();
    assert_eq!(restored, original, "round-trip must be byte-exact");

    for dir in [&in_dir, &out_dir, &back_dir] {
        std::fs::remove_dir_all(dir).ok();
    }
}

#[test]
fn offload_copies_and_verifies_to_two_destinations() {
    let src = unique_dir("src");
    std::fs::write(src.join("a.dng"), b"frame-a").unwrap();
    std::fs::create_dir_all(src.join("clip")).unwrap();
    std::fs::write(src.join("clip/b.dng"), b"frame-b").unwrap();

    let d1 = unique_dir("d1");
    let d2 = unique_dir("d2");

    let out = run(&["offload", &s(&src), "--to", &s(&d1), "--to", &s(&d2)]);
    assert!(out.status.success(), "offload failed: {out:?}");

    assert_eq!(std::fs::read(d1.join("a.dng")).unwrap(), b"frame-a");
    assert_eq!(std::fs::read(d2.join("clip/b.dng")).unwrap(), b"frame-b");

    for dir in [&src, &d1, &d2] {
        std::fs::remove_dir_all(dir).ok();
    }
}

#[test]
fn no_arguments_is_an_error() {
    let out = run(&[]);
    assert!(!out.status.success(), "missing subcommand must fail");
}
