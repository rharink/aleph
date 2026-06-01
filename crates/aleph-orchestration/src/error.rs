//! The orchestration error type, spanning codec, container, I/O, and policy.

use std::path::PathBuf;

/// Anything that can go wrong while compressing, verifying, or offloading.
#[derive(Debug, thiserror::Error)]
pub enum OrchestrationError {
    /// A container (TIFF/DNG) read or write failure.
    #[error(transparent)]
    Container(#[from] aleph_container::ContainerError),

    /// A codec (lossless JPEG) encode or decode failure.
    #[error(transparent)]
    Codec(#[from] aleph_codec::CodecError),

    /// A filesystem operation failed on a specific path.
    #[error("i/o error at {path}: {source}")]
    Io {
        /// The path being operated on.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// Pixel data uses a bit depth this build cannot (un)pack.
    #[error("unsupported bit depth {0}: only 8 and 16 bits per sample are supported")]
    UnsupportedBitDepth(u32),

    /// `BitsPerSample` entries are not all equal across components.
    #[error("inconsistent bits-per-sample across components")]
    InconsistentBitDepth,

    /// `PlanarConfiguration` is not chunky (1).
    #[error("unsupported planar configuration {0}: only chunky (1) is supported")]
    UnsupportedPlanarConfig(u32),

    /// A required tag is missing from an image directory.
    #[error("image directory is missing required tag {0}")]
    MissingTag(u16),

    /// A pixel dimension exceeds what lossless JPEG can address (u16).
    #[error("pixel dimension {0} exceeds the 65535 limit of lossless JPEG")]
    DimensionTooLarge(u32),

    /// Component count is outside the codec's supported range.
    #[error("unsupported component count {0}: must be 1..=4")]
    UnsupportedComponents(u32),

    /// A strip/tile segment is shorter than the pixel data its geometry implies.
    #[error("segment too short: need {need} bytes for the pixel data, have {have}")]
    SegmentTooShort {
        /// Bytes required for the declared pixels.
        need: usize,
        /// Bytes actually present in the segment.
        have: usize,
    },

    /// A sub-byte depth whose row is not byte-aligned, which this build cannot
    /// pack/unpack reversibly.
    #[error(
        "row of {samples_per_row} samples at {precision} bits is not byte-aligned; \
         sub-byte depths require whole-byte rows"
    )]
    UnsupportedRowAlignment {
        /// Samples per image row (width times components).
        samples_per_row: usize,
        /// Bits per sample.
        precision: u8,
    },

    /// The number of image segments is inconsistent with the declared geometry.
    #[error("segment geometry inconsistent: {0}")]
    SegmentGeometry(String),

    /// A compress -> decompress round-trip did not reproduce the input.
    #[error("round-trip verification failed: {0}")]
    RoundTrip(String),

    /// Metadata tags were not preserved across the round-trip.
    #[error("metadata not preserved: {0:?}")]
    TagViolations(Vec<aleph_metadata::Violation>),

    /// Offload was asked to copy to zero destinations.
    #[error("no destinations specified")]
    NoDestinations,

    /// An input path is neither a regular file nor a directory.
    #[error("path is neither a file nor a directory: {0}")]
    NotFileOrDir(PathBuf),

    /// A directory contained no DNG frames to process.
    #[error("no DNG frames found under {0}")]
    NoFrames(PathBuf),

    /// An input DNG had no uncompressed image data to compress.
    #[error("nothing to compress in {0}: all image data is already compressed")]
    NothingToCompress(PathBuf),

    /// An input DNG had no Aleph-compressed image data to decompress.
    #[error("nothing to decompress in {0}: no Aleph-compressed image data found")]
    NothingToDecompress(PathBuf),

    /// The input already uses Aleph's private marker tag with a non-Aleph value,
    /// so compressing would overwrite existing metadata.
    #[error(
        "input already uses Aleph's private marker tag {0:#06x} with a foreign value; \
         refusing to overwrite it"
    )]
    MarkerCollision(u16),
}

// Only the native (filesystem) modules construct path-carrying I/O errors.
#[cfg(feature = "native")]
impl OrchestrationError {
    /// Build an [`OrchestrationError::Io`] carrying `path` for a failed op.
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
