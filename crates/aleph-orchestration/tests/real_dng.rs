//! Conformance tests against a real camera DNG (`fixtures/1.DNG`, a SIGMA fp
//! still): its main raw image (`SubIFD0`) is a 6064 by 4042 14-bit CFA Bayer
//! stored as 384 lossless-JPEG tiles produced by the camera. These prove our
//! codec interoperates with third-party lossless JPEG and our container survives
//! a real, complex file.
//!
//! Golden sample values come from an independent reference LJPEG decoder
//! (written from `ITU-T81`), so a match means two independent implementations
//! agree on real-world data.

use aleph_container::Dng;
use aleph_container::Endian;
use aleph_container::Ifd;
use aleph_container::Layout;
use aleph_container::Value;

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/1.DNG");

fn read_fixture() -> Option<Dng> {
    let path = std::path::Path::new(FIXTURE);
    if !path.exists() {
        eprintln!("skipping: {FIXTURE} not present (large LFS fixture)");
        return None;
    }
    Some(aleph_container::read_file(path).expect("fixture fixtures/1.DNG must parse"))
}

fn u32_at(ifd: &Ifd, tag: u16) -> Option<u32> {
    ifd.get(tag)
        .and_then(Value::as_u32_vec)
        .and_then(|v| v.into_iter().next())
}

fn raw_subifd(dng: &Dng) -> &Ifd {
    // IFD0's first SubIFD (tag 330 order) is the full-resolution raw.
    &dng.ifds[0].sub_ifds[0]
}

fn fnv1a(samples: &[u16]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &value in samples {
        for byte in value.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}

#[test]
fn fixture_structure_matches_camera_dng() {
    let Some(dng) = read_fixture() else { return };
    assert_eq!(dng.endian, Endian::Little);
    assert_eq!(dng.ifds.len(), 1);

    let ifd0 = &dng.ifds[0];
    assert_eq!(u32_at(ifd0, 256), Some(160)); // thumbnail width
    assert_eq!(u32_at(ifd0, 257), Some(120)); // thumbnail height
    assert_eq!(u32_at(ifd0, 259), Some(7)); // JPEG-compressed preview
    assert_eq!(u32_at(ifd0, 262), Some(6)); // YCbCr
    assert_eq!(ifd0.sub_ifds.len(), 3);
    assert!(ifd0.image.is_some(), "IFD0 carries the thumbnail strip");

    let raw = raw_subifd(&dng);
    assert_eq!(u32_at(raw, 256), Some(6064));
    assert_eq!(u32_at(raw, 257), Some(4042));
    assert_eq!(raw.get(258), Some(&Value::Short(vec![14]))); // 14 bits/sample
    assert_eq!(u32_at(raw, 259), Some(7)); // lossless JPEG
    assert_eq!(u32_at(raw, 262), Some(32803)); // CFA
    assert_eq!(u32_at(raw, 277), Some(1)); // single CFA plane

    let image = raw.image.as_ref().expect("raw SubIFD has pixel data");
    assert_eq!(
        image.layout,
        Layout::Tiles {
            tile_width: 256,
            tile_length: 256
        }
    );
    assert_eq!(image.segments.len(), 384);
}

#[test]
fn decodes_real_camera_lossless_jpeg_tiles() {
    let Some(dng) = read_fixture() else { return };
    let segments = &raw_subifd(&dng).image.as_ref().unwrap().segments;

    // (tile index, expected first8, last8, fnv1a) from the reference decoder.
    let golden: &[(usize, [u16; 8], [u16; 8], u64)] = &[
        (
            0,
            [1020, 1025, 1027, 1022, 1028, 1024, 1023, 1027],
            [1026, 1026, 1031, 1025, 1027, 1029, 1023, 1023],
            0x3c0c_5613_2477_7010,
        ),
        (
            383,
            [1030, 1029, 1031, 1028, 1028, 1030, 1028, 1037],
            [1034, 1025, 1034, 1025, 1034, 1025, 1034, 1025],
            0xc442_a57b_74b6_edee,
        ),
    ];

    for &(index, first8, last8, expected_fnv) in golden {
        let decoded = aleph_codec::decode(&segments[index])
            .unwrap_or_else(|e| panic!("tile {index} must decode: {e}"));

        // Camera uses the 2-component CFA arrangement: 256 rows x 128 cols x 2.
        assert_eq!(decoded.width, 128);
        assert_eq!(decoded.height, 256);
        assert_eq!(decoded.components, 2);
        assert_eq!(decoded.precision, 14);
        assert_eq!(decoded.samples.len(), 256 * 128 * 2);

        assert!(
            decoded.samples.iter().all(|&s| s < (1 << 14)),
            "tile {index} sample exceeds 14-bit range"
        );
        assert_eq!(&decoded.samples[..8], &first8, "tile {index} first8");
        assert_eq!(
            &decoded.samples[decoded.samples.len() - 8..],
            &last8,
            "tile {index} last8"
        );
        assert_eq!(
            fnv1a(&decoded.samples),
            expected_fnv,
            "tile {index} full-sample checksum must match the reference decoder"
        );
    }
}

#[test]
fn decodes_every_raw_tile_without_error() {
    let Some(dng) = read_fixture() else { return };
    let segments = &raw_subifd(&dng).image.as_ref().unwrap().segments;

    for (index, segment) in segments.iter().enumerate() {
        let decoded = aleph_codec::decode(segment)
            .unwrap_or_else(|e| panic!("tile {index} of 384 failed to decode: {e}"));
        assert_eq!(decoded.samples.len(), 256 * 128 * 2, "tile {index} size");
        assert!(
            decoded.samples.iter().all(|&s| s < (1 << 14)),
            "tile {index} out of 14-bit range"
        );
    }
}

#[test]
fn reencode_decode_round_trips_real_samples() {
    let Some(dng) = read_fixture() else { return };
    let segments = &raw_subifd(&dng).image.as_ref().unwrap().segments;
    let decoded = aleph_codec::decode(&segments[0]).unwrap();

    let frame = aleph_codec::Frame {
        width: decoded.width,
        height: decoded.height,
        components: decoded.components,
        precision: decoded.precision,
        samples: &decoded.samples,
    };
    let reencoded = aleph_codec::encode(&frame).expect("re-encode real samples");
    let again = aleph_codec::decode(&reencoded).expect("decode our own stream");

    assert_eq!(again.samples, decoded.samples);
    assert_eq!(again.precision, 14);
    assert_eq!(again.components, 2);
}

#[test]
fn container_round_trips_real_dng() {
    let path = std::path::Path::new(FIXTURE);
    if !path.exists() {
        eprintln!("skipping: {FIXTURE} not present (large LFS fixture)");
        return;
    }
    let bytes = std::fs::read(path).expect("read fixture bytes");
    let parsed = aleph_container::read(&bytes).expect("parse fixture");
    let rewritten = aleph_container::write(&parsed).expect("rewrite fixture");
    let reparsed = aleph_container::read(&rewritten).expect("reparse rewritten fixture");

    assert_eq!(
        parsed, reparsed,
        "our model must survive a write/read cycle on a real DNG"
    );
}
