use crate::bitio::BitWriter;
use crate::error::CodecError;
use crate::huffman::HuffmanTable;
use crate::marker;
use crate::sample;

pub struct Frame<'a> {
    pub width: u16,
    pub height: u16,
    pub components: u8,
    pub precision: u8,
    pub samples: &'a [u16],
}

/// Encode a rectangular sample array as a lossless-JPEG (SOF3, process 14)
/// bitstream that `decode` reverses bit-exactly.
///
/// # Errors
/// Returns `CodecError` if `components` is outside `1..=4`, `precision` outside
/// `2..=16`, either dimension is zero, `samples.len()` does not equal
/// `width * height * components`, or any sample does not fit in `precision` bits.
pub fn encode(frame: &Frame<'_>) -> Result<Vec<u8>, CodecError> {
    validate(frame)?;

    let width = usize::from(frame.width);
    let height = usize::from(frame.height);
    let components = usize::from(frame.components);

    let diffs = differences(frame, width, height, components);

    let mut histogram = [0u32; 17];
    for &d in &diffs {
        histogram[usize::from(category(d))] += 1;
    }
    let table = HuffmanTable::from_histogram(&histogram);

    let mut out = Vec::new();
    write_marker(&mut out, marker::SOI);
    write_frame_header(&mut out, frame);
    write_huffman_table(&mut out, &table);
    write_scan_header(&mut out, frame);
    write_entropy(&mut out, &diffs, &table);
    write_marker(&mut out, marker::EOI);

    Ok(out)
}

fn validate(frame: &Frame<'_>) -> Result<(), CodecError> {
    if !(1..=4).contains(&frame.components) {
        return Err(CodecError::InvalidComponents(frame.components));
    }
    if !(2..=16).contains(&frame.precision) {
        return Err(CodecError::InvalidPrecision(frame.precision));
    }
    if frame.width == 0 || frame.height == 0 {
        return Err(CodecError::InvalidDimensions {
            width: frame.width,
            height: frame.height,
        });
    }

    let expected =
        usize::from(frame.width) * usize::from(frame.height) * usize::from(frame.components);
    if frame.samples.len() != expected {
        return Err(CodecError::SampleCountMismatch {
            expected,
            actual: frame.samples.len(),
        });
    }

    let max = (1u32 << frame.precision) - 1;
    for &s in frame.samples {
        if u32::from(s) > max {
            return Err(CodecError::SampleOutOfRange {
                value: s,
                precision: frame.precision,
            });
        }
    }
    Ok(())
}

// Reconstruction is exact, so the encoder predicts from the original samples
// (they equal the decoder's reconstructed neighbors).
fn differences(frame: &Frame<'_>, width: usize, height: usize, components: usize) -> Vec<i16> {
    let mut diffs = vec![0i16; frame.samples.len()];
    for y in 0..height {
        for x in 0..width {
            for c in 0..components {
                let i = sample::index(x, y, c, width, components);
                let px =
                    sample::predict(frame.samples, x, y, c, width, components, frame.precision);
                let d = frame.samples[i].wrapping_sub(px);
                diffs[i] = i16::from_ne_bytes(d.to_ne_bytes());
            }
        }
    }
    diffs
}

fn write_entropy(out: &mut Vec<u8>, diffs: &[i16], table: &HuffmanTable) {
    let mut writer = BitWriter::with_capacity(diffs.len());
    for &d in diffs {
        let ssss = category(d);
        let (code, size) = table.code(ssss);
        writer.write_bits(u32::from(code), u32::from(size));

        if ssss > 0 && ssss < 16 {
            let (value, count) = value_bits(d, ssss);
            writer.write_bits(value, count);
        }
    }
    out.extend_from_slice(&writer.finish());
}

// Magnitude category SSSS: bit count of |d|, with the d == -32768 special case
// mapped to 16 (it carries no value bits).
fn category(d: i16) -> u8 {
    if d == 0 {
        return 0;
    }
    let bits = u16::BITS - d.unsigned_abs().leading_zeros();
    u8::try_from(bits).expect("category in 0..=16")
}

// JPEG value mapping: V = d for d >= 0, else V = d + (2^s - 1); always in
// [0, 2^s - 1].
fn value_bits(d: i16, ssss: u8) -> (u32, u32) {
    let s = u32::from(ssss);
    let v = if d >= 0 {
        i32::from(d)
    } else {
        i32::from(d) + (1i32 << s) - 1
    };
    (u32::try_from(v).expect("mapped value is non-negative"), s)
}

fn write_frame_header(out: &mut Vec<u8>, frame: &Frame<'_>) {
    write_marker(out, marker::SOF3);
    let length = 8 + 3 * u16::from(frame.components);
    write_u16(out, length);
    out.push(frame.precision);
    write_u16(out, frame.height);
    write_u16(out, frame.width);
    out.push(frame.components);
    for c in 0..frame.components {
        out.push(c + 1);
        out.push(0x11);
        out.push(0x00);
    }
}

fn write_huffman_table(out: &mut Vec<u8>, table: &HuffmanTable) {
    write_marker(out, marker::DHT);
    let length = 2 + 1 + 16 + u16::try_from(table.values.len()).expect("at most 17 symbols");
    write_u16(out, length);
    out.push(0x00);
    out.extend_from_slice(&table.bits);
    out.extend_from_slice(&table.values);
}

fn write_scan_header(out: &mut Vec<u8>, frame: &Frame<'_>) {
    write_marker(out, marker::SOS);
    let length = 6 + 2 * u16::from(frame.components);
    write_u16(out, length);
    out.push(frame.components);
    for c in 0..frame.components {
        out.push(c + 1);
        out.push(0x00);
    }
    out.push(0x01);
    out.push(0x00);
    out.push(0x00);
}

fn write_marker(out: &mut Vec<u8>, value: u16) {
    write_u16(out, value);
}

fn write_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::Frame;
    use crate::decode::decode;
    use crate::encode::encode;
    use proptest::prelude::Just;
    use proptest::prelude::Strategy;
    use proptest::prop_assert_eq;
    use proptest::proptest;

    fn assert_roundtrip(width: u16, height: u16, components: u8, precision: u8, samples: &[u16]) {
        let frame = Frame {
            width,
            height,
            components,
            precision,
            samples,
        };
        let bytes = encode(&frame).expect("encode succeeds");
        let decoded = decode(&bytes).expect("decode succeeds");
        assert_eq!(decoded.width, width);
        assert_eq!(decoded.height, height);
        assert_eq!(decoded.components, components);
        assert_eq!(decoded.precision, precision);
        assert_eq!(decoded.samples, samples);
    }

    #[test]
    fn single_pixel() {
        assert_roundtrip(1, 1, 1, 8, &[123]);
    }

    #[test]
    fn single_row() {
        let samples: Vec<u16> = (0..16u16).collect();
        assert_roundtrip(16, 1, 1, 12, &samples);
    }

    #[test]
    fn single_column() {
        let samples: Vec<u16> = (0..16u16).map(|v| v * 3).collect();
        assert_roundtrip(1, 16, 1, 12, &samples);
    }

    #[test]
    fn all_zeros() {
        let samples = vec![0u16; 8 * 8];
        assert_roundtrip(8, 8, 1, 16, &samples);
    }

    #[test]
    fn all_max_precision_16() {
        let samples = vec![u16::MAX; 8 * 8];
        assert_roundtrip(8, 8, 1, 16, &samples);
    }

    #[test]
    fn all_max_precision_2() {
        let samples = vec![3u16; 5 * 7];
        assert_roundtrip(5, 7, 1, 2, &samples);
    }

    #[test]
    fn three_components() {
        let samples: Vec<u16> = (0..(4u16 * 4 * 3)).map(|v| (v * 17) % 256).collect();
        assert_roundtrip(4, 4, 3, 8, &samples);
    }

    #[test]
    fn four_components_min_max_mix() {
        let samples = vec![0u16, 4095, 2048, 1, 4094, 7, 4095, 0];
        assert_roundtrip(1, 2, 4, 12, &samples);
    }

    #[test]
    fn structural_markers() {
        let frame = Frame {
            width: 3,
            height: 3,
            components: 1,
            precision: 8,
            samples: &[1, 2, 3, 4, 5, 6, 7, 8, 9],
        };
        let bytes = encode(&frame).expect("encode succeeds");
        assert_eq!(&bytes[..2], &[0xFF, 0xD8]);
        assert_eq!(&bytes[bytes.len() - 2..], &[0xFF, 0xD9]);
    }

    #[test]
    fn rejects_bad_components() {
        let frame = Frame {
            width: 1,
            height: 1,
            components: 0,
            precision: 8,
            samples: &[],
        };
        assert!(encode(&frame).is_err());
    }

    #[test]
    fn rejects_bad_precision() {
        let frame = Frame {
            width: 1,
            height: 1,
            components: 1,
            precision: 1,
            samples: &[0],
        };
        assert!(encode(&frame).is_err());
    }

    #[test]
    fn rejects_count_mismatch() {
        let frame = Frame {
            width: 2,
            height: 2,
            components: 1,
            precision: 8,
            samples: &[1, 2, 3],
        };
        assert!(encode(&frame).is_err());
    }

    #[test]
    fn rejects_sample_out_of_range() {
        let frame = Frame {
            width: 1,
            height: 1,
            components: 1,
            precision: 8,
            samples: &[256],
        };
        assert!(encode(&frame).is_err());
    }

    proptest! {
        #[test]
        fn roundtrip_random(
            (width, height, components, precision, raw) in
                (1u16..=64, 1u16..=64, 1u8..=4, 2u8..=16).prop_flat_map(
                    |(w, h, comp, prec)| {
                        let n = usize::from(w) * usize::from(h) * usize::from(comp);
                        let max = (1u32 << prec) - 1;
                        (
                            Just((w, h, comp, prec)),
                            proptest::collection::vec(0u32..=max, n),
                        )
                    },
                ).prop_map(|((w, h, comp, prec), v)| (w, h, comp, prec, v)),
        ) {
            let samples: Vec<u16> = raw
                .iter()
                .map(|&v| u16::try_from(v).expect("masked to precision"))
                .collect();
            let frame = Frame {
                width,
                height,
                components,
                precision,
                samples: &samples,
            };
            let bytes = encode(&frame).expect("encode succeeds");
            let decoded = decode(&bytes).expect("decode succeeds");
            prop_assert_eq!(decoded.width, width);
            prop_assert_eq!(decoded.height, height);
            prop_assert_eq!(decoded.components, components);
            prop_assert_eq!(decoded.precision, precision);
            prop_assert_eq!(decoded.samples, samples);
        }
    }
}
