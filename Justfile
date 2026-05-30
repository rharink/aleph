set dotenv-load

# List available recipes
default:
    @just --list

# ── Build ────────────────────────────────────────────────────────────────────

build:
    cargo build --all-targets

build-release:
    cargo build --release

# ── Test ─────────────────────────────────────────────────────────────────────

test:
    cargo nextest run --all-features

# Run tests and emit lcov coverage data (required by `crap`)
test-cov:
    cargo llvm-cov nextest --all-features --lcov --output-path lcov.info

# ── Lint / format ────────────────────────────────────────────────────────────

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# ── Quality gates ────────────────────────────────────────────────────────────

# License and vulnerability policy
deny:
    cargo deny check

# CRAP metric — requires coverage data; runs test-cov first
crap: test-cov
    cargo crap --lcov lcov.info --fail-above 30

# ── Benchmarks ───────────────────────────────────────────────────────────────

bench:
    cargo criterion

# Run a single benchmark by name fragment, e.g.: just bench-one codec_lossless
bench-one name:
    cargo criterion --bench {{ name }}

# ── Composite ────────────────────────────────────────────────────────────────

# Full local quality gate — run before pushing
check: fmt-check clippy test deny

# Heavier check including the CRAP metric (slower due to instrumented build)
check-full: fmt-check clippy test-cov deny crap

# ── Housekeeping ─────────────────────────────────────────────────────────────

clean:
    cargo clean
    rm -f lcov.info
