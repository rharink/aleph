//! Conversion between opaque container segment bytes and `u16` samples.
//!
//! Byte-aligned depths (8, 16 bit) use the container's byte order. Sub-byte
//! depths (10, 12, 14 bit) use DNG's default big-endian, MSB-first bit packing
//! (`FillOrder = 1`), which is independent of the file's byte order — verified
//! against real SIGMA fp 12-bit `CinemaDNG`. Rows must be byte-aligned; the caller
//! enforces this before packing so packing is exactly reversible.

use aleph_container::Endian;

use crate::error::OrchestrationError;

/// Decode the first `sample_count` samples of `bytes` at `precision`. Bytes
/// beyond the decoded samples (strip/tile padding) are ignored.
///
/// # Errors
/// [`OrchestrationError::UnsupportedBitDepth`] for unsupported depths, or
/// [`OrchestrationError::SegmentTooShort`] if `bytes` cannot supply
/// `sample_count` samples.
pub fn unpack(
    bytes: &[u8],
    precision: u8,
    endian: Endian,
    sample_count: usize,
) -> Result<Vec<u16>, OrchestrationError> {
    match precision {
        8 => {
            if bytes.len() < sample_count {
                return Err(OrchestrationError::SegmentTooShort {
                    need: sample_count,
                    have: bytes.len(),
                });
            }
            Ok(bytes[..sample_count]
                .iter()
                .map(|&b| u16::from(b))
                .collect())
        }
        16 => {
            let need = sample_count * 2;
            if bytes.len() < need {
                return Err(OrchestrationError::SegmentTooShort {
                    need,
                    have: bytes.len(),
                });
            }
            let mut out = Vec::with_capacity(sample_count);
            for pair in bytes[..need].chunks_exact(2) {
                let bytes = [pair[0], pair[1]];
                out.push(match endian {
                    Endian::Little => u16::from_le_bytes(bytes),
                    Endian::Big => u16::from_be_bytes(bytes),
                });
            }
            Ok(out)
        }
        10 | 12 | 14 => unpack_msb(bytes, precision, sample_count),
        other => Err(OrchestrationError::UnsupportedBitDepth(u32::from(other))),
    }
}

/// Encode `samples` to segment bytes at `precision`.
///
/// # Errors
/// [`OrchestrationError::UnsupportedBitDepth`] for unsupported depths.
pub(crate) fn pack(
    samples: &[u16],
    precision: u8,
    endian: Endian,
) -> Result<Vec<u8>, OrchestrationError> {
    match precision {
        // Samples decoded at precision 8 are < 256, so the low byte is the value.
        8 => Ok(samples.iter().map(|&s| s.to_le_bytes()[0]).collect()),
        16 => {
            let mut out = Vec::with_capacity(samples.len() * 2);
            for &s in samples {
                let bytes = match endian {
                    Endian::Little => s.to_le_bytes(),
                    Endian::Big => s.to_be_bytes(),
                };
                out.extend_from_slice(&bytes);
            }
            Ok(out)
        }
        10 | 12 | 14 => Ok(pack_msb(samples, precision)),
        other => Err(OrchestrationError::UnsupportedBitDepth(u32::from(other))),
    }
}

/// Whether this build can (un)pack `precision`-bit samples.
pub(crate) fn is_supported_depth(precision: u32) -> bool {
    matches!(precision, 8 | 10 | 12 | 14 | 16)
}

// Big-endian, MSB-first bit reader (DNG FillOrder = 1).
fn unpack_msb(
    bytes: &[u8],
    precision: u8,
    sample_count: usize,
) -> Result<Vec<u16>, OrchestrationError> {
    let width = u32::from(precision);
    let mask = (1u32 << width) - 1;

    let mut out = Vec::with_capacity(sample_count);
    let mut acc = 0u32;
    let mut nbits = 0u32;
    let mut index = 0usize;
    for _ in 0..sample_count {
        while nbits < width {
            let byte = *bytes
                .get(index)
                .ok_or(OrchestrationError::SegmentTooShort {
                    need: (sample_count * precision as usize).div_ceil(8),
                    have: bytes.len(),
                })?;
            index += 1;
            acc = (acc << 8) | u32::from(byte);
            nbits += 8;
        }
        nbits -= width;
        let sample = (acc >> nbits) & mask;
        out.push(u16::try_from(sample).expect("masked to <= 16 bits"));
        acc &= (1u32 << nbits) - 1; // drop the bits just consumed
    }
    Ok(out)
}

// Big-endian, MSB-first bit writer; inverse of `unpack_msb`.
fn pack_msb(samples: &[u16], precision: u8) -> Vec<u8> {
    let width = u32::from(precision);

    let mut out = Vec::with_capacity((samples.len() * precision as usize).div_ceil(8));
    let mut acc = 0u32;
    let mut nbits = 0u32;
    for &sample in samples {
        acc = (acc << width) | u32::from(sample);
        nbits += width;
        while nbits >= 8 {
            nbits -= 8;
            out.push(u8::try_from((acc >> nbits) & 0xFF).expect("masked to a byte"));
        }
        acc &= (1u32 << nbits) - 1; // keep only the not-yet-emitted bits
    }
    if nbits > 0 {
        // Pad the final partial byte with zero bits in the low positions.
        out.push(u8::try_from((acc << (8 - nbits)) & 0xFF).expect("masked to a byte"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::pack;
    use super::unpack;
    use aleph_container::Endian;
    use proptest::collection::vec;
    use proptest::prelude::any;
    use proptest::prop_assert_eq;
    use proptest::proptest;

    #[test]
    fn unpack_16bit_respects_endianness() {
        let bytes = [0x34, 0x12, 0x78, 0x56];
        assert_eq!(
            unpack(&bytes, 16, Endian::Little, 2).unwrap(),
            vec![0x1234, 0x5678]
        );
        assert_eq!(
            unpack(&bytes, 16, Endian::Big, 2).unwrap(),
            vec![0x3412, 0x7856]
        );
    }

    #[test]
    fn twelve_bit_is_msb_first() {
        // Locks the DNG 12-bit convention validated against real footage:
        // [0x10,0x21,0x03] -> 0x102, 0x103.
        let bytes = [0x10, 0x21, 0x03];
        assert_eq!(
            unpack(&bytes, 12, Endian::Little, 2).unwrap(),
            vec![0x102, 0x103]
        );
        // Byte order must not matter for sub-byte packing.
        assert_eq!(
            unpack(&bytes, 12, Endian::Big, 2).unwrap(),
            vec![0x102, 0x103]
        );
        assert_eq!(pack(&[0x102, 0x103], 12, Endian::Little).unwrap(), bytes);
    }

    #[test]
    fn unpack_ignores_trailing_padding() {
        // Two 12-bit samples occupy 3 bytes; extra bytes are padding.
        let bytes = [0x10, 0x21, 0x03, 0xFF, 0xFF];
        assert_eq!(
            unpack(&bytes, 12, Endian::Little, 2).unwrap(),
            vec![0x102, 0x103]
        );
    }

    #[test]
    fn too_short_segment_is_rejected() {
        assert!(unpack(&[0u8; 3], 16, Endian::Little, 2).is_err());
        assert!(unpack(&[0u8; 2], 12, Endian::Little, 2).is_err());
    }

    #[test]
    fn unsupported_depth_is_rejected() {
        assert!(unpack(&[0u8; 4], 9, Endian::Little, 2).is_err());
        assert!(pack(&[0u16; 4], 9, Endian::Little).is_err());
    }

    proptest! {
        #[test]
        fn pack_unpack_round_trips_16bit(
            samples in vec(any::<u16>(), 0..512),
            big in any::<bool>(),
        ) {
            let endian = if big { Endian::Big } else { Endian::Little };
            let bytes = pack(&samples, 16, endian).unwrap();
            prop_assert_eq!(unpack(&bytes, 16, endian, samples.len()).unwrap(), samples);
        }

        #[test]
        fn pack_unpack_round_trips_subbyte(
            count in 0usize..512,
            depth_sel in 0u8..3,
        ) {
            // 10, 12, or 14-bit. Use a deterministic, in-range sample pattern.
            let precision = [10u8, 12, 14][depth_sel as usize];
            let limit = 1u16 << precision;
            let samples: Vec<u16> =
                (0..count).map(|i| u16::try_from(i % 4096).unwrap().wrapping_mul(37) % limit).collect();
            let bytes = pack(&samples, precision, Endian::Little).unwrap();
            prop_assert_eq!(
                unpack(&bytes, precision, Endian::Little, samples.len()).unwrap(),
                samples
            );
        }

        #[test]
        fn pack_unpack_round_trips_8bit(values in vec(0u16..256, 0..512)) {
            let bytes = pack(&values, 8, Endian::Little).unwrap();
            prop_assert_eq!(unpack(&bytes, 8, Endian::Little, values.len()).unwrap(), values);
        }
    }
}
