//! Structural TIFF tag numbers and field-type metadata.
//!
//! Structural tags describe physical layout (where pixel bytes live) rather than
//! semantics. The reader removes them from [`crate::Ifd::entries`] and the writer
//! re-emits them from the typed [`crate::Layout`] / [`crate::Image`] fields.

pub(crate) const IMAGE_LENGTH: u16 = 257;
pub(crate) const STRIP_OFFSETS: u16 = 273;
pub(crate) const ROWS_PER_STRIP: u16 = 278;
pub(crate) const STRIP_BYTE_COUNTS: u16 = 279;
pub(crate) const TILE_WIDTH: u16 = 322;
pub(crate) const TILE_LENGTH: u16 = 323;
pub(crate) const TILE_OFFSETS: u16 = 324;
pub(crate) const TILE_BYTE_COUNTS: u16 = 325;
pub(crate) const SUB_IFDS: u16 = 330;

pub(crate) const EXIF_IFD: u16 = 34665;
pub(crate) const GPS_IFD: u16 = 34853;
pub(crate) const INTEROP_IFD: u16 = 40965;

/// IFD-pointer tags whose value is an offset to a sub-directory, parsed into
/// `Ifd::pointer_ifds` and re-emitted by the writer. Followed in this order.
pub(crate) const POINTER_IFDS: [u16; 3] = [EXIF_IFD, GPS_IFD, INTEROP_IFD];

pub(crate) const TYPE_LONG: u16 = 4;
pub(crate) const TYPE_IFD: u16 = 13;

/// EXIF `MakerNote` (in the ExifIFD): a vendor blob that may carry file-absolute
/// internal offsets which break when the blob is relocated.
pub(crate) const MAKER_NOTE: u16 = 37500;
/// DNG `MakerNoteSafety` (in IFD0): 1 = self-contained/relocatable, else unsafe.
pub(crate) const MAKER_NOTE_SAFETY: u16 = 50741;

/// True for IFD-pointer tags the container follows and rewrites; like structural
/// tags, these never appear in `entries`.
pub(crate) fn is_pointer_ifd(tag: u16) -> bool {
    matches!(tag, EXIF_IFD | GPS_IFD | INTEROP_IFD)
}

/// True for tags the container owns and rewrites; these never appear in `entries`.
pub(crate) fn is_structural(tag: u16) -> bool {
    matches!(
        tag,
        STRIP_OFFSETS
            | ROWS_PER_STRIP
            | STRIP_BYTE_COUNTS
            | TILE_WIDTH
            | TILE_LENGTH
            | TILE_OFFSETS
            | TILE_BYTE_COUNTS
            | SUB_IFDS
    )
}

/// Size in bytes of one element of a TIFF field type, or `None` if unknown.
pub(crate) fn type_size(code: u16) -> Option<usize> {
    match code {
        1 | 2 | 6 | 7 => Some(1),
        3 | 8 => Some(2),
        4 | 9 | 11 => Some(4),
        5 | 10 | 12 => Some(8),
        _ => None,
    }
}
