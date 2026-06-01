# Aleph ℵ

> *You only lose ε.*

Fast, near-lossless RAW compression for cinematographers and photographers. Lossless modes are bit-perfect — `decompress(compress(x)) == x`, provably.

**Status:** early development. Not yet usable.

---

## What it does

Aleph takes uncompressed open-format RAW files and produces smaller files with no visible quality loss, full metadata preservation, and fast multi-threaded processing.

| Feature | v1 | v2 | v3 |
|---|---|---|---|
| Lossless CinemaDNG compression | ✓ | | |
| Dual-destination offload + checksums | ✓ | | |
| DNG stills (Leica, Sigma, Panasonic L-mount) | | ✓ | |
| Watch folder / daemon mode | | ✓ | |
| Proxy generation (H.264 / ProRes) | | ✓ | |
| JPEG XL delivery pipeline | | | ✓ |
| GUI | | | ✓ |

Open formats only. No proprietary camera SDKs.

---

## Install

Binaries are not yet published. To build from source:

```sh
cargo build --release
# binary at: target/release/aleph
```

Requires Rust 1.80+.

---

## Usage

```sh
# Compress a CinemaDNG sequence losslessly
aleph compress ./footage/A001 --out ./compressed/A001

# Offload a card to two destinations with checksums
aleph offload /Volumes/CARD_A --to /Volumes/RAID --to /Volumes/BACKUP
```

---

## Workspace

```
crates/
  aleph-codec/          lossless codec — no I/O, no metadata
  aleph-container/      DNG/TIFF container read/write
  aleph-metadata/       tag preservation and verification
  aleph-orchestration/  parallel job execution, checksums, offload
apps/
  cli/                  aleph binary
  website/              marketing site (SvelteKit + Motion)
  gui/                  desktop GUI (future)
proof/                  Lean 4 / Alloy formal specs
```

---

## Development

Requires: `cargo-nextest`, `cargo-deny`, `cargo-crap`, `cargo-llvm-cov`, `criterion`, `just`.

```sh
just          # list all recipes
just check    # fmt + clippy + tests + deny (fast, run before push)
just check-full   # + coverage + CRAP metric
just bench    # criterion benchmarks
```

Correctness is the non-negotiable constraint. Every codec change must include a round-trip property test. See [AGENTS.md](AGENTS.md) for full contributor and agent guidance.

---

## License

Proprietary. All rights reserved.
