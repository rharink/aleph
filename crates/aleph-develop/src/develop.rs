//! The pixel pipeline: normalise → demosaic → white balance → colour → transfer.

use crate::params::DevelopParams;

/// Output transfer function applied to linear RGB.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Transfer {
    /// sRGB — for web/display.
    Srgb,
    /// Rec. 709 — for editing proxies.
    Rec709,
}

/// A developed 8-bit RGBA image (row-major, 4 bytes/pixel).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

// Linear XYZ (D65) → linear sRGB.
const XYZ_TO_SRGB: [[f32; 3]; 3] = [
    [3.2406, -1.5372, -0.4986],
    [-0.9689, 1.8758, 0.0415],
    [0.0557, -0.2040, 1.0570],
];

/// Develop unpacked CFA `samples` into a display RGBA image.
///
/// `samples` holds `width * height` raw values at `params.bits`, row-major.
#[must_use]
pub fn develop(samples: &[u16], params: &DevelopParams, transfer: Transfer) -> RgbaImage {
    let width = params.width as usize;
    let height = params.height as usize;

    let linear = normalise(samples, params, width, height);
    // Camera→linear-sRGB, normalised so the as-shot neutral maps to white. That
    // single normalisation folds in white balance *and* the illuminant of the
    // colour matrix, so neutrals render neutral (no cast) at sensible exposure.
    let color = params.cam_to_xyz.map(|cam_to_xyz| {
        let c = matmul(XYZ_TO_SRGB, cam_to_xyz);
        let white = apply(c, params.neutral);
        (c, white)
    });
    let mut data = vec![0u8; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let cam = demosaic(&linear, params, width, height, x, y);
            let rgb = match color {
                Some((c, white)) => {
                    let lin = apply(c, cam);
                    [
                        lin[0] / white[0].max(1e-4),
                        lin[1] / white[1].max(1e-4),
                        lin[2] / white[2].max(1e-4),
                    ]
                }
                None => [
                    cam[0] / params.neutral[0],
                    cam[1] / params.neutral[1],
                    cam[2] / params.neutral[2],
                ],
            };
            let o = (y * width + x) * 4;
            data[o] = encode(rgb[0] * params.exposure, transfer);
            data[o + 1] = encode(rgb[1] * params.exposure, transfer);
            data[o + 2] = encode(rgb[2] * params.exposure, transfer);
            data[o + 3] = 255;
        }
    }

    RgbaImage {
        width: params.width,
        height: params.height,
        data,
    }
}

// Black-subtracted, white-normalised samples in [0, 1], per CFA position.
fn normalise(samples: &[u16], params: &DevelopParams, width: usize, height: usize) -> Vec<f32> {
    let mut linear = vec![0.0f32; width * height];
    for y in 0..height {
        for x in 0..width {
            let pos = (y & 1) * 2 + (x & 1);
            let black = params.black[pos];
            let range = (params.white - black).max(1.0);
            let idx = y * width + x;
            let raw = f32::from(*samples.get(idx).unwrap_or(&0));
            linear[idx] = ((raw - black) / range).clamp(0.0, 1.0);
        }
    }
    linear
}

// 3×3 colour-matched average: each output channel is the mean of the same-colour
// CFA samples in the neighbourhood (the centre contributes to its own colour).
fn demosaic(
    linear: &[f32],
    params: &DevelopParams,
    width: usize,
    height: usize,
    x: usize,
    y: usize,
) -> [f32; 3] {
    let mut sum = [0.0f32; 3];
    let mut count = [0u32; 3];
    let y0 = y.saturating_sub(1);
    let x0 = x.saturating_sub(1);
    let y1 = (y + 1).min(height - 1);
    let x1 = (x + 1).min(width - 1);
    for ny in y0..=y1 {
        for nx in x0..=x1 {
            let color = params.cfa[(ny & 1) * 2 + (nx & 1)] as usize;
            if color < 3 {
                sum[color] += linear[ny * width + nx];
                count[color] += 1;
            }
        }
    }
    [
        if count[0] > 0 {
            sum[0] / count[0] as f32
        } else {
            0.0
        },
        if count[1] > 0 {
            sum[1] / count[1] as f32
        } else {
            0.0
        },
        if count[2] > 0 {
            sum[2] / count[2] as f32
        } else {
            0.0
        },
    ]
}

fn apply(m: [[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

fn matmul(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    // Each result column is `a` applied to the matching column of `b`.
    let c0 = apply(a, [b[0][0], b[1][0], b[2][0]]);
    let c1 = apply(a, [b[0][1], b[1][1], b[2][1]]);
    let c2 = apply(a, [b[0][2], b[1][2], b[2][2]]);
    [
        [c0[0], c1[0], c2[0]],
        [c0[1], c1[1], c2[1]],
        [c0[2], c1[2], c2[2]],
    ]
}

fn encode(value: f32, transfer: Transfer) -> u8 {
    let c = value.clamp(0.0, 1.0);
    let g = match transfer {
        Transfer::Srgb => {
            if c <= 0.003_130_8 {
                12.92 * c
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            }
        }
        Transfer::Rec709 => {
            if c < 0.018 {
                4.5 * c
            } else {
                1.099 * c.powf(0.45) - 0.099
            }
        }
    };
    (g * 255.0 + 0.5).clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::develop;
    use super::Transfer;
    use crate::params::DevelopParams;

    fn params(width: u32, height: u32) -> DevelopParams {
        DevelopParams {
            width,
            height,
            bits: 12,
            black: [0.0; 4],
            white: 4095.0,
            cfa: [0, 1, 1, 2],
            neutral: [1.0, 1.0, 1.0],
            exposure: 1.0,
            cam_to_xyz: None, // treat camera RGB as linear sRGB for a deterministic test
        }
    }

    #[test]
    fn white_frame_renders_white() {
        let p = params(4, 4);
        let img = develop(&[4095u16; 16], &p, Transfer::Srgb);
        assert_eq!(img.width, 4);
        assert!(img.data.iter().all(|&b| b == 255));
    }

    #[test]
    fn black_frame_renders_black() {
        let p = params(4, 4);
        let img = develop(&[0u16; 16], &p, Transfer::Srgb);
        // RGB all zero, alpha 255.
        for px in img.data.chunks_exact(4) {
            assert_eq!([px[0], px[1], px[2], px[3]], [0, 0, 0, 255]);
        }
    }

    #[test]
    fn mid_gray_is_gamma_encoded() {
        let p = params(2, 2);
        // Half of white level → linear 0.5 → sRGB ~0.735 → ~188.
        let img = develop(&[2047u16; 4], &p, Transfer::Srgb);
        let v = img.data[0];
        assert!((185..=191).contains(&v), "got {v}");
        assert_eq!(img.data[0], img.data[1]); // neutral: R == G == B
        assert_eq!(img.data[1], img.data[2]);
    }

    #[test]
    fn demosaic_recovers_channels() {
        // 2×2 RGGB: R=4095, G=2047/2047, B=0. Every pixel sees all three colours
        // in its 3×3 neighbourhood, so each resolves R high, B zero.
        let p = params(2, 2);
        let img = develop(&[4095, 2047, 2047, 0], &p, Transfer::Srgb);
        for px in img.data.chunks_exact(4) {
            assert_eq!(px[0], 255, "red channel"); // R sample = white
            assert_eq!(px[2], 0, "blue channel"); // B sample = black
        }
    }
}
