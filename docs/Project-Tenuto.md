# PROJECT TENUTO: THE UNIFIED THEORY OF DIGITAL MUSIC
**The 5-Year Master Plan for the "HTML of Music"**

## 1. The Thesis
For 40 years, digital music has been trapped in a fragmented, schizophrenic paradigm. If you want to *print* music, you use bloated, visual-first XML software (Sibelius, Dorico) that doesn't understand audio. If you want to *produce* music, you use binary-heavy, proprietary DAWs (Ableton, Logic) that don't understand music theory, notation, or time beyond arbitrary integer ticks. If you want to *generate* music with AI, you are forced to output messy, uneditable raw audio waveforms because no text format is dense enough for an LLM to reliably compute.

**Tenuto is the solution.** It is a mathematically perfect, token-efficient, declarative programming language that captures the ontological truth of music. It separates *Physics* (instruments, DSP, tuning) from *Logic* (rhythm, pitch, relationships). 

With infinite resources, the goal of Project Tenuto is not to build another app. The goal is to establish **the definitive, open-source computational standard for all human and machine musical expression**, obsoleting legacy formats (MIDI 1.0, MusicXML, DAW `.project` files) and becoming the native nervous system of the Web and AI.

---

## 2. The 5-Division Resource Allocation

With massive capital and specialized teams, we will scale the Tenuto architecture out of a single Rust compiler and into a ubiquitous global platform. We will divide the enterprise into five elite engineering strike teams.

### 🌐 DIVISION 1: The Web & Tooling Ecosystem (The "DX" Team)
**Goal:** Make Tenuto as easy to write, share, and embed as Markdown or HTML.
*   **The Language Server (`tenuto-lsp`):** We build an industry-leading language server for VS Code and Neovim. It features real-time rational-time debugging, autocomplete for SMuFL glyphs, and macro-expansion hovering.
*   **The `<tenuto>` Web Component:** A Wasm-compiled runtime that allows any web developer on Earth to drop `<tenuto src="song.ten" controls>` into their HTML. It renders beautiful interactive sheet music and plays flawless WebAudio natively in the browser, instantly killing the market for embedded PDFs and clunky Soundcloud players.
*   **The Global Package Manager (`tpm`):** A Cargo-like registry for music. Composers publish packages containing hyper-realistic Synth ADSRs, microtonal tuning files (`.scl`), and complex Euclidean macro algorithms. You type `import "tpm://lofi-hiphop"` and instantly inherit the exact physics of that genre.

### 🎨 DIVISION 2: `tenuto-engrave` (The "Typst of Music" Team)
**Goal:** Build the most advanced, mathematically rigorous music layout engine in history.
*   **The End of MusicXML:** We stop bridging to legacy notation software and build the ultimate pure-Rust SVG/PDF renderer, implementing the *Tenuto Engraving Architecture Specification (TEAS)*.
*   **Data-Oriented Typography:** We employ an ECS (Entity-Component-System) memory model paired with a **Cassowary Linear Constraint Solver** to perfectly justify measures horizontally.
*   **SIMD-Accelerated Skylines:** We utilize hardware-accelerated vectors to calculate collision bounding boxes for lyrics, dynamics, and slurs in nanoseconds. 
*   **Real-Time Interactive Rendering:** Using the `salsa` incremental computation framework, a composer typing in a 100-page orchestral `.ten` file sees the exact printed page update visually at 60 FPS as they type.

### 🎛️ DIVISION 3: `tenuto-studio` (The Headless DAW)
**Goal:** Translate Tenuto's advanced Producer features (`style=concrete`, `style=synth`) into a professional-grade, native digital audio workstation environment without the GUI bloat.
*   **Native DSP Graph:** We build a pure-Rust, lock-free audio graph. Instead of delegating to SuperCollider via OSC, Tenuto natively computes granular sampling (`.slice(8)`), phase-vocoder time stretching (`.stretch`), and continuous 808 portamento (`.glide`).
*   **The VST3 / CLAP Wrapper:** Tenuto becomes a plugin host. You write `def lead style=vst src="Serum.vst3"`, and the Tenuto compiler maps your dot-chained attributes (`.cc(11,[0, 127])`) directly into Serum’s automation API with sample-accurate precision.
*   **Algorithmic Live-Coding:** We build a CRDT-powered synchronization engine (`tenutod`). Multiple producers can connect to the same Tenuto daemon over a network, editing the text file in real-time, executing live sidechain ducking and Euclidean beats in a collaborative "Google Docs for DAWs" environment.

### 🤖 DIVISION 4: Tenuto Foundation AI (The Machine Learning Team)
**Goal:** Build the world’s first LLM natively pretrained on musical logic, treating Tenuto as its primary language.
*   **The Token Advantage:** Because Tenuto relies on the "Sticky State" (inheriting octaves and durations), it compresses complex polyphony by 90%. We can fit an entire symphony into a standard LLM context window.
*   **The Tenuto Transformer:** We acquire massive GPU clusters and train a foundation model not on audio waveforms, but on a massive corpus of Tenuto ASTs. The AI learns the deep mathematics of counterpoint, harmony, and rhythm.
*   **The Ultimate Copilot:** You prompt the AI: *"Write a 16-measure jazz solo over a ii-V-I progression, use ghost notes on the snare, and bend the climax note a quarter-flat."* The AI generates a flawless, token-efficient `.ten` file. You can then edit the text, recompile it, and instantly generate the sheet music, MIDI, and Audio. 

### 🏛️ DIVISION 5: Standardization & Hardware (The Consortium)
**Goal:** Make Tenuto an immutable, global standard.
*   **W3C / ISO Standardization:** We push the Tenuto specification through formal standardization bodies, establishing it as the official archival format for the Library of Congress and global music publishers.
*   **Hardware Integration:** We partner with synthesizer manufacturers (Moog, Roland) and digital sheet music tablet makers (iPad, ForScore). Hardware natively ingests `.ten` files via Bluetooth, bypassing MIDI completely. The synth perfectly executes the rational time math and microtonal bends, while the tablet natively renders the SVGs and automatically turns the pages based on the timeline IR.

---

## 3. The Endgame: A World Built on Tenuto

If this moon-shot is fully funded and executed, the landscape of music creation is fundamentally altered within 5 years.

1. **For the Composer:** You no longer fight with a mouse, wrestling with Sibelius layout bugs or Ableton automation curves. You write pure, beautiful, version-controlled logic. You push it to GitHub. It automatically runs a CI/CD pipeline, generating PDFs for the orchestra and a perfectly mixed audio mockup for the director.
2. **For the Developer:** You want procedural music in your indie video game? You don't buy massive `.wav` stems. You ship your game with a lightweight 5KB `.ten` file and the Tenuto Wasm runtime. As the player's health drops, you simply swap a variable in the Tenuto script, instantly compiling a faster, darker arrangement on the fly.
3. **For Humanity:** Music is no longer locked in proprietary, decaying software formats. A `.ten` file written today is grounded in pure acoustic physics and UTF-8 text. It will be readable, playable, and perfect a thousand years from now.
