# Tenuto Reference Compiler (`tenutoc`)

> **A programming language for music.**  
> What HTML did for documents, what Markdown did for writing, what Mermaid did for diagrams — Tenuto does for music.

![Version](https://img.shields.io/badge/version-2.2.0-green)
![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-MIT-blue)
![Language](https://img.shields.io/badge/language-Rust-orange)

---

## The Vision

Music notation has been stuck for centuries. First it was trapped on paper. Then it got locked inside bloated, machine‑generated XML that no human can read or write. MIDI captured performances but threw away the music's structure — its measures, key signatures, tuplets, all of it. Every digital music format forced a compromise.

**Tenuto changes everything.**

This isn't just another file format. It's a complete reimagining of how we write, share, and understand music. It's:

- **A language** — clean, expressive, and designed for humans.
- **A compiler** — turning your ideas into sound and ink with mathematical precision.
- **A philosophy** — separating *what* the music is from *how* it looks or sounds.
- **A bridge** — connecting composers to AI collaborators, and the past to the future.

Tenuto is to music what HTML is to the web: the foundation everyone builds on. It's what every composer who ever dreamed of writing music as code has been waiting for.

**And it exists now. It's built. It works. It's complete.**

---

## Why Tenuto?

Let's look at the options we've had until now:

- **MusicXML** is a visual straitjacket. It cares about where ink sits on a page, not the music itself. It's incredibly verbose — a single measure of polyphony can take over a thousand tokens. You can't edit it by hand. You can't put it in version control and see what changed. It's a format for machines, by machines.

- **MIDI** is a mechanical ghost. It records button presses and slider movements, but it forgets everything about musical structure. Measures? Gone. Key signatures? Gone. Tuplets? Gone. It's a recording of a performance, not a representation of the music.

- **DAW projects** are proprietary black boxes. Open a project from ten years ago and it's gibberish. Open it in a different DAW and it's useless. Your music gets locked inside software that might not exist tomorrow.

- **ASCII tab** is readable but rhythmically ambiguous. You can see the notes, but you can't tell exactly when to play them.

Tenuto solves all of this by going back to first principles: **What does music actually need to represent?**

- Pitch — including microtones.
- Rhythm — as exact fractions, not floating‑point approximations.
- Articulation and expression.
- Instrument definitions and capabilities.
- Structural flow — repeats, jumps, codas.
- Lyrics and text.

Nothing more. Nothing less.

---

## How It Works

### 1. Define Your Instruments

```tenuto
def violin "Violin I" style=standard clef=treble
def cello "Cello" style=standard clef=bass
def guitar "Guitar" style=tab tuning=guitar_std
def drums "Drum Kit" style=grid map=@{ k:[0,36], s:[2,38], h:[4,42] }
```

Each instrument gets its own "physics" — how it behaves, how it's notated, how it sounds.

### 2. Write the Music

```tenuto
measure 1 {
  %% Right hand melody with articulation
  piano: <[
    v1: c5:8.stacc d e f g a b c6:2.ten |
    v2: c3:4 c2:4                        |
  ]>
  
  %% Guitar tab with a bend
  guitar: 10-2:4.bu(full) 10-2.bd(0) |
  
  %% Drums with a roll
  drums: k:4 s:8.roll(3) c:1 |
}
```

### 3. Compile to Anything

```bash
# Get MIDI for playback
tenutoc --input score.ten --output score.mid

# Get sheet music for engraving (MusicXML)
tenutoc --input score.ten --output score.musicxml

# Validate with strict mode for archival quality
tenutoc --input score.ten --strict
```

---

## Key Features

### Semantic, Not Visual
Tenuto encodes musical intent, not graphical accidents. The same code can generate MIDI, MusicXML, and eventually beautifully engraved scores, Braille music, or interactive web displays. Change the output — keep the music.

### Sticky State (Inference)
Like a human sight‑reader, Tenuto remembers the last duration, octave, and articulation. You write less and say more:

```tenuto
c4:4 d e f g a b c5
```

The `d` knows it's a quarter note. The `c5` knows it's in octave 5. No repetition needed.

### Exact Rational Time
Triplets are exactly 1/3, not 0.33333334. No floating‑point drift. No quantization errors. Perfect mathematical time.

### Native Microtonality
```tenuto
c4qs     %% Quarter-sharp C
f4qf     %% Quarter-flat F
c4+25    %% C plus 25 cents
```

### Macros and Variables
```tenuto
var verse_velocity = 90
macro arpeggio(root) = { $root:16 e g b c5 }

piano: $arpeggio(c4).vol($verse_velocity)
```

### Polyphonic Voices
```tenuto
piano: <[
  v1: c5:4.ten e5 g5 c6:2 |  %% Melody (stems up)
  v2: c3:2 g3:2           |  %% Bass (stems down)
]>
```

---

## What Makes Tenuto Different

| Format        | Human Readable | Musical Structure | Version Control Friendly | Compact |
|---------------|----------------|-------------------|--------------------------|---------|
| MusicXML      | ❌             | ✅                | ❌                       | ❌      |
| MIDI          | ❌             | ❌                | ❌                       | ✅      |
| DAW Project   | ❌             | ✅                | ❌                       | ❌      |
| ASCII Tab     | ✅             | ❌                | ✅                       | ✅      |
| **Tenuto**    | ✅             | ✅                | ✅                       | ✅      |

---

## Built for the AI Age

Large Language Models think in tokens. Every token is precious. Every bit of context matters.

**Tenuto was designed for this moment.**

Where MusicXML burns over a thousand tokens on a single measure, Tenuto uses twenty. Where other formats bury musical relationships under layers of cruft, Tenuto exposes them in clean, parseable grammar. Where AI struggles to generate coherent music in bloated formats, Tenuto gives it a native language for musical thought.

This is by design. Tenuto was built from the ground up to be:

- **Token‑efficient** — more music, less noise.
- **Semantically rich** — microtonality, performance techniques, expressive gestures are all first‑class citizens.
- **Algorithmically friendly** — `$macro` and `$variable` systems turn musical motifs into parameterized functions.
- **Deterministically parseable** — an LL(1) grammar means no surprises, no ambiguity.

The AI revolution in music won't be built on MIDI bytes or XML bloat. It will be built on Tenuto.

---

## Current Status

**Tenuto v2.2.0 is complete and working.**

The compiler can:
- Parse the full Tenuto language.
- Generate standard MIDI files with all performance data.
- Export MusicXML 4.0 for use in MuseScore, Dorico, Sibelius.
- Validate scores in strict mode for archival quality.

---

## The Road Ahead

### Phase V: Developer Experience (In Progress)
- **`tenuto-lsp`** — A Language Server for VS Code, Neovim, and more. Real‑time error checking, hover documentation, auto‑completion.
- **`tenuto-fmt`** — An opinionated code formatter that aligns bar lines and indents voices consistently.
- **`tenuto-playground`** — A web editor where anyone can write Tenuto, hear it instantly, and share it with a URL.

### Phase VI: Real-Time Collaboration (Planned)
- **`tenutod`** — A daemon for collaborative editing, using CRDTs to keep everyone in sync.
- **Live coding** — Connect Tenuto to Sonic Pi, TidalCycles, or custom environments. Change the music while it plays.

### Phase VII: Native Engraving — `tenuto-engrave` (Spec Complete)
**The crown jewel.** A native Rust engraving engine that renders publication‑ready sheet music directly from Tenuto source — no MusicXML required.

The **Tenuto Engraving Architecture Specification (TEAS)** is 100% complete — a 150‑page architectural blueprint that solves every major problem in music typography:

- **ECS Memory Model** — Flat, cache‑local generational arenas for sub‑millisecond access to millions of events.
- **Cassowary Constraint Solver** — Horizontal spacing treated as linear inequalities, perfect measure justification.
- **SIMD‑Accelerated Skylines** — Quantized 1D arrays for hardware‑accelerated collision detection.
- **Bezier Routing** — Collision‑avoidant curves that gracefully split across system and page breaks.
- **Complete Notation Coverage** — From piano polyphony to mensural ligatures, aleatoric clusters to figured bass.
- **Accessibility by Design** — Native WAI‑ARIA semantic SVG output, direct Braille Music (BRF) generation.
- **Incremental Computation** — Built on `salsa` to memoize the layout graph. Change one note, re‑render only that measure in under 50 milliseconds, even for a 100‑page score.

**`tenuto-engrave` will be the Typst of music.** The first modern, programmable, mathematically rigorous engraving engine. When it's complete, Tenuto will be a closed loop: write → hear → see, all from the same source, all deterministic, all beautiful.

---

## Who Is Tenuto For?

**Composers** who want to work in text, fight fewer GUIs, and version control their scores.

**Educators** who teach music theory and composition with code.

**Developers** building music applications who need a clean, parseable music format.

**Researchers** analyzing musical structure algorithmically.

**AI/ML engineers** training models on a token‑efficient representation of music.

**Publishers** who want scores in a format that will outlive any software.

**The Future** — A century from now, musicians will open Tenuto files and see exactly what you intended, because Tenuto encodes *intent*, not the quirks of some long‑dead software.

---

## Getting Started

### Installation

```bash
git clone https://github.com/alec-borman/TenutoNotationLanguage.git
cd TenutoNotationLanguage
cargo build --release
```

### Write Your First Score

Create `hello.ten`:

```tenuto
tenuto "2.2" {
  meta @{ title: "Hello World", tempo: 120, time: "4/4", key: "C" }
  def piano "Piano" style=standard
  
  measure 1 {
    piano: c4:4 d e f g a b c5:1 |
  }
}
```

### Compile and Listen

```bash
tenutoc --input hello.ten --output hello.mid
# Open hello.mid in any music player or DAW
```

---

## Join Us

This is bigger than one person. Tenuto is an open standard, an open source compiler, and an open invitation.

We need:

- **Rust developers** to help with the compiler and engraving engine.
- **Music theorists** to ensure the language captures every nuance.
- **Composers** to write music in Tenuto and share their work.
- **Educators** to build teaching materials.
- **Tool builders** to create editor plugins, web apps, and more.

The language is stable. The compiler works. The vision is clear.

**Now we build the ecosystem together.**

---

## Learn More

- [Full Language Specification](./docs/SPEC.md) — The complete Tenuto grammar and semantics.
- [Engraving Architecture (TEAS)](./docs/TEAS.md) — The blueprint for native score rendering.
- [GitHub Discussions](https://github.com/alec-borman/TenutoNotationLanguage/discussions) — Ask questions, share ideas.

---

**Tenuto: Write music as code. Compile to everything.**

[GitHub Repository](https://github.com/alec-borman/TenutoNotationLanguage) | [MIT License](./LICENSE)
