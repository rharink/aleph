mod bitio;
mod decode;
mod encode;
mod error;
mod huffman;
mod marker;
mod sample;

pub use decode::decode;
pub use decode::Decoded;
pub use encode::encode;
pub use encode::Frame;
pub use error::CodecError;

pub mod prelude {
    pub use super::*;
}
