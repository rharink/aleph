//! Extracting an embedded JPEG preview from a parsed container.
//!
//! DNG stores display previews as JPEG-compressed image directories (Compression
//! 6/7); the raw frame is never JPEG, so it is not mistaken for one. Browsers
//! cannot decode raw DNG but can render this preview directly.

use crate::ifd::Dng;
use crate::ifd::Ifd;

const TAG_IMAGE_WIDTH: u16 = 256;
const TAG_IMAGE_LENGTH: u16 = 257;
const TAG_COMPRESSION: u16 = 259;

/// An embedded JPEG preview lifted out of a DNG/TIFF.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Preview {
    /// A complete JPEG byte stream, ready to decode or display.
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// The largest embedded JPEG preview in a parsed container, or `None` for a
/// raw-only frame that carries no preview.
#[must_use]
pub fn preview(dng: &Dng) -> Option<Preview> {
    let mut best: Option<Preview> = None;
    for ifd in &dng.ifds {
        collect(ifd, &mut best);
    }
    best
}

fn collect(ifd: &Ifd, best: &mut Option<Preview>) {
    if let Some(found) = jpeg_preview(ifd) {
        if area(&found) > best.as_ref().map_or(0, area) {
            *best = Some(found);
        }
    }
    for child in &ifd.sub_ifds {
        collect(child, best);
    }
}

fn jpeg_preview(ifd: &Ifd) -> Option<Preview> {
    if !matches!(scalar(ifd, TAG_COMPRESSION), Some(6 | 7)) {
        return None;
    }
    let image = ifd.image.as_ref()?;

    // A new-style JPEG preview is one complete stream (occasionally split across
    // strips). Concatenate, then require the JPEG SOI marker so a non-JPEG
    // segment can never be served as an image.
    let mut bytes = Vec::with_capacity(image.segments.iter().map(Vec::len).sum());
    for segment in &image.segments {
        bytes.extend_from_slice(segment);
    }
    if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return None;
    }

    Some(Preview {
        bytes,
        width: scalar(ifd, TAG_IMAGE_WIDTH).unwrap_or(0),
        height: scalar(ifd, TAG_IMAGE_LENGTH).unwrap_or(0),
    })
}

fn area(preview: &Preview) -> u64 {
    u64::from(preview.width) * u64::from(preview.height)
}

fn scalar(ifd: &Ifd, tag: u16) -> Option<u32> {
    ifd.get(tag)?.as_u32_vec()?.first().copied()
}

#[cfg(test)]
mod tests {
    use super::preview;
    use super::Preview;
    use crate::ifd::Dng;
    use crate::ifd::Ifd;
    use crate::ifd::Image;
    use crate::ifd::Layout;
    use crate::value::Endian;
    use crate::value::Value;

    fn jpeg(width: u32, height: u32, compression: u32, segment: Vec<u8>) -> Ifd {
        let mut ifd = Ifd::default();
        ifd.set(256, Value::Long(vec![width]));
        ifd.set(257, Value::Long(vec![height]));
        ifd.set(259, Value::Short(vec![u16::try_from(compression).unwrap()]));
        ifd.image = Some(Image {
            layout: Layout::Strips {
                rows_per_strip: height,
            },
            segments: vec![segment],
        });
        ifd
    }

    #[test]
    fn extracts_strip_jpeg_preview() {
        let bytes = vec![0xFF, 0xD8, 0xFF, 0xD9];
        let dng = Dng {
            endian: Endian::Little,
            ifds: vec![jpeg(320, 240, 7, bytes.clone())],
        };
        assert_eq!(
            preview(&dng),
            Some(Preview {
                bytes,
                width: 320,
                height: 240
            })
        );
    }

    #[test]
    fn ignores_raw_only_frames() {
        let mut raw = Ifd::default();
        raw.set(259, Value::Short(vec![1])); // uncompressed
        raw.image = Some(Image {
            layout: Layout::Strips { rows_per_strip: 4 },
            segments: vec![vec![0u8; 16]],
        });
        let dng = Dng {
            endian: Endian::Little,
            ifds: vec![raw],
        };
        assert_eq!(preview(&dng), None);
    }

    #[test]
    fn rejects_non_jpeg_segment() {
        // Compression says JPEG but the bytes lack the SOI marker.
        let dng = Dng {
            endian: Endian::Little,
            ifds: vec![jpeg(8, 8, 7, vec![0x00, 0x01, 0x02, 0x03])],
        };
        assert_eq!(preview(&dng), None);
    }

    #[test]
    fn picks_largest_preview_across_sub_ifds() {
        let small = jpeg(160, 120, 7, vec![0xFF, 0xD8, 0xAA, 0xD9]);
        let mut root = jpeg(320, 240, 7, vec![0xFF, 0xD8, 0xBB, 0xD9]);
        root.sub_ifds.push(small);
        let dng = Dng {
            endian: Endian::Little,
            ifds: vec![root],
        };
        let found = preview(&dng).unwrap();
        assert_eq!((found.width, found.height), (320, 240));
    }
}
