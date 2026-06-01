//! Browser (WASM) bindings: in-memory compress / decompress over DNG byte
//! slices, powering the in-browser "provably lossless" compression trial.
//!
//! Thin wasm-bindgen shim. All codec, container, and metadata logic lives in the
//! core crates, which keep `unsafe_code = "forbid"`; wasm-bindgen's generated
//! glue needs `unsafe`, so this edge crate opts out of that lint (see Cargo.toml).

use aleph_container::preview as extract_preview;
use aleph_container::read;
use aleph_container::write;
use aleph_develop::develop;
use aleph_develop::raw_frame;
use aleph_develop::Transfer;
use aleph_orchestration::compress_dng;
use aleph_orchestration::decompress_dng;
use aleph_orchestration::unpack;
use aleph_orchestration::verify_compressed;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsError;

/// Outcome of compressing one frame: the compressed DNG bytes, the input and
/// output sizes, and whether the in-memory round-trip verified bit-perfect.
#[wasm_bindgen]
pub struct CompressResult {
    bytes: Vec<u8>,
    original_len: usize,
    compressed_len: usize,
    verified: bool,
}

#[wasm_bindgen]
impl CompressResult {
    /// The compressed DNG, ready to download or feed back into `decompress`.
    ///
    /// Each read copies the buffer out of wasm memory, so callers should read
    /// this once and reuse the result rather than re-accessing it in a loop.
    #[wasm_bindgen(getter)]
    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn original_len(&self) -> usize {
        self.original_len
    }

    #[wasm_bindgen(getter)]
    pub fn compressed_len(&self) -> usize {
        self.compressed_len
    }

    /// True when `decompress(compress(x))` reproduces `x` exactly — checked in
    /// memory before returning, so the compression is provably lossless.
    #[wasm_bindgen(getter)]
    pub fn verified(&self) -> bool {
        self.verified
    }
}

/// Compress one uncompressed DNG frame losslessly, verifying the round-trip.
///
/// # Errors
/// Returns a JS error if `dng` is not a parseable DNG or holds pixel data this
/// build cannot encode (unsupported bit depth, planar config, dimensions).
#[wasm_bindgen]
pub fn compress(dng: &[u8]) -> Result<CompressResult, JsError> {
    let original = read(dng).map_err(to_js)?;
    let compressed = compress_dng(&original).map_err(to_js)?;
    let verified = verify_compressed(&original, &compressed).is_ok();
    let bytes = write(&compressed).map_err(to_js)?;
    Ok(CompressResult {
        original_len: dng.len(),
        compressed_len: bytes.len(),
        verified,
        bytes,
    })
}

/// Decompress an Aleph-compressed DNG back to its original bytes.
///
/// # Errors
/// Returns a JS error if `dng` is not a parseable DNG or a compressed segment
/// fails to decode.
#[wasm_bindgen]
pub fn decompress(dng: &[u8]) -> Result<Vec<u8>, JsError> {
    let parsed = read(dng).map_err(to_js)?;
    let restored = decompress_dng(&parsed).map_err(to_js)?;
    write(&restored).map_err(to_js)
}

/// An embedded JPEG preview lifted from a DNG: the JPEG bytes plus its pixel
/// dimensions. Browsers can render `bytes` directly in an `<img>`.
#[wasm_bindgen]
pub struct PreviewResult {
    bytes: Vec<u8>,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl PreviewResult {
    /// The JPEG byte stream. Copies out of wasm memory on each read — read once.
    #[wasm_bindgen(getter)]
    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }
}

/// Extract the largest embedded JPEG preview from a DNG, or `undefined` when the
/// frame is raw-only with no preview.
///
/// # Errors
/// Returns a JS error if `dng` is not a parseable DNG.
#[wasm_bindgen]
pub fn preview(dng: &[u8]) -> Result<Option<PreviewResult>, JsError> {
    let parsed = read(dng).map_err(to_js)?;
    Ok(extract_preview(&parsed).map(|p| PreviewResult {
        bytes: p.bytes,
        width: p.width,
        height: p.height,
    }))
}

/// A developed RGBA frame: 8-bit `width * height * 4` bytes plus dimensions.
#[wasm_bindgen]
pub struct RenderResult {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl RenderResult {
    /// Row-major RGBA8 pixels. Copies out of wasm memory on each read — read once.
    #[wasm_bindgen(getter)]
    pub fn rgba(&self) -> Vec<u8> {
        self.rgba.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }
}

/// Develop a DNG's raw CFA frame to a display RGBA image, or `undefined` when the
/// frame can't be developed here (compressed raw, non-CFA, unsupported depth).
///
/// # Errors
/// Returns a JS error if `dng` is not a parseable DNG, or its raw samples are
/// truncated.
#[wasm_bindgen]
pub fn render(dng: &[u8]) -> Result<Option<RenderResult>, JsError> {
    let parsed = read(dng).map_err(to_js)?;
    let Some(frame) = raw_frame(&parsed) else {
        return Ok(None);
    };
    let samples = unpack(
        &frame.packed,
        frame.params.bits,
        parsed.endian,
        frame.params.pixel_count(),
    )
    .map_err(to_js)?;
    let image = develop(&samples, &frame.params, Transfer::Srgb);
    Ok(Some(RenderResult {
        rgba: image.data,
        width: image.width,
        height: image.height,
    }))
}

fn to_js<E: std::fmt::Display>(error: E) -> JsError {
    JsError::new(&error.to_string())
}
