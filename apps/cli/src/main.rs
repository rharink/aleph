use clap::Parser;
use miette::Result;

#[derive(Parser)]
#[command(name = "aleph", about = "Near-lossless RAW compressor")]
struct Cli {}

fn main() -> Result<()> {
    let _cli = Cli::parse();
    Ok(())
}
