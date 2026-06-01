//! Raw develop: turn a parsed DNG's CFA mosaic into a display-ready RGB image.
//!
//! I/O-free and frame-at-a-time, so the browser (one frame, via WASM) and v2
//! proxy generation (every frame, via rayon in orchestration) share one
//! developer. Parameter extraction reads the container model; pixel work is pure
//! math. Bit-unpacking the raw samples is the caller's job (the codec already
//! owns it), keeping this crate dependent only on the container.
//!
//! The pipeline is a faithful approximation: black/white-level normalisation,
//! bilinear demosaic, as-shot white balance, `ColorMatrix1` (camera→XYZ) and
//! XYZ→sRGB, then the output transfer. It omits the `ForwardMatrix`, dual
//! illuminant, chromatic adaptation, and tone curve — enough for an accurate
//! preview/proxy, not a final grade.

// Image develop is intentionally lossy integer/float conversion throughout.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

mod develop;
mod params;

pub use develop::develop;
pub use develop::RgbaImage;
pub use develop::Transfer;
pub use params::raw_frame;
pub use params::DevelopParams;
pub use params::RawFrame;

pub mod prelude {
    pub use super::*;
}
