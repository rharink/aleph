//! Bridge from container [`Ifd`] entries to a metadata [`TagSnapshot`].
//!
//! The container and metadata crates are independent siblings, so orchestration
//! owns the normalization: each tag value is flattened to canonical bytes that
//! the metadata verifier compares only for equality.

use aleph_container::Ifd;
use aleph_container::Value;
use aleph_metadata::TagSnapshot;

/// Snapshot every semantic entry of `ifd` for round-trip verification.
pub(crate) fn snapshot(ifd: &Ifd) -> TagSnapshot {
    let mut snap = TagSnapshot::new();
    for entry in &ifd.entries {
        snap.insert(entry.tag, canonical_bytes(&entry.value));
    }
    snap
}

/// Flatten a value to type-tagged little-endian bytes. The exact encoding is
/// irrelevant as long as it is deterministic and distinguishes distinct values;
/// metadata only checks equality.
fn canonical_bytes(value: &Value) -> Vec<u8> {
    let mut out = value.type_code().to_le_bytes().to_vec();
    match value {
        Value::Byte(v) | Value::Undefined(v) | Value::Ascii(v) => out.extend_from_slice(v),
        Value::SByte(v) => out.extend(v.iter().flat_map(|&x| x.to_le_bytes())),
        Value::Short(v) => out.extend(v.iter().flat_map(|&x| x.to_le_bytes())),
        Value::SShort(v) => out.extend(v.iter().flat_map(|&x| x.to_le_bytes())),
        Value::Long(v) => out.extend(v.iter().flat_map(|&x| x.to_le_bytes())),
        Value::SLong(v) => out.extend(v.iter().flat_map(|&x| x.to_le_bytes())),
        Value::Float(v) => out.extend(v.iter().flat_map(|&x| x.to_le_bytes())),
        Value::Double(v) => out.extend(v.iter().flat_map(|&x| x.to_le_bytes())),
        Value::Rational(v) => out.extend(
            v.iter()
                .flat_map(|&(n, d)| [n.to_le_bytes(), d.to_le_bytes()])
                .flatten(),
        ),
        Value::SRational(v) => out.extend(
            v.iter()
                .flat_map(|&(n, d)| [n.to_le_bytes(), d.to_le_bytes()])
                .flatten(),
        ),
    }
    out
}

#[cfg(test)]
mod tests {
    use super::canonical_bytes;
    use aleph_container::Value;

    #[test]
    fn distinct_types_with_same_payload_differ() {
        let as_byte = canonical_bytes(&Value::Byte(vec![1]));
        let as_short = canonical_bytes(&Value::Short(vec![1]));
        assert_ne!(as_byte, as_short);
    }

    #[test]
    fn distinct_values_differ() {
        assert_ne!(
            canonical_bytes(&Value::Short(vec![1, 2])),
            canonical_bytes(&Value::Short(vec![1, 3]))
        );
    }
}
