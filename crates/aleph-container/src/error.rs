//! Error type for container read/write.

/// Failure modes for parsing and emitting TIFF/DNG containers.
#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    /// A read accessed bytes past the end of the buffer.
    #[error("truncated data: need {need} byte(s) at offset {offset}")]
    Truncated {
        /// Byte offset at which the read began.
        offset: usize,
        /// Number of bytes the read required.
        need: usize,
    },

    /// The byte-order marker was neither `II` nor `MM`.
    #[error("invalid byte-order marker")]
    BadByteOrder,

    /// The TIFF magic number was not 42.
    #[error("bad TIFF magic: expected 42, found {0}")]
    BadMagic(u16),

    /// An IFD entry used a field type code outside 1..=12.
    #[error("unknown TIFF field type {0}")]
    UnknownFieldType(u16),

    /// The in-memory model could not be encoded consistently.
    #[error("inconsistent DNG: {0}")]
    Inconsistent(&'static str),

    /// A count, offset, or length exceeded what classic TIFF can encode.
    #[error("value too large to encode in classic TIFF")]
    ValueTooLarge,

    /// Filesystem error from `read_file` / `write_file`.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
}
