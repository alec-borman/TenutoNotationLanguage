# Tenuto Reference Compiler (`tenutoc`)

> **A declarative, domain-specific language (DSL) unifying classical notation and electronic DSP. Tenuto does for music what Mermaid did for diagrams: transforming cumbersome DAWs and bloated XML into a token-efficient, mathematically perfect coding experience built for human composers and AI alike.**

---

## The Vision: The Universal Language of Music

Music notation has been stuck for centuries. First, it was trapped on paper. Then it got locked inside bloated, machine‑generated XML that no human can read. MIDI captured performances but threw away the music's structure. Modern DAWs gave us total control over audio but locked our compositions inside proprietary, unreadable black boxes.

And now, AI music generation has arrived as a "slot machine." You prompt a model, and it spits out a flattened, uneditable audio file. If the snare is slightly too loud, you can't edit it. You have to reroll and pray.

**Tenuto solves this by returning to first principles.** Tenuto is a clean, deterministic text language that separates *what* the music is from *how* it sounds. Because it is incredibly token-efficient and built on universal computer science paradigms (dot-chaining, JSON-like dictionaries, variables), it is the **perfect ontological bridge between human composers, digital rendering engines, and Artificial Intelligence.**

---

## 🟢 Current Status: What You Can Do *Right Now* (v2.2.0 Stable)

The current `main` branch contains the stable v2.2.0 Rust compiler (`tenutoc`). It is a fully functional, highly optimized engine for acoustic composition and traditional sequencing.

### Features Available Today:

* **Semantic Inference (Sticky State):** Write `c4:4 d e f`. The compiler remembers the quarter-note duration and the 4th octave. No XML bloat.
* **Exact Rational Time:** Triplets are exactly 1/3, not a floating-point approximation.
* **Microtonality & Advanced Polyphony:** Native support for quarter-tones, tuplets, and complex multi-voice structural routing.
* **Multiple Output Targets:** Compile a single `.ten` file into a `.mid` file for DAW playback or a `.musicxml` file for beautiful sheet music engraving in Dorico/Sibelius.

### Write Your First Score (v2.2 Syntax)

```tenuto
tenuto "2.2" {
  meta @{ title: "Hello World", tempo: 120, time: "4/4", key: "C" }
  
  def piano "Piano" style=standard clef=treble
  def drums "Drum Kit" style=grid map=@{ k:[0,36], s:[2,38], h:[4,42] }
  
  measure 1 {
    %% Acoustic Piano polyphony
    piano: <[
      v1: c5:8.stacc d e f g a b c6:2.ten |
      v2: c3:4 c2:4                       |
    ]>
    
    %% Drum mapping with exact tuplets
    drums: k:4 (h):3/8 s:4 |
  }
}

```

### Compile It Today

```bash
cargo install --path .
tenutoc --input score.ten --output score.mid
tenutoc --input score.ten --output score.musicxml

```

---

## 🔵 The Horizon: The Producer Update (v3.0.0 Spec Complete)

We have officially finalized the **Tenuto v3.0.0 Specification + Addendum A**. This massive update elevates Tenuto from a sheet music compiler into a **Universal Semantic Conductor**, unifying acoustic orchestration with live electronic digital signal processing (DSP).

*The `tenutoc` compiler is currently being actively upgraded to support these v3.0 features.*

### What v3.0 Unlocks:

1. **The Concrete & Synth Engines:** Natively define ADSR envelopes (`style=synth`) and granular audio samplers (`style=concrete`). Execute continuous 808 pitch glides (`.glide(150ms)`) and mathematically perfect time-stretching right from the code.
2. **"The Pocket" (Micro-Timing):** Offset notes in absolute physical time (`.pull(15ms)`) to create human, unquantized grooves without destroying the underlying sheet music grid.
3. **The AI-to-DSP Bridge:** Tenuto acts as the master logic brain, but delegates the heavy audio lifting. It fires Open Sound Control (OSC) bundles to **SuperCollider** or spawns **ChucK** shreds to render physical modeling in real-time.
4. **Live Algoraves (Ableton Link):** The new `tenutod` runtime daemon locks to Ableton Link. An AI or human can inject new code live, and Tenuto mathematically guarantees it executes flawlessly on the shared network downbeat.
5. **Generative AI Plugins:** Map an instrument to a URI (`src="plugin://ai-vocal-gen"`). Tenuto sends the pitch/lyrics skeleton to an AI model, receives a rendered audio buffer, and perfectly locks it into your timeline.

### A Glimpse at v3.0 Code (In Development)

```tenuto
tenuto "3.0" {
  %% Global Sidechain Ducking (Kick -> Bass)
  meta @{ sidechain: @{ source: "drums.k", target: "sub", ratio: "8:1" } }

  %% SuperCollider OSC Delegation
  def sub "808 Bass" style=synth env=@{ a: 5ms, d: 1s, s: 100%, r: 50ms } cut_group=1
  
  %% AI Generative Plugin
  def lead_vox "Lead Singer" style=concrete src="plugin://ai-vocal-gen"
  
  measure 1 {
    %% 808 Synth triggering an OSC glide in SuperCollider
    sub: c2:2.glide(150ms) c3:2 |
    
    %% Tenuto maps the lyrics and exact timing to the AI vocal generator
    lead_vox: c4:4 d e f |
    lead_vox.lyric: "Do- ing it right"
  }
}

```

---

## Why Tenuto is the Holy Grail for AI Music

Large Language Models (LLMs) think in tokens, and their context windows are precious. Where MusicXML burns over a thousand tokens on a single measure of music, Tenuto uses twenty.

Because Tenuto is incredibly token-efficient and utilizes standard programming paradigms (dot-chaining, dictionaries, macros), **an LLM can hold an entire multi-track song's logic in its working memory without losing the plot.**

It turns AI from a random audio slot machine into a deterministic, collaborative studio engineer. You ask the AI for a track, listen to the compiled result, and say, *"The groove is too stiff."* The AI doesn't hallucinate a new `.wav` file—it simply edits the code, adding a global `swing: 66` and a `.pull(15ms)` on the snare. You get the infinite creativity of AI with the surgical precision of a text-based DAW.

---

## The Roadmap

* **Phase IV (Current):** Stabilize `tenutoc` v2.2.0 MIDI/MusicXML generation.
* **Phase V (In Progress):** Upgrade the Rust AST and internal IR to resolve the v3.0 `style=synth` and `style=concrete` paradigms.
* **Phase VI:** Build the `tenutod` runtime daemon, Ableton Link integration, and OSC emitter backend for SuperCollider/ChucK.
* **Phase VII:** `tenuto-lsp` (Language Server for VS Code) and the web-based Playground.
* **Phase VIII:** `tenuto-engrave` — A native Rust engraving engine that renders publication‑ready sheet music directly from Tenuto source. **It will be the Typst of music.**

---

## Join the Ecosystem

This is an open standard and an open invitation. We need:

* **Rust Developers** to help upgrade the compiler to v3.0 and build the OSC/Ableton Link backends.
* **AI / ML Researchers** to fine-tune open-source models on Tenuto syntax.
* **Audio Engineers** to refine the SuperDirt and ChucK mapping protocols.

The language is stable. The v3.0 blueprint is drawn. **Now we build the future of music together.**

[Read the Full v3.0 Specification](https://www.google.com/search?q=./docs/SPEC.md) | [GitHub Discussions](https://github.com/alec-borman/TenutoNotationLanguage/discussions)
