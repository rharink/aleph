//! Pulling develop parameters and the packed raw segment out of a parsed DNG.

use aleph_container::Dng;
use aleph_container::Ifd;
use aleph_container::Image;
use aleph_container::Layout;
use aleph_container::Value;

const TAG_IMAGE_WIDTH: u16 = 256;
const TAG_IMAGE_LENGTH: u16 = 257;
const TAG_BITS_PER_SAMPLE: u16 = 258;
const TAG_COMPRESSION: u16 = 259;
const TAG_PHOTOMETRIC: u16 = 262;
const TAG_SAMPLES_PER_PIXEL: u16 = 277;
const TAG_CFA_PATTERN: u16 = 33422;
const TAG_BLACK_LEVEL: u16 = 50714;
const TAG_WHITE_LEVEL: u16 = 50717;
const TAG_AS_SHOT_NEUTRAL: u16 = 50728;
const TAG_COLOR_MATRIX_1: u16 = 50721;
const TAG_COLOR_MATRIX_2: u16 = 50722;
const TAG_BASELINE_EXPOSURE: u16 = 50730;

const PHOTOMETRIC_CFA: u32 = 32803;
const COMPRESSION_NONE: u32 = 1;
const SUPPORTED_DEPTHS: [u32; 5] = [8, 10, 12, 14, 16];

/// Everything the developer needs for one CFA frame.
pub struct DevelopParams {
    pub width: u32,
    pub height: u32,
    pub bits: u8,
    /// Black level per 2×2 CFA position (top-left, top-right, bottom-left, bottom-right).
    pub(crate) black: [f32; 4],
    pub(crate) white: f32,
    /// 2×2 CFA colours: 0 = red, 1 = green, 2 = blue.
    pub(crate) cfa: [u8; 4],
    /// As-shot neutral: the camera-space RGB of a neutral grey (DNG `AsShotNeutral`).
    pub(crate) neutral: [f32; 3],
    /// Linear gain from DNG `BaselineExposure` (2^EV); 1.0 when absent.
    pub(crate) exposure: f32,
    /// Camera-RGB → XYZ (inverse of the colour matrix); `None` skips colour conversion.
    pub(crate) cam_to_xyz: Option<[[f32; 3]; 3]>,
}

impl DevelopParams {
    #[must_use]
    pub fn pixel_count(&self) -> usize {
        self.width as usize * self.height as usize
    }
}

/// A raw frame ready to develop: its parameters and the packed sample bytes
/// (the caller unpacks them at `params.bits` before calling `develop`).
pub struct RawFrame {
    pub params: DevelopParams,
    pub packed: Vec<u8>,
}

/// Find the largest develop-able CFA frame in a parsed DNG, or `None` when there
/// is no uncompressed single-component CFA image this build can render.
#[must_use]
pub fn raw_frame(dng: &Dng) -> Option<RawFrame> {
    let mut best: Option<RawFrame> = None;
    for ifd in &dng.ifds {
        consider(ifd, &mut best);
    }
    best
}

fn consider(ifd: &Ifd, best: &mut Option<RawFrame>) {
    if let Some(frame) = frame_from(ifd) {
        let area = u64::from(frame.params.width) * u64::from(frame.params.height);
        let best_area = best.as_ref().map_or(0, |f| {
            u64::from(f.params.width) * u64::from(f.params.height)
        });
        if best.is_none() || area > best_area {
            *best = Some(frame);
        }
    }
    for child in &ifd.sub_ifds {
        consider(child, best);
    }
}

fn frame_from(ifd: &Ifd) -> Option<RawFrame> {
    if scalar(ifd, TAG_PHOTOMETRIC)? as u32 != PHOTOMETRIC_CFA {
        return None;
    }
    if scalar(ifd, TAG_SAMPLES_PER_PIXEL).unwrap_or(1.0) as u32 != 1 {
        return None;
    }
    if scalar(ifd, TAG_COMPRESSION).unwrap_or(COMPRESSION_NONE as f32) as u32 != COMPRESSION_NONE {
        return None;
    }
    let bits = scalar(ifd, TAG_BITS_PER_SAMPLE)? as u32;
    if !SUPPORTED_DEPTHS.contains(&bits) {
        return None;
    }
    let width = scalar(ifd, TAG_IMAGE_WIDTH)? as u32;
    let height = scalar(ifd, TAG_IMAGE_LENGTH)? as u32;
    if width == 0 || height == 0 {
        return None;
    }
    let image = ifd.image.as_ref()?;

    let packed = pack_rows(image, width, height, bits)?;
    let params = DevelopParams {
        width,
        height,
        bits: bits as u8,
        black: black_level(ifd),
        white: scalar(ifd, TAG_WHITE_LEVEL).unwrap_or(((1u32 << bits) - 1) as f32),
        cfa: cfa_pattern(ifd),
        neutral: as_shot_neutral(ifd),
        exposure: scalar(ifd, TAG_BASELINE_EXPOSURE).map_or(1.0, |ev| 2.0f32.powf(ev)),
        cam_to_xyz: invert3(color_matrix(ifd)),
    };
    Some(RawFrame { params, packed })
}

// Reassemble row-major packed sample bytes from the image segments, trimming any
// per-strip padding. Mirrors the codec's per-segment layout. Returns None for
// layouts we can't faithfully linearise here: tiles (stored in tile order, not
// row order) and rows that aren't byte-aligned (a contiguous bit-unpacker would
// drift across rows).
fn pack_rows(image: &Image, width: u32, height: u32, bits: u32) -> Option<Vec<u8>> {
    let rows_per_strip = match image.layout {
        Layout::Strips { rows_per_strip } => rows_per_strip as usize,
        Layout::Tiles { .. } => return None,
    };

    let w = width as usize;
    let h = height as usize;
    let bits = bits as usize;
    // CFA preview is single-component; require byte-aligned rows.
    if !(bits.is_multiple_of(8) || (w * bits).is_multiple_of(8)) {
        return None;
    }
    let bytes_per_row = (w * bits).div_ceil(8);
    let rows_per_strip = if rows_per_strip == 0 {
        h
    } else {
        rows_per_strip
    };

    let mut packed = Vec::with_capacity(h * bytes_per_row);
    for (index, segment) in image.segments.iter().enumerate() {
        let start = index.checked_mul(rows_per_strip)?;
        if start >= h {
            break;
        }
        let need = (h - start).min(rows_per_strip) * bytes_per_row;
        if segment.len() < need {
            return None; // truncated strip
        }
        packed.extend_from_slice(&segment[..need]);
    }

    (packed.len() == h * bytes_per_row).then_some(packed)
}

fn black_level(ifd: &Ifd) -> [f32; 4] {
    let values = nums(ifd, TAG_BLACK_LEVEL);
    match values.len() {
        0 => [0.0; 4],
        1 => [values[0]; 4],
        _ => [
            values[0],
            values[1 % values.len()],
            values[2 % values.len()],
            values[3 % values.len()],
        ],
    }
}

fn cfa_pattern(ifd: &Ifd) -> [u8; 4] {
    let values = nums(ifd, TAG_CFA_PATTERN);
    if values.len() >= 4 {
        [
            values[0] as u8,
            values[1] as u8,
            values[2] as u8,
            values[3] as u8,
        ]
    } else {
        [0, 1, 1, 2] // RGGB
    }
}

fn as_shot_neutral(ifd: &Ifd) -> [f32; 3] {
    let n = nums(ifd, TAG_AS_SHOT_NEUTRAL);
    if n.len() < 3 || n.iter().any(|&v| v <= 0.0) {
        return [1.0, 1.0, 1.0];
    }
    [n[0], n[1], n[2]]
}

fn color_matrix(ifd: &Ifd) -> Option<[[f32; 3]; 3]> {
    // Prefer the D65 matrix (ColorMatrix2) — it matches the sRGB white point;
    // fall back to ColorMatrix1 (often a tungsten illuminant).
    let v = {
        let cm2 = nums(ifd, TAG_COLOR_MATRIX_2);
        if cm2.len() >= 9 {
            cm2
        } else {
            nums(ifd, TAG_COLOR_MATRIX_1)
        }
    };
    if v.len() < 9 {
        return None;
    }
    Some([[v[0], v[1], v[2]], [v[3], v[4], v[5]], [v[6], v[7], v[8]]])
}

fn invert3(m: Option<[[f32; 3]; 3]>) -> Option<[[f32; 3]; 3]> {
    let m = m?;
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    if det.abs() < 1e-9 {
        return None;
    }
    let inv_det = 1.0 / det;
    Some([
        [
            (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * inv_det,
            (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * inv_det,
            (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * inv_det,
        ],
        [
            (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * inv_det,
            (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * inv_det,
            (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * inv_det,
        ],
        [
            (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * inv_det,
            (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * inv_det,
            (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * inv_det,
        ],
    ])
}

/// First element of a tag's value as `f32`, if present.
fn scalar(ifd: &Ifd, tag: u16) -> Option<f32> {
    nums(ifd, tag).first().copied()
}

/// A tag's numeric payload as `f32`, widening any integer/rational type.
fn nums(ifd: &Ifd, tag: u16) -> Vec<f32> {
    let Some(value) = ifd.get(tag) else {
        return Vec::new();
    };
    match value {
        Value::Byte(v) | Value::Undefined(v) => v.iter().map(|&x| f32::from(x)).collect(),
        Value::SByte(v) => v.iter().map(|&x| f32::from(x)).collect(),
        Value::Short(v) => v.iter().map(|&x| f32::from(x)).collect(),
        Value::SShort(v) => v.iter().map(|&x| f32::from(x)).collect(),
        Value::Long(v) => v.iter().map(|&x| x as f32).collect(),
        Value::SLong(v) => v.iter().map(|&x| x as f32).collect(),
        Value::Float(v) => v.clone(),
        Value::Double(v) => v.iter().map(|&x| x as f32).collect(),
        Value::Rational(v) => v
            .iter()
            .map(|&(n, d)| if d == 0 { 0.0 } else { n as f32 / d as f32 })
            .collect(),
        Value::SRational(v) => v
            .iter()
            .map(|&(n, d)| if d == 0 { 0.0 } else { n as f32 / d as f32 })
            .collect(),
        Value::Ascii(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::raw_frame;
    use aleph_container::Dng;
    use aleph_container::Endian;
    use aleph_container::Ifd;
    use aleph_container::Image;
    use aleph_container::Layout;
    use aleph_container::Value;

    fn cfa_ifd() -> Ifd {
        let mut ifd = Ifd::default();
        ifd.set(256, Value::Long(vec![4]));
        ifd.set(257, Value::Long(vec![2]));
        ifd.set(258, Value::Short(vec![12]));
        ifd.set(259, Value::Short(vec![1]));
        ifd.set(262, Value::Short(vec![32803]));
        ifd.set(277, Value::Short(vec![1]));
        ifd.set(33422, Value::Byte(vec![0, 1, 1, 2]));
        ifd.set(50717, Value::Long(vec![4095]));
        ifd.set(50714, Value::Short(vec![256]));
        ifd.set(50728, Value::Rational(vec![(1, 2), (1, 1), (3, 4)]));
        ifd.image = Some(Image {
            layout: Layout::Strips { rows_per_strip: 2 },
            segments: vec![vec![0u8; 12]], // 4*2 samples * 12bit = 96 bits = 12 bytes
        });
        ifd
    }

    #[test]
    fn extracts_cfa_params() {
        let dng = Dng {
            endian: Endian::Little,
            ifds: vec![cfa_ifd()],
        };
        let frame = raw_frame(&dng).expect("frame");
        assert_eq!((frame.params.width, frame.params.height), (4, 2));
        assert_eq!(frame.params.bits, 12);
        assert!((frame.params.white - 4095.0).abs() < 1e-3);
        assert!(frame.params.black.iter().all(|&b| (b - 256.0).abs() < 1e-3));
        assert_eq!(frame.params.cfa, [0, 1, 1, 2]);
        // AsShotNeutral = [1/2, 1/1, 3/4].
        assert!((frame.params.neutral[0] - 0.5).abs() < 1e-5);
        assert!((frame.params.neutral[1] - 1.0).abs() < 1e-5);
        assert!((frame.params.neutral[2] - 0.75).abs() < 1e-5);
        assert_eq!(frame.packed.len(), 12);
    }

    #[test]
    fn skips_non_cfa() {
        let mut rgb = Ifd::default();
        rgb.set(262, Value::Short(vec![2])); // RGB, not CFA
        rgb.set(256, Value::Long(vec![4]));
        rgb.set(257, Value::Long(vec![2]));
        let dng = Dng {
            endian: Endian::Little,
            ifds: vec![rgb],
        };
        assert!(raw_frame(&dng).is_none());
    }

    #[test]
    fn skips_compressed_raw() {
        let mut ifd = cfa_ifd();
        ifd.set(259, Value::Short(vec![7])); // JPEG-compressed — can't unpack here
        let dng = Dng {
            endian: Endian::Little,
            ifds: vec![ifd],
        };
        assert!(raw_frame(&dng).is_none());
    }

    #[test]
    fn reassembles_padded_strips() {
        let mut ifd = Ifd::default();
        ifd.set(256, Value::Long(vec![4]));
        ifd.set(257, Value::Long(vec![2]));
        ifd.set(258, Value::Short(vec![8]));
        ifd.set(259, Value::Short(vec![1]));
        ifd.set(262, Value::Short(vec![32803]));
        ifd.set(277, Value::Short(vec![1]));
        ifd.image = Some(Image {
            layout: Layout::Strips { rows_per_strip: 1 },
            segments: vec![
                vec![10, 11, 12, 13, 99], // one 4-byte row + trailing strip padding
                vec![20, 21, 22, 23, 99],
            ],
        });
        let dng = Dng {
            endian: Endian::Little,
            ifds: vec![ifd],
        };
        let frame = raw_frame(&dng).expect("frame");
        // Padding trimmed; rows concatenated in order.
        assert_eq!(frame.packed, vec![10, 11, 12, 13, 20, 21, 22, 23]);
    }

    #[test]
    fn declines_tiles() {
        let mut ifd = cfa_ifd();
        ifd.image = Some(Image {
            layout: Layout::Tiles {
                tile_width: 2,
                tile_length: 2,
            },
            segments: vec![vec![0u8; 12]],
        });
        let dng = Dng {
            endian: Endian::Little,
            ifds: vec![ifd],
        };
        assert!(raw_frame(&dng).is_none());
    }
}
