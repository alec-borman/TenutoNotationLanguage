# Tenuto Reference Compiler (`tenutoc`)

> **A programming language for music.** > What HTML did for documents, what Markdown did for writing, what Mermaid did for diagrams — Tenuto does for music.

---

## The Vision

Music notation has been stuck for centuries. First, it was trapped on paper. Then it got locked inside bloated, machine‑generated XML that no human can read or write. MIDI captured performances but threw away the music's structure. Modern DAWs gave us total control over audio but locked our compositions inside proprietary, unreadable black boxes.

And now, AI music generation has arrived as a "slot machine." You prompt a model, and it spits out a flattened, uneditable `.wav` file. If the vibe is perfect but the snare is slightly too loud, or you want to change a specific chord inversion in the chorus? You can't. You have to reroll the prompt and pray the black box gives you something usable.

**Tenuto v3.0.0 changes everything.**

Tenuto is a deterministic, declarative language that unifies classical sheet music with modern electronic digital signal processing (DSP). It is designed to be the ultimate bridge between human composers, software compilers, and Artificial Intelligence.

## The Holy Grail of AI Music

Large Language Models (LLMs) think in tokens. Every token is precious.

Where MusicXML burns over a thousand tokens on a single measure of music, Tenuto uses twenty. Because Tenuto is incredibly token-efficient and semantically dense, **an LLM can hold an entire multi-track song's logic in its working memory without losing the plot.**

Tenuto turns AI from a random audio generator into a deterministic, collaborative studio engineer. Imagine the workflow:

1. **The Prompt:** You ask an LLM for a funky, Daft Punk-style track.
2. **The Generation:** The AI writes Tenuto code. It sets up a 110 BPM pocket groove, defines a `style=synth` sub-bass with a heavy sidechain mapped to the kick, and uses Euclidean tuplets for a syncopated hi-hat pattern.
3. **The Playback:** You compile it instantly to hear the track.
4. **The Iteration:** You tell the AI, *"The groove is too stiff, and the bass needs more glide."*
5. **The Edit:** The AI doesn't hallucinate a new audio file. It simply edits the code—adding a global `swing: 66`, dropping a `.pull(15ms)` on the snare to drag it behind the beat, and adding `.glide(100ms)` to the bass notes.

### Hybrid Generative Vocals (The Skeleton and the Flesh)

Pure generative AI is brilliant at synthesizing the *timbre* of a human voice, but terrible at placing it exactly where you want it. Tenuto's extension architecture solves this.

You write the exact melody, rhythm, and lyrics in Tenuto. A compiler plugin sends this mathematically perfect skeleton (pitch, duration, and phonetic mapping) to a generative vocal AI. The AI renders a hyper-realistic vocal performance, and Tenuto automatically imports it back into your project as a perfectly synchronized audio slice. **You get the realism of generative AI with the surgical precision of a text-based DAW.**

---

## Use Cases

Tenuto is built for anyone who believes music should be readable, editable, and future-proof.

### 🤖 AI-Assisted Composition

For AI developers and prompt engineers, Tenuto is the missing native language for music generation. It provides a token-efficient, LL(1) parseable grammar that allows models to generate, analyze, iteratively edit, and invoke external audio-generation plugins for multi-track music deterministically.

### 🎛️ Electronic Producers & Beatmakers

Write Euclidean rhythms (`(k):3/8`), trigger granular sample slices (`.slice(4)`), execute continuous 808 pitch glides (`.glide()`), and write invisible LFO automation curves for sidechain ducking—all directly from a text editor, without fighting a DAW piano roll.

### 🎼 Classical Composers & Engravers

Write your symphonies in plain text. Tenuto encodes musical intent natively, supporting microtonality, tuplets, polyphonic voices, and structural repeats. Compile your code directly into MusicXML for use in Dorico/Sibelius, or output pristine MIDI.

### 👩‍🏫 Educators & Music Theorists

Teach music theory using clean, readable code. Tenuto strips away graphical accidents and proprietary software, leaving only the pure logic of the music. It is perfect for algorithmic analysis and generating educational worksheets.

### 💾 Archivists

Proprietary DAW project files from ten years ago are often unopenable today. Tenuto is plain text. A century from now, musicians will be able to open a Tenuto file and see exactly what you intended to write.

---

## How It Works

### 1. Define Your Instruments (The Physics)

Each instrument gets its own "physics"—how it behaves, how it's notated, and how it sounds.

```tenuto
%% Acoustic Instruments
def violin "Violin I" style=standard clef=treble

%% Electronic & Producer Instruments
def sub "808 Bass" style=synth env=@{ a: 5ms, d: 1s, s: 100%, r: 50ms } cut_group=1
def drums "Drum Kit" style=grid map=@{ k:[0,36], s:[2,38], h:[4,42] }

%% AI Generative Plugin
def lead_vox "Lead Singer" style=concrete src="plugin://ai-vocal-gen"

```

### 2. Write the Music (The Logic)

```tenuto
measure 1 {
  %% 808 Synth with a portamento glide
  sub: c2:2.glide(150ms) c3:2 |
  
  %% Drums using Euclidean rhythm and micro-timing for the "pocket"
  drums: (k):3/8 s:4.pull(20ms) |
  
  %% Tenuto maps the lyrics and exact timing to the AI vocal generator
  lead_vox: c4:4 d e f |
  lead_vox.lyric: "Do- ing it right"
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

## What Makes Tenuto Different

| Format | Editable by Humans | AI Context-Friendly | Semantic Structure | Electronic/DSP Native | Future-Proof |
| --- | --- | --- | --- | --- | --- |
| MusicXML | ❌ | ❌ | ✅ | ❌ | ✅ |
| MIDI | ❌ | ❌ | ❌ | ❌ | ✅ |
| DAW Project | ❌ | ❌ | ✅ | ✅ | ❌ |
| Audio (.wav) | ❌ | ❌ | ❌ | ✅ | ✅ |
| **Tenuto 3.0** | ✅ | ✅ | ✅ | ✅ | ✅ |

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
* Building the plugin architecture for external AI synthesis pipelines.
* Building the audio-buffer target for direct `.wav` export.

### Phase VI: Developer Experience

* **`tenuto-lsp`** — A Language Server for VS Code, Neovim, and more. Real‑time error checking, hover documentation, auto‑completion.
* **`tenuto-fmt`** — An opinionated code formatter that aligns bar lines and indents voices consistently.
* **`tenuto-playground`** — A web editor where anyone can write Tenuto, hear it instantly, and edit collaboratively with an AI assistant.

### Phase VII: Native Engraving — `tenuto-engrave` (Spec Complete)

**The crown jewel.** A native Rust engraving engine that renders publication‑ready sheet music directly from Tenuto source — no MusicXML required. Built on ECS memory models and Cassowary constraint solvers, **`tenuto-engrave` will be the Typst of music.**

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
* **AI / ML Researchers** to fine-tune open-source models on Tenuto syntax and build vocal synthesis plugins.
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
