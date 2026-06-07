//! DNG/TIFF container read and write. No codec logic: image segment bytes are
//! stored opaquely. The writer is authoritative on physical layout.

mod error;
mod ifd;
mod makernote;
mod preview;
mod read;
mod tags;
mod value;
mod write;

pub use error::ContainerError;
pub use ifd::Dng;
pub use ifd::Entry;
pub use ifd::Ifd;
pub use ifd::Image;
pub use ifd::Layout;
pub use ifd::PointerIfd;
pub use preview::preview;
pub use preview::Preview;
pub use read::read;
pub use read::read_file;
pub use value::Endian;
pub use value::Value;
pub use write::write;
pub use write::write_file;

pub mod prelude {
    pub use super::*;
}
