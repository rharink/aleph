//! End-to-end test against real uncompressed 12-bit `CinemaDNG` footage (a SIGMA
//! fp clip frame). Skipped when the large fixture is absent (it is committed via
//! git LFS and may not be fetched in every environment).

use std::path::Path;

const FRAME: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/A001_001/A001_001_20260601_000001.DNG"
);

#[test]
fn real_12bit_cinemadng_round_trips_byte_exact() {
    let path = Path::new(FRAME);
    if !path.exists() {
        eprintln!("skipping real_12bit_cinemadng_round_trips_byte_exact: {FRAME} not present");
        return;
    }

    let original = aleph_container::read_file(path).expect("parse frame");

    // The frame is a single uncompressed 12-bit CFA strip.
    let raw = &original.ifds[0];
    assert_eq!(raw.get(258), Some(&aleph_container::Value::Short(vec![12])));
    let original_segment_len = raw.image.as_ref().expect("image").segments[0].len();

    // The frame carries an ExifIFD whose metadata (focal length, lens, ...) must
    // survive compression. `restored == original` below proves it round-trips.
    let exif = raw
        .pointer_ifds
        .iter()
        .find(|p| p.tag == 34665)
        .expect("frame has an ExifIFD");
    assert!(
        exif.ifd.entries.iter().any(|e| e.tag == 0x920a),
        "ExifIFD carries FocalLength (0x920a)"
    );

    let compressed = aleph_orchestration::compress_dng(&original).expect("compress");

    // It must actually compress real footage.
    let compressed_segment_len =
        compressed.ifds[0].image.as_ref().expect("image").segments[0].len();
    assert!(
        compressed_segment_len < original_segment_len,
        "12-bit footage should shrink: {original_segment_len} -> {compressed_segment_len}"
    );

    // The compress-time verifier must accept it...
    aleph_orchestration::verify_compressed(&original, &compressed).expect("verify");

    // ...and a full decompress must reproduce the original model byte-exactly,
    // including the camera's trailing strip padding.
    let restored = aleph_orchestration::decompress_dng(&compressed).expect("decompress");
    assert_eq!(
        restored, original,
        "12-bit CinemaDNG round-trip must be exact"
    );
}

// Minimal little-endian readers (the fixture is `II`).
fn rd16(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}
fn rd32(b: &[u8], o: usize) -> usize {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]) as usize
}
fn field_at(b: &[u8], ifd: usize, tag: u16) -> Option<(u16, usize, usize)> {
    let count = rd16(b, ifd) as usize;
    (0..count)
        .map(|i| ifd + 2 + i * 12)
        .find(|&p| rd16(b, p) == tag)
        .map(|p| (rd16(b, p + 2), rd32(b, p + 4), p + 8))
}
fn type_size(t: u16) -> usize {
    match t {
        1 | 2 | 6 | 7 => 1,
        3 | 8 => 2,
        4 | 9 | 11 => 4,
        5 | 10 | 12 => 8,
        _ => 0,
    }
}

#[test]
fn real_frame_makernote_offsets_relocate_cleanly() {
    let path = Path::new(FRAME);
    if !path.exists() {
        eprintln!("skipping real_frame_makernote_offsets_relocate_cleanly: {FRAME} not present");
        return;
    }

    // Re-serialize the frame: the writer relocates the unsafe SIGMA MakerNote.
    let dng = aleph_container::read_file(path).expect("read");
    let bytes = aleph_container::write(&dng).expect("write");

    // Walk IFD0 -> ExifIFD -> MakerNote to find its new position and length.
    let ifd0 = rd32(&bytes, 4);
    let exif = rd32(&bytes, field_at(&bytes, ifd0, 34665).expect("ExifIFD").2);
    let (_, mn_len, mn_field) = field_at(&bytes, exif, 37500).expect("MakerNote");
    let mn_off = rd32(&bytes, mn_field);
    assert_eq!(&bytes[mn_off..mn_off + 5], b"SIGMA");

    // Every out-of-line offset in the SIGMA MakerNote IFD (header is 10 bytes)
    // must now point inside the *relocated* blob; a missed shift would point at
    // the original location, outside this range.
    let ifd = mn_off + 10;
    let count = rd16(&bytes, ifd) as usize;
    let mut checked = 0;
    for i in 0..count {
        let e = ifd + 2 + i * 12;
        if type_size(rd16(&bytes, e + 2)) * rd32(&bytes, e + 4) > 4 {
            let off = rd32(&bytes, e + 8);
            assert!(
                (mn_off..mn_off + mn_len).contains(&off),
                "MakerNote offset {off} is outside the relocated blob [{mn_off}, {})",
                mn_off + mn_len
            );
            checked += 1;
        }
    }
    assert!(
        checked > 0,
        "expected out-of-line MakerNote values to validate"
    );
}
