**Tenuto Studio Architecture (TSA)**
**Edition:** 1.0.0
**Status:** Final
**Prepared by:** Alec Borman - The Tenuto Working Group

---

### Foreword

While the Tenuto Music Representation Language (T-MRL v3.0.0) defines the absolute mathematical truth, grammatical syntax, and rational temporal execution of musical logic, it intentionally omits the mechanisms of tooling, artificial intelligence integration, and legacy format ingestion. Blurring the line between a language specification and its execution environment leads to architectural bloat and non-deterministic execution.

This document establishes the **Tenuto Studio Architecture (TSA)**. It is a normative specification defining the runtime environment (`tenutod`), the bidirectional Agentic REPL loop, the acoustic static analysis linter (`tenuto-lint`), and the O(n) Semantic Decompiler (`tenuto-decompile`). The TSA guarantees that while external AI models and user workflows may be inherently non-deterministic, the core musical grid remains mathematically uncorrupted.

---

### 1. Scope

This specification defines the normative interfaces for software toolchains interacting with the Tenuto language. It covers:

1. **The Plugin Sandbox:** The strict parameter handoff and audio splicing protocols for external AI generative models (e.g., voice, synthesis).
2. **The Agentic REPL:** Structured JSON error reporting designed natively for Large Language Model (LLM) auto-correction loops.
3. **Static Analysis:** Acoustic and ergonomic linting heuristics.
4. **Semantic Decompilation:** Deterministic O(n) algorithms for reverse-engineering legacy machine formats (MIDI/MusicXML) into highly compressed Tenuto source code.

---

### 2. The Deterministic AI Contract (The Plugin Boundary)

The TSA explicitly isolates the deterministic Tenuto compiler from non-deterministic external AI models. When the `tenutod` runtime evaluates a track instantiated with `style=concrete src="plugin://..."`, it **MUST** enforce the Plugin Boundary.

#### 2.1 Parameter Handoff

The runtime **SHALL** halt execution of the specific track and serialize the targeted measure block into a strict JSON payload. The payload **MUST** contain absolute rational durations, calculated frequency targets (Hz), and any assigned textual lyrics or expression states.

```json
{
  "request_id": "req_8829a",
  "endpoint": "plugin://ai-vocal-gen",
  "events": [
    { "pitch_hz": 261.63, "duration_ms": 500, "lyric": "Sing" },
    { "pitch_hz": 293.66, "duration_ms": 500, "lyric": "this" }
  ]
}

```

#### 2.2 Splicing and Deterministic Quantization

External AI models are non-deterministic; an AI vocal generator requested to produce 1000ms of audio may return a buffer of 1042ms due to latent phonetic rendering.
To protect the core mandate of the language, the runtime **MUST NOT** allow the returned audio to shift the master rational time grid.
Upon receiving the `.wav` or raw audio buffer, the `tenutod` daemon **SHALL** execute a phase-vocoder time-stretching algorithm (e.g., WSOLA or equivalent) to forcefully quantize the audio buffer to the exact rational boundary defined by the Tenuto AST, ensuring sample-accurate synchronization.

---

### 3. The Agentic REPL (Read-Eval-Print Loop)

To facilitate seamless collaboration between human producers and LLM co-producers (Agentic IDEs), all compiler faults and static analysis warnings **MUST** be emitted in a machine-readable format. The compiler **SHALL NOT** return unstructured string traces to the standard output when in Studio Mode.

#### 3.1 Error Payload Structure

If an AI generates a mathematically invalid sequence (e.g., polyphonic voices that do not sum to the same absolute duration), the compiler returns a JSON schema allowing the LLM to autonomously parse the failure and rewrite the offending line.

```json
{
  "status": "fatal",
  "code": "E3002",
  "type": "Voice Sync Failure",
  "location": { "measure": 12, "voice": "v1" },
  "diagnostics": {
    "expected_ticks": 1920,
    "received_ticks": 1440,
    "delta": 480,
    "suggestion": "Append r:4 to v1 to balance the rational grid."
  }
}

```

---

### 4. Static Analysis & Acoustic Linting (`tenuto-lint`)

The TSA defines a non-fatal Static Analyzer designed to catch musical anti-patterns before audio rendering. A compliant linter **SHOULD** implement the following checks:

#### 4.1 Ergonomic Hand-Span Threshold (W4001)

If an acoustic instrument assigned a human physical model (e.g., `def piano`) contains a simultaneous vertical chord cluster where the intervallic distance between the lowest and highest pitch exceeds a Major 10th (16 semitones), the linter **SHALL** emit warning `W4001: Ergonomic span exceeded`. The LLM client may automatically resolve this by appending the `.arp` (arpeggio) modifier.

#### 4.2 Acoustic Mud Zone Detection (W4002)

Acoustic physics dictate that dense tertian harmony below the C3 frequency boundary creates severe harmonic interference due to overlapping lower overtones. If the linter detects closed triads below $f = 130.81$ Hz, it **SHALL** emit warning `W4002: Acoustic mud zone`.

---

### 5. Semantic Decompilation (The Reverse Inference Engine)

The Tenuto Studio Architecture includes a normative, offline Decompiler (`tenuto-decompile`) to ingest explicit, verbose machine formats (MIDI, MusicXML) and refactor them into compressed, human-readable Tenuto code. The Decompiler relies on strict $O(n)$ mathematical heuristics, **prohibiting** the use of non-deterministic AI hallucination during translation.

#### 5.1 Lexical Compression (LZ77 Macro Extraction)

The Decompiler **SHALL** utilize dictionary coding algorithms (such as LZ77) to scan the linear event stream for recurring structural arrays. When an identical sequence of musical events is detected multiple times, it is extracted, defined in the Global Symbol Table as a `$macro`, and replaced locally to optimize token density.

#### 5.2 Algorithmic Euclidean Reverse-Engineering

When evaluating percussion or grid-based MIDI tracks, the Decompiler evaluates the discrete physical hits ($K$) against the total grid slots ($N$). It runs the Bresenham line-drawing formula $E(K,N)$ in reverse. If the hit array perfectly matches the mathematical output, the explicit notes **MUST** be collapsed into Tenuto Euclidean notation (e.g., `(k):3/8`).

#### 5.3 Micro-Timing ("The Pocket") Extraction

The Decompiler calculates an idealized rational grid based on the ingested tempo map. If a note physically falls $t_{\Delta}$ milliseconds ahead of or behind the rational boundary, the engine quantizes the logical note to the nearest exact fraction and appends the absolute delta as a Tenuto physical modifier (e.g., `c4:4.push(12ms)`).

#### 5.4 Idiomatic Control Lane Decoupling

To reverse-engineer professional piano MIDI performances, the Decompiler **SHALL** detect "syncopated pedaling" (delayed CC 64 triggers occurring milliseconds after a chord strike). The engine mathematically realigns these asynchronous events into a dedicated, visually decoupled `pedal:` control lane, perfectly synchronized with the logical downbeat.

#### 5.5 Graph Folding (Structural Loop Recognition)

The Decompiler generates a cryptographic hash for the absolute payload of every measure block. When a sequential array of identical measure hashes is detected (e.g., a chorus repeated twice), the engine folds the linear timeline back into itself, wrapping the source code in standard barline repeat tokens (`|:` and `:|`).

---

### Conclusion of the Studio Architecture

By isolating the Studio Architecture from the Core Language Specification, Tenuto ensures that its foundational grammar remains immutable. The TSA provides the precise API contracts, sandboxing protocols, and static analysis tools required to safely integrate bleeding-edge Artificial Intelligence models without ever compromising the deterministic, mathematical truth of the composer's grid.
