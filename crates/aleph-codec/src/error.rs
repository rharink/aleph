#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("invalid component count {0}: must be 1..=4")]
    InvalidComponents(u8),

    #[error("invalid precision {0}: must be 2..=16")]
    InvalidPrecision(u8),

    #[error("invalid dimensions {width}x{height}: width and height must be >= 1")]
    InvalidDimensions { width: u16, height: u16 },

    #[error("sample buffer length {actual} does not match expected {expected}")]
    SampleCountMismatch { expected: usize, actual: usize },

    #[error("sample value {value} does not fit in {precision}-bit precision")]
    SampleOutOfRange { value: u16, precision: u8 },

    #[error("unexpected end of bitstream")]
    UnexpectedEof,

    #[error("invalid marker: expected {expected:#06x}, found {found:#06x}")]
    InvalidMarker { expected: u16, found: u16 },

    #[error("unsupported marker {0:#06x}")]
    UnsupportedMarker(u16),

    #[error("unsupported bitstream feature: {0}")]
    Unsupported(&'static str),

    #[error("malformed bitstream: {0}")]
    Malformed(&'static str),
}
