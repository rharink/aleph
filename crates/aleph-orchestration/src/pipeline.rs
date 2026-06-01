//! File- and sequence-level compression: the operations the CLI drives.
//!
//! Inputs may be a single DNG or a directory of frames; frames are processed in
//! parallel. Compression optionally re-verifies each frame in memory before it
//! is written, so a written file is provably recoverable.

use std::path::Path;
use std::path::PathBuf;

use rayon::iter::IntoParallelRefIterator as _;
use rayon::iter::ParallelIterator as _;

use crate::compress::compress_dng;
use crate::compress::decompress_dng;
use crate::error::OrchestrationError;
use crate::frames::enumerate_dng;
use crate::verify::verify_compressed;

/// Options governing a compression run.
#[derive(Clone, Copy, Debug)]
pub struct CompressOptions {
    /// Re-verify each frame round-trips before writing it (recommended).
    pub verify: bool,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self { verify: true }
    }
}

/// Per-frame size accounting.
#[derive(Clone, Debug)]
pub struct FrameStat {
    /// Source path.
    pub input: PathBuf,
    /// Written output path.
    pub output: PathBuf,
    /// Source size in bytes.
    pub input_bytes: u64,
    /// Output size in bytes.
    pub output_bytes: u64,
}

/// Aggregate result of a compress or decompress run.
#[derive(Clone, Debug)]
pub struct Summary {
    /// One entry per processed frame.
    pub frames: Vec<FrameStat>,
}

impl FrameStat {
    /// Output size as a fraction of input size (`< 1.0` means it shrank).
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // byte counts well within f64's exact range
    pub fn ratio(&self) -> f64 {
        if self.input_bytes == 0 {
            return 0.0;
        }
        self.output_bytes as f64 / self.input_bytes as f64
    }
}

impl Summary {
    /// Total input bytes across all frames.
    #[must_use]
    pub fn total_input(&self) -> u64 {
        self.frames.iter().map(|f| f.input_bytes).sum()
    }

    /// Total output bytes across all frames.
    #[must_use]
    pub fn total_output(&self) -> u64 {
        self.frames.iter().map(|f| f.output_bytes).sum()
    }

    /// Number of frames processed.
    #[must_use]
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Total output bytes as a fraction of total input bytes.
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // byte counts well within f64's exact range
    pub fn ratio(&self) -> f64 {
        let input = self.total_input();
        if input == 0 {
            return 0.0;
        }
        self.total_output() as f64 / input as f64
    }
}

/// Compress a DNG file or a directory of DNG frames into `out_dir`.
///
/// # Errors
/// Returns [`OrchestrationError`] if no frames are found, a frame cannot be read
/// or encoded, verification fails, or output cannot be written.
pub fn compress(
    input: &Path,
    out_dir: &Path,
    options: &CompressOptions,
) -> Result<Summary, OrchestrationError> {
    run(input, out_dir, |path, dng| {
        let compressed = compress_dng(dng)?;
        if &compressed == dng {
            return Err(OrchestrationError::NothingToCompress(path.to_path_buf()));
        }
        if options.verify {
            verify_compressed(dng, &compressed)?;
        }
        Ok(compressed)
    })
}

/// Decompress a DNG file or a directory of DNG frames into `out_dir`.
///
/// # Errors
/// Returns [`OrchestrationError`] if no frames are found, a frame cannot be read
/// or decoded, or output cannot be written.
pub fn decompress(input: &Path, out_dir: &Path) -> Result<Summary, OrchestrationError> {
    run(input, out_dir, |path, dng| {
        let decompressed = decompress_dng(dng)?;
        if &decompressed == dng {
            return Err(OrchestrationError::NothingToDecompress(path.to_path_buf()));
        }
        Ok(decompressed)
    })
}

fn run<F>(input: &Path, out_dir: &Path, transform: F) -> Result<Summary, OrchestrationError>
where
    F: Fn(&Path, &aleph_container::Dng) -> Result<aleph_container::Dng, OrchestrationError> + Sync,
{
    let inputs = resolve_inputs(input)?;
    std::fs::create_dir_all(out_dir).map_err(|e| OrchestrationError::io(out_dir, e))?;

    let frames = inputs
        .par_iter()
        .map(|path| process_one(path, out_dir, &transform))
        .collect::<Result<Vec<FrameStat>, OrchestrationError>>()?;

    Ok(Summary { frames })
}

fn process_one<F>(
    input: &Path,
    out_dir: &Path,
    transform: &F,
) -> Result<FrameStat, OrchestrationError>
where
    F: Fn(&Path, &aleph_container::Dng) -> Result<aleph_container::Dng, OrchestrationError>,
{
    let name = input
        .file_name()
        .ok_or_else(|| OrchestrationError::NotFileOrDir(input.to_path_buf()))?;
    let output = out_dir.join(name);

    let dng = aleph_container::read_file(input)?;
    let transformed = transform(input, &dng)?;
    let bytes = aleph_container::write(&transformed)?;

    let input_bytes = std::fs::metadata(input)
        .map_err(|e| OrchestrationError::io(input, e))?
        .len();
    std::fs::write(&output, &bytes).map_err(|e| OrchestrationError::io(&output, e))?;

    Ok(FrameStat {
        input: input.to_path_buf(),
        output,
        input_bytes,
        output_bytes: bytes.len() as u64,
    })
}

fn resolve_inputs(input: &Path) -> Result<Vec<PathBuf>, OrchestrationError> {
    let meta = std::fs::metadata(input).map_err(|e| OrchestrationError::io(input, e))?;
    if meta.is_file() {
        return Ok(vec![input.to_path_buf()]);
    }
    if meta.is_dir() {
        let frames = enumerate_dng(input)?;
        if frames.is_empty() {
            return Err(OrchestrationError::NoFrames(input.to_path_buf()));
        }
        return Ok(frames);
    }
    Err(OrchestrationError::NotFileOrDir(input.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::compress;
    use super::decompress;
    use super::CompressOptions;
    use crate::error::OrchestrationError;
    use aleph_container::Dng;
    use aleph_container::Endian;
    use aleph_container::Ifd;
    use aleph_container::Image;
    use aleph_container::Layout;
    use aleph_container::Value;
    use std::fs;

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let mut dir = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        dir.push(format!("aleph-pipeline-{tag}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn gradient_dng() -> Dng {
        // 8x8 single-channel 16-bit, smooth gradient (compressible).
        let mut samples = Vec::with_capacity(64);
        for y in 0..8u16 {
            for x in 0..8u16 {
                samples.push(x * 7 + y * 3);
            }
        }
        let segment: Vec<u8> = samples.iter().flat_map(|&v| v.to_le_bytes()).collect();
        let mut ifd = Ifd::default();
        ifd.set(256, Value::Long(vec![8]));
        ifd.set(257, Value::Long(vec![8]));
        ifd.set(258, Value::Short(vec![16]));
        ifd.set(259, Value::Short(vec![1]));
        ifd.set(277, Value::Short(vec![1]));
        ifd.set(0x9000, Value::Ascii(b"frame-meta\0".to_vec()));
        ifd.image = Some(Image {
            layout: Layout::Strips { rows_per_strip: 8 },
            segments: vec![segment],
        });
        Dng {
            endian: Endian::Little,
            ifds: vec![ifd],
        }
    }

    #[test]
    fn file_roundtrip_recovers_original_bytes() {
        let in_dir = temp_dir("in");
        let out_dir = temp_dir("out");
        let back_dir = temp_dir("back");

        let original = gradient_dng();
        let in_path = in_dir.join("A001.dng");
        aleph_container::write_file(&in_path, &original).unwrap();
        let original_bytes = fs::read(&in_path).unwrap();

        let summary = compress(&in_path, &out_dir, &CompressOptions::default()).unwrap();
        assert_eq!(summary.frame_count(), 1);
        // Gradient must actually compress.
        assert!(summary.total_output() < summary.total_input());

        let compressed_path = out_dir.join("A001.dng");
        decompress(&compressed_path, &back_dir).unwrap();
        let restored_bytes = fs::read(back_dir.join("A001.dng")).unwrap();

        assert_eq!(restored_bytes, original_bytes);

        for dir in [&in_dir, &out_dir, &back_dir] {
            fs::remove_dir_all(dir).ok();
        }
    }

    #[test]
    fn directory_sequence_is_processed() {
        let in_dir = temp_dir("seq-in");
        let out_dir = temp_dir("seq-out");
        for n in 0..3 {
            aleph_container::write_file(&in_dir.join(format!("f{n:03}.dng")), &gradient_dng())
                .unwrap();
        }
        // A non-DNG file must be ignored.
        fs::write(in_dir.join("notes.txt"), b"ignore me").unwrap();

        let summary = compress(&in_dir, &out_dir, &CompressOptions::default()).unwrap();
        assert_eq!(summary.frame_count(), 3);

        for dir in [&in_dir, &out_dir] {
            fs::remove_dir_all(dir).ok();
        }
    }

    #[test]
    fn already_compressed_input_is_reported() {
        let in_dir = temp_dir("ac-in");
        let out_dir = temp_dir("ac-out");
        let again_dir = temp_dir("ac-again");

        let in_path = in_dir.join("A001.dng");
        aleph_container::write_file(&in_path, &gradient_dng()).unwrap();
        compress(&in_path, &out_dir, &CompressOptions::default()).unwrap();

        // Re-compressing the already-compressed output reports nothing to do
        // instead of erroring deep inside the codec/packer.
        let compressed_path = out_dir.join("A001.dng");
        let result = compress(&compressed_path, &again_dir, &CompressOptions::default());
        assert!(matches!(
            result,
            Err(OrchestrationError::NothingToCompress(_))
        ));

        for dir in [&in_dir, &out_dir, &again_dir] {
            fs::remove_dir_all(dir).ok();
        }
    }

    #[test]
    fn non_aleph_decompress_is_reported() {
        let in_dir = temp_dir("nd-in");
        let out_dir = temp_dir("nd-out");

        // A plain uncompressed DNG with no Aleph marker: nothing for us to undo.
        let in_path = in_dir.join("plain.dng");
        aleph_container::write_file(&in_path, &gradient_dng()).unwrap();

        let result = decompress(&in_path, &out_dir);
        assert!(matches!(
            result,
            Err(OrchestrationError::NothingToDecompress(_))
        ));

        for dir in [&in_dir, &out_dir] {
            fs::remove_dir_all(dir).ok();
        }
    }
}
