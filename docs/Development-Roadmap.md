# 🗺️ Tenuto v3.0: Vertical Slice Development Roadmap

**Objective:** Transition the Tenuto v2.2.0 compiler into the v3.0.0 "Producer Engine" (incorporating advanced DSP, sampling, synth physics, and micro-timing) without destabilizing the core architecture.

**Methodology:** Strict **Vertical Slicing**. Instead of updating the Lexer for all features, then the Parser for all features, we will select *one single feature* and implement it completely from Lexer → AST → IR → Output. Each slice culminates in a stable, production-ready minor release.

---

## 🟢 Slice 1: Euclidean Topologies
**Target Release:** `v2.3.0`  
**Goal:** Natively support algorithmic rhythm distribution via `(Event):K/N`.

*   **The "Why":** A massive win for electronic production with minimal compiler risk. It uses existing lexer tokens `(`, `)`, `:`, and `/`.
*   **AST:** Add `Event::Euclidean { target: String, k: u64, n: u64, attrs: Vec<Attribute> }`.
*   **Parser (`chumsky`):** Branch the Tuplet parser. If the parentheses contain exactly one token with no spaces (e.g., `(k):3/8`), route to `Event::Euclidean`.
*   **IR Engine:** Implement a Bresenham line-drawing (or Bjorklund) algorithm in `process_voice_events` to mathematically calculate the $K$ hits distributed evenly across $N$ subdivisions, injecting them into the absolute timeline.
*   **Deliverable:** Writing `drm: (k):3/8` successfully generates a perfect reggaeton/tresillo MIDI beat.

## 🟡 Slice 2: Action Notation & The Spacer Token
**Target Release:** `v2.4.0`  
**Goal:** Enable pure automation curves via the Spacer (`s`) token for sidechain ducking.

*   **The "Why":** Formalizes invisible LFOs. The current parser panics if attributes are attached to rests. We need a dedicated invisible logic carrier.
*   **AST:** Add `Event::Spacer` (distinct from `Event::Rest`).
*   **Parser:** Update the event parser to separate `s` from `r`, and allow `s` to accept chained `.attributes()`.
*   **IR Engine:** Ensure `Spacer` consumes logical ticks but generates $0$ NoteOn/NoteOff events. Ensure attributes like `.cc(7,[0,127])` attached to it *are* executed.
*   **XML Exporter:** Explicitly ignore `Event::Spacer` so it doesn't draw bizarre, invisible text artifacts on the sheet music.
*   **Deliverable:** `v2: s:4.cc(11, [0, 127])` smoothly ramps a MIDI CC curve in the background without affecting the visual score.

## 🟠 Slice 3: Physical Time & Micro-Timing (`TimeVal`)
**Target Release:** `v2.5.0`  
**Goal:** Implement J Dilla-style "pocket" grooves via `.push()` and `.pull()`.

*   **The "Why":** Decouples strict visual layout time from humanized audio playback time.
*   **Lexer (`logos`):** Introduce `Token::TimeVal` to parse `150ms`, `0.5s`, and `20ticks`.
*   **AST / Parser:** Update the `Value` enum and `attribute_parser` to accept `TimeVal` arguments.
*   **IR Engine:** Add `physical_tick_offset: i64` to `AtomicEvent`. Update the dynamic parser to calculate the math: if `.pull(10ms)`, algebraically convert 10ms to absolute PPQ ticks based on the active tempo, and store it in the offset.
*   **MIDI Exporter:** Apply `physical_tick_offset` when calculating Delta Times for NoteOn/NoteOff.
*   **Deliverable:** Snare drums can be pushed milliseconds off the grid in MIDI, while the MusicXML renders perfectly quantized.

## 🔴 Slice 4: The Synth Engine (Portamento & Choke)
**Target Release:** `v2.6.0`  
**Goal:** Support continuous frequency physics (`.glide`, `.accelerate`) and monophonic sub-bass limits.

*   **The "Why":** Moves Tenuto beyond discrete piano keys into continuous electronic frequencies (808 dives).
*   **AST:** Add `style=synth`, `env=@{}`, and `cut_group` to Instrument Definitions.
*   **IR Engine (Choke):** During linearization, if a note fires on `cut_group=1`, seek backward in the track history and forcibly truncate the `gate_ticks` of the previous note to prevent overlap.
*   **IR Engine (Glide):** Evaluate `.glide(150ms)`. Look at the previous pitch, calculate the interval difference, and generate a 14-bit MIDI Pitch Bend sweep bridging the two notes over exactly 150ms.
*   **Deliverable:** Overlapping 808s automatically cut each other off, and pitch glides synthesize perfectly in MIDI.

## 🟣 Slice 5: The Concrete Engine (Granular Sampling)
**Target Release:** `v2.7.0`  
**Goal:** Elevate raw audio slicing to semantic code (`style=concrete`).

*   **The "Why":** Fulfills the Musique Concrète mandate, mapping logic to audio buffer slices.
*   **AST:** Parse `src="file.wav"` and complex mapping dictionaries (`map=@{ a:[0.0s, 1.5s] }`).
*   **IR Engine:** Implement `.slice(N)`. Mathematically divide the mapped audio bounds into $N$ equal chunks, emitting a sequence of `AtomicEvent`s that pass those audio-buffer start/stop bounds to the backend.
*   **Deliverable:** A fully semantic AST representation of chopped sample data. *(Note: Execution relies on Slice 6).*

## 🔵 Slice 6: The Delegation Backend (OSC)
**Target Release:** `v2.8.0`  
**Goal:** Bypass MIDI entirely. Drive enterprise audio engines (SuperDirt / ChucK) natively.

*   **The "Why":** MIDI 1.0 cannot physically trigger complex audio stretching or granular synthesis. We need a modern network protocol.
*   **Exporter (`tenutoc::osc`):** Add the `rosc` (Rust OSC) crate. Write an exporter that translates the `Timeline` IR into UDP OSC packets.
*   **Mapping:** Translate `.slice` bounds to `/dirt/play` `begin/end` parameters, and `physical_tick_offset` to `nudge`.
*   **Deliverable:** Running `tenutoc --target osc` instantly triggers a SuperCollider server to play high-fidelity samples and synths.

## 🚀 Slice 7: The Zero-Friction Web Runtime
**Target Release:** `v3.0.0` (The Final Milestone)  
**Goal:** Compile Tenuto to WebAssembly and execute it directly in the browser via Web Audio API.

*   **The "Why":** Solves the adoption problem. Allows millions of web developers to embed Tenuto scores natively without installing DAWs.
*   **Compilation:** Add the `wasm32-unknown-unknown` target. Expose JS bindings (`wasm-bindgen`).
*   **Audio Engine:** Write `tenutoc::webaudio`. Map `style=synth` to browser `AudioWorkletNode`s, and `style=concrete` to `AudioBufferSourceNode`s.
*   **HTML Integration:** Create the `<tenuto-score>` web component for instantaneous browser rendering and playback.
*   **Deliverable:** A static HTML page playing and rendering a full Tenuto electronic production.

---

### The Engineering Pact
By adhering strictly to these Vertical Slices, the `tenutoc` codebase will remain 100% stable. At the end of every slice, `cargo test` must be entirely green, and the compiler must successfully output a valid `.mid` or `.musicxml` file.

**We do not move to the next slice until the current one is bulletproof.**
