use std::cmp::Ordering;

use crate::snapshot::TagSnapshot;

/// How a preserved tag failed to round-trip.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViolationKind {
    /// Present in the restored snapshot but not the original.
    Added,
    /// Present in the original snapshot but not the restored.
    Removed,
    /// Present in both, with differing value bytes.
    Changed,
}

/// A single preserved tag that did not survive the round-trip unchanged.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Violation {
    pub tag: u16,
    pub kind: ViolationKind,
}

/// Verify the restored snapshot preserves every original tag bit-exactly, except
/// tags allowed to change during compression (currently only 259 Compression,
/// whose value and presence are both ignored).
///
/// Violations are returned sorted ascending by tag.
///
/// # Errors
/// Returns the full list of `Violation`s if any preserved tag was added,
/// removed, or changed.
pub fn verify_preserved(
    original: &TagSnapshot,
    restored: &TagSnapshot,
) -> Result<(), Vec<Violation>> {
    let mut violations = Vec::new();

    let orig = original.entries();
    let rest = restored.entries();

    // Sorted merge join over the union of tags. BTreeMap iteration is already
    // ascending, so violations come out sorted without a later re-sort.
    let mut o = orig.iter().peekable();
    let mut r = rest.iter().peekable();
    loop {
        let next = match (o.peek(), r.peek()) {
            (Some((&ot, ov)), Some((&rt, rv))) => match ot.cmp(&rt) {
                Ordering::Equal => {
                    if !is_allowed(ot) && ov != rv {
                        violations.push(Violation {
                            tag: ot,
                            kind: ViolationKind::Changed,
                        });
                    }
                    o.next();
                    r.next();
                    continue;
                }
                Ordering::Less => {
                    o.next();
                    Some((ot, ViolationKind::Removed))
                }
                Ordering::Greater => {
                    r.next();
                    Some((rt, ViolationKind::Added))
                }
            },
            (Some((&ot, _)), None) => {
                o.next();
                Some((ot, ViolationKind::Removed))
            }
            (None, Some((&rt, _))) => {
                r.next();
                Some((rt, ViolationKind::Added))
            }
            (None, None) => break,
        };
        if let Some((tag, kind)) = next {
            if !is_allowed(tag) {
                violations.push(Violation { tag, kind });
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

// A tag whose difference is ignored by the policy.
fn is_allowed(tag: u16) -> bool {
    tag == tags::COMPRESSION || tag == tags::ALEPH_MARKER
}

/// Well-known DNG tag numbers used by the policy.
pub mod tags {
    /// TIFF/DNG `Compression`; legitimately flips between 1 (uncompressed) and 7
    /// (lossless JPEG) across a compress/decompress round-trip.
    pub const COMPRESSION: u16 = 259;

    /// Aleph-private marker tag. Aleph stamps this onto an image directory it
    /// has lossless-JPEG encoded, recording the original `Compression` value so
    /// decompression can identify and restore exactly the directories Aleph
    /// produced (never a camera's pre-existing compressed preview). Added on
    /// compress, removed on decompress, so it is ignored by the policy.
    ///
    /// Provisional private-range value; register with Adobe before release.
    pub const ALEPH_MARKER: u16 = 0xA1E9;
}

#[cfg(test)]
mod tests {
    use super::tags;
    use super::verify_preserved;
    use super::Violation;
    use super::ViolationKind;
    use crate::snapshot::TagSnapshot;
    use proptest::prelude::*;

    fn snap(pairs: &[(u16, &[u8])]) -> TagSnapshot {
        let mut s = TagSnapshot::new();
        for &(tag, bytes) in pairs {
            s.insert(tag, bytes.to_vec());
        }
        s
    }

    #[test]
    fn identical_snapshots_ok() {
        let s = snap(&[(256, &[1, 0]), (258, &[16]), (259, &[7])]);
        assert_eq!(verify_preserved(&s, &s.clone()), Ok(()));
    }

    #[test]
    fn compression_value_change_ok() {
        let before = snap(&[(256, &[1, 0]), (tags::COMPRESSION, &[1])]);
        let after = snap(&[(256, &[1, 0]), (tags::COMPRESSION, &[7])]);
        assert_eq!(verify_preserved(&before, &after), Ok(()));
    }

    #[test]
    fn compression_presence_only_in_one_ok() {
        let before = snap(&[(256, &[1, 0]), (tags::COMPRESSION, &[1])]);
        let after = snap(&[(256, &[1, 0])]);
        assert_eq!(verify_preserved(&before, &after), Ok(()));
        assert_eq!(verify_preserved(&after, &before), Ok(()));
    }

    #[test]
    fn aleph_marker_presence_is_allowed() {
        // compress adds the marker; decompress removes it. Its presence/value must
        // never count as a violation.
        let original = snap(&[(256, &[1, 0]), (258, &[16])]);
        let compressed = snap(&[(256, &[1, 0]), (258, &[16]), (tags::ALEPH_MARKER, &[1])]);
        assert_eq!(verify_preserved(&original, &compressed), Ok(()));
    }

    #[test]
    fn changed_tag_is_violation() {
        let before = snap(&[(256, &[1, 0])]);
        let after = snap(&[(256, &[2, 0])]);
        assert_eq!(
            verify_preserved(&before, &after),
            Err(vec![Violation {
                tag: 256,
                kind: ViolationKind::Changed
            }])
        );
    }

    #[test]
    fn missing_in_restored_is_removed() {
        let before = snap(&[(256, &[1, 0]), (320, &[9])]);
        let after = snap(&[(256, &[1, 0])]);
        assert_eq!(
            verify_preserved(&before, &after),
            Err(vec![Violation {
                tag: 320,
                kind: ViolationKind::Removed
            }])
        );
    }

    #[test]
    fn extra_in_restored_is_added() {
        let before = snap(&[(256, &[1, 0])]);
        let after = snap(&[(256, &[1, 0]), (320, &[9])]);
        assert_eq!(
            verify_preserved(&before, &after),
            Err(vec![Violation {
                tag: 320,
                kind: ViolationKind::Added
            }])
        );
    }

    #[test]
    fn violations_sorted_ascending_by_tag() {
        // Mix of Added (700), Changed (256), Removed (320); Compression ignored.
        let before = snap(&[(256, &[1, 0]), (tags::COMPRESSION, &[1]), (320, &[9])]);
        let after = snap(&[(256, &[9, 9]), (tags::COMPRESSION, &[7]), (700, &[0])]);
        assert_eq!(
            verify_preserved(&before, &after),
            Err(vec![
                Violation {
                    tag: 256,
                    kind: ViolationKind::Changed
                },
                Violation {
                    tag: 320,
                    kind: ViolationKind::Removed
                },
                Violation {
                    tag: 700,
                    kind: ViolationKind::Added
                },
            ])
        );
    }

    proptest! {
        #[test]
        fn mutating_only_compression_always_ok(
            pairs in proptest::collection::vec(
                (any::<u16>(), proptest::collection::vec(any::<u8>(), 0..8)),
                0..16,
            ),
            new_compression in proptest::option::of(proptest::collection::vec(any::<u8>(), 0..8)),
        ) {
            let mut original = TagSnapshot::new();
            for (tag, bytes) in pairs {
                original.insert(tag, bytes);
            }
            let mut restored = original.clone();
            if let Some(bytes) = new_compression {
                restored.insert(tags::COMPRESSION, bytes);
            } else {
                // Drop compression entirely; presence is ignored too.
                let mut rebuilt = TagSnapshot::new();
                for (&tag, bytes) in restored.entries() {
                    if tag != tags::COMPRESSION {
                        rebuilt.insert(tag, bytes.clone());
                    }
                }
                restored = rebuilt;
            }
            prop_assert_eq!(verify_preserved(&original, &restored), Ok(()));
        }
    }
}
