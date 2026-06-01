//! Serializing a [`Dng`] back to a self-consistent classic TIFF byte stream.
//!
//! The writer is authoritative on physical layout: it recomputes every offset,
//! re-emits the structural tags from the typed [`Layout`]/[`Image`]/sub-directory
//! fields, and word-aligns all out-of-line data.

use std::borrow::Cow;
use std::path::Path;

use crate::error::ContainerError;
use crate::ifd::Dng;
use crate::ifd::Ifd;
use crate::ifd::Layout;
use crate::makernote;
use crate::tags;
use crate::value::put_u32;
use crate::value::Endian;

/// Serialize a [`Dng`] to a TIFF/DNG byte stream.
///
/// # Errors
/// Returns [`ContainerError::ValueTooLarge`] when a count, length, or offset
/// exceeds the classic-TIFF 32-bit range.
pub fn write(dng: &Dng) -> Result<Vec<u8>, ContainerError> {
    let mut nodes: Vec<Node> = Vec::new();
    let mut top = Vec::with_capacity(dng.ifds.len());
    for ifd in &dng.ifds {
        top.push(build_node(ifd, dng.endian, &mut nodes)?);
    }

    let total = assign_offsets(&mut nodes)?;
    link_chain(&mut nodes, &top);

    let mut buf = vec![0u8; total];
    write_header(&mut buf, dng.endian, &nodes, &top)?;
    for index in 0..nodes.len() {
        write_node(&mut buf, dng.endian, &nodes, index)?;
    }
    Ok(buf)
}

/// Serialize a [`Dng`] and write it to `path`.
///
/// # Errors
/// Returns [`ContainerError`] on an I/O failure or any [`write`] error.
pub fn write_file(path: &Path, dng: &Dng) -> Result<(), ContainerError> {
    let bytes = write(dng)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

/// A flattened IFD ready for offset assignment and emission.
struct Node<'a> {
    entries: Vec<EmitEntry>,
    segments: &'a [Vec<u8>],
    children: Vec<usize>,
    pointer_children: Vec<(u16, usize)>,
    maker_note_origin: Option<u32>,
    dir_offset: usize,
    next_offset: usize,
    seg_offsets: Vec<usize>,
}

struct EmitEntry {
    tag: u16,
    type_code: u16,
    count: u32,
    bytes: Vec<u8>,
    blob_offset: usize,
    kind: EntryKind,
}

enum EntryKind {
    Plain,
    SegmentOffsets,
    ChildOffsets,
    PointerIfd,
}

fn build_node<'a>(
    ifd: &'a Ifd,
    endian: Endian,
    nodes: &mut Vec<Node<'a>>,
) -> Result<usize, ContainerError> {
    let mut entries: Vec<EmitEntry> = Vec::with_capacity(ifd.entries.len() + 4);
    for entry in &ifd.entries {
        entries.push(EmitEntry {
            tag: entry.tag,
            type_code: entry.value.type_code(),
            count: entry.value.count(),
            bytes: entry.value.encode(endian),
            blob_offset: 0,
            kind: EntryKind::Plain,
        });
    }

    let segments: &[Vec<u8>] = if let Some(image) = &ifd.image {
        let counts = segment_counts(&image.segments)?;
        let n = image.segments.len();
        match image.layout {
            Layout::Strips { rows_per_strip } => {
                entries.push(offset_entry(
                    tags::STRIP_OFFSETS,
                    n,
                    EntryKind::SegmentOffsets,
                ));
                entries.push(long_entry(tags::ROWS_PER_STRIP, &[rows_per_strip], endian));
                entries.push(long_entry(tags::STRIP_BYTE_COUNTS, &counts, endian));
            }
            Layout::Tiles {
                tile_width,
                tile_length,
            } => {
                entries.push(long_entry(tags::TILE_WIDTH, &[tile_width], endian));
                entries.push(long_entry(tags::TILE_LENGTH, &[tile_length], endian));
                entries.push(offset_entry(
                    tags::TILE_OFFSETS,
                    n,
                    EntryKind::SegmentOffsets,
                ));
                entries.push(long_entry(tags::TILE_BYTE_COUNTS, &counts, endian));
            }
        }
        &image.segments
    } else {
        &[]
    };

    if !ifd.sub_ifds.is_empty() {
        entries.push(offset_entry(
            tags::SUB_IFDS,
            ifd.sub_ifds.len(),
            EntryKind::ChildOffsets,
        ));
    }

    for pointer in &ifd.pointer_ifds {
        entries.push(offset_entry(pointer.tag, 1, EntryKind::PointerIfd));
    }

    entries.sort_by_key(|e| e.tag);

    let index = nodes.len();
    nodes.push(Node {
        entries,
        segments,
        children: Vec::new(),
        pointer_children: Vec::new(),
        maker_note_origin: ifd.maker_note_origin,
        dir_offset: 0,
        next_offset: 0,
        seg_offsets: Vec::new(),
    });

    let mut children = Vec::with_capacity(ifd.sub_ifds.len());
    for child in &ifd.sub_ifds {
        children.push(build_node(child, endian, nodes)?);
    }
    nodes[index].children = children;

    let mut pointer_children = Vec::with_capacity(ifd.pointer_ifds.len());
    for pointer in &ifd.pointer_ifds {
        let child = build_node(&pointer.ifd, endian, nodes)?;
        pointer_children.push((pointer.tag, child));
    }
    nodes[index].pointer_children = pointer_children;
    Ok(index)
}

fn assign_offsets(nodes: &mut [Node]) -> Result<usize, ContainerError> {
    let mut cursor = 8;
    for node in &mut *nodes {
        node.dir_offset = cursor;
        cursor += 2 + 12 * node.entries.len() + 4;
        for entry in &mut node.entries {
            if entry.bytes.len() > 4 {
                cursor = align_even(cursor);
                entry.blob_offset = cursor;
                cursor += entry.bytes.len();
            }
        }
        for segment in node.segments {
            cursor = align_even(cursor);
            node.seg_offsets.push(cursor);
            cursor += segment.len();
        }
    }
    if u32::try_from(cursor).is_err() {
        return Err(ContainerError::ValueTooLarge);
    }
    Ok(cursor)
}

fn link_chain(nodes: &mut [Node], top: &[usize]) {
    for window in top.windows(2) {
        nodes[window[0]].next_offset = nodes[window[1]].dir_offset;
    }
}

fn write_header(
    buf: &mut [u8],
    endian: Endian,
    nodes: &[Node],
    top: &[usize],
) -> Result<(), ContainerError> {
    let (b0, b1) = match endian {
        Endian::Little => (0x49, 0x49),
        Endian::Big => (0x4D, 0x4D),
    };
    buf[0] = b0;
    buf[1] = b1;
    put_u16_at(buf, 2, endian, 42);
    let ifd0 = top.first().map_or(0, |&n| nodes[n].dir_offset);
    put_u32_at(buf, 4, endian, offset32(ifd0)?);
    Ok(())
}

fn write_node(
    buf: &mut [u8],
    endian: Endian,
    nodes: &[Node],
    index: usize,
) -> Result<(), ContainerError> {
    let node = &nodes[index];
    let dir = node.dir_offset;
    let n = node.entries.len();

    put_u16_at(
        buf,
        dir,
        endian,
        u16::try_from(n).map_err(|_| ContainerError::ValueTooLarge)?,
    );
    for (i, entry) in node.entries.iter().enumerate() {
        let pos = dir + 2 + i * 12;
        put_u16_at(buf, pos, endian, entry.tag);
        put_u16_at(buf, pos + 2, endian, entry.type_code);
        put_u32_at(buf, pos + 4, endian, entry.count);

        match entry.kind {
            EntryKind::Plain => {
                let bytes = relocated_makernote(node, entry, endian)?;
                write_field(buf, pos + 8, endian, bytes.as_ref(), entry.blob_offset)?;
            }
            EntryKind::SegmentOffsets => {
                let bytes = encode_offsets(endian, &node.seg_offsets)?;
                write_field(buf, pos + 8, endian, &bytes, entry.blob_offset)?;
            }
            EntryKind::ChildOffsets => {
                let child_offsets: Vec<usize> =
                    node.children.iter().map(|&c| nodes[c].dir_offset).collect();
                let bytes = encode_offsets(endian, &child_offsets)?;
                write_field(buf, pos + 8, endian, &bytes, entry.blob_offset)?;
            }
            EntryKind::PointerIfd => {
                let offset = node
                    .pointer_children
                    .iter()
                    .find(|(t, _)| *t == entry.tag)
                    .map(|&(_, c)| nodes[c].dir_offset)
                    .ok_or(ContainerError::Inconsistent("pointer ifd child missing"))?;
                let bytes = encode_offsets(endian, &[offset])?;
                write_field(buf, pos + 8, endian, &bytes, entry.blob_offset)?;
            }
        }
    }
    put_u32_at(buf, dir + 2 + 12 * n, endian, offset32(node.next_offset)?);

    for (segment, &offset) in node.segments.iter().zip(&node.seg_offsets) {
        buf[offset..offset + segment.len()].copy_from_slice(segment);
    }
    Ok(())
}

/// An unsafe `MakerNote`'s file-absolute internal offsets must be shifted when the
/// blob is relocated. Returns the original bytes unchanged unless this is a
/// recognized, repairable `MakerNote`.
fn relocated_makernote<'a>(
    node: &Node,
    entry: &'a EmitEntry,
    endian: Endian,
) -> Result<Cow<'a, [u8]>, ContainerError> {
    if entry.tag == tags::MAKER_NOTE && entry.blob_offset != 0 {
        if let Some(origin) = node.maker_note_origin {
            let new_base = offset32(entry.blob_offset)?;
            if let Some(fixed) = makernote::relocate(&entry.bytes, endian, origin, new_base) {
                return Ok(Cow::Owned(fixed));
            }
        }
    }
    Ok(Cow::Borrowed(&entry.bytes))
}

fn write_field(
    buf: &mut [u8],
    field: usize,
    endian: Endian,
    bytes: &[u8],
    blob_offset: usize,
) -> Result<(), ContainerError> {
    if bytes.len() <= 4 {
        buf[field..field + bytes.len()].copy_from_slice(bytes);
    } else {
        put_u32_at(buf, field, endian, offset32(blob_offset)?);
        buf[blob_offset..blob_offset + bytes.len()].copy_from_slice(bytes);
    }
    Ok(())
}

fn long_entry(tag: u16, values: &[u32], endian: Endian) -> EmitEntry {
    EmitEntry {
        tag,
        type_code: tags::TYPE_LONG,
        count: u32::try_from(values.len()).unwrap_or(u32::MAX),
        bytes: encode_longs(endian, values),
        blob_offset: 0,
        kind: EntryKind::Plain,
    }
}

fn offset_entry(tag: u16, count: usize, kind: EntryKind) -> EmitEntry {
    EmitEntry {
        tag,
        type_code: tags::TYPE_LONG,
        count: u32::try_from(count).unwrap_or(u32::MAX),
        bytes: vec![0u8; 4 * count],
        blob_offset: 0,
        kind,
    }
}

fn segment_counts(segments: &[Vec<u8>]) -> Result<Vec<u32>, ContainerError> {
    segments
        .iter()
        .map(|s| u32::try_from(s.len()).map_err(|_| ContainerError::ValueTooLarge))
        .collect()
}

fn encode_longs(endian: Endian, values: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(values.len() * 4);
    for &v in values {
        put_u32(&mut out, endian, v);
    }
    out
}

fn encode_offsets(endian: Endian, offsets: &[usize]) -> Result<Vec<u8>, ContainerError> {
    let mut out = Vec::with_capacity(offsets.len() * 4);
    for &offset in offsets {
        put_u32(&mut out, endian, offset32(offset)?);
    }
    Ok(out)
}

fn offset32(value: usize) -> Result<u32, ContainerError> {
    u32::try_from(value).map_err(|_| ContainerError::ValueTooLarge)
}

fn align_even(value: usize) -> usize {
    value + (value & 1)
}

fn put_u16_at(buf: &mut [u8], pos: usize, endian: Endian, value: u16) {
    let bytes = match endian {
        Endian::Little => value.to_le_bytes(),
        Endian::Big => value.to_be_bytes(),
    };
    buf[pos..pos + 2].copy_from_slice(&bytes);
}

fn put_u32_at(buf: &mut [u8], pos: usize, endian: Endian, value: u32) {
    let bytes = match endian {
        Endian::Little => value.to_le_bytes(),
        Endian::Big => value.to_be_bytes(),
    };
    buf[pos..pos + 4].copy_from_slice(&bytes);
}

#[cfg(test)]
mod tests {
    use super::write;
    use crate::ifd::Dng;
    use crate::ifd::Ifd;
    use crate::ifd::Image;
    use crate::ifd::Layout;
    use crate::ifd::PointerIfd;
    use crate::read::read;
    use crate::value::Endian;
    use crate::value::Value;
    use proptest::collection::vec as prop_vec;
    use proptest::option::of as prop_option;
    use proptest::prelude::any;
    use proptest::prelude::Just;
    use proptest::prelude::Strategy;
    use proptest::prop_oneof;
    use proptest::proptest;

    fn rich_dng(endian: Endian) -> Dng {
        let mut child = Ifd::default();
        child.set(256, Value::Short(vec![16]));
        child.set(0x9000, Value::Long(vec![1, 2, 3]));
        child.image = Some(Image {
            layout: Layout::Tiles {
                tile_width: 16,
                tile_length: 16,
            },
            segments: vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]],
        });

        let mut ifd0 = Ifd::default();
        ifd0.set(256, Value::Short(vec![4]));
        ifd0.set(258, Value::Short(vec![16, 16, 16]));
        ifd0.set(259, Value::Short(vec![1]));
        ifd0.set(271, Value::Ascii(b"Aleph\0".to_vec()));
        ifd0.set(0x9001, Value::Long(vec![10, 20, 30, 40]));
        ifd0.set(0x9002, Value::Rational(vec![(1, 2), (3, 4)]));

        let mut interop = Ifd::default();
        interop.set(1, Value::Ascii(b"R98\0".to_vec()));
        let mut exif = Ifd::default();
        exif.set(0x829a, Value::Rational(vec![(1, 100)])); // ExposureTime
        exif.set(0x920a, Value::Rational(vec![(280, 10)])); // FocalLength
        exif.set(0x927c, Value::Undefined(vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01])); // MakerNote
        exif.set(0xA434, Value::Ascii(b"Test Lens\0".to_vec())); // LensModel
        exif.pointer_ifds = vec![PointerIfd {
            tag: 40965,
            ifd: interop,
        }];
        let mut gps = Ifd::default();
        gps.set(0, Value::Byte(vec![2, 3, 0, 0]));
        ifd0.pointer_ifds = vec![
            PointerIfd {
                tag: 34665,
                ifd: exif,
            },
            PointerIfd {
                tag: 34853,
                ifd: gps,
            },
        ];
        ifd0.sub_ifds = vec![child];
        ifd0.image = Some(Image {
            layout: Layout::Strips { rows_per_strip: 2 },
            segments: vec![vec![0xAA, 0xBB, 0xCC], vec![0xDD]],
        });

        let mut ifd1 = Ifd::default();
        ifd1.set(256, Value::Short(vec![1]));
        ifd1.set(305, Value::Ascii(b"sw\0".to_vec()));

        Dng {
            endian,
            ifds: vec![ifd0, ifd1],
        }
    }

    #[test]
    fn round_trips_rich_little_endian() {
        let dng = rich_dng(Endian::Little);
        assert_eq!(read(&write(&dng).unwrap()).unwrap(), dng);
    }

    #[test]
    fn round_trips_rich_big_endian() {
        let dng = rich_dng(Endian::Big);
        assert_eq!(read(&write(&dng).unwrap()).unwrap(), dng);
    }

    #[test]
    fn write_is_stable_under_reparse() {
        for endian in [Endian::Little, Endian::Big] {
            let dng = rich_dng(endian);
            let first = write(&dng).unwrap();
            let second = write(&read(&first).unwrap()).unwrap();
            assert_eq!(first, second);
        }
    }

    fn arb_value() -> impl Strategy<Value = Value> {
        prop_oneof![
            prop_vec(any::<u8>(), 0..6).prop_map(Value::Byte),
            prop_vec(any::<u8>(), 0..6).prop_map(Value::Ascii),
            prop_vec(any::<u8>(), 0..6).prop_map(Value::Undefined),
            prop_vec(any::<i8>(), 0..6).prop_map(Value::SByte),
            prop_vec(any::<u16>(), 0..4).prop_map(Value::Short),
            prop_vec(any::<i16>(), 0..4).prop_map(Value::SShort),
            prop_vec(any::<u32>(), 0..4).prop_map(Value::Long),
            prop_vec(any::<i32>(), 0..4).prop_map(Value::SLong),
            prop_vec((any::<u32>(), any::<u32>()), 0..3).prop_map(Value::Rational),
            prop_vec((any::<i32>(), any::<i32>()), 0..3).prop_map(Value::SRational),
            prop_vec(-1e6f32..1e6f32, 0..4).prop_map(Value::Float),
            prop_vec(-1e6f64..1e6f64, 0..4).prop_map(Value::Double),
        ]
    }

    fn arb_image() -> impl Strategy<Value = Image> {
        let segments = prop_vec(prop_vec(any::<u8>(), 0..5), 1..4);
        let layout = prop_oneof![
            (0u32..64).prop_map(|r| Layout::Strips { rows_per_strip: r }),
            (1u32..64, 1u32..64).prop_map(|(w, h)| Layout::Tiles {
                tile_width: w,
                tile_length: h,
            }),
        ];
        (layout, segments).prop_map(|(layout, segments)| Image { layout, segments })
    }

    fn arb_ifd(allow_children: bool) -> impl Strategy<Value = Ifd> {
        let entries = prop_vec((0u16..1000, arb_value()), 0..6);
        let child = if allow_children {
            prop_option(arb_ifd(false).prop_map(Box::new)).boxed()
        } else {
            Just(None).boxed()
        };
        (entries, prop_option(arb_image()), child).prop_map(|(raw, image, child)| {
            let mut ifd = Ifd::default();
            for (tag, value) in raw {
                if !crate::tags::is_structural(tag) {
                    ifd.set(tag, value);
                }
            }
            ifd.image = image;
            ifd.sub_ifds = child.into_iter().map(|c| *c).collect();
            ifd
        })
    }

    proptest! {
        #[test]
        fn round_trips_generated_dng(
            little in any::<bool>(),
            ifds in prop_vec(arb_ifd(true), 1..3),
        ) {
            let endian = if little { Endian::Little } else { Endian::Big };
            let dng = Dng { endian, ifds };
            let bytes = write(&dng).unwrap();
            let parsed = read(&bytes).unwrap();
            assert_eq!(parsed, dng);
            assert_eq!(write(&parsed).unwrap(), bytes);
        }
    }
}
