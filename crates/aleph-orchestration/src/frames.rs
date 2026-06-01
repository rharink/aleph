//! Enumerating inputs: DNG frame sequences and arbitrary file trees.

use std::path::Path;
use std::path::PathBuf;

use crate::error::OrchestrationError;

/// List the `.dng` frames directly inside `dir`, sorted by path.
///
/// A `CinemaDNG` clip is a flat directory of per-frame files, so this does not
/// recurse. The extension match is case-insensitive.
///
/// # Errors
/// Returns [`OrchestrationError::Io`] if `dir` cannot be read.
pub fn enumerate_dng(dir: &Path) -> Result<Vec<PathBuf>, OrchestrationError> {
    let mut frames = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| OrchestrationError::io(dir, e))? {
        let entry = entry.map_err(|e| OrchestrationError::io(dir, e))?;
        let path = entry.path();
        let is_file = entry
            .file_type()
            .map_err(|e| OrchestrationError::io(&path, e))?
            .is_file();
        if is_file && has_dng_extension(&path) {
            frames.push(path);
        }
    }
    frames.sort();
    Ok(frames)
}

/// Collect every regular file at or under `root` as `(absolute, relative)`.
///
/// A file `root` yields a single entry relative to its own name. Symlinks are
/// skipped to avoid cycles. Results are sorted by relative path.
///
/// # Errors
/// Returns [`OrchestrationError::Io`] on a read failure, or
/// [`OrchestrationError::NotFileOrDir`] if `root` is neither.
pub(crate) fn walk_files(root: &Path) -> Result<Vec<(PathBuf, PathBuf)>, OrchestrationError> {
    let meta = std::fs::symlink_metadata(root).map_err(|e| OrchestrationError::io(root, e))?;

    let mut out = Vec::new();
    if meta.is_file() {
        let name = root
            .file_name()
            .map_or_else(|| PathBuf::from("file"), PathBuf::from);
        out.push((root.to_path_buf(), name));
        return Ok(out);
    }
    if meta.is_dir() {
        collect_dir(root, Path::new(""), &mut out)?;
        out.sort_by(|a, b| a.1.cmp(&b.1));
        return Ok(out);
    }
    Err(OrchestrationError::NotFileOrDir(root.to_path_buf()))
}

fn collect_dir(
    dir: &Path,
    rel: &Path,
    out: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<(), OrchestrationError> {
    for entry in std::fs::read_dir(dir).map_err(|e| OrchestrationError::io(dir, e))? {
        let entry = entry.map_err(|e| OrchestrationError::io(dir, e))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| OrchestrationError::io(&path, e))?;
        let child_rel = rel.join(entry.file_name());

        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            collect_dir(&path, &child_rel, out)?;
        } else if file_type.is_file() {
            out.push((path, child_rel));
        }
    }
    Ok(())
}

fn has_dng_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("dng"))
}
