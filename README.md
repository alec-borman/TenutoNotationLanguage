# Tenuto

**The Universal Language for Music and Audio**

Tenuto is a declarative, domain‑specific language (DSL) that unifies classical music notation and modern digital signal processing (DSP) into a single, token‑efficient text format. Built on a mathematically rigorous architecture, Tenuto acts as a universal logic layer: you write the musical intent once, and the compiler can render it as sheet music, MIDI, real‑time synthesis instructions, or even native browser audio.

---

## The Problem Tenuto Solves

Today’s digital music landscape is fractured:

- **DAWs are closed ecosystems.** Project files are proprietary binary blobs that suffer from bit rot and vendor lock‑in. A session saved today may be unreadable in ten years.
- **MusicXML is bloated.** Classical notation formats are machine‑generated, visually bound, and incredibly verbose—a single measure can consume over a thousand tokens, making them hostile to version control and prohibitively expensive for AI models to process.
- **MIDI lacks structure.** MIDI captures performance data but discards semantic intent, loops, key signatures, and tuplets. It’s a recording of button presses, not a representation of music.
- **AI music is static.** Generative models output flattened audio files. If a single note is wrong, the entire file must be regenerated—there is no editable source.

Tenuto solves these problems by providing a **semantic, archival‑safe, and editable** standard. It separates *what* the music is (the intent) from *how* it sounds (the execution), creating a single source of truth that can be compiled to any format.

---

## Core Architecture & Features

Tenuto is not just another text‑to‑MIDI converter. The `tenutoc` compiler acts as an **asset linker and decompression engine**, reading a lightweight textual blueprint and dynamically orchestrating external assets (audio samples, tuning maps, AI generative URIs, and DSP scripts).

### 1. Token‑Efficient & AI‑Native (Sticky State)

Tenuto uses semantic inference to eliminate redundancy. A line like `c4:4 d e f` automatically applies the duration (`:4`) and octave (`4`) to the subsequent notes. Where a single measure of MusicXML might consume 1,500 tokens in an LLM context window, Tenuto requires only 20. This allows AI models to natively read, write, and iteratively edit multi‑track symphonies within their working memory.

### 2. Rational Temporal Execution

Most sequencers suffer from floating‑point quantization drift. Tenuto’s Intermediate Representation (IR) calculates time using pure rational arithmetic (fractions), guaranteeing that complex tuplets and phase alignments remain mathematically perfect across thousands of measures.

### 3. DSP Delegation & Live Sync

Tenuto does not reinvent audio synthesis—it orchestrates it. Code can be compiled to Open Sound Control (OSC) bundles, triggering high‑performance engines like SuperCollider, ChucK, or Ableton Live. The `tenutod` runtime daemon integrates **Ableton Link**, allowing Tenuto scripts (and AI‑generated injections) to sync perfectly with shared network tempos during live performances.

### 4. Zero‑Friction Web Runtime (Wasm)

Tenuto is built for the web. The parser compiles to WebAssembly (`wasm32-unknown-unknown`), mapping its execution graph directly to the browser’s native Web Audio API. This means you can embed procedural, interactive music in any web application with zero configuration—no plugins, no servers.

```html
<tenuto-score src="./soundtrack.ten" controls autoplay loop>
    Your browser does not support the Tenuto Web Runtime.
</tenuto-score>
```

---

## How It Works: The Pipeline

The `tenutoc` compiler transforms a Tenuto source file into absolute‑time instructions through a six‑stage pipeline:

```mermaid
graph TD
    subgraph Workspace [1. Authorship & Assets]
        Composer[Human / AI Composer] -->|Writes Logic & Pointers| Source[(score.ten)]
        Source -.->|src: amen.wav| Samples[(Audio Samples .wav)]
        Source -.->|src: plugin://ai| Plugins((AI Generative URIs))
    end

    subgraph Compiler [2. Compiler: tenutoc]
        Source --> Parse[Lexer & AST Parser]
        Parse --> AssetMgr{Asset Manager & Buffer}
        Samples & Plugins -->|Loads into RAM| AssetMgr
        AssetMgr --> Unroller[Graph Unroller]
        Unroller -->|Decompresses code to absolute ticks| IR[Absolute Timeline IR]
    end

    subgraph Execution [3. Execution Environment]
        IR --> Vis[Visual Engine] --> XML[MusicXML / PDF]
        IR --> AudioRouter{Execution Router}
        AudioRouter -->|OSC / MIDI| ProAudio[[SuperCollider / DAWs]]
        AudioRouter -->|Wasm Target| WebRuntime((Web Audio API / HTML Element))
    end
```

1. **Authorship** – A human or AI writes a `.ten` file that references external assets (samples, AI plugins, etc.).
2. **Compilation** – `tenutoc` parses the source, resolves all references, and generates an absolute‑timeline Intermediate Representation (IR) using rational arithmetic.
3. **Execution** – The IR is routed to one or more targets: visual notation (MusicXML/PDF), hardware/software synthesis (OSC/MIDI), or the browser’s Web Audio API.

---

## Writing Tenuto

A single `.ten` file can simultaneously orchestrate acoustic sheet music, granular sample playback, and remote AI vocal generation. Here’s a complete example:

```tenuto
tenuto "3.0" {
  meta @{ title: "Example", tempo: 120, time: "4/4" }

  %% Define instruments and their physics
  def piano "Piano" style=standard clef=treble
  def sub "808 Bass" style=synth env=@{ a: 5ms, d: 1s, s: 100%, r: 50ms }
  def vox "Lead AI" style=concrete src="plugin://ai-vocal-gen"

  measure 1 {
    %% Acoustic routing → sheet music / MIDI
    piano: c5:8.stacc d e f g a b c6:2.ten |

    %% DSP synthesis → OSC glides for SuperCollider
    sub: c2:2.glide(150ms) c3:2 |

    %% AI generative mapping → passes lyrics and pitch to a local WebGPU model
    vox: c4:4 d e f |
    vox.lyric: "He- llo a- gain"
  }
}
```

---

## Current Status & Roadmap

The **Tenuto v3.0 Specification** is mathematically finalized. We are actively implementing the Rust infrastructure to bring this universal conductor to life.

- **🟢 Available Now (v2.2.0 Stable):**  
  The `main` branch contains a highly optimized Rust compiler (`tenutoc`) capable of parsing Tenuto syntax, evaluating rational time grids, and exporting to standard `.mid` (MIDI) and `.musicxml` (MusicXML 4.0) for immediate use in DAWs and engraving software.  
  ```bash
  cargo install --path .
  tenutoc --input score.ten --output score.mid
  ```

- **🔵 In Development (v3.0.0 Infrastructure):**
  - **Phase V:** Upgrading the Rust AST to resolve `style=synth` and `style=concrete` URI fetching.
  - **Phase VI:** Building the `tenutod` daemon, Ableton Link API integration, and OSC emitter backend.
  - **Phase VII:** Compiling the parser to `wasm32-unknown-unknown` and building the `<tenuto-score>` Web Component.
  - **Phase VIII:** `tenuto-engrave` — a native Rust engraving engine that renders publication‑ready sheet music directly from Tenuto source, bypassing third‑party XML software.

---

## Contributing

Tenuto is an open standard and an open‑source project. We are actively seeking contributors in the following areas:

- **Rust Engineering:** AST expansion, WebAssembly compilation, and network daemons (Ableton Link / OSC).
- **AI/ML Research:** Fine‑tuning LLMs on the Tenuto syntax and integrating generative audio plugin endpoints.
- **Audio DSP:** Refining the SuperDirt and ChucK mapping protocols.

- [Read the Full v3.0 Specification](https://github.com/alec-borman/TenutoNotationLanguage/blob/main/docs/SPEC.md)
- [Join the Discussions](https://github.com/alec-borman/TenutoNotationLanguage/discussions)
- [Report an Issue](https://github.com/alec-borman/TenutoNotationLanguage/issues)

**License:** MIT

---

*Tenuto: Write music as code. Compile to everything.*
