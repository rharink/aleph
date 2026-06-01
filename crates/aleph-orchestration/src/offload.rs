//! Dual-destination offload: copy a card to N destinations and verify every
//! copy with an independent blake3 checksum. Files are processed in parallel.

use std::fs::File;
use std::io::Read as _;
use std::path::Path;
use std::path::PathBuf;

use rayon::iter::IntoParallelRefIterator as _;
use rayon::iter::ParallelIterator as _;

use crate::error::OrchestrationError;
use crate::frames::walk_files;

const CHUNK: usize = 1 << 16;

/// Outcome of an offload run, one entry per source file.
#[derive(Clone, Debug)]
pub struct OffloadReport {
    /// Per-file results, in source-relative path order.
    pub files: Vec<FileOutcome>,
}

/// Result of copying one source file to every destination.
#[derive(Clone, Debug)]
pub struct FileOutcome {
    /// Absolute source path.
    pub source: PathBuf,
    /// Path relative to the offload root (the layout reproduced at each dest).
    pub relative: PathBuf,
    /// Source size in bytes (0 when the source could not be read).
    pub size: u64,
    /// blake3 checksum of the source, or `None` if it could not be read.
    pub checksum: Option<blake3::Hash>,
    /// Error reading the source, if any (destinations are then skipped).
    pub error: Option<String>,
    /// Per-destination copy results.
    pub destinations: Vec<DestinationOutcome>,
}

/// Result of copying one file to one destination.
#[derive(Clone, Debug)]
pub struct DestinationOutcome {
    /// Full path written at the destination.
    pub path: PathBuf,
    /// Whether the copy verified against the source checksum.
    pub status: DestinationStatus,
}

/// Verification status of a single destination copy.
#[derive(Clone, Debug)]
pub enum DestinationStatus {
    /// The copy's checksum matched the source.
    Verified,
    /// The copy was written but its checksum did not match the source.
    ChecksumMismatch,
    /// The copy could not be written or re-read.
    Error(String),
}

/// Copy every file at `source` to each of `destinations`, verifying each copy.
///
/// Returns a per-file report rather than failing fast: a single unreadable file
/// or bad destination does not abort the run. Inspect [`OffloadReport`] for the
/// outcome.
///
/// # Errors
/// Returns [`OrchestrationError::NoDestinations`] if `destinations` is empty, or
/// an I/O error if `source` itself cannot be enumerated.
pub fn offload(
    source: &Path,
    destinations: &[PathBuf],
) -> Result<OffloadReport, OrchestrationError> {
    if destinations.is_empty() {
        return Err(OrchestrationError::NoDestinations);
    }

    let files = walk_files(source)?;
    let outcomes = files
        .par_iter()
        .map(|(absolute, relative)| offload_one(absolute, relative, destinations))
        .collect();

    Ok(OffloadReport { files: outcomes })
}

impl OffloadReport {
    /// Whether every source was read and every copy verified.
    #[must_use]
    pub fn all_verified(&self) -> bool {
        self.files.iter().all(FileOutcome::is_ok)
    }

    /// Number of files copied and verified to all destinations.
    #[must_use]
    pub fn verified_files(&self) -> usize {
        self.files.iter().filter(|f| f.is_ok()).count()
    }

    /// Number of files that failed to read or failed at any destination.
    #[must_use]
    pub fn failed_files(&self) -> usize {
        self.files.len() - self.verified_files()
    }

    /// Total bytes across all source files.
    #[must_use]
    pub fn total_bytes(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }
}

impl FileOutcome {
    /// Whether this file was read and verified at every destination.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.error.is_none()
            && !self.destinations.is_empty()
            && self.destinations.iter().all(DestinationOutcome::verified)
    }
}

impl DestinationOutcome {
    /// Whether this destination copy verified.
    #[must_use]
    pub fn verified(&self) -> bool {
        matches!(self.status, DestinationStatus::Verified)
    }
}

fn offload_one(source: &Path, relative: &Path, destinations: &[PathBuf]) -> FileOutcome {
    let (checksum, size) = match hash_file(source) {
        Ok(result) => result,
        Err(error) => {
            return FileOutcome {
                source: source.to_path_buf(),
                relative: relative.to_path_buf(),
                size: 0,
                checksum: None,
                error: Some(error.to_string()),
                destinations: Vec::new(),
            };
        }
    };

    let outcomes = destinations
        .iter()
        .map(|dest| {
            let path = dest.join(relative);
            let status = match copy_and_verify(source, &path, checksum) {
                Ok(true) => DestinationStatus::Verified,
                Ok(false) => DestinationStatus::ChecksumMismatch,
                Err(error) => DestinationStatus::Error(error.to_string()),
            };
            DestinationOutcome { path, status }
        })
        .collect();

    FileOutcome {
        source: source.to_path_buf(),
        relative: relative.to_path_buf(),
        size,
        checksum: Some(checksum),
        error: None,
        destinations: outcomes,
    }
}

fn copy_and_verify(source: &Path, target: &Path, expected: blake3::Hash) -> std::io::Result<bool> {
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(source, target)?;
    let (actual, _) = hash_file(target)?;
    Ok(actual == expected)
}

fn hash_file(path: &Path) -> std::io::Result<(blake3::Hash, u64)> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0u8; CHUNK];
    let mut total: u64 = 0;
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        total = total.saturating_add(u64::try_from(read).unwrap_or(0));
    }
    Ok((hasher.finalize(), total))
}

#[cfg(test)]
mod tests {
    use super::offload;
    use std::fs;

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let mut dir = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        dir.push(format!("aleph-offload-{tag}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn copies_and_verifies_a_tree_to_two_destinations() {
        let root = temp_dir("src");
        fs::create_dir_all(root.join("sub")).unwrap();
        fs::write(root.join("a.dng"), b"frame-a-contents").unwrap();
        fs::write(root.join("sub/b.dng"), b"frame-b-contents").unwrap();

        let dest1 = temp_dir("d1");
        let dest2 = temp_dir("d2");

        let report = offload(&root, &[dest1.clone(), dest2.clone()]).unwrap();

        assert!(report.all_verified());
        assert_eq!(report.files.len(), 2);
        assert_eq!(report.verified_files(), 2);
        assert_eq!(report.failed_files(), 0);
        assert_eq!(report.total_bytes(), 16 + 16);

        assert_eq!(fs::read(dest1.join("a.dng")).unwrap(), b"frame-a-contents");
        assert_eq!(
            fs::read(dest2.join("sub/b.dng")).unwrap(),
            b"frame-b-contents"
        );

        fs::remove_dir_all(&root).ok();
        fs::remove_dir_all(&dest1).ok();
        fs::remove_dir_all(&dest2).ok();
    }

    #[test]
    fn single_file_source_is_offloaded() {
        let root = temp_dir("file-src");
        let file = root.join("only.dng");
        fs::write(&file, b"solo").unwrap();
        let dest = temp_dir("file-dest");

        let report = offload(&file, std::slice::from_ref(&dest)).unwrap();
        assert!(report.all_verified());
        assert_eq!(report.files.len(), 1);
        assert_eq!(fs::read(dest.join("only.dng")).unwrap(), b"solo");

        fs::remove_dir_all(&root).ok();
        fs::remove_dir_all(&dest).ok();
    }

    #[test]
    fn empty_destinations_is_rejected() {
        let root = temp_dir("nodest");
        fs::write(root.join("x.dng"), b"x").unwrap();
        assert!(offload(&root, &[]).is_err());
        fs::remove_dir_all(&root).ok();
    }
}
