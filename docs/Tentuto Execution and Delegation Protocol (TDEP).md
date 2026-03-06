## Tenuto Execution & Delegation Protocol (TEDP)

**Edition:** 1.0.0
**Status:** Final
**Prepared by:** Alec Borman - The Tenuto Working Group

---

### Foreword

The Tenuto Music Representation Language (T-MRL v3.0.0) is fundamentally abstract; it calculates an Intermediate Representation (IR) consisting of absolute rational time grids and semantic logic. To produce physical sound or visual notation, this abstract data must be delegated to concrete execution environments.

This document establishes the **Tenuto Execution and Delegation Protocol (TEDP)**. It defines the normative networking schemas, memory bindings, and synchronization protocols required for Tenuto to interoperate seamlessly with open-source digital signal processing (DSP) environments (SuperCollider, ChucK), network synchronization standards (Ableton Link), native browser APIs (Web Audio), and legacy engraving formats (MusicXML, MIDI).

---

### 1. Scope

This specification standardizes the outbound delegation layer of the Tenuto ecosystem. It mandates:

1. **Networked DSP Orchestration:** Open Sound Control (OSC) payload schemas and look-ahead scheduling for external synthesis engines.
2. **Phase Synchronization:** Integration with the Ableton Link protocol to align Tenuto's rational timeline with external network tempos.
3. **Web-Native Execution:** WebAssembly (Wasm) bindings and HTML Custom Element instantiation for browser-based playback.
4. **Static Interchange:** Deterministic translation of the Tenuto IR into standard MusicXML 4.0 and MIDI 1.0/2.0 files.

---

### 2. Networked DSP Orchestration (OSC Delegation)

When a Tenuto script instantiates an instrument with `style=synth`, `style=concrete`, or `style=chuck`, the `tenutod` runtime **SHALL NOT** attempt to synthesize the audio internally. It **MUST** act as a master sequencer, delegating execution instructions to an external DSP daemon via Open Sound Control (OSC 1.0) over UDP or TCP.

#### 2.1 Deterministic Look-Ahead Scheduling

To prevent network jitter from corrupting temporal accuracy, the runtime **MUST NOT** send OSC messages exactly when they are meant to be heard. All outgoing OSC bundles **SHALL** include a Network Time Protocol (NTP) timestamp representing an absolute future execution time (the "Look-Ahead Horizon").

The receiving DSP engine (e.g., SuperCollider) caches the bundle and executes it with sample-accuracy at the designated physical microsecond.

#### 2.2 Standard Address Patterns

Compliant runtimes **SHOULD** utilize the following normative OSC address schema for interoperability:

* `/tenuto/play`: Triggers an acoustic or synthetic event.
* *Arguments:* `[string: voice_id, float: freq_hz, float: duration_ms, float: velocity]`


* `/tenuto/param`: Automates continuous control data (e.g., filter sweeps, macros).
* *Arguments:* `[string: voice_id, string: param_name, float: value, float: glide_ms]`


* `/tenuto/spawn`: Instantiates a new parallel execution thread (e.g., adding a ChucK shred).
* *Arguments:* `[string: script_uri]`



---

### 3. Temporal Synchronization (Ableton Link)

For live algorithmic performances (Algoraves) and hybrid studio environments, Tenuto **MUST** support dynamic tempo integration without compromising its internal mathematical exactness.

#### 3.1 The Phase-Locked Rational Grid

When the `tenutod` daemon joins an Ableton Link session, it **SHALL** decouple its internal logical tempo from the physical clock.

1. The internal AST evaluates rhythms as pure rational fractions ($P/Q$).
2. The Link Protocol dictates the absolute microsecond duration of a system beat.
3. The runtime performs Just-In-Time (JIT) multiplication of the rational fraction against the Link microsecond phase to determine physical execution.

#### 3.2 Generative Injection (Beat-Matched Evaluation)

If an LLM or live-coder injects new Tenuto source code during active playback, the runtime **MUST** wait for the next mathematical downbeat (defined by the Link phase) before seamlessly appending the compiled IR to the execution queue. This guarantees that code changes never result in acoustic stuttering or phase misalignment.

---

### 4. Browser-Native Execution (Web Runtime)

To guarantee zero-friction adoption, the TEDP defines normative behaviors for executing Tenuto entirely client-side within modern web browsers, eliminating the need for local DAWs or OSC network daemons.

#### 4.1 WebAssembly (Wasm) Compilation

The core Tenuto parser and IR unroller **MUST** support compilation to `wasm32-unknown-unknown`. This allows the exact deterministic algorithms used in the desktop CLI to execute securely within a browser's V8 or SpiderMonkey JavaScript engine.

#### 4.2 Web Audio API Bindings

When executing via Wasm, the runtime **SHALL** fallback to the browser's native Web Audio API.

* AST events mapped to `style=synth` **MUST** be translated into `AudioWorkletNode` commands for sample-accurate scheduling.
* AST events mapped to `style=concrete` **MUST** be fetched via HTTP/WSS, decoded into `AudioBuffer`, and scheduled via `AudioBufferSourceNode.start(when)`.

#### 4.3 The HTML Custom Element

Compliant web runtimes **SHOULD** register the `<tenuto-score>` Custom Element to allow declarative instantiation by web developers. The element **SHALL** handle its own WebAudio context lifecycle and Wasm instantiation.

```html
<tenuto-score src="generative_theme.ten" controls="true" autoplay="false">
    </tenuto-score>

```

---

### 5. Static Interchange & Engraving (MusicXML / MIDI)

While Tenuto is designed to replace legacy formats, the TEDP mandates strict translation protocols for backwards compatibility with historical engraving software (Dorico, Sibelius) and hardware synthesizers.

#### 5.1 MusicXML 4.0 Extrapolation

When exporting to `.musicxml`, the TEDP engine **MUST** strip all micro-timing variations (`.push`, `.pull`) and output only the idealized logical rational grid. Tenuto semantic modifiers (e.g., `.stacc`, `.marc`) **SHALL** be translated into their corresponding `<articulations>` tags rather than physical duration truncation.

#### 5.2 MIDI 2.0 High-Resolution Export

When exporting to `.mid`, the engine **MUST** utilize the highest available PPQ (Pulses Per Quarter Note) resolution of the target standard. For MIDI 2.0, rational time fractions **SHALL** be calculated directly against the expanded tick resolution, and semantic dynamic markings (e.g., `ppp`, `ff`) must be mapped logarithmically to MIDI velocity vectors.
