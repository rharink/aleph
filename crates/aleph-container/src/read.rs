//! Parsing a classic TIFF/DNG byte stream into a [`Dng`].

use std::path::Path;

use crate::error::ContainerError;
use crate::ifd::Dng;
use crate::ifd::Entry;
use crate::ifd::Ifd;
use crate::ifd::Image;
use crate::ifd::Layout;
use crate::ifd::PointerIfd;
use crate::tags;
use crate::value::get_u16;
use crate::value::get_u32;
use crate::value::get_u64;
use crate::value::Endian;
use crate::value::Value;

/// Parse a TIFF/DNG container from memory.
///
/// # Errors
/// Returns [`ContainerError`] on a malformed header, an unknown field type, a
/// truncated buffer, or an inconsistent structural layout.
pub fn read(bytes: &[u8]) -> Result<Dng, ContainerError> {
    if bytes.len() < 8 {
        return Err(ContainerError::Truncated { offset: 0, need: 8 });
    }
    let endian = match (bytes[0], bytes[1]) {
        (0x49, 0x49) => Endian::Little,
        (0x4D, 0x4D) => Endian::Big,
        _ => return Err(ContainerError::BadByteOrder),
    };
    let reader = Reader { bytes, endian };

    let magic = reader.u16(2)?;
    if magic != 42 {
        return Err(ContainerError::BadMagic(magic));
    }

    let mut offset = reader.u32(4)? as usize;
    let mut ifds = Vec::new();
    let mut seen = Vec::new();
    while offset != 0 {
        if seen.contains(&offset) {
            break;
        }
        seen.push(offset);
        // Top level: MakerNoteSafety is absent here and resolved per-IFD; start
        // from the spec's "unsafe" default so an unflagged MakerNote is repaired.
        let (ifd, next) = reader.parse_ifd(offset, false)?;
        ifds.push(ifd);
        offset = next;
    }

    Ok(Dng { endian, ifds })
}

/// Read and parse a TIFF/DNG file via `std::fs::read` (never mmap).
///
/// # Errors
/// Returns [`ContainerError`] on an I/O failure or any [`read`] error.
pub fn read_file(path: &Path) -> Result<Dng, ContainerError> {
    let bytes = std::fs::read(path)?;
    read(&bytes)
}

struct Reader<'a> {
    bytes: &'a [u8],
    endian: Endian,
}

impl Reader<'_> {
    fn parse_ifd(
        &self,
        offset: usize,
        maker_note_safe: bool,
    ) -> Result<(Ifd, usize), ContainerError> {
        let count = usize::from(self.u16(offset)?);
        let mut raw: Vec<Entry> = Vec::with_capacity(count);
        let mut maker_note_offset: Option<u32> = None;
        for i in 0..count {
            let pos = offset + 2 + i * 12;
            let tag = self.u16(pos)?;
            let type_code = self.u16(pos + 2)?;
            let elems = self.u32(pos + 4)? as usize;
            let size =
                tags::type_size(type_code).ok_or(ContainerError::UnknownFieldType(type_code))?;
            let total = size
                .checked_mul(elems)
                .ok_or(ContainerError::ValueTooLarge)?;
            let data = if total <= 4 {
                self.slice(pos + 8, total)?
            } else {
                let value_offset = self.u32(pos + 8)?;
                if tag == tags::MAKER_NOTE {
                    maker_note_offset = Some(value_offset);
                }
                self.slice(value_offset as usize, total)?
            };
            raw.push(Entry {
                tag,
                value: parse_value(self.endian, type_code, elems, data),
            });
        }
        let next = self.u32(offset + 2 + count * 12)? as usize;

        // MakerNoteSafety (IFD0) describes the file's MakerNote; children inherit
        // it. Absent means unsafe per the DNG spec.
        let children_safe = match scalar(&raw, tags::MAKER_NOTE_SAFETY) {
            Some(1) => true,
            Some(_) => false,
            None => maker_note_safe,
        };
        let sub_ifds = self.parse_sub_ifds(&raw, children_safe)?;
        let pointer_ifds = self.parse_pointer_ifds(&raw, children_safe)?;
        let image = self.parse_image(&raw)?;

        let maker_note_origin = if maker_note_safe {
            None
        } else {
            maker_note_offset
        };

        let mut entries: Vec<Entry> = raw
            .into_iter()
            .filter(|e| !tags::is_structural(e.tag) && !tags::is_pointer_ifd(e.tag))
            .collect();
        entries.sort_by_key(|e| e.tag);

        Ok((
            Ifd {
                entries,
                sub_ifds,
                pointer_ifds,
                image,
                maker_note_origin,
            },
            next,
        ))
    }

    fn parse_sub_ifds(
        &self,
        raw: &[Entry],
        maker_note_safe: bool,
    ) -> Result<Vec<Ifd>, ContainerError> {
        let Some(offsets) = find(raw, tags::SUB_IFDS).and_then(Value::as_u32_vec) else {
            return Ok(Vec::new());
        };
        let mut children = Vec::with_capacity(offsets.len());
        for child_offset in offsets {
            let (child, _) = self.parse_ifd(child_offset as usize, maker_note_safe)?;
            children.push(child);
        }
        Ok(children)
    }

    fn parse_pointer_ifds(
        &self,
        raw: &[Entry],
        maker_note_safe: bool,
    ) -> Result<Vec<PointerIfd>, ContainerError> {
        let mut out = Vec::new();
        for &tag in &tags::POINTER_IFDS {
            let Some(offset) = find(raw, tag)
                .and_then(Value::as_u32_vec)
                .and_then(|v| v.into_iter().next())
            else {
                continue;
            };
            let (ifd, _next) = self.parse_ifd(offset as usize, maker_note_safe)?;
            out.push(PointerIfd { tag, ifd });
        }
        Ok(out)
    }

    fn parse_image(&self, raw: &[Entry]) -> Result<Option<Image>, ContainerError> {
        let tile_offsets = find(raw, tags::TILE_OFFSETS).and_then(Value::as_u32_vec);
        let tile_counts = find(raw, tags::TILE_BYTE_COUNTS).and_then(Value::as_u32_vec);
        if let (Some(offsets), Some(counts)) = (tile_offsets, tile_counts) {
            let tile_width =
                scalar(raw, tags::TILE_WIDTH).ok_or(ContainerError::Inconsistent("tile width"))?;
            let tile_length = scalar(raw, tags::TILE_LENGTH)
                .ok_or(ContainerError::Inconsistent("tile length"))?;
            return Ok(Some(Image {
                layout: Layout::Tiles {
                    tile_width,
                    tile_length,
                },
                segments: self.read_segments(&offsets, &counts)?,
            }));
        }

        let strip_offsets = find(raw, tags::STRIP_OFFSETS).and_then(Value::as_u32_vec);
        let strip_counts = find(raw, tags::STRIP_BYTE_COUNTS).and_then(Value::as_u32_vec);
        if let (Some(offsets), Some(counts)) = (strip_offsets, strip_counts) {
            let rows_per_strip = scalar(raw, tags::ROWS_PER_STRIP)
                .or_else(|| scalar(raw, tags::IMAGE_LENGTH))
                .unwrap_or(0);
            return Ok(Some(Image {
                layout: Layout::Strips { rows_per_strip },
                segments: self.read_segments(&offsets, &counts)?,
            }));
        }

        Ok(None)
    }

    fn read_segments(
        &self,
        offsets: &[u32],
        counts: &[u32],
    ) -> Result<Vec<Vec<u8>>, ContainerError> {
        if offsets.len() != counts.len() {
            return Err(ContainerError::Inconsistent(
                "segment offset/count length mismatch",
            ));
        }
        let mut segments = Vec::with_capacity(offsets.len());
        for (&offset, &len) in offsets.iter().zip(counts) {
            segments.push(self.slice(offset as usize, len as usize)?.to_vec());
        }
        Ok(segments)
    }

    fn u16(&self, pos: usize) -> Result<u16, ContainerError> {
        Ok(get_u16(self.endian, self.slice(pos, 2)?))
    }

    fn u32(&self, pos: usize) -> Result<u32, ContainerError> {
        Ok(get_u32(self.endian, self.slice(pos, 4)?))
    }

    fn slice(&self, pos: usize, len: usize) -> Result<&[u8], ContainerError> {
        let end = pos.checked_add(len).ok_or(ContainerError::Truncated {
            offset: pos,
            need: len,
        })?;
        self.bytes.get(pos..end).ok_or(ContainerError::Truncated {
            offset: pos,
            need: len,
        })
    }
}

fn parse_value(endian: Endian, type_code: u16, elems: usize, data: &[u8]) -> Value {
    match type_code {
        1 => Value::Byte(data.to_vec()),
        2 => Value::Ascii(data.to_vec()),
        7 => Value::Undefined(data.to_vec()),
        6 => Value::SByte(data.iter().map(|&x| i8::from_ne_bytes([x])).collect()),
        3 => Value::Short(decode(elems, data, 2, |b| get_u16(endian, b))),
        8 => Value::SShort(decode(elems, data, 2, |b| {
            i16::from_ne_bytes(get_u16(endian, b).to_ne_bytes())
        })),
        4 => Value::Long(decode(elems, data, 4, |b| get_u32(endian, b))),
        9 => Value::SLong(decode(elems, data, 4, |b| {
            i32::from_ne_bytes(get_u32(endian, b).to_ne_bytes())
        })),
        11 => Value::Float(decode(elems, data, 4, |b| {
            f32::from_bits(get_u32(endian, b))
        })),
        12 => Value::Double(decode(elems, data, 8, |b| {
            f64::from_bits(get_u64(endian, b))
        })),
        5 => Value::Rational(decode(elems, data, 8, |b| {
            (get_u32(endian, b), get_u32(endian, &b[4..]))
        })),
        // Only type 10 (SRational) remains; type validity checked by the caller.
        _ => Value::SRational(decode(elems, data, 8, |b| {
            (
                i32::from_ne_bytes(get_u32(endian, b).to_ne_bytes()),
                i32::from_ne_bytes(get_u32(endian, &b[4..]).to_ne_bytes()),
            )
        })),
    }
}

fn decode<T>(elems: usize, data: &[u8], stride: usize, f: impl Fn(&[u8]) -> T) -> Vec<T> {
    let mut out = Vec::with_capacity(elems);
    for i in 0..elems {
        out.push(f(&data[i * stride..]));
    }
    out
}

fn find(entries: &[Entry], tag: u16) -> Option<&Value> {
    entries.iter().find(|e| e.tag == tag).map(|e| &e.value)
}

fn scalar(entries: &[Entry], tag: u16) -> Option<u32> {
    find(entries, tag)
        .and_then(Value::as_u32_vec)
        .and_then(|v| v.first().copied())
}

#[cfg(test)]
mod tests {
    use super::read;
    use crate::error::ContainerError;
    use crate::value::Endian;
    use crate::value::Value;
    use proptest::collection::vec;
    use proptest::prelude::any;
    use proptest::proptest;

    #[test]
    fn parses_hand_crafted_minimal_little_endian_tiff() {
        #[rustfmt::skip]
        let bytes: Vec<u8> = vec![
            0x49, 0x49,             // II
            0x2A, 0x00,             // magic 42
            0x08, 0x00, 0x00, 0x00, // IFD0 at offset 8
            0x02, 0x00,             // 2 entries
            // entry: ImageWidth (256) Short count 1 = 8
            0x00, 0x01, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
            // entry: Make (271) Ascii count 3 = "Hi\0"
            0x0F, 0x01, 0x02, 0x00, 0x03, 0x00, 0x00, 0x00, 0x48, 0x69, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, // next IFD = 0
        ];

        let dng = read(&bytes).expect("parse");
        assert_eq!(dng.endian, Endian::Little);
        assert_eq!(dng.ifds.len(), 1);
        let ifd = &dng.ifds[0];
        assert!(ifd.image.is_none());
        assert!(ifd.sub_ifds.is_empty());
        assert_eq!(ifd.get(256), Some(&Value::Short(vec![8])));
        assert_eq!(ifd.get(271), Some(&Value::Ascii(b"Hi\0".to_vec())));
        // Entries are sorted ascending by tag.
        assert_eq!(ifd.entries[0].tag, 256);
        assert_eq!(ifd.entries[1].tag, 271);
    }

    #[test]
    fn rejects_truncated_header_without_panic() {
        assert!(matches!(
            read(&[0x49, 0x49, 0x2A]),
            Err(ContainerError::Truncated { .. })
        ));
    }

    #[test]
    fn rejects_bad_byte_order() {
        let bytes = [0x00, 0x00, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00];
        assert!(matches!(read(&bytes), Err(ContainerError::BadByteOrder)));
    }

    #[test]
    fn rejects_bad_magic() {
        let bytes = [0x49, 0x49, 0x2B, 0x00, 0x08, 0x00, 0x00, 0x00];
        assert!(matches!(read(&bytes), Err(ContainerError::BadMagic(43))));
    }

    #[test]
    fn rejects_unknown_field_type_without_panic() {
        #[rustfmt::skip]
        let bytes: Vec<u8> = vec![
            0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00,
            0x01, 0x00,             // 1 entry
            0x00, 0x01, 0x63, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // type 99
            0x00, 0x00, 0x00, 0x00,
        ];
        assert!(matches!(
            read(&bytes),
            Err(ContainerError::UnknownFieldType(99))
        ));
    }

    #[test]
    fn rejects_out_of_range_ifd_offset_without_panic() {
        let bytes = [0x49, 0x49, 0x2A, 0x00, 0xFF, 0x00, 0x00, 0x00];
        assert!(matches!(
            read(&bytes),
            Err(ContainerError::Truncated { .. })
        ));
    }

    proptest! {
        // The reader must reject malformed input with an error, never panic.
        #[test]
        fn read_never_panics_on_arbitrary_bytes(data in vec(any::<u8>(), 0..4096)) {
            let _ = read(&data);
        }

        // A valid little-endian header routes arbitrary bytes straight into the
        // IFD parser, fuzzing its entry-count, type, and offset bounds handling.
        #[test]
        fn read_never_panics_behind_valid_header(tail in vec(any::<u8>(), 0..4096)) {
            let mut bytes = vec![0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00];
            bytes.extend_from_slice(&tail);
            let _ = read(&bytes);
        }
    }
}
