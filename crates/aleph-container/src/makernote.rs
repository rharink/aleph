//! Repairing a vendor `MakerNote` whose internal offsets are file-absolute.
//!
//! Unsafe `MakerNotes` (DNG `MakerNoteSafety` != 1) store offsets relative to the
//! file start, so they break when the container rewrites layout and moves the
//! blob. For recognized, flat, self-contained forms we shift every internal
//! offset by the relocation delta — the same repair exiftool's `FixBase`
//! performs. Anything we cannot fully recognize and validate is returned as
//! `None` so the caller keeps the bytes byte-for-byte intact (never corrupted).

use crate::tags;
use crate::value::get_u16;
use crate::value::get_u32;
use crate::value::Endian;

// Sigma/Foveon: an 8-byte signature plus 2 version bytes, then a standard TIFF
// IFD whose value offsets are absolute (relative to the file start).
const SIGMA_SIGNATURE: &[u8] = b"SIGMA\x00\x00\x00";
const FOVEON_SIGNATURE: &[u8] = b"FOVEON\x00\x00";
const SIGMA_IFD_START: usize = 10;

/// Rewrite a relocated `MakerNote`'s file-absolute internal offsets so they remain
/// valid after the blob moves from `old_base` to `new_base`.
///
/// Returns `None` (keep the bytes verbatim) unless the `MakerNote` is a recognized,
/// flat, fully self-contained absolute-offset form that we can repair with
/// certainty.
pub(crate) fn relocate(
    bytes: &[u8],
    endian: Endian,
    old_base: u32,
    new_base: u32,
) -> Option<Vec<u8>> {
    if old_base == new_base {
        return Some(bytes.to_vec());
    }
    let ifd_start = ifd_start(bytes)?;
    let delta = i64::from(new_base) - i64::from(old_base);
    let blob_end = old_base.checked_add(u32::try_from(bytes.len()).ok()?)?;

    let mut out = bytes.to_vec();
    let count = usize::from(get_u16(endian, out.get(ifd_start..ifd_start + 2)?));
    let entries_end = ifd_start + 2 + count.checked_mul(12)?;

    // A chained next-IFD pointer would need its own walk; bail rather than
    // half-repair.
    if get_u32(endian, out.get(entries_end..entries_end + 4)?) != 0 {
        return None;
    }

    for i in 0..count {
        let entry = ifd_start + 2 + i * 12;
        let type_code = get_u16(endian, out.get(entry + 2..entry + 4)?);
        if type_code == tags::TYPE_IFD {
            return None; // nested IFD pointer: out of scope for the simple shift
        }
        let size = tags::type_size(type_code)?;
        let elems = get_u32(endian, out.get(entry + 4..entry + 8)?) as usize;
        if size.checked_mul(elems)? <= 4 {
            continue; // value is stored inline; no offset field to fix
        }
        let offset = get_u32(endian, out.get(entry + 8..entry + 12)?);
        if !(old_base..blob_end).contains(&offset) {
            return None; // offset is not a self-contained, in-blob absolute offset
        }
        let shifted = u32::try_from(i64::from(offset) + delta).ok()?;
        put_u32_at(&mut out, entry + 8, endian, shifted);
    }
    Some(out)
}

fn ifd_start(bytes: &[u8]) -> Option<usize> {
    if bytes.starts_with(SIGMA_SIGNATURE) || bytes.starts_with(FOVEON_SIGNATURE) {
        Some(SIGMA_IFD_START)
    } else {
        None
    }
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
    use super::relocate;
    use crate::value::Endian;

    // Build a minimal Sigma-style MakerNote at `base`: header + 1-entry IFD whose
    // single out-of-line value (8 bytes) sits right after the IFD, addressed
    // file-absolute.
    fn sigma_makernote(base: u32) -> Vec<u8> {
        let mut mn = b"SIGMA\x00\x00\x00".to_vec();
        mn.extend_from_slice(&[0x01, 0x04]); // version; IFD starts at byte 10
        mn.extend_from_slice(&1u16.to_le_bytes()); // entry count = 1
                                                   // entry: tag=2, type=5 (rational), count=1 -> 8 bytes, out-of-line
        mn.extend_from_slice(&2u16.to_le_bytes());
        mn.extend_from_slice(&5u16.to_le_bytes());
        mn.extend_from_slice(&1u32.to_le_bytes());
        let value_pos: u32 = 10 + 2 + 12 + 4; // header+count+entry+next
        mn.extend_from_slice(&(base + value_pos).to_le_bytes()); // absolute offset
        mn.extend_from_slice(&0u32.to_le_bytes()); // next IFD = 0
        mn.extend_from_slice(&[1, 0, 0, 0, 2, 0, 0, 0]); // the rational value (8 bytes)
        mn
    }

    #[test]
    fn shifts_absolute_offsets_by_relocation_delta() {
        let old = 1000u32;
        let new = 4000u32;
        let mn = sigma_makernote(old);
        let moved = relocate(&mn, Endian::Little, old, new).expect("relocatable");

        // The single offset field must have moved by (new - old).
        let off_pos = 10 + 2 + 8; // header + count + (tag,type,count)
        let original = u32::from_le_bytes(mn[off_pos..off_pos + 4].try_into().unwrap());
        let shifted = u32::from_le_bytes(moved[off_pos..off_pos + 4].try_into().unwrap());
        assert_eq!(shifted, original + (new - old));
        // Everything except the offset field is untouched.
        assert_eq!(&moved[..off_pos], &mn[..off_pos]);
        assert_eq!(&moved[off_pos + 4..], &mn[off_pos + 4..]);
    }

    #[test]
    fn no_op_when_not_relocated() {
        let mn = sigma_makernote(1000);
        assert_eq!(relocate(&mn, Endian::Little, 1000, 1000).unwrap(), mn);
    }

    #[test]
    fn leaves_unrecognized_makernotes_alone() {
        let other = b"NIKON\x00stuff".to_vec();
        assert_eq!(relocate(&other, Endian::Little, 1000, 4000), None);
    }

    #[test]
    fn rejects_offset_outside_the_blob() {
        // Same shape, but the offset points outside [old, old+len): not self-contained.
        let mut mn = sigma_makernote(1000);
        let off_pos = 10 + 2 + 8;
        mn[off_pos..off_pos + 4].copy_from_slice(&9_000_000u32.to_le_bytes());
        assert_eq!(relocate(&mn, Endian::Little, 1000, 4000), None);
    }
}
