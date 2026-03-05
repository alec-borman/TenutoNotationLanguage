# Tenuto Reference Compiler (`tenutoc`)

> **A programming language for music.** > What HTML did for documents, what Markdown did for writing, what Mermaid did for diagrams — Tenuto does for music.

---

## The Vision

Music notation has been stuck for centuries. First, it was trapped on paper. Then it got locked inside bloated, machine‑generated XML that no human can read or write. MIDI captured performances but threw away the music's structure — its measures, key signatures, and tuplets. Modern DAWs gave us total control over audio but locked our compositions inside proprietary, unreadable black boxes. Every digital music format forced a compromise between the acoustic and the electronic.

**Tenuto v3.0.0 changes everything.**

This isn't just another file format. It's a complete reimagining of how we write, share, and understand music. It unifies the classical sheet music paradigm with modern electronic digital signal processing (DSP). It's:

* **A language** — clean, expressive, and designed for humans.
* **A compiler** — turning your ideas into sound, MIDI, and ink with mathematical precision.
* **A philosophy** — separating *what* the music is from *how* it looks or sounds.
* **A bridge** — connecting composers to producers, AI collaborators to human performers, and the past to the future.

Tenuto is to music what HTML is to the web: the foundation everyone builds on. It's what every composer and producer who ever dreamed of writing music as code has been waiting for.

---

## Why Tenuto?

Let's look at the options we've had until now:

* **MusicXML** is a visual straitjacket. It cares about where ink sits on a page, not the music itself. It's incredibly verbose — a single measure of polyphony can take over a thousand tokens. You can't edit it by hand. You can't put it in version control and see what changed.
* **MIDI** is a mechanical ghost. It records button presses and slider movements, but it forgets everything about musical structure. Measures? Gone. Key signatures? Gone.
* **DAW projects** are proprietary black boxes. Open a project from ten years ago and it's gibberish. Open it in a different DAW and it's useless. Your music gets locked inside software that might not exist tomorrow.
* **ASCII tab** is readable but rhythmically ambiguous. You can see the notes, but you can't tell exactly when to play them.

Tenuto solves all of this by going back to first principles: **What does music actually need to represent?**

* Pitch — including microtones and continuous portamento glides.
* Rhythm — as exact fractions, but with millisecond-accurate micro-timing for the "pocket."
* Instrumentation — from classical violins to ADSR synth basses and granular audio samples.
* Articulation, expression, and dynamic signal routing (sidechaining).
* Structural flow — repeats, jumps, codas.

Nothing more. Nothing less.

---

## How It Works

### 1. Define Your Instruments (The Physics)

Each instrument gets its own "physics" — how it behaves, how it's notated, and how it sounds. Tenuto 3.0 natively supports standard notation, tablature, drum grids, audio samplers, and synthesizers.

```tenuto
%% Acoustic Instruments
def violin "Violin I" style=standard clef=treble
def guitar "Guitar" style=tab tuning=guitar_std

%% Electronic & Producer Instruments
def sub "808 Bass" style=synth env=@{ a: 5ms, d: 1s, s: 100%, r: 50ms } cut_group=1
def vox "Vocal Chop" style=concrete src="./vocals.wav" map=@{ a:[0.0s, 1.2s] }
def drums "Drum Kit" style=grid map=@{ k:[0,36], s:[2,38], h:[4,42] }

```

### 2. Write the Music (The Logic)

```tenuto
measure 1 {
  %% 808 Synth with a portamento glide
  sub: c2:2.glide(150ms) c3:2 |

  %% Sampler applying granular slicing (4 equal cuts)
  vox: a:2.slice(4) r:2 |
  
  %% Drums using Euclidean rhythm and micro-timing for the "pocket"
  drums: (k):3/8 s:4.pull(20ms) |
  
  %% Acoustic Piano polyphony
  piano: <[
    v1: c5:8.stacc d e f g a b c6:2.ten |
    v2: c3:4 c2:4                       |
  ]>
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

### Acoustic & Electronic Parity (v3.0)

Abstract DSP manipulations are elevated to semantic primitives. Define ADSR envelopes, trigger granular sample slices (`.slice(N)`), execute continuous 808 pitch glides (`.glide()`), and write invisible LFO automation curves for sidechain ducking—all directly alongside standard notation.

### Semantic, Not Visual

Tenuto encodes musical intent, not graphical accidents. The same code can generate MIDI, MusicXML, and eventually beautifully engraved scores, Braille music, or direct audio buffers. Change the output — keep the music.

### Sticky State (Inference)

Like a human sight‑reader, Tenuto remembers the last duration, octave, and articulation. You write less and say more:

```tenuto
c4:4 d e f g a b c5

```

The `d` knows it's a quarter note. The `c5` knows it's in octave 5. No repetition needed.

### Exact Rational Time & "The Pocket"

Triplets are exactly 1/3. No floating‑point drift. No quantization errors. But music isn't rigid—Tenuto 3.0 allows you to physically offset notes using `.pull(15ms)` or `.push(10ticks)` to create unquantized human grooves without destroying the underlying sheet music grid.

### Euclidean Rhythms

Generate complex, algorithmic syncopations instantly. `(k):3/8` mathematically distributes 3 kick drum hits as evenly as possible across an 8-step grid (the classic *tresillo* pattern), replacing tedious manual placement.

### Macros and Variables

```tenuto
var verse_velocity = 90
macro arpeggio(root) = { $root:16 e g b c5 }

piano: $arpeggio(c4).vol($verse_velocity)

```

---

## What Makes Tenuto Different

| Format | Human Readable | Musical Structure | Version Control Friendly | Compact | Electronic/DSP Native |
| --- | --- | --- | --- | --- | --- |
| MusicXML | ❌ | ✅ | ❌ | ❌ | ❌ |
| MIDI | ❌ | ❌ | ❌ | ✅ | ❌ |
| DAW Project | ❌ | ✅ | ❌ | ❌ | ✅ |
| ASCII Tab | ✅ | ❌ | ✅ | ✅ | ❌ |
| **Tenuto 3.0** | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Built for the AI Age

Large Language Models think in tokens. Every token is precious. Every bit of context matters.

**Tenuto was designed for this moment.**

Where MusicXML burns over a thousand tokens on a single measure, Tenuto uses twenty. Where other formats bury musical relationships under layers of cruft, Tenuto exposes them in clean, parseable grammar. Where AI struggles to generate coherent music in bloated formats, Tenuto gives it a native language for musical thought.

This is by design. Tenuto was built from the ground up to be:

* **Token‑efficient** — more music, less noise.
* **Semantically rich** — microtonality, sidechaining, and expressive gestures are all first‑class citizens.
* **Algorithmically friendly** — `$macro` and `$variable` systems turn musical motifs into parameterized functions.
* **Deterministically parseable** — an LL(1) grammar means no surprises, no ambiguity.

The AI revolution in music won't be built on MIDI bytes or XML bloat. It will be built on Tenuto.

---

## Current Status

**The Tenuto Language Specification v3.0.0 is officially complete.**

The reference compiler (`tenutoc`) is currently in active development to upgrade from the v2.2.0 architecture to fully support the new v3.0.0 engines (including `style=concrete`, `style=synth`, Euclidean tuplets, and continuous automation).

Currently, the `main` branch compiler can:

* Parse the v2.2+ Tenuto language.
* Generate standard MIDI files with all performance data.
* Export MusicXML 4.0 for use in MuseScore, Dorico, Sibelius.
* Validate scores in strict mode for archival quality.

---

## The Road Ahead

### Phase V: The Producer Engines (In Progress)

* Updating the AST and internal IR (Intermediate Representation) to resolve `style=synth` and `style=concrete`.
* Building the audio-buffer target for direct `.wav` export.

### Phase VI: Developer Experience

* **`tenuto-lsp`** — A Language Server for VS Code, Neovim, and more. Real‑time error checking, hover documentation, auto‑completion.
* **`tenuto-fmt`** — An opinionated code formatter that aligns bar lines and indents voices consistently.
* **`tenuto-playground`** — A web editor where anyone can write Tenuto, hear it instantly, and share it with a URL.

### Phase VII: Native Engraving — `tenuto-engrave` (Spec Complete)

**The crown jewel.** A native Rust engraving engine that renders publication‑ready sheet music directly from Tenuto source — no MusicXML required.

The **Tenuto Engraving Architecture Specification (TEAS)** is 100% complete — a 150‑page architectural blueprint that solves every major problem in music typography using ECS memory models, Cassowary constraint solvers, and Bezier routing. **`tenuto-engrave` will be the Typst of music.** ---

## Who Is Tenuto For?

**Producers** who want exact mathematical control over granular sampling and LFO sidechaining without fighting DAW GUIs.

**Composers** who want to work in text, fight fewer constraints, and version control their scores.

**AI/ML engineers** training models on a token‑efficient representation of music.

**Developers** building music applications who need a clean, parseable music format.

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
tenuto "3.0" {
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

* **Rust developers** to help with the compiler, audio backend, and engraving engine.
* **Music theorists & Producers** to ensure the language captures every nuance of both classical and electronic paradigms.
* **Tool builders** to create editor plugins, web apps, and more.

The language is stable. The vision is clear. **Now we build the ecosystem together.**

---

## Learn More

* [Full Language Specification](https://www.google.com/search?q=./docs/SPEC.md) — The complete Tenuto v3.0.0 grammar and semantics.
* [Engraving Architecture (TEAS)](https://www.google.com/search?q=./docs/TEAS.md) — The blueprint for native score rendering.
* [GitHub Discussions](https://github.com/alec-borman/TenutoNotationLanguage/discussions) — Ask questions, share ideas.

---

**Tenuto: Write music as code. Compile to everything.**

[GitHub Repository](https://github.com/alec-borman/TenutoNotationLanguage) | [MIT License](https://www.google.com/search?q=./LICENSE)
