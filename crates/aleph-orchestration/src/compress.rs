//! In-memory lossless (de)compression of a parsed [`Dng`].
//!
//! Compression replaces every uncompressed, byte-aligned image directory's
//! segments with lossless-JPEG streams, flips its `Compression` tag to 7, and
//! stamps an Aleph marker tag recording the original compression. Decompression
//! reverses *only* Aleph-marked directories, so a camera's pre-existing
//! compressed previews are never touched. Both recurse through `SubIFDs`. The
//! container writer is authoritative on layout, so byte counts and offsets are
//! recomputed there.

use aleph_codec::Frame;
use aleph_container::Dng;
use aleph_container::Endian;
use aleph_container::Ifd;
use aleph_container::Layout;
use aleph_container::Value;

use crate::error::OrchestrationError;
use crate::sample;

const IMAGE_WIDTH: u16 = 256;
const IMAGE_LENGTH: u16 = 257;
const BITS_PER_SAMPLE: u16 = 258;
const COMPRESSION: u16 = 259;
const SAMPLES_PER_PIXEL: u16 = 277;
const PLANAR_CONFIGURATION: u16 = 284;

const COMPRESSION_NONE: u32 = 1;
const COMPRESSION_LOSSLESS_JPEG: u32 = 7;

// Aleph-private marker stamped on directories we encode; see aleph_metadata.
const ALEPH_MARKER: u16 = aleph_metadata::tags::ALEPH_MARKER;

// Distinctive value written at `ALEPH_MARKER` ("ALPH" in ASCII). Identifying our
// directories by this signature — not mere presence of the tag — stops a
// coincidental third-party tag at the same number from being mistaken for ours.
const ALEPH_MAGIC: u32 = 0x414C_5048;

/// Compress every supported image directory in `dng` losslessly.
///
/// # Errors
/// Returns [`OrchestrationError`] if an uncompressed image directory uses a
/// feature this build cannot encode (bit depth, planar config, dimensions) or
/// the codec rejects a segment.
pub fn compress_dng(dng: &Dng) -> Result<Dng, OrchestrationError> {
    ensure_no_foreign_marker(dng)?;

    let mut out = dng.clone();
    let endian = out.endian;
    for ifd in &mut out.ifds {
        compress_ifd(ifd, endian)?;
    }
    Ok(out)
}

/// The signature value Aleph writes at [`ALEPH_MARKER`].
fn aleph_marker_value() -> Value {
    Value::Long(vec![ALEPH_MAGIC])
}

/// Whether `value` is Aleph's marker signature (not just any value at the tag).
fn is_aleph_marker(value: &Value) -> bool {
    matches!(value, Value::Long(v) if v.as_slice() == [ALEPH_MAGIC])
}

/// Whether `ifd` is a directory Aleph itself compressed: it carries our marker
/// signature **and** is lossless-JPEG. Decompression and the round-trip verifier
/// must agree on exactly this predicate, or verify could pass output that
/// decompress would later skip.
pub(crate) fn is_aleph_compressed(ifd: &Ifd) -> bool {
    ifd.get(ALEPH_MARKER).is_some_and(is_aleph_marker)
        && first_u32(ifd, COMPRESSION).unwrap_or(COMPRESSION_NONE) == COMPRESSION_LOSSLESS_JPEG
}

/// Reject inputs that already carry the marker tag with a non-Aleph value, so we
/// never overwrite a camera's or user's metadata that happens to live at the same
/// private tag number. An existing *Aleph* signature is fine (already compressed).
///
/// # Errors
/// [`OrchestrationError::MarkerCollision`] if any directory holds a foreign value
/// at the marker tag.
fn ensure_no_foreign_marker(dng: &Dng) -> Result<(), OrchestrationError> {
    for ifd in &dng.ifds {
        check_no_foreign_marker(ifd)?;
    }
    Ok(())
}

fn check_no_foreign_marker(ifd: &Ifd) -> Result<(), OrchestrationError> {
    if let Some(value) = ifd.get(ALEPH_MARKER) {
        if !is_aleph_marker(value) {
            return Err(OrchestrationError::MarkerCollision(ALEPH_MARKER));
        }
    }
    for child in &ifd.sub_ifds {
        check_no_foreign_marker(child)?;
    }
    Ok(())
}

/// Reverse [`compress_dng`]: decode every lossless-JPEG image directory.
///
/// # Errors
/// Returns [`OrchestrationError`] if a `Compression = 7` segment is not a
/// decodable lossless-JPEG stream or its precision is not byte-aligned.
pub fn decompress_dng(dng: &Dng) -> Result<Dng, OrchestrationError> {
    let mut out = dng.clone();
    let endian = out.endian;
    for ifd in &mut out.ifds {
        decompress_ifd(ifd, endian)?;
    }
    Ok(out)
}

struct Params {
    precision: u8,
    components: u8,
    image_width: u32,
    image_length: u32,
}

fn compress_ifd(ifd: &mut Ifd, endian: Endian) -> Result<(), OrchestrationError> {
    for child in &mut ifd.sub_ifds {
        compress_ifd(child, endian)?;
    }

    let Some(params) = compressible_params(ifd)? else {
        return Ok(());
    };

    let image = ifd
        .image
        .as_mut()
        .expect("compressible_params yields Some only for image directories");
    let mut encoded = Vec::with_capacity(image.segments.len());
    for (index, segment) in image.segments.iter().enumerate() {
        let (width, height) = segment_dims(&image.layout, &params, index)?;
        let precision = usize::from(params.precision);
        let samples_per_row = usize::from(width) * usize::from(params.components);

        // Sub-byte depths require byte-aligned rows so packing is exactly
        // reversible (no shared pad bits at a row/segment boundary).
        if !precision.is_multiple_of(8) && !(samples_per_row * precision).is_multiple_of(8) {
            return Err(OrchestrationError::UnsupportedRowAlignment {
                samples_per_row,
                precision: params.precision,
            });
        }

        let sample_count = samples_per_row * usize::from(height);
        let pixel_bytes = sample_count * precision / 8;
        if segment.len() < pixel_bytes {
            return Err(OrchestrationError::SegmentTooShort {
                need: pixel_bytes,
                have: segment.len(),
            });
        }

        // Pixel data, then any strip/tile padding the camera appended. The codec
        // stream tolerates trailing bytes, so we stash the padding after EOI to
        // restore it byte-exactly on decompress.
        let (pixels, trailing) = segment.split_at(pixel_bytes);
        let samples = sample::unpack(pixels, params.precision, endian, sample_count)?;
        let frame = Frame {
            width,
            height,
            components: params.components,
            precision: params.precision,
            samples: &samples,
        };
        let mut stream = aleph_codec::encode(&frame)?;
        stream.extend_from_slice(trailing);
        encoded.push(stream);
    }
    image.segments = encoded;

    ifd.set(COMPRESSION, Value::Short(vec![7]));
    ifd.set(ALEPH_MARKER, aleph_marker_value());
    Ok(())
}

fn decompress_ifd(ifd: &mut Ifd, endian: Endian) -> Result<(), OrchestrationError> {
    for child in &mut ifd.sub_ifds {
        decompress_ifd(child, endian)?;
    }

    // Only undo directories Aleph itself encoded (marker signature + lossless
    // JPEG). A camera's pre-existing compressed previews — or a foreign tag that
    // merely collides with the marker number — are left untouched.
    if !is_aleph_compressed(ifd) {
        return Ok(());
    }
    let Some(image) = ifd.image.as_mut() else {
        return Ok(());
    };

    let mut decoded = Vec::with_capacity(image.segments.len());
    for segment in &image.segments {
        let frame = aleph_codec::decode(segment)?;
        let mut bytes = sample::pack(&frame.samples, frame.precision, endian)?;
        // Reattach the padding compress stashed after the codec stream.
        bytes.extend_from_slice(segment.get(frame.consumed..).unwrap_or_default());
        decoded.push(bytes);
    }
    image.segments = decoded;

    // compress only ever encodes uncompressed (Compression = 1) directories.
    ifd.set(COMPRESSION, Value::Short(vec![1]));
    ifd.remove(ALEPH_MARKER);
    Ok(())
}

/// Decide whether `ifd` holds uncompressed pixel data this build can compress.
///
/// `Ok(None)` means "nothing to do" (no pixels, or already compressed).
/// `Err` means the directory holds uncompressed pixels we refuse to silently
/// skip because we cannot faithfully encode them.
fn compressible_params(ifd: &Ifd) -> Result<Option<Params>, OrchestrationError> {
    if ifd.image.is_none() {
        return Ok(None);
    }
    if first_u32(ifd, COMPRESSION).unwrap_or(COMPRESSION_NONE) != COMPRESSION_NONE {
        return Ok(None);
    }

    let planar = first_u32(ifd, PLANAR_CONFIGURATION).unwrap_or(1);
    if planar != 1 {
        return Err(OrchestrationError::UnsupportedPlanarConfig(planar));
    }

    let bits = ifd
        .get(BITS_PER_SAMPLE)
        .and_then(Value::as_u32_vec)
        .ok_or(OrchestrationError::MissingTag(BITS_PER_SAMPLE))?;
    let precision = *bits
        .first()
        .ok_or(OrchestrationError::MissingTag(BITS_PER_SAMPLE))?;
    if bits.iter().any(|&b| b != precision) {
        return Err(OrchestrationError::InconsistentBitDepth);
    }
    if !sample::is_supported_depth(precision) {
        return Err(OrchestrationError::UnsupportedBitDepth(precision));
    }

    let samples_per_pixel = first_u32(ifd, SAMPLES_PER_PIXEL).unwrap_or(1);
    let components = u32::try_from(bits.len()).unwrap_or(u32::MAX);
    if components != samples_per_pixel {
        return Err(OrchestrationError::InconsistentBitDepth);
    }
    if !(1..=4).contains(&components) {
        return Err(OrchestrationError::UnsupportedComponents(components));
    }

    let image_width =
        first_u32(ifd, IMAGE_WIDTH).ok_or(OrchestrationError::MissingTag(IMAGE_WIDTH))?;
    let image_length =
        first_u32(ifd, IMAGE_LENGTH).ok_or(OrchestrationError::MissingTag(IMAGE_LENGTH))?;

    Ok(Some(Params {
        precision: u8::try_from(precision).expect("supported depth fits u8"),
        components: u8::try_from(components).expect("1..=4 fits u8"),
        image_width,
        image_length,
    }))
}

/// Pixel dimensions of segment `index` under `layout`.
fn segment_dims(
    layout: &Layout,
    params: &Params,
    index: usize,
) -> Result<(u16, u16), OrchestrationError> {
    match *layout {
        Layout::Tiles {
            tile_width,
            tile_length,
        } => Ok((dim_u16(tile_width)?, dim_u16(tile_length)?)),
        Layout::Strips { rows_per_strip } => {
            if rows_per_strip == 0 {
                return Err(OrchestrationError::SegmentGeometry(
                    "rows_per_strip is zero".to_owned(),
                ));
            }
            let index = u32::try_from(index).unwrap_or(u32::MAX);
            let start = index.checked_mul(rows_per_strip).ok_or_else(|| {
                OrchestrationError::SegmentGeometry("strip start row overflow".to_owned())
            })?;
            if start >= params.image_length {
                return Err(OrchestrationError::SegmentGeometry(format!(
                    "strip {index} starts past image height {}",
                    params.image_length
                )));
            }
            let height = rows_per_strip.min(params.image_length - start);
            Ok((dim_u16(params.image_width)?, dim_u16(height)?))
        }
    }
}

fn dim_u16(value: u32) -> Result<u16, OrchestrationError> {
    u16::try_from(value).map_err(|_| OrchestrationError::DimensionTooLarge(value))
}

fn first_u32(ifd: &Ifd, tag: u16) -> Option<u32> {
    ifd.get(tag)
        .and_then(Value::as_u32_vec)
        .and_then(|v| v.into_iter().next())
}

#[cfg(test)]
mod tests {
    use super::compress_dng;
    use super::decompress_dng;
    use crate::error::OrchestrationError;
    use aleph_container::Dng;
    use aleph_container::Endian;
    use aleph_container::Entry;
    use aleph_container::Ifd;
    use aleph_container::Image;
    use aleph_container::Layout;
    use aleph_container::Value;

    fn raw_ifd(
        endian: Endian,
        width: u32,
        length: u32,
        bits: u16,
        spp: u16,
        layout: Layout,
        segments: Vec<Vec<u8>>,
    ) -> Ifd {
        let mut ifd = Ifd::default();
        ifd.set(256, Value::Long(vec![width]));
        ifd.set(257, Value::Long(vec![length]));
        ifd.set(258, Value::Short(vec![bits; spp as usize]));
        ifd.set(259, Value::Short(vec![1]));
        ifd.set(277, Value::Short(vec![spp]));
        ifd.set(0x9000, Value::Ascii(b"keepme\0".to_vec()));
        ifd.image = Some(Image { layout, segments });
        let _ = endian;
        ifd
    }

    fn bytes16(values: &[u16], endian: Endian) -> Vec<u8> {
        values
            .iter()
            .flat_map(|&v| match endian {
                Endian::Little => v.to_le_bytes(),
                Endian::Big => v.to_be_bytes(),
            })
            .collect()
    }

    #[test]
    fn round_trips_tiled_16bit() {
        let endian = Endian::Little;
        // 4x4 image, 2x2 tiles -> 4 tiles, each 4 samples.
        let tiles: Vec<Vec<u8>> = (0..4)
            .map(|t| {
                let base = u16::try_from(t).unwrap() * 100;
                bytes16(&[base, base + 1, base + 2, base + 3], endian)
            })
            .collect();
        let ifd = raw_ifd(
            endian,
            4,
            4,
            16,
            1,
            Layout::Tiles {
                tile_width: 2,
                tile_length: 2,
            },
            tiles,
        );
        let dng = Dng {
            endian,
            ifds: vec![ifd],
        };

        let compressed = compress_dng(&dng).unwrap();
        assert_eq!(compressed.ifds[0].get(259), Some(&Value::Short(vec![7])));
        let restored = decompress_dng(&compressed).unwrap();
        assert_eq!(restored, dng);
    }

    #[test]
    fn round_trips_stripped_big_endian_three_components() {
        let endian = Endian::Big;
        // 2x4 RGB image, rows_per_strip = 2 -> 2 strips of 2 rows * 2 px * 3 ch.
        let strip0 = bytes16(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], endian);
        let strip1 = bytes16(&[13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24], endian);
        let ifd = raw_ifd(
            endian,
            2,
            4,
            16,
            3,
            Layout::Strips { rows_per_strip: 2 },
            vec![strip0, strip1],
        );
        let dng = Dng {
            endian,
            ifds: vec![ifd],
        };

        let compressed = compress_dng(&dng).unwrap();
        let restored = decompress_dng(&compressed).unwrap();
        assert_eq!(restored, dng);
    }

    #[test]
    fn round_trips_through_subifd() {
        let endian = Endian::Little;
        let raw = raw_ifd(
            endian,
            2,
            2,
            16,
            1,
            Layout::Strips { rows_per_strip: 2 },
            vec![bytes16(&[100, 200, 300, 400], endian)],
        );
        let mut top = Ifd::default();
        top.entries.push(Entry {
            tag: 254,
            value: Value::Long(vec![1]),
        });
        top.sub_ifds.push(raw);
        let dng = Dng {
            endian,
            ifds: vec![top],
        };

        let compressed = compress_dng(&dng).unwrap();
        assert_eq!(
            compressed.ifds[0].sub_ifds[0].get(259),
            Some(&Value::Short(vec![7]))
        );
        let restored = decompress_dng(&compressed).unwrap();
        assert_eq!(restored, dng);
    }

    #[test]
    fn already_compressed_directory_is_left_alone() {
        let endian = Endian::Little;
        let mut ifd = raw_ifd(
            endian,
            2,
            2,
            16,
            1,
            Layout::Strips { rows_per_strip: 2 },
            vec![bytes16(&[1, 2, 3, 4], endian)],
        );
        ifd.set(259, Value::Short(vec![7]));
        let dng = Dng {
            endian,
            ifds: vec![ifd],
        };
        let compressed = compress_dng(&dng).unwrap();
        assert_eq!(compressed, dng);
    }

    #[test]
    fn round_trips_12bit_with_trailing_padding() {
        let endian = Endian::Little;
        // 2 samples (0x102, 0x103) MSB-first = [0x10,0x21,0x03], then strip padding.
        let mut segment = vec![0x10, 0x21, 0x03];
        segment.extend_from_slice(&[0xAB, 0xCD, 0xEF]);
        let ifd = raw_ifd(
            endian,
            2,
            1,
            12,
            1,
            Layout::Strips { rows_per_strip: 1 },
            vec![segment],
        );
        let dng = Dng {
            endian,
            ifds: vec![ifd],
        };

        let compressed = compress_dng(&dng).unwrap();
        let restored = decompress_dng(&compressed).unwrap();
        // Pixels and the trailing padding must both round-trip byte-exactly.
        assert_eq!(restored, dng);
    }

    #[test]
    fn unsupported_bit_depth_is_rejected() {
        // 11-bit is not in the supported set {8,10,12,14,16}.
        let endian = Endian::Little;
        let ifd = raw_ifd(
            endian,
            2,
            2,
            11,
            1,
            Layout::Strips { rows_per_strip: 2 },
            vec![vec![0u8; 8]],
        );
        let dng = Dng {
            endian,
            ifds: vec![ifd],
        };
        assert!(matches!(
            compress_dng(&dng),
            Err(OrchestrationError::UnsupportedBitDepth(11))
        ));
    }

    #[test]
    fn unaligned_subbyte_row_is_rejected() {
        // 3 samples/row * 12 bits = 36 bits: not byte-aligned, so not reversible.
        let endian = Endian::Little;
        let ifd = raw_ifd(
            endian,
            3,
            1,
            12,
            1,
            Layout::Strips { rows_per_strip: 1 },
            vec![vec![0u8; 8]],
        );
        let dng = Dng {
            endian,
            ifds: vec![ifd],
        };
        assert!(matches!(
            compress_dng(&dng),
            Err(OrchestrationError::UnsupportedRowAlignment { .. })
        ));
    }

    #[test]
    fn decompress_leaves_preexisting_compressed_preview_untouched() {
        let endian = Endian::Little;
        let raw = raw_ifd(
            endian,
            2,
            2,
            16,
            1,
            Layout::Strips { rows_per_strip: 2 },
            vec![bytes16(&[100, 200, 300, 400], endian)],
        );
        // A pre-existing lossless-JPEG preview (Compression=7, NO Aleph marker) —
        // exactly the shape that previously tripped decompress.
        let preview_jpeg = aleph_codec::encode(&aleph_codec::Frame {
            width: 2,
            height: 2,
            components: 1,
            precision: 16,
            samples: &[11, 22, 33, 44],
        })
        .unwrap();
        let mut preview = Ifd::default();
        preview.set(256, Value::Long(vec![2]));
        preview.set(257, Value::Long(vec![2]));
        preview.set(258, Value::Short(vec![16]));
        preview.set(259, Value::Short(vec![7]));
        preview.set(277, Value::Short(vec![1]));
        preview.image = Some(aleph_container::Image {
            layout: Layout::Strips { rows_per_strip: 2 },
            segments: vec![preview_jpeg],
        });
        let dng = Dng {
            endian,
            ifds: vec![raw, preview],
        };

        let compressed = compress_dng(&dng).unwrap();
        assert!(compressed.ifds[0]
            .get(aleph_metadata::tags::ALEPH_MARKER)
            .is_some());
        assert!(compressed.ifds[1]
            .get(aleph_metadata::tags::ALEPH_MARKER)
            .is_none());

        let restored = decompress_dng(&compressed).unwrap();
        // The pre-existing preview must be byte-identical (never decoded), and the
        // raw must round-trip.
        assert_eq!(restored, dng);
        assert_eq!(restored.ifds[1].image, dng.ifds[1].image);
    }

    #[test]
    fn rejects_input_with_conflicting_marker_tag() {
        let endian = Endian::Little;
        let mut ifd = raw_ifd(
            endian,
            2,
            2,
            16,
            1,
            Layout::Strips { rows_per_strip: 2 },
            vec![bytes16(&[1, 2, 3, 4], endian)],
        );
        // A foreign value already occupying Aleph's private marker number.
        ifd.set(aleph_metadata::tags::ALEPH_MARKER, Value::Short(vec![42]));
        let dng = Dng {
            endian,
            ifds: vec![ifd],
        };
        assert!(matches!(
            compress_dng(&dng),
            Err(OrchestrationError::MarkerCollision(_))
        ));
    }

    #[test]
    fn decompress_ignores_foreign_marker_tag() {
        let endian = Endian::Little;
        // A Compression=7 directory carrying a FOREIGN value at the marker number
        // (not Aleph's signature) must be left byte-identical by decompress.
        let mut ifd = raw_ifd(
            endian,
            2,
            2,
            16,
            1,
            Layout::Strips { rows_per_strip: 2 },
            vec![b"opaque-not-our-stream".to_vec()],
        );
        ifd.set(259, Value::Short(vec![7]));
        ifd.set(aleph_metadata::tags::ALEPH_MARKER, Value::Short(vec![42]));
        let dng = Dng {
            endian,
            ifds: vec![ifd],
        };
        let restored = decompress_dng(&dng).unwrap();
        assert_eq!(restored, dng);
    }
}
