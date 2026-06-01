//! Round-trip correctness harness: prove that what `compress` produced decodes
//! back to the original, bit-exact.
//!
//! Verification is scoped to the directories `compress` actually changed: each is
//! reconstructed the way `decompress` would (decode, repack, reattach trailing)
//! and compared to the original bytes. Directories `compress` left
//! alone (already-compressed previews, unsupported formats) must be byte-identical
//! and are never decoded — so a file's pre-existing compressed content can never
//! make verification fail or panic. Metadata is checked via
//! [`aleph_metadata::verify_preserved`], which permits only `Compression` to differ.

use aleph_container::Dng;
use aleph_container::Endian;
use aleph_container::Ifd;
use aleph_container::Value;

use crate::compress::compress_dng;
use crate::compress::is_aleph_compressed;
use crate::error::OrchestrationError;
use crate::sample;
use crate::snapshot::snapshot;

const COMPRESSION: u16 = 259;
const COMPRESSION_NONE: u32 = 1;

/// Compress `original` in memory and assert the result decodes back to it.
///
/// # Errors
/// Returns [`OrchestrationError`] if compression fails or the compressed output
/// does not losslessly reproduce `original`.
pub fn verify_roundtrip(original: &Dng) -> Result<(), OrchestrationError> {
    let compressed = compress_dng(original)?;
    verify_compressed(original, &compressed)
}

/// Assert `compressed` (the output of [`compress_dng`] on `original`) losslessly
/// preserves `original`: every directory we compressed decodes back to the
/// original samples, every other directory is untouched, and metadata survives.
///
/// # Errors
/// Returns [`OrchestrationError::RoundTrip`] on a structural or pixel mismatch,
/// [`OrchestrationError::TagViolations`] when a preserved tag changed, or a codec
/// error if a compressed segment fails to decode.
pub fn verify_compressed(original: &Dng, compressed: &Dng) -> Result<(), OrchestrationError> {
    if original.endian != compressed.endian {
        return Err(OrchestrationError::RoundTrip(format!(
            "byte order changed: {:?} -> {:?}",
            original.endian, compressed.endian
        )));
    }
    if original.ifds.len() != compressed.ifds.len() {
        return Err(OrchestrationError::RoundTrip(format!(
            "top-level IFD count changed: {} -> {}",
            original.ifds.len(),
            compressed.ifds.len()
        )));
    }
    for (index, (o, c)) in original.ifds.iter().zip(&compressed.ifds).enumerate() {
        verify_ifd(o, c, original.endian, index)?;
    }
    Ok(())
}

fn verify_ifd(
    original: &Ifd,
    compressed: &Ifd,
    endian: Endian,
    index: usize,
) -> Result<(), OrchestrationError> {
    aleph_metadata::verify_preserved(&snapshot(original), &snapshot(compressed))
        .map_err(OrchestrationError::TagViolations)?;

    if we_compressed(original, compressed) {
        verify_compressed_image(original, compressed, endian, index)?;
    } else if original.image != compressed.image {
        return Err(OrchestrationError::RoundTrip(format!(
            "directory {index} was modified but not by compression"
        )));
    }

    if original.sub_ifds.len() != compressed.sub_ifds.len() {
        return Err(OrchestrationError::RoundTrip(format!(
            "SubIFD count changed in IFD {index}"
        )));
    }
    for (child, (o, c)) in original
        .sub_ifds
        .iter()
        .zip(&compressed.sub_ifds)
        .enumerate()
    {
        verify_ifd(o, c, endian, child)?;
    }

    // The container preserves Exif/GPS/Interop directories in `pointer_ifds`;
    // verify they survive too, or a tampered/dropped ExifIFD (focal length, lens,
    // GPS, MakerNote, ...) would pass the harness silently.
    if original.pointer_ifds.len() != compressed.pointer_ifds.len() {
        return Err(OrchestrationError::RoundTrip(format!(
            "pointer-IFD count changed in IFD {index}"
        )));
    }
    for (o, c) in original.pointer_ifds.iter().zip(&compressed.pointer_ifds) {
        if o.tag != c.tag {
            return Err(OrchestrationError::RoundTrip(format!(
                "pointer-IFD tag changed in IFD {index}: {:#06x} -> {:#06x}",
                o.tag, c.tag
            )));
        }
        verify_ifd(&o.ifd, &c.ifd, endian, index)?;
    }
    Ok(())
}

/// True when `compress` encoded this directory: it was an uncompressed image in
/// `original` and is now a genuinely Aleph-compressed directory (marker signature
/// and lossless JPEG) in `compressed`. Sharing the exact `decompress_ifd`
/// predicate keeps the harness faithful: output that decompress would skip
/// (missing or foreign marker) cannot pass verification.
fn we_compressed(original: &Ifd, compressed: &Ifd) -> bool {
    original.image.is_some()
        && compression(original) == COMPRESSION_NONE
        && is_aleph_compressed(compressed)
}

fn verify_compressed_image(
    original: &Ifd,
    compressed: &Ifd,
    endian: Endian,
    index: usize,
) -> Result<(), OrchestrationError> {
    let source = original
        .image
        .as_ref()
        .expect("we_compressed guarantees an image");
    let encoded = compressed.image.as_ref().ok_or_else(|| {
        OrchestrationError::RoundTrip(format!("IFD {index} lost its image during compression"))
    })?;

    if source.layout != encoded.layout {
        return Err(OrchestrationError::RoundTrip(format!(
            "IFD {index} layout changed during compression"
        )));
    }
    if source.segments.len() != encoded.segments.len() {
        return Err(OrchestrationError::RoundTrip(format!(
            "IFD {index} segment count changed during compression"
        )));
    }

    for (segment, (source_bytes, encoded_bytes)) in
        source.segments.iter().zip(&encoded.segments).enumerate()
    {
        // Reconstruct the original segment exactly the way decompress would, and
        // compare bytes: this proves both the pixels and any trailing padding
        // round-trip, not merely the samples.
        let decoded = aleph_codec::decode(encoded_bytes)?;
        let mut reconstructed = sample::pack(&decoded.samples, decoded.precision, endian)?;
        reconstructed.extend_from_slice(encoded_bytes.get(decoded.consumed..).unwrap_or_default());
        if &reconstructed != source_bytes {
            return Err(OrchestrationError::RoundTrip(format!(
                "segment {segment} in IFD {index} did not round-trip byte-exactly"
            )));
        }
    }
    Ok(())
}

fn compression(ifd: &Ifd) -> u32 {
    ifd.get(COMPRESSION)
        .and_then(Value::as_u32_vec)
        .and_then(|v| v.into_iter().next())
        .unwrap_or(COMPRESSION_NONE)
}

#[cfg(test)]
mod tests {
    use super::verify_compressed;
    use super::verify_roundtrip;
    use crate::compress::compress_dng;
    use aleph_container::Dng;
    use aleph_container::Endian;
    use aleph_container::Ifd;
    use aleph_container::Image;
    use aleph_container::Layout;
    use aleph_container::PointerIfd;
    use aleph_container::Value;

    fn raw_dng(endian: Endian) -> Dng {
        let mut ifd = Ifd::default();
        ifd.set(256, Value::Long(vec![2]));
        ifd.set(257, Value::Long(vec![2]));
        ifd.set(258, Value::Short(vec![16]));
        ifd.set(259, Value::Short(vec![1]));
        ifd.set(277, Value::Short(vec![1]));
        ifd.set(0x9000, Value::Ascii(b"meta\0".to_vec()));
        let segment: Vec<u8> = [10u16, 9999, 0, 65535]
            .iter()
            .flat_map(|&v| v.to_le_bytes())
            .collect();
        ifd.image = Some(Image {
            layout: Layout::Strips { rows_per_strip: 2 },
            segments: vec![segment],
        });
        Dng {
            endian,
            ifds: vec![ifd],
        }
    }

    #[test]
    fn verifies_clean_round_trip() {
        verify_roundtrip(&raw_dng(Endian::Little)).expect("round-trip holds");
    }

    #[test]
    fn ignores_preexisting_compressed_directories() {
        // An uncompressed raw plus a directory that is already compressed with
        // opaque, non-lossless-JPEG bytes (e.g. a baseline-JPEG preview). The
        // raw must compress and verify; the pre-existing one must never be decoded.
        let mut dng = raw_dng(Endian::Little);
        let mut preview = Ifd::default();
        preview.set(256, Value::Long(vec![4]));
        preview.set(257, Value::Long(vec![4]));
        preview.set(258, Value::Short(vec![8, 8, 8]));
        preview.set(259, Value::Short(vec![7])); // already compressed, not ours
        preview.set(277, Value::Short(vec![3]));
        preview.image = Some(Image {
            layout: Layout::Strips { rows_per_strip: 4 },
            segments: vec![b"not a decodable jpeg".to_vec()],
        });
        dng.ifds[0].sub_ifds.push(preview);

        verify_roundtrip(&dng).expect("must verify without decoding the preview");
    }

    #[test]
    fn detects_tampered_metadata() {
        let original = raw_dng(Endian::Little);
        let mut compressed = compress_dng(&original).unwrap();
        compressed.ifds[0].set(0x9000, Value::Ascii(b"oops\0".to_vec()));
        assert!(verify_compressed(&original, &compressed).is_err());
    }

    #[test]
    fn detects_corrupted_pixels() {
        let original = raw_dng(Endian::Little);
        let mut compressed = compress_dng(&original).unwrap();
        // Replace the encoded segment with a different (but valid) stream.
        let other = raw_dng(Endian::Little);
        let mut wrong = compress_dng(&{
            let mut d = other;
            d.ifds[0].image.as_mut().unwrap().segments[0] = [1u16, 2, 3, 4]
                .iter()
                .flat_map(|&v| v.to_le_bytes())
                .collect();
            d
        })
        .unwrap();
        compressed.ifds[0].image = wrong.ifds[0].image.take();
        assert!(verify_compressed(&original, &compressed).is_err());
    }

    #[test]
    fn detects_dropped_or_tampered_pointer_ifd() {
        let mut original = raw_dng(Endian::Little);
        let mut exif = Ifd::default();
        exif.set(0x920a, Value::Rational(vec![(280, 10)])); // FocalLength
        original.ifds[0].pointer_ifds.push(PointerIfd {
            tag: 34665,
            ifd: exif,
        });

        let compressed = compress_dng(&original).unwrap();
        verify_compressed(&original, &compressed).expect("clean round-trip verifies");

        // A dropped ExifIFD must be caught.
        let mut dropped = compressed.clone();
        dropped.ifds[0].pointer_ifds.clear();
        assert!(verify_compressed(&original, &dropped).is_err());

        // A tampered focal length inside the ExifIFD must be caught.
        let mut tampered = compressed.clone();
        tampered.ifds[0].pointer_ifds[0]
            .ifd
            .set(0x920a, Value::Rational(vec![(999, 10)]));
        assert!(verify_compressed(&original, &tampered).is_err());
    }

    #[test]
    fn requires_valid_marker_for_compressed_directories() {
        let original = raw_dng(Endian::Little);
        let compressed = compress_dng(&original).unwrap();
        verify_compressed(&original, &compressed).expect("clean verifies");

        // Compression=7 but the Aleph marker stripped: decompress would skip this,
        // so verify must NOT accept it as a valid round-trip.
        let mut no_marker = compressed.clone();
        no_marker.ifds[0].remove(aleph_metadata::tags::ALEPH_MARKER);
        assert!(verify_compressed(&original, &no_marker).is_err());

        // Marker present but foreign (not our signature): likewise rejected.
        let mut foreign = compressed.clone();
        foreign.ifds[0].set(aleph_metadata::tags::ALEPH_MARKER, Value::Short(vec![42]));
        assert!(verify_compressed(&original, &foreign).is_err());
    }
}
