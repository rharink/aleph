//! The directory model: [`Ifd`] and its components, plus the top-level [`Dng`].

use crate::value::Endian;
use crate::value::Value;

/// One image file directory: semantic entries, child directories, pixel data.
#[derive(Clone, Debug, Default)]
pub struct Ifd {
    /// Non-structural entries, sorted ascending by tag.
    pub entries: Vec<Entry>,
    /// Child directories referenced by tag 330, in order.
    pub sub_ifds: Vec<Ifd>,
    /// Sub-directories referenced by IFD-pointer tags (Exif/GPS/Interop), so the
    /// metadata they hold (focal length, lens, exposure, GPS, ...) is preserved.
    pub pointer_ifds: Vec<PointerIfd>,
    /// Pixel data, present when this directory has strip or tile segments.
    pub image: Option<Image>,
    /// Original file offset of an unsafe `MakerNote` value in this directory, so
    /// the writer can repair its file-absolute internal offsets after relocation.
    /// Transient layout provenance — deliberately excluded from equality.
    pub maker_note_origin: Option<u32>,
}

impl PartialEq for Ifd {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
            && self.sub_ifds == other.sub_ifds
            && self.pointer_ifds == other.pointer_ifds
            && self.image == other.image
    }
}

/// A sub-directory referenced from `entries` by an IFD-pointer tag (e.g. 34665
/// `ExifIFD`). The reader follows the pointer; the writer re-emits the directory
/// and patches the tag to its new offset.
#[derive(Clone, PartialEq, Debug)]
pub struct PointerIfd {
    /// The pointer tag (e.g. 34665 `ExifIFD`, 34853 GPS, 40965 Interop).
    pub tag: u16,
    /// The referenced directory.
    pub ifd: Ifd,
}

/// A single tag/value pair.
#[derive(Clone, PartialEq, Debug)]
pub struct Entry {
    /// TIFF tag number.
    pub tag: u16,
    /// Decoded value.
    pub value: Value,
}

/// How an [`Image`]'s segments tile the pixel grid.
#[derive(Clone, PartialEq, Debug)]
pub enum Layout {
    /// Horizontal strips, each `rows_per_strip` rows tall (last may be shorter).
    Strips {
        /// Rows per strip (tag 278).
        rows_per_strip: u32,
    },
    /// Rectangular tiles.
    Tiles {
        /// Tile width in pixels (tag 322).
        tile_width: u32,
        /// Tile height in pixels (tag 323).
        tile_length: u32,
    },
}

/// Opaque pixel data: raw stored bytes per segment, in index order.
#[derive(Clone, PartialEq, Debug)]
pub struct Image {
    /// Segment geometry.
    pub layout: Layout,
    /// Raw bytes per strip/tile, exactly as stored on disk.
    pub segments: Vec<Vec<u8>>,
}

/// A parsed DNG/TIFF file: byte order plus the top-level IFD chain.
#[derive(Clone, PartialEq, Debug)]
pub struct Dng {
    /// Byte order of the stream.
    pub endian: Endian,
    /// Top-level directories, in `NextIFD` order.
    pub ifds: Vec<Ifd>,
}

impl Ifd {
    /// The value of `tag`, if present in `entries`.
    #[must_use]
    pub fn get(&self, tag: u16) -> Option<&Value> {
        self.entries
            .binary_search_by_key(&tag, |e| e.tag)
            .ok()
            .map(|i| &self.entries[i].value)
    }

    /// Insert or replace `tag`'s value, keeping `entries` sorted by tag.
    pub fn set(&mut self, tag: u16, value: Value) {
        match self.entries.binary_search_by_key(&tag, |e| e.tag) {
            Ok(i) => self.entries[i].value = value,
            Err(i) => self.entries.insert(i, Entry { tag, value }),
        }
    }

    /// Remove `tag` and return its value, if present.
    pub fn remove(&mut self, tag: u16) -> Option<Value> {
        match self.entries.binary_search_by_key(&tag, |e| e.tag) {
            Ok(i) => Some(self.entries.remove(i).value),
            Err(_) => None,
        }
    }
}
