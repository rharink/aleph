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

# CRAP metric (complexity x coverage) for production code — runs test-cov first.
# tests/ and benches/ are excluded: they carry no LCOV entries, so scoring them
# would pin every helper at 0% coverage and produce false positives.
crap: test-cov
    cargo crap --workspace --lcov lcov.info --threshold 30 --fail-above --exclude 'tests/**' --exclude 'benches/**'

# ── Benchmarks ───────────────────────────────────────────────────────────────
# Driven via `cargo bench` (criterion). cargo-criterion is an optional nicer
# runner but is not required and is absent from the dev shell.
#
# Bench targets are listed explicitly: `cargo bench --benches` would also invoke
# the libtest harnesses of the lib/bin/test targets, which reject criterion's
# --save-baseline/--baseline flags. Add new benches here.
bench_targets := "--bench codec_lossless --bench container_read --bench orchestration_parallel"

# Run every criterion benchmark.
bench:
    cargo bench {{ bench_targets }}

# Run a single benchmark target, e.g.: just bench-one codec_lossless
bench-one name:
    cargo bench --bench {{ name }}

# Snapshot current performance under a named baseline — run BEFORE a change.
# e.g.: just bench-save before
bench-save name:
    cargo bench {{ bench_targets }} -- --save-baseline {{ name }}

# Re-run benchmarks and report deltas against a saved baseline — run AFTER a
# change. e.g.: just bench-cmp before
bench-cmp name:
    cargo bench {{ bench_targets }} -- --baseline {{ name }}

# ── Composite ────────────────────────────────────────────────────────────────

# Full local quality gate — run before pushing
check: fmt-check clippy test deny

# Heavier check including the CRAP metric (slower due to instrumented build)
check-full: fmt-check clippy test-cov deny crap

# ── Housekeeping ─────────────────────────────────────────────────────────────

clean:
    cargo clean
    rm -f lcov.info
