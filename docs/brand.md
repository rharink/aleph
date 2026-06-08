# Aleph — Brand Foundation

The single source of truth for how Aleph is positioned, named, and written. Name
and logo are locked; this document covers everything built around them. When site,
store, README, or pitch copy disagrees with this file, this file wins — or this
file is updated on purpose.

Last updated: 2026-06-08.

---

## At a glance

- **Name:** Aleph (the ℵ glyph is mark-only — see Naming).
- **Signature line:** *Smaller files. Same footage.*
- **Descriptor:** Lossless RAW compression and verified offload — bought once.
- **The spine:** **Smaller · Safe · Yours.**
- **What only Aleph can say:** Aleph gets your RAW off the card smaller, proves
  every copy is safe, and it's yours to keep.
- **Calibration:** plain over clever; verification is a quiet reason-to-believe,
  not the headline; the math identity is the name and mark only.

---

## 1. Positioning

### The one thing only Aleph can say

> Aleph gets your RAW off the card smaller, proves every copy is safe, and it's
> yours to keep.

Neither competitor can say the whole sentence. SlimRAW compresses but doesn't
offload or verify. Silverstack offloads but rents you the tool forever and isn't
built around lossless compression. The combination plus ownership is open ground.

### Positioning statement

> For **DITs and shooter-owners** who have to clear the card fast and cannot afford
> to wonder whether the footage survived, **Aleph** is **on-set offload-and-compression
> software** that shrinks CinemaDNG without altering a single sample and checks every
> copy before the card is wiped.
> Unlike **subscription DIT suites you rent forever** or **single-purpose compressors
> that leave you to handle the copy and the checks yourself**, Aleph does both in one
> fast pass, runs on the hardware you already own, and you buy it once.

### The spine

| | Promise | What backs it |
|---|---|---|
| **Smaller** | Less to store, back up, and wait on — without throwing anything away | Lossless CinemaDNG compression; multi-core speed; two drives in one pass |
| **Safe** | What comes out is what went in — and you can show it | Bit-perfect round-trip (provable, in the background); BLAKE3 on every destination; every tag, sidecar, and folder preserved |
| **Yours** | The tool, the files, and the workflow stay under your control | Buy-once perpetual license; open formats, no lock-in archive; no proprietary SDKs |

### Audience

- **Primary (current spotlight):** the person responsible for data on set — DIT,
  data wrangler, or the owner-operator who is their own DIT.
- **Durable (brand promise):** professionals who shoot open-format RAW — CinemaDNG
  today, L-mount stills and the JXL pipeline later. The promise holds for all of
  them, so the brand doesn't need a rebuild when later features ship.

### Competitive frame — a two-sided squeeze

- **Above (Silverstack & rent-forever suites):** own your data tool the way you own
  your camera — ownership, focus, speed, no subscription.
- **Beside (SlimRAW & pure compressors):** compression is only half the job. Aleph
  also offloads and verifies; the card isn't trusted until the bytes match.
- **Never compete on price.** Position on correctness, trust, and ownership.

### What Aleph is *not*

- Not "a compressor." Compression is the wedge; the position is trustworthy on-set
  data you own.
- Not a full DIT suite yet — don't claim future workflow (proxies, watch folder,
  GUI) until it ships.
- Not a math product. Provability and ℵ are texture, not the pitch.

---

## 2. Messaging architecture

### Master message

> Get your RAW off the card in one fast pass — smaller, checked on every drive, and
> exactly as it was shot. Aleph is the on-set data tool you buy once, not the one
> you rent forever.

### Pillars, expanded

**Smaller** — less footage to store, copy, and back up, with nothing discarded.
RAW is enormous; duplicate backups are a recurring cost and a daily time sink.
*Proof:* lossless CinemaDNG compression; runs across all cores so it isn't the set's
bottleneck; writes primary + backup in a single pass.

**Safe** — the footage that comes out is the footage that went in, and every copy is
verified before the card is wiped. A silently corrupted file found after the card is
cleared is the job-ending nightmare; trust *is* the DIT's job.
*Proof:* bit-perfect round-trip, not one sample changed (this is where machine-checked
verification quietly earns the claim); BLAKE3 checksum on every destination; timecode,
color matrices, lens data, EXIF/GPS, MakerNotes, WAV, LUTs, sidecars, and reel
structure preserved; only the compression tag changes.

**Yours** — you own the tool, the files stay open, nothing holds your work hostage.
Subscriptions and proprietary archive formats put your footage and your budget under
someone else's control.
*Proof:* buy-once perpetual license; standard lossless CinemaDNG out, no proprietary
container; no camera-SDK lock-in; runs on the hardware you already have.

### Objection handling

| They think… | We answer |
|---|---|
| "Compression that touches my RAW? No thanks." | It doesn't alter a single sample. The round-trip is bit-perfect and machine-checked, and Aleph verifies its output against the source before you rely on it. |
| "I already copy with Hedge/Shotput and keep RAW uncompressed." | Aleph copies and verifies the same way — and makes the footage losslessly smaller in the same pass. Fewer steps, less storage, same safety. |
| "SlimRAW compresses CinemaDNG and it's cheap." | SlimRAW stops at compression. Aleph closes the loop card → two drives → verified, in open formats. Compression is half the job. |
| "Will it open in Resolve / Premiere?" | Standard lossless CinemaDNG. It opens in the NLE you already use — no special importer, no relink pain. |
| "It's early / not usable yet." | Say so plainly: private development, shipping soon, founder pricing for early buyers. Honesty is on-brand here. |
| "If I buy now, will it die when Pro arrives?" | Buy Aleph, keep Aleph forever. Pro is for future workflow features; your license never stops working. |

### Audience angling

- **DIT / data wrangler:** lead **Safe**, then speed and folder/reel integrity.
  Native vocabulary — offload, reels, sidecars, checksums, relink.
- **Shooter-owner:** lead **Yours** and **Smaller**, jargon-light. "Your footage,"
  "off the card," "buy once."

### Claims discipline

- **"Smaller" is safe to assert** — the ~45% figure is real and demonstrated, but
  keep it tied to the demo; don't generalize it into boilerplate as a guarantee.
- **"Faster than SlimRAW" is not assertable yet.** It's a target, not a measured
  result. Until benchmarked on identical footage, frame speed as design intent
  ("won't bottleneck the set"), never as a head-to-head number.
- **No future-feature claims** (proxies, watch folder, GUI, JXL) in current copy.

---

## 3. Verbal identity

### Voice — five principles

1. **Plain over clever.** Say the thing. If a line needs a second of translation,
   cut it. ("Smaller files. Same footage." beats anything that needs decoding.)
2. **Show, don't claim.** Numbers and mechanisms over adjectives. Proof carries the
   weight; that's the whole brand.
3. **Calm and certain.** Short declaratives. No exclamation marks, no hype, no
   fear-selling. The verbal match to the monochrome mark.
4. **Respect the operator.** You're talking to a professional clearing a card at
   2am. Use their nouns. Never condescend, never over-explain the obvious.
5. **Honest about stage and scope.** Say "private development." Say what ships now
   and what doesn't. Trust is the product — never spend it on an overclaim.

### Tone range — one voice, four dials

| Context | Dial |
|---|---|
| Marketing / hero | Confident, spare, benefit-first |
| CLI & docs | Precise, instructional, zero fluff |
| Errors & verification output | Exact *and* reassuring — what happened, what's safe, what to do. An error message is a brand moment; treat it like one |
| Pricing / sales | Straight, ownership-framed, no dark patterns |

### Lexicon

**Use:** lossless · bit-perfect · byte-for-byte · verified · checked · offload ·
off the card · smaller · same footage · every copy · preserved · relink · open
formats · buy once · own · on set.

**Avoid:** revolutionary · blazing(ly) · magic · effortless · seamless ·
enterprise-grade · next-gen · AI (unless literally true) · epsilon / "lose ε" ·
"guarantee" (legalistic) · "perfect" as hype (only ever "bit-perfect") ·
"fastest" / "faster than SlimRAW" until benchmarked.

**Formulas:** no formulas in plain-audience copy. One contained technical signal —
`decompress(compress(x)) == x` — is permitted where the technical reader is the
explicit target (e.g. the developer-facing "Verify" step). One signal, not a habit.

### Signature & line system

- **Brand line (the signature):** *Smaller files. Same footage.* Protect it from
  variants — don't paraphrase.
- **Descriptor (sits by the logo):** *On-set RAW offload & lossless compression.*
- **One-liner (elevator):** *Aleph gets your RAW off the card smaller, checks every
  copy, and it's yours to keep.*
- **Pillar lines (context-rotating, never competing with the signature):**
  - Smaller — *Same footage. Smaller files. Nothing left behind.*
  - Safe — *Verified before the card leaves.*
  - Yours — *Own it like you own your camera.*

### Name-in-copy rules

- Always **Aleph** — capital A, lowercase rest. Never `ALEPH` in prose, never
  lowercase at sentence start.
- The **ℵ glyph** is mark, favicon, and occasional flourish only. Never inside a
  running sentence — it forces a decode.
- Provability is a phrase ("bit-perfect, verified"), never a formula, never the lead.

---

## 4. Naming system

Aleph produces standard **DNG** output — extensions are industry-dictated and the
filename is preserved (DNG in, DNG out, smaller). **There is no Aleph file type, and
there must never be one;** a proprietary extension would signal exactly the lock-in
the "Yours" pillar promises to avoid.

One deliberate exception: the in-browser inspector saves its demo download as
`name.aleph.dng` to tell the compressed copy apart from the original in the user's
Downloads folder. It is still a standard DNG (the extension is `.dng`), not a new
file type — the "no Aleph file type" rule holds. The pipeline (CLI) output keeps the
original filename.

### Principles

1. **Branded house, plain modifiers.** Everything is *Aleph + a plain word*. No
   standalone sub-brands, no codenames.
2. **A name must not imply lock-in.**
3. **The name carries the ownership model** — own it, subscribe to it, or rent it
   for a job.

### Tiers

| Name | What it is | Model |
|---|---|---|
| **Aleph** | The core: lossless compression + verified dual-destination offload, CLI | Perpetual — buy once |
| **Aleph Pro** | On-set workflow layer: watch folder, proxies, reports, GUI, advanced re-verify | Ongoing — subscription or paid major upgrade |
| **Aleph Project** | The full tool licensed for a single production | Time-boxed |
| **Aleph Founder** | Early-adopter cohort of the core (limited, discounted, Pro perks) | Perpetual + founder benefits |

### "v1" is a version, not a product name

- **In marketing/pricing it is just "Aleph,"** sold with a perpetual license.
- **Version numbers (v1.x) live only in release notes and changelogs.**
- **What "forever" covers** is defined by license terms ("the Aleph core feature
  set"), not by branding the product with a number. A version in the name fights the
  permanence the perpetual license promises.

### CLI verbs

- Lowercase, imperative English verb the user can guess: `aleph compress`,
  `aleph decompress`, `aleph offload`. Flags are plain `--kebab-case`
  (`--out`, `--to`, `--no-verify`).
- Reserve future verbs in the same pattern: `aleph watch`, `aleph verify`
  (re-verify before ingest), `aleph report`, `aleph proxy` (preferred over
  `transcode` — "proxy" is the on-set noun).

### Casing & the brand/binary split

- **Prose:** Aleph, Aleph Pro, Aleph Project, Aleph Founder — title case.
- **Code/commands:** the binary is lowercase `aleph` only inside terminal/code
  contexts.

---

## 5. Narrative & boilerplate

### Brand narrative — long

> RAW is the truth of a shot — and it's enormous, fragile, and expensive to keep. On
> set, the footage has to come off the card fast, copy to more than one drive, and be
> trusted completely, because a file that corrupts silently is only discovered after
> the card is wiped, when it's too late. The tools built for this either rent you the
> workflow forever or wrap your footage in formats you don't control.
>
> Aleph takes a different position. It makes CinemaDNG smaller without changing a
> single sample, checks every copy against the source before you trust it, and hands
> the footage back in the same open files your NLE already reads. Nothing is added,
> nothing is locked in, and the tool is yours — bought once, like the camera it serves.
>
> Smaller files. Same footage. Proof that it survived, and a tool you own. That's the
> whole idea.

### Brand narrative — short

> On set, RAW has to come off the card fast and be trusted completely — a file that
> corrupts unnoticed is found too late. Aleph makes CinemaDNG smaller without touching
> a sample, verifies every copy before you trust it, and keeps everything in open
> files you own. Smaller files. Same footage.

### Boilerplate library (paste-ready)

**Tagline (never paraphrase):** Smaller files. Same footage.

**Descriptor (≤10 words):** Lossless RAW compression and verified offload — bought once.

**25 words:**

> Aleph compresses CinemaDNG losslessly and offloads it to multiple drives, every copy
> verified bit-perfect. Open formats, bought once. Smaller files, same footage — a tool
> you own.

**50 words:**

> Aleph is on-set software for cinematographers and DITs that makes CinemaDNG and
> open-format RAW smaller without altering a single sample, then offloads to multiple
> drives with a BLAKE3 check on every copy. Open formats in and out, no lock-in, no
> subscription — buy it once. Smaller files. Same footage.

**100 words:**

> Aleph is on-set data software for cinematographers, photographers, and DITs. It makes
> CinemaDNG and open-format RAW meaningfully smaller without changing a single raw
> sample, then offloads to multiple destinations with a BLAKE3 checksum on every copy,
> so the card is never trusted until the bytes match. Timecode, color matrices, lens
> data, sidecars, and folder structure are preserved, and the output is standard
> lossless CinemaDNG that opens in the NLE you already use. No proprietary archive, no
> camera SDK, no subscription — you buy Aleph once and keep it. Smaller files. Same
> footage. Built first for macOS, Windows planned.

**Store / marketplace descriptor:**

> Aleph — lossless RAW compression and verified offload for the set. Smaller files,
> same footage, bought once.

**Press boilerplate ("About Aleph"):**

> Aleph is on-set data software for film and photo professionals that compresses
> open-format RAW losslessly and offloads it with verified, bit-perfect copies. Built
> on open formats with no subscription and no lock-in, Aleph is sold as a perpetual
> license and is available first for macOS.

**Elevator pitch (spoken):**

> Getting RAW off the card means copying huge files to two drives and hoping nothing
> corrupted. Aleph shrinks the footage losslessly and checks every copy before you
> wipe the card — same files, smaller, and you buy it once instead of renting another
> subscription.

### Usage discipline

- **Pre-launch,** prepend the honest qualifier ("Aleph is in private development…").
  Don't write shipped-tense claims before it ships.
- **Never** insert a "faster than SlimRAW" claim until it's benchmarked.
- **Product name is "Aleph"** (perpetual). "v1" never appears in customer copy;
  tiers are Aleph / Aleph Pro / Aleph Project / Aleph Founder.
- The number "45%" stays tied to the demo, not generalized into boilerplate.
