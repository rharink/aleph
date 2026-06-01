use std::collections::BTreeMap;

/// A normalized, comparable view of a DNG IFD's tag set: each tag mapped to its
/// canonical value bytes. The encoding of those bytes is caller-defined; this
/// type only ever compares them for equality, never interprets them.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct TagSnapshot {
    tags: BTreeMap<u16, Vec<u8>>,
}

impl TagSnapshot {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a tag's canonical value bytes, replacing any prior value.
    pub fn insert(&mut self, tag: u16, value: Vec<u8>) {
        self.tags.insert(tag, value);
    }

    #[must_use]
    pub fn get(&self, tag: u16) -> Option<&[u8]> {
        self.tags.get(&tag).map(Vec::as_slice)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.tags.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }
}

impl TagSnapshot {
    // Crate-internal accessor for the verifier's tag-by-tag merge join.
    pub(crate) fn entries(&self) -> &BTreeMap<u16, Vec<u8>> {
        &self.tags
    }
}

#[cfg(test)]
mod tests {
    use super::TagSnapshot;

    #[test]
    fn insert_replaces_and_reads_back() {
        let mut snap = TagSnapshot::new();
        assert!(snap.is_empty());
        snap.insert(256, vec![1, 2, 3]);
        snap.insert(256, vec![4, 5]);
        assert_eq!(snap.get(256), Some([4, 5].as_slice()));
        assert_eq!(snap.len(), 1);
        assert!(!snap.is_empty());
        assert_eq!(snap.get(999), None);
    }
}
