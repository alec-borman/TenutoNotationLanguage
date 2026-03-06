# ISO/IEC Draft International Standard: 80235-1

## Information Technology — Tenuto Studio Architecture (TSA)

**Edition:** 1.0.0
**Status:** Final Draft International Standard (FDIS)
**Prepared by:** Technical Committee on Music & Audio Representation

---

### Foreword

The Tenuto Music Representation Language (T-MRL v3.0.0) strictly defines the mathematical truth, grammatical syntax, and rational temporal execution of musical logic. To preserve its deterministic core, T-MRL intentionally omits the mechanisms of external tooling, artificial intelligence integration, and legacy format ingestion.

This document establishes the **Tenuto Studio Architecture (TSA)** as a normative companion specification. It defines the runtime environment (`tenutod`), the bidirectional Agentic REPL loop, the modular static analysis framework (`tenuto-lint`), and the O(n) Semantic Decompiler (`tenuto-decompile`). The TSA guarantees that while external AI models and user workflows may be inherently non-deterministic, the core mathematical grid of the Tenuto compilation remains uncorrupted.

### 1. Scope

This specification defines the normative interfaces for software toolchains interacting with the Tenuto language. It mandates the protocols for safely executing generative AI plugins, establishing machine-readable communication loops with Large Language Models (LLMs), defining modular static analysis APIs, and executing deterministic reverse-inference from explicit machine formats (MIDI/MusicXML).

---

### 2. The Deterministic AI Contract (The Plugin Sandbox)

The TSA strictly isolates the deterministic Tenuto compiler from non-deterministic external AI generative models. When the `tenutod` runtime evaluates an Abstract Syntax Tree (AST) node instantiated with `style=concrete` referencing a `plugin://` URI, it **MUST** enforce the Plugin Sandbox.

#### 2.1 Parameter Handoff

The runtime **SHALL** halt execution of the target track and serialize the contextual measure block into a strict JSON payload. The payload **MUST** supply the external AI model with absolute rational durations, calculated frequency targets (Hz), and associated textual metadata (e.g., lyrics, expression states).

```json
{
  "request_id": "req_8829a",
  "endpoint": "plugin://ai-vocal-gen",
  "context": {
    "tempo_bpm": 120,
    "time_signature": "4/4"
  },
  "events": [
    { "pitch_hz": 261.63, "duration_ms": 500, "lyric": "Sing" },
    { "pitch_hz": 293.66, "duration_ms": 500, "lyric": "this" }
  ]
}

```

#### 2.2 Splicing and Deterministic Quantization

External generative AI models are inherently non-deterministic; an AI requested to produce 1000ms of audio may return a buffer of 1042ms due to latent phonetic rendering.
To protect the integrity of the compilation, the runtime **MUST NOT** allow returned audio to shift or corrupt the master rational time grid. Upon receiving the raw audio buffer, the `tenutod` daemon **SHALL** execute a phase-vocoder time-stretching algorithm to forcefully quantize the audio buffer to the exact rational boundaries defined by the Tenuto AST.

---

### 3. The Agentic REPL (Read-Eval-Print Loop)

To facilitate seamless collaboration between human producers and LLM co-producers, all compiler faults **MUST** be emitted in a structured, machine-readable format. The compiler **SHALL NOT** return unstructured string traces to the standard output when operating in Studio Mode.

#### 3.1 Error Payload Structure

If a generation yields a mathematically invalid sequence (e.g., polyphonic voices that do not sum to the identical duration), the compiler returns a JSON schema allowing the LLM to autonomously parse the failure, reason about the mathematical discrepancy, and inject the corrected Tenuto string without human intervention.

```json
{
  "status": "fatal",
  "code": "E3002",
  "type": "Voice Sync Failure",
  "location": { "measure": 12, "voice": "v1" },
  "diagnostics": {
    "expected_ticks": 1920,
    "received_ticks": 1440,
    "delta_ticks": 480,
    "suggestion": "Append r:4 to v1 to balance the rational grid."
  }
}

```

---

### 4. Static Analysis Protocol (`tenuto-lint`)

The TSA defines a non-fatal Static Analyzer designed to evaluate the AST against external heuristic profiles. To prevent architectural scope creep, the core `tenuto-lint` engine **SHALL NOT** contain any hardcoded rules regarding music theory, acoustic physics, or instrument ergonomics. It functions strictly as a rule-evaluation engine and message broker.

#### 4.1 The Plugin Architecture

The linter **MUST** operate via a plugin architecture. Heuristics are defined in external, community-authored packages (e.g., `@tenuto/rules-piano`, `@tenuto/rules-counterpoint`). The core engine parses the AST and passes the nodes to the loaded plugins for evaluation.

#### 4.2 The Agentic Warning Payload

When an external plugin flags an AST node, the core engine catches the flag and serializes it into the Agentic REPL payload. The core compiler **MUST NOT** halt execution. It emits the JSON warning to the standard output or Language Server Protocol (LSP) client for the user or LLM to handle.

```json
{
  "status": "warning",
  "plugin": "@tenuto/rules-orchestral",
  "rule_id": "mud-zone",
  "location": { "measure": 4, "track": "piano" },
  "diagnostics": {
    "message": "Dense tertian harmony detected below C3 (130.81 Hz).",
    "suggestion": "Consider open fifth voicings to prevent harmonic interference."
  }
}

```

---

### 5. Semantic Decompilation Engine (`tenuto-decompile`)

The TSA includes a normative, offline Decompiler to ingest explicit machine formats (MIDI, MusicXML) and refactor them into highly compressed Tenuto source code. The Decompiler **SHALL** rely strictly on deterministic algorithms and active linting plugins to infer intent, prohibiting the use of non-deterministic AI hallucination during translation.

#### 5.1 Lexical Compression & State Restoration

* **State Diffing:** The Decompiler maintains a virtual state cursor. If consecutive notes share exact octave or duration parameters, the redundant data **SHALL** be programmatically stripped to leverage Tenuto's semantic Sticky State.
* **Macro Extraction:** Dictionary coding algorithms (e.g., LZ77) **SHALL** be utilized to scan the linear event stream. Recurring structural arrays must be extracted, assigned to the Global Symbol Table as a `$macro`, and replaced locally.

#### 5.2 Temporal Quantization

* **Euclidean Reverse-Engineering:** For rhythmic grids, the Decompiler evaluates discrete physical hits (K) against total grid slots (N). By running the Bresenham line-drawing algorithm E(K,N) in reverse, matching arrays **MUST** be collapsed into Tenuto Euclidean notation (e.g., `(k):3/8`).
* **Micro-Timing Extraction:** If a physical note falls milliseconds ahead of or behind an idealized rational boundary, the engine quantizes the logical note to the exact fraction and appends the absolute delta as a physical modifier (e.g., `c4:4.push(12ms)`).

#### 5.3 Profile-Driven Refactoring

The Decompiler utilizes the active `.tenutorc.json` profile in reverse to infer semantic intent. For example, if analyzing a piano performance while the `@tenuto/rules-piano` plugin is active, the engine mathematically realigns delayed CC 64 (sustain pedal) messages into a dedicated, visually decoupled `pedal:` control lane synchronized with the logical downbeat.

#### 5.4 Graph Folding (Structural Loop Recognition)

The Decompiler generates a cryptographic hash for the absolute payload of every measure block. When a sequential array of identical measure hashes is detected, the engine **SHALL** fold the linear timeline back into itself, wrapping the source code in standard barline repeat tokens (`|:` and `:|`).
