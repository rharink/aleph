# Aleph — Agent & Contributor Guide

## What Aleph is

Aleph is a fast, near-lossless RAW compressor built in Rust for professional cinematographers and photographers. The long-term job is **on-set data management**: compress open-format RAW, verify integrity, offload to multiple destinations, and generate proxies — replacing expensive subscription tools like Silverstack (~€400/seat/year) with a fast, buy-once alternative.

The immediate comparables are **SlimRAW** (lossless CinemaDNG compressor, ~€65 one-time) and **Silverstack** (full DIT/offload workflow, subscription). Aleph competes on capability, correctness, and pricing model — not on undercutting.

**Market note:** pure CinemaDNG video is a narrowing segment (Blackmagic moved to BRAW; RED and Arri are proprietary). The durable market is open formats: CinemaDNG, DNG stills (Leica, Sigma fp, Panasonic L-mount), and the JPEG XL delivery pipeline. Stay on open specs — no proprietary SDK licensing.

**Formal verification as a differentiator:** Aleph aims to be the only RAW tool with machine-checked proofs of its lossless invariants. For professionals where data loss is catastrophic, "formally verified bit-perfect round-trip" is a genuine trust signal no competitor can match.

---

## Roadmap

**v1 — lossless foundation (current focus)**
- CLI only
- Lossless CinemaDNG compression with full metadata preservation
- Dual-destination offload with blake3 checksums
- Multi-threaded via rayon
- Round-trip correctness harness

**v2 — on-set workflow**
- DNG stills compression (L-mount: Panasonic/Sigma/Leica)
- Watch folder / daemon mode (process cards on plug-in)
- Proxy generation — transcode to H.264 / ProRes for offline edit
- Integrity re-verification before NLE ingest

**v3+ — delivery and reach**
- JPEG XL pipeline for stills delivery (RAW → JXL archive + delivery)
- Lossy / visually-lossless modes (quantization)
- GUI (Tauri or egui, TBD)
- Expanded format support as open specs permit

---

## Code style

Follow <https://epage.github.io/dev/rust-style/>. Key rules:

**Module layout**
- Use `mod.rs` as the directory root, not `name.rs`. (`lib.rs` and `main.rs` are their own roots.)
- Root files (`mod.rs`, `lib.rs`) contain only re-exports — they are a table of contents, not a definition site.
- Prelude modules contain only re-exports.
- Avoid `#[path]` except for `build.rs`-generated files.
- Modules re-export only from child and sibling modules; do not reach across the tree.
- Visibility: use only `pub`, `pub(crate)`, or private. Avoid `pub(super)` and other scoped forms.

**File structure within a module**
- Private imports first, then public imports alongside public API.
- Import traits anonymously; import other items individually (no compound `{a, b}` imports — reduces merge conflicts).
- Lead with the central type or function (table-of-contents principle).
- Order: type definitions → associated `impl` blocks → trait impls → private helpers.
- Callers precede callees. Public items precede private items.

**Function structure**
- Group related statements into visual paragraphs (blank line between logical units).
- Open a block with the output variable being constructed.
- Emphasize mutually exclusive branches with `if`/`else`/`match`; use early returns for bookkeeping, not business logic.
- Keep closures passed to combinators free of side effects with business meaning.
- Don't mix mutation and pure expression in the same statement.

**General**
- Code is technical writing. Apply the inverted pyramid: salient facts first.
- Strong abstractions hide detail. Weak abstractions live close to their usage point.
- No comments that describe *what* the code does. Only comments that explain *why* a non-obvious invariant, constraint, or workaround exists.
- No multi-line docstring blocks unless a public API genuinely needs them.

---

## Repository layout

Cargo workspace. Library crates live under `crates/`, user-facing applications under `apps/`.

```
aleph/
├── Cargo.toml                  # workspace root — shared deps and lint policy
├── Justfile
├── crates/
│   ├── aleph-codec/            # lossless codec (no I/O, no metadata, no CLI)
│   ├── aleph-container/        # DNG/TIFF container read/write (no codec logic)
│   ├── aleph-metadata/         # tag preservation, round-trip verification
│   └── aleph-orchestration/    # frame enumeration, rayon parallelism, checksums
├── apps/
│   ├── cli/                    # `aleph` binary — thin clap shell over orchestration
│   ├── website/                # marketing/landing site (SvelteKit + Motion, static)
│   └── gui/                    # future GUI (Tauri or egui, TBD)
├── benches/                    # workspace-level benchmarks (per-crate benches live in crates/)
└── proof/                      # Lean 4 / Alloy formal specs
```

`apps/website/` is a standalone SvelteKit app (pnpm, not a Cargo workspace member). It uses Motion (motion.dev) — the framework-agnostic engine behind Framer Motion — for animation, and builds to fully prerendered static output via `adapter-static`. Drive it with the `web-*` Just recipes or `pnpm` inside the directory.

## Architecture

Dependencies flow strictly inward:

```
apps/cli  ──►  aleph-orchestration  ──►  aleph-codec
                                    ──►  aleph-container
                                    ──►  aleph-metadata
```

No upward or sideways dependencies. `aleph-codec` must never import `aleph-container` or vice versa.

Error handling: `thiserror` in library crates, `miette` (with `fancy` feature) in CLI/GUI apps.

---

## Toolchain and quality gates

| Tool | Purpose |
|------|---------|
| `cargo nextest` | Default test runner — use instead of `cargo test` |
| `proptest` | Property-based tests; required for any codec or algorithm with an invariant |
| `cargo-deny` | License and vulnerability policy enforcement |
| `cargo-crap` | CRAP metric (cyclomatic complexity × coverage); flag functions above threshold 30 |
| `criterion` | Micro-benchmarks for codec and container hot paths |
| `bencher` | Tracks criterion results over time to catch performance regressions |

Use the `Justfile` for all common tasks (`just --list` to see them). Run before any commit:

```
just check        # fmt-check + clippy + test + deny
just check-full   # + instrumented coverage + crap metric
```

---

## Testing strategy

**Correctness is the non-negotiable constraint.** Every codec change must pass a round-trip property test: `decompress(compress(x)) == x` for all valid inputs, byte-for-byte.

- Unit tests: per module, in an inline `#[cfg(test)]` block.
- Property tests (`proptest`): for any function with a mathematical invariant (codec, checksum, tag round-trip).
- Integration tests: full-pipeline tests against a corpus of real uncompressed CinemaDNG frames. These must confirm the output opens in DaVinci Resolve/Premiere (verified manually; automate where possible).
- Round-trip harness: a dedicated test that compresses then decompresses each corpus file and asserts byte identity.

---

## Formal verification

Lean 4 and/or Alloy are in scope for partially proving core invariants — for example:

- The codec's lossless round-trip property modeled in Lean.
- The DNG container state machine modeled in Alloy to rule out tag corruption by construction.

Formal specs live in `proof/`. They are not part of the build pipeline but must stay in sync with the implementation. When modifying `codec/` or `container/`, check whether a corresponding proof needs updating.

---

## Performance

Criterion benchmarks live in `benches/`. Name them by subsystem: `codec_lossless`, `container_read`, `orchestration_parallel`.

Benchmark results are tracked via Bencher. Do not remove or rename existing benchmarks without updating the Bencher baseline — it will produce false regressions.

Target: meaningfully faster than SlimRAW on multi-core. Measure on the same footage, same hardware, note the comparison in the benchmark description.

---

## Working rules for agents

- **Correctness first.** If a correct implementation is slower, ship it and optimize later. Never trade round-trip fidelity for speed.
- **Spike the hardest risk early.** The lossless JPEG encoder for DNG is the make-or-break component. Evaluate `rawler` first; scope a custom encoder if no crate suffices.
- **Respect the roadmap.** v1 is CinemaDNG video, lossless, CLI only. Do not implement v2+ features (stills, proxies, watch folder, JPEG XL, GUI) until v1 is complete. Surface any scope creep.
- **Stay on open formats.** Never add a dependency on a proprietary camera SDK (BRAW, R3D, ARRIRAW). Open specs only.
- **No features without tests.** Property tests for anything algorithmic; integration tests for any pipeline change.
- **Don't add abstractions beyond the task.** Three similar lines beat a premature helper.
- **Keep the architecture boundaries.** Never let codec logic reach into the container layer or vice versa.
- **Open questions to resolve before implementation:** lossless JPEG crate vs custom (spike first), GUI framework (Tauri vs egui, decide when GUI work begins). Do not silently pick one — surface it.
