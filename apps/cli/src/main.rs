use std::path::PathBuf;

use aleph_orchestration::CompressOptions;
use aleph_orchestration::OffloadReport;
use aleph_orchestration::Summary;
use clap::Parser;
use clap::Subcommand;
use miette::bail;
use miette::IntoDiagnostic as _;
use miette::Result;

#[derive(Parser)]
#[command(name = "aleph", version, about = "Near-lossless RAW compressor")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

// Doc comments here are clap `--help` text, not rustdoc: backticks would render
// literally in the terminal.
#[allow(clippy::doc_markdown)]
#[derive(Subcommand)]
enum Command {
    /// Losslessly compress a CinemaDNG file or a directory of frames.
    Compress {
        /// Input `.dng` file or directory of frames.
        input: PathBuf,
        /// Output directory for compressed frames.
        #[arg(long)]
        out: PathBuf,
        /// Skip the per-frame round-trip verification (faster, less safe).
        #[arg(long)]
        no_verify: bool,
    },
    /// Decompress an aleph-compressed CinemaDNG file or directory of frames.
    Decompress {
        /// Input `.dng` file or directory of frames.
        input: PathBuf,
        /// Output directory for decompressed frames.
        #[arg(long)]
        out: PathBuf,
    },
    /// Copy a card to one or more destinations, verifying every copy.
    Offload {
        /// Source file or directory to offload.
        source: PathBuf,
        /// Destination directory (repeat `--to` for multiple destinations).
        #[arg(long = "to", required = true)]
        to: Vec<PathBuf>,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Compress {
            input,
            out,
            no_verify,
        } => {
            let options = CompressOptions { verify: !no_verify };
            let summary =
                aleph_orchestration::compress(&input, &out, &options).into_diagnostic()?;
            report_compression("Compressed", &summary);
        }
        Command::Decompress { input, out } => {
            let summary = aleph_orchestration::decompress(&input, &out).into_diagnostic()?;
            report_compression("Decompressed", &summary);
        }
        Command::Offload { source, to } => {
            let report = aleph_orchestration::offload(&source, &to).into_diagnostic()?;
            report_offload(&report);
            if !report.all_verified() {
                bail!(
                    "offload failed verification for {} file(s)",
                    report.failed_files()
                );
            }
        }
    }
    Ok(())
}

fn report_compression(verb: &str, summary: &Summary) {
    println!(
        "{verb} {} frame(s): {} -> {} ({:.1}% of original)",
        summary.frame_count(),
        human_bytes(summary.total_input()),
        human_bytes(summary.total_output()),
        summary.ratio() * 100.0,
    );
}

fn report_offload(report: &OffloadReport) {
    for file in &report.files {
        let status = if file.is_ok() { "ok" } else { "FAIL" };
        println!("[{status}] {}", file.relative.display());
        if let Some(error) = &file.error {
            println!("    source unreadable: {error}");
        }
    }
    println!(
        "Offloaded {}/{} file(s) verified, {} total",
        report.verified_files(),
        report.files.len(),
        human_bytes(report.total_bytes()),
    );
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    #[allow(clippy::cast_precision_loss)] // display only; exactness not required
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
