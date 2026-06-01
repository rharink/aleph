//! Frame enumeration, parallel job execution, checksum-verified offload, and the
//! lossless compress/decompress pipeline that ties codec, container, and metadata
//! together.

mod compress;
mod error;
mod sample;
mod snapshot;
mod verify;

// Native-only: filesystem + multi-frame parallelism + checksum offload.
#[cfg(feature = "native")]
mod frames;
#[cfg(feature = "native")]
mod offload;
#[cfg(feature = "native")]
mod pipeline;

pub use compress::compress_dng;
pub use compress::decompress_dng;
pub use error::OrchestrationError;
pub use verify::verify_compressed;
pub use verify::verify_roundtrip;

#[cfg(feature = "native")]
pub use frames::enumerate_dng;
#[cfg(feature = "native")]
pub use offload::offload;
#[cfg(feature = "native")]
pub use offload::DestinationOutcome;
#[cfg(feature = "native")]
pub use offload::DestinationStatus;
#[cfg(feature = "native")]
pub use offload::FileOutcome;
#[cfg(feature = "native")]
pub use offload::OffloadReport;
#[cfg(feature = "native")]
pub use pipeline::compress;
#[cfg(feature = "native")]
pub use pipeline::decompress;
#[cfg(feature = "native")]
pub use pipeline::CompressOptions;
#[cfg(feature = "native")]
pub use pipeline::FrameStat;
#[cfg(feature = "native")]
pub use pipeline::Summary;

pub mod prelude {
    pub use super::*;
}
