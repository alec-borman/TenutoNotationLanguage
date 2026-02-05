# Tenuto Language Specification

**Version:** 2.0.0

**Status:** Normative / Final

**License:** MIT

**Maintainer:** The Tenuto Working Group

## Table of Contents

## Table of Contents

1. [Introduction](#1-introduction)
2. [Lexical Structure](#2-lexical-structure)
3. [Document Structure](#3-document-structure)
4. [Instrument Definitions (The Physics)](#4-instrument-definitions-the-physics)
5. [The Event Engine: Rhythm & Time](#5-the-event-engine-rhythm--time)
6. [The Pitch Engine](#6-the-pitch-engine)
7. [Notational Attributes](#7-notational-attributes)
8. [The Tablature Engine](#8-the-tablature-engine)
9. [The Percussion Engine](#9-the-percussion-engine)
10. [Advanced Polyphony](#10-advanced-polyphony)
11. [Structure & Flow Control](#11-structure--flow-control)
12. [The Lyric Engine](#12-the-lyric-engine)
13. [Layout Directives](#13-layout-directives)
14. [Playback Control (The Synth Engine)](#14-playback-control-the-synth-engine)
15. [Macros & Variables](#15-macros--variables)
16. [File Organization](#16-file-organization)
17. [Advanced Engraving Controls](#17-advanced-engraving-controls)
18. [Ornamentation & Lines](#18-ornamentation--lines)
19. [Microtonality & Tuning Systems](#19-microtonality--tuning-systems)
20. [Visual Styling (The Theme Engine)](#20-visual-styling-the-theme-engine)
21. [Advanced MIDI & Automation](#21-advanced-midi--automation)
22. [Compiler Directives & Debugging](#22-compiler-directives--debugging)
23. [The Standard Library](#23-the-standard-library)
24. [Error Reference](#24-error-reference)
25. [Implementation Guidelines](#25-implementation-guidelines)
26. [Formal Grammar (EBNF)](#26-formal-grammar-ebnf)
27. [Interoperability & Exchange](#27-interoperability--exchange)
28. [Reference Example](#28-reference-example-the-kitchen-sink)

---

### Addendum A: Advanced Implementation & Extensions
* [A.1 Live Execution Model (REPL & Daemon)](#a1-live-execution-model-repl--daemon)
* [A.2 Binary Format (.tenb)](#a2-binary-format-tenb)
* [A.3 Cryptographic Integrity & Archival](#a3-cryptographic-integrity--archival)
* [A.4 Feature Degradation Matrix](#a4-feature-degradation-matrix)
* [A.5 Error Correction (Leniency)](#a5-error-correction-leniency)
* [A.6 Real-Time Collaboration Protocol](#a6-real-time-collaboration-protocol)
* [A.7 Implementation Checklist](#a7-implementation-checklist)

---

### Addendum A: Advanced Implementation & Extensions

* [A.1 Live Execution Model (REPL & Daemon)](https://www.google.com/search?q=%23a1-live-execution-model-repl--daemon)
* [A.2 Binary Format (.tenb)](https://www.google.com/search?q=%23a2-binary-format-tenb)
* [A.3 Cryptographic Integrity & Archival](https://www.google.com/search?q=%23a3-cryptographic-integrity--archival)
* [A.4 Feature Degradation Matrix](https://www.google.com/search?q=%23a4-feature-degradation-matrix)
* [A.5 Error Correction (Leniency)](https://www.google.com/search?q=%23a5-error-correction-leniency)
* [A.6 Real-Time Collaboration Protocol](https://www.google.com/search?q=%23a6-real-time-collaboration-protocol)
* [A.7 Implementation Checklist](https://www.google.com/search?q=%23a7-implementation-checklist)

## 1. Introduction

Tenuto is a declarative, domain-specific language (DSL) designed to serialize musical logic, notation, and performance data into a human-readable text format. Unlike XML-based interchange formats (such as MusicXML) which prioritize visual layout coordinates and graphical preservation, Tenuto prioritizes **musical intent**.

It utilizes a deterministic inference engine to calculate layout, beaming, and audio synthesis at render-time, allowing the user to focus purely on composition structure. This document defines the syntax, grammar, and processing rules for Tenuto Version 2.0.0.

### 1.1 Design Philosophy

The language adheres to three core principles designed to maximize efficiency, maintainability, and interoperability:

1. **Inference Over Redundancy (The "Sticky State"):** Musical notation is inherently repetitive. Tenuto leverages this by maintaining a stateful cursor. Attributes such as duration, octave, and articulation persist until explicitly changed. This reduces file size and aligns the code with the mental model of a performer reading a score.
2. **Semantic Separation:** The definition of an instrument (its "Physics"—range, transposition, tuning) is strictly separated from the event data (the "Notes"). This allows a single musical pattern to be re-assigned from a Violin to a Guitar without rewriting the notation logic.
3. **Human Readability:** Source code must be intelligible to a musician without the need for rendering software. The syntax mimics standard music theory shorthand, serving as a valid form of archival documentation in its raw state.

### 1.2 The Coordinate System

Tenuto maps auditory events onto a high-dimensional logical grid. Understanding this coordinate system is essential for implementing the standard correctly.

* **X-Axis (Time):** Linear absolute time. While the code is organized into `Measure` blocks for human convenience, the compiler views the X-axis as a continuous stream of "Ticks" (pulses).
* **Y-Axis (Source):** Distinct logical threads defined by **Staff IDs**. These represent the instruments or entities producing sound.
* **Z-Axis (Polyphony):** Vertical layering within a single Time/Source coordinate. These are handled via **Voices** (e.g., Soprano/Alto on a single staff).

### 1.3 The Compilation Pipeline

A compliant Tenuto compiler **MUST** implement a multi-stage transformation pipeline to convert source text into a renderable artifact. This pipeline ensures that all context (Global Definitions) is resolved before the linear logic is processed.

1. **Lexing & Parsing:** The raw UTF-8 text is tokenized and validated against the Tenuto Formal Grammar.
2. **Context Building:** The compiler scans the `meta` and `def` blocks to establish the global physics of the piece (Time signatures, Instrument capabilities, Tempos).
3. **Linearization:** The compiler iterates through the `measure` blocks. This is where the "Inference Engine" operates:

* It **MUST** resolve "Sticky" attributes (filling in missing durations or octaves based on previous state).
* It **MUST** calculate absolute tick positions for every event.
* It **MUST** validate vertical synchronization (ensuring all voices in a measure sum to the correct duration).

4. **Rendering:** The linearized, fully resolved data is mapped to the target output (SVG for scores, MIDI for audio, or MusicXML for interchange).

### 1.4 Scope & Limitations

It is critical to understand the boundaries of the Tenuto specification.

* **In Scope:**
* Definition of musical pitch, rhythm, and structural flow.
* Definition of instrument characteristics (transposition, string tuning, percussion mapping).
* High-level layout directives (system breaks, page turns).
* Abstract playback controls (dynamics, tempo, MIDI CC automation).
* **Out of Scope:**
* **Binary Audio:** Tenuto does not embed .wav or .mp3 data. It generates instructions for a synthesizer, but is not a sampler.
* **DAW Session Data:** Tenuto is not a replacement for Ableton Live or Pro Tools project files. It does not store plugin states, EQ settings, or routing graphs.
* **Pixel-Perfect Engraving:** While Tenuto supports layout hints, the final calculation of bezier curves and font kerning is the responsibility of the *Renderer*, not the *Language*.

### 1.5 Conformance & Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://tools.ietf.org/html/rfc2119).

#### 1.5.1 Conformance Classes

This specification defines two classes of conformance:

1. **Strict Conformance:** A compiler or interpreter that validates every aspect of the grammar and halts on any ambiguity or "Sticky State" violation across prohibited boundaries (e.g., Staff-to-Staff stickiness). Strict conformance is **REQUIRED** for archival and interchange tools.
2. **Lenient Conformance:** A tool (such as a live-coding environment) that attempts to infer missing data where the spec is ambiguous (e.g., auto-closing beams at bar lines). Lenient tools **SHOULD** emit warnings when auto-correcting syntax.

### 1.6 Typographic Conventions

* `Code` font is used for keywords, identifiers, and literals.
* **Bold** text is used for defining terms or emphasizing normative requirements.
* *Italic* text is used for non-normative notes, examples, and commentary.

```tenuto
%% Code blocks formatted like this are Non-Normative examples.
measure 1 {
  vln: c4 d e f |
}


```

---

## 2. Lexical Structure

This section defines the lexical grammar for Tenuto. A conformant parser **MUST** convert a stream of Unicode characters into a stream of tokens based on the rules defined below.

### 2.1 Character Set & Encoding

Source files **MUST** be encoded in **UTF-8**.

* The Byte Order Mark (BOM) is **OPTIONAL**; parsers **SHOULD** ignore it if present.
* Control characters (U+0000 through U+001F), excluding standard whitespace, are **FORBIDDEN** outside of string literals.

### 2.2 Whitespace & Formatting

Whitespace characters are used to separate tokens. Beyond separation, whitespace is **insignificant**.

* **Valid Whitespace:** Space (`U+0020`), Horizontal Tab (`U+0009`), Line Feed (`U+000A`), Carriage Return (`U+000D`).
* **Indentation:** Unlike Python or YAML, indentation has **NO** syntactic meaning. Developers are encouraged to use 2-space or 4-space indentation for human readability, but the parser treats `measure 1 vln: c4` and `measure 1\n  vln: c4` identically.
* **Line Terminators:** Statements **MAY** span multiple lines. There is no explicit "End of Statement" character (like `;` in C-family languages); the structure is inferred from the block nesting hierarchy.

### 2.3 Comments

Comments are non-executable text segments used for documentation. They are stripped from the token stream during the Lexing phase.

* **Line Comments:** Indicated by a double percentage sign `%%`. All text following `%%` on the same line **MUST** be ignored.

```tenuto
%% This is a comment
c4:4 %% Inline comment


```

* **Block Comments:** Block comments are **NOT SUPPORTED** in Version 2.0.0. This design choice prevents parsing ambiguities regarding nested comments.

### 2.4 Case Sensitivity

Tenuto enforces strict rules regarding case sensitivity to avoid ambiguity between musical data and structural logic.

1. **Keywords:** All reserved keywords (`def`, `measure`, `meta`, `style`) are **Case-Insensitive**. `DEF`, `Def`, and `def` are identical. *Convention:* Lowercase `def` is preferred.
2. **Note Names:** Pitch literals (`c4`, `F#5`) are **Case-Insensitive**. `c4` and `C4` refer to the same pitch object.
3. **Identifiers:** User-defined identifiers (Staff IDs, Macro Names) are **Case-Sensitive**. `vln` is distinct from `Vln`. This distinguishes between semantic groups (e.g., `Strings` group vs `strings` patch).
4. **String Literals:** Content within quotes is **Case-Sensitive** and preserves original formatting.

### 2.5 Literals & Data Types

The following table defines the primitive data types supported by the Tenuto lexer.

| Type | Regex Pattern | Examples | Description |
| --- | --- | --- | --- |
| **Integer** | `[0-9]+` | `1`, `120` | Used for octaves, BPM, counts. |
| **Float** | `[0-9]+\.[0-9]+` | `1.5`, `0.75` | Used for precise durations/multipliers. |
| **String** | `".*"` | `"Violin I"` | UI Labels, Lyrics, Metadata. Supports UTF-8. |
| **Pitch** | `[a-gA-G][# | b | x...` |
| **TabCoord** | `[0-9]+-[1-9][0-9]*` | `0-6`, `12-2` | Format: `Fret-String`. |
| **Boolean** | `true | false` | `true` |
| **Identifier** | `[a-zA-Z_][a-zA-Z0-9_]*` | `vln`, `My_Macro` | Used for naming Staves and Macros. |

### 2.6 Operators & Punctuators

The following symbols act as delimiters or operators and **MUST** be tokenized as distinct lexical elements, even if adjacent to other tokens.

| Symbol | Name | Usage |
| --- | --- | --- |
| `{` `}` | Braces | Scope definition (Groups, Voices, Meta blocks). |
| `[` `]` | Brackets | Chord grouping, Volta endings. |
| `:` | Colon | Assignment (Staff), Duration separator (`c4:4`). |
| `.` | Dot | Attribute accessor (`.stacc`), Dotted rhythm (`:4.`). |
| `,` | Comma | List separator (Meta keys, Arrays). |
| `~` | Tilde | Tie connection. |
| ` | ` | Pipe |
| `$` | Dollar | Macro invocation prefix. |
| `(` `)` | Parentheses | Tuplet grouping, Argument lists. |
| `=` | Equals | Definition assignment. |

### 2.7 Reserved Keywords

The following tokens are reserved and **MUST NOT** be used as Identifiers to avoid ambiguity in the LL(1) parser:

`tenuto`, `meta`, `def`, `measure`, `group`, `import`, `macro`, `if`, `else`, `return`, `break`, `stretch`, `style`, `clef`, `transpose`, `tuning`, `map`, `patch`, `vol`, `pan`, `lyric`.

### 2.8 Escape Sequences

Within String literals, the following escape sequences **MUST** be supported: `\"`, `\\`, `\n`, `\t`, `\uXXXX` (Unicode).

---

## 3. Document Structure

A valid Tenuto document represents a self-contained unit of musical logic. This section defines the syntactic hierarchy, block ordering, and scoping rules required for a well-formed document.

### 3.1 The Root Block

The outermost scope of any Tenuto file **MUST** be enclosed in a generic root block initiated by the `tenuto` keyword. This block establishes the **Global Namespace** for the compilation unit.

* **Implicit Root:** In "Lenient" conformance mode, the `tenuto` wrapper **MAY** be omitted.
* **Explicit Root:** In "Strict" conformance mode, the wrapper is **REQUIRED**.

```tenuto
tenuto {
  %% Content goes here
}


```

### 3.2 Block Ordering (The Three Phases)

To facilitate efficient single-pass compilation, Tenuto enforces a **Declaration-Before-Use** policy. A valid document structure consists of three sequential phases.

#### 3.2.1 Phase 1: Configuration (Meta)

The `meta` block establishes the global environment (Time Signature, Tempo, Key).

* **Constraint:** Global physics constants (like `time: 4/4`) **MUST** be defined before the first `measure` block.

#### 3.2.2 Phase 2: Definition (Defs)

The `def` statements register identifiers (Staff IDs) into the Symbol Table.

* **Constraint:** An Instrument ID (e.g., `vln`) **MUST** be defined before it is referenced in a `measure` block. Using an undefined identifier is a **Fatal Error (E2001)**.
* **Grouping:** `def` statements **MAY** be nested inside `group` blocks for visual organization.

#### 3.2.3 Phase 3: Logic (Flow)

The `measure` blocks contain the event data.

* **Constraint:** Once the Logic phase begins, new `def` statements are **FORBIDDEN** within the same scope.
* **Import Exception:** The `import` statement is a pre-processor directive. If an imported file contains `def` blocks, they are processed as if they appeared at the point of import. Therefore, imports containing definitions **MUST** appear before the Logic phase begins.

### 3.3 Metadata Scope & Keys

The `meta` block defines properties that persist until overridden (Sticky State). Use of `meta` inside a measure creates a **Local Scope** that inherits from the Global Scope.

**Syntax:** `meta { key: value, ... }`

| Key | Type | Default | Description |
| --- | --- | --- | --- |
| `title` | String | "Untitled" | The work's title (Global only). |
| `composer` | String | "Unknown" | The author (Global only). |
| `tempo` | Integer | 120 | Beats per minute (BPM). |
| `time` | String | "4/4" | Time signature. |
| `key` | String | "C" | Key signature. |
| `swing` | Integer | 0 | Swing percentage (0-100). |

### 3.4 Definition Scope

The `def` statement registers a Staff ID.

**Syntax:** `def [ID] [Label] [Attributes]`

* **ID Uniqueness:** Staff IDs **MUST** be unique within the Global Namespace.
* **Visibility:** Once defined, an ID is visible to all subsequent `measure` blocks in the compilation unit.

### 3.5 The Logic Stream & Measure Syntax

The `measure` block serves as the container for temporal events.

**Syntax:** `measure [Range] ...`

#### 3.5.1 Measure Range Grammar

The `[Range]` parameter defines which time-slices the block populates.

* **Single Index:** `measure 1` (Standard usage).
* **Range:** `measure 1-4` (Populates 4 sequential measures with the same logic).
* **List:** `measure 1, 3, 5` (Populates specific indices).

#### 3.5.2 Additive Merging (The "Open Measure")

Tenuto utilizes an **Additive Merge Strategy**.

* **Re-definition:** If a `measure` index is encountered that has already been defined (e.g., `measure 1` appears in `flute.ten` and later in `cello.ten`), the compiler **MUST** merge the new content into the existing time slice.
* **Conflict:** If the re-definition attempts to modify Global Meta (e.g., changing Time Signature), it is a **Fatal Error**.

### 3.6 File Standards

To ensure interoperability between operating systems and applications:

* **File Extension:** Source files **MUST** use the `.ten` extension.
* **MIME Type:** The official MIME type is `text/x-tenuto`.
* **Shebang:** Executable scripts **MAY** start with `#!/usr/bin/env tenutoc`.

---

## 4. Instrument Definitions (The Physics)

This section defines the syntax and semantics for registering instruments. The `def` statement establishes the **"Physics"** of a staff—how the parsing engine interprets the logic stream and maps it to audio/visual outputs.

### 4.1 The Definition Statement

The `def` keyword registers a new Staff ID in the Global Symbol Table.

**Syntax:** `def [ID] [Label] [Attributes]`

* **ID:** An alphanumeric identifier (e.g., `vln`). It **MUST** be unique within the Global Namespace.
* **Label:** A String literal (e.g., `"Violin I"`). Used by the Renderer for visual staff labels.
* **Attributes:** A space-separated list of `key=value` pairs.

### 4.2 Staff Styles (The Engines)

The `style` attribute determines the parsing mode.

#### 4.2.1 Standard Style (`style=standard`)

* **Input Data:** Pitch Literals (`c4`) or Rests (`r`).
* **Transposition Logic:** Tenuto logic is **Concert Pitch** by default. The logic stream represents the absolute sounding pitch. The `transpose` attribute affects the **Visual Rendering** only (transposing the display to the appropriate key for the performer).

#### 4.2.2 Tablature Style (`style=tab`)

* **Input Data:** Tab Coordinates (`0-6`).
* **Physics:** Requires a `tuning` array. The number of strings is inferred from the array length.
* **Range Validation:** Providing a string index outside the bounds of the `tuning` array is a **Range Error**.

#### 4.2.3 Grid Style (`style=grid`)

* **Input Data:** Mapped Characters (`k`, `s`, `h`).
* **Map Logic:** Requires a `map` dictionary linking keys to visual positions and MIDI notes.

### 4.3 Attribute Reference

The following table defines the normative attributes for instrument definition.

| Attribute | Valid Styles | Type | Default | Description |
| --- | --- | --- | --- | --- |
| **`style`** | All | Enum | `standard` | The parsing engine (`standard`, `tab`, `grid`). |
| **`clef`** | `standard` | Enum | `treble` | Visual clef (`treble`, `bass`, `alto`, `tenor`, `perc`). |
| **`transpose`** | `standard` | Integer | `0` | Visual semitone offset. |
| **`tuning`** | `tab` | Array | `guitar_std` | Open string pitches (Low to High). |
| **`capo`** | `tab` | Integer | `0` | Fret offset. |
| **`map`** | `grid` | Map | `gm_std` | Input Token mapping. |
| **`patch`** | All | String | "Grand Piano" | General MIDI Name or SoundFont ID. |
| **`channel`** | All | Integer | Auto | MIDI Channel (1-16). Defaults to auto-increment. |
| **`bank`** | All | Integer | `0` | MIDI Bank Select (MSB). |
| **`keyswitch`** | `standard` | Map | `{}` | Articulation map (See 4.6). |
| **`vol`** | All | Integer | `100` | Default Volume (CC 7). |
| **`pan`** | All | Integer | `0` | Default Pan (CC 10). |

### 4.4 Percussion Mapping

For `style=grid`, the `map` attribute defines the input schema.

**Syntax:** `map={ key: [position, midi_note], ... }`

* **Key:** The character used in the logic stream (e.g., `k`).
* **Position:** Integer. Vertical staff offset (0 = bottom line).
* **MIDI Note:** Integer. The note triggered during playback.

### 4.5 Grouping

Staves **MAY** be grouped using the `group` block.

**Syntax:** `group [Label] symbol=[brace|bracket|line] { ... }`

* **Scope:** `group` blocks do **NOT** create a variable scope; IDs remain global.
* **Nesting:** Groups **MAY** be nested.

### 4.6 Articulation Maps (Keyswitches)

To support realistic playback of complex sample libraries, instruments **MAY** define a `keyswitch` map. This binds an attribute modifier (logic) to a specific MIDI note (audio trigger).

**Syntax:** `keyswitch={ modifier: note_number, ... }`

```tenuto
def vln "Violin" style=standard keyswitch={
  arco: 24,   %% MIDI Note 24 (C0) triggers Arco
  pizz: 25    %% MIDI Note 25 (C#0) triggers Pizzicato
}


```

*Usage:* In the logic stream, appending `.pizz` to an event will silently trigger MIDI note 25 before playing the event.

---

## 5. The Event Engine: Rhythm & Time

The core of Tenuto's efficiency lies in its handling of time. Unlike coordinate-based formats that require explicit start positions for every event, Tenuto treats music as a **Linear Stream of Durations**. The absolute start time of an event is deterministically calculated as the sum of the durations of all preceding events in that voice.

### 5.1 The Duration Syntax

Duration is denoted by a colon followed by a value (`:value`). The base values correspond to the reciprocal of the note type (e.g., 4 = Quarter Note).

| Token | Musical Value | Relative Value (Whole=1.0) | Tick Count (at 1920 PPQ) |
| --- | --- | --- | --- |
| `:0.5` | Double Whole (Breve) | 2.0 | 7680 |
| `:1` | Whole Note | 1.0 | 3840 |
| `:2` | Half Note | 0.5 | 1920 |
| `:4` | Quarter Note | 0.25 | 960 |
| `:8` | Eighth Note | 0.125 | 480 |
| `:16` | Sixteenth Note | 0.0625 | 240 |
| `:32` | Thirty-Second Note | 0.03125 | 120 |
| `:64` | Sixty-Fourth Note | 0.015625 | 60 |

#### 5.1.1 Augmentation Dots

Durations **MAY** be modified by appending periods (`.`).

* **Single Dot (`.`):** Adds 50% to the base value ().
* *Example:* `:4.` (Dotted Quarter) =  ticks.
* **Double Dot (`..`):** Adds 75% to the base value ().

#### 5.1.2 Duration Multipliers

For repetitive events or structural blocks (like Multi-Measure Rests), a multiplier syntax **MAY** be used.

* **Syntax:** `duration * count`
* *Example:* `r:1 * 4` denotes a rest with the duration of 4 whole notes.

### 5.2 Sticky State (Contextual Inference)

To minimize file size and visual noise, Tenuto employs **Sticky State** logic. If a duration is omitted from an event token, the parser **MUST** infer it from the immediately preceding event in the same Staff/Voice context.

#### 5.2.1 The Inference Rules

1. **Staff-Local:** Stickiness is strictly isolated to the Staff ID.
2. **Voice-Local:** Within a staff, stickiness is isolated to the active Voice layer.
3. **Measure-Crossing:** Stickiness **PERSISTS** across bar lines. It is a continuous state cursor.
4. **Tuplet-Penetrating:** Stickiness flows into and out of Tuplet groups linearly.

* *Example:* `c4:4 (d:8 e f):3/2 g` -> `g` is inferred as `:8`.

5. **Initialization:** If the first event of a staff has no explicit duration, the compiler **SHOULD** default to `:4` and emit a **Warning**.

### 5.3 Tuplets (Irrational Rhythms)

Tuplets define a span of time divided into equal parts contrary to the prevailing meter.

**Syntax:** `( [Events...] ):Actual/Normal`

* **Ratio ():** "Play  notes in the time of ."
* **Calculation:** .
* **Nesting:** Tuplets **MAY** be nested. The timing modification is multiplicative.

```tenuto
%% Triplet: 3 eighths in the space of 2
(c4:8 d e):3/2 

%% Quintuplet: 5 sixteenths in the space of 4
(c d e f g):5/4


```

### 5.4 Grace Notes (Atemporal Events)

Grace notes are ornamental events that theoretically occupy "zero duration" in the measure's metrical grid but consume real time during playback.

**Syntax:** `:grace` or `:grace.slash` (Acciaccatura) or `:grace.noSlash` (Appoggiatura).

* **Metric Logic:** Grace notes **DO NOT** advance the measure's internal clock (the "Grid Cursor").
* **Playback Logic:** Grace notes "steal" time from the *following* event (Appoggiatura) or the *previous* event (Acciaccatura), depending on the renderer's settings. The default behavior is to steal from the following note (On-the-beat execution).
* **Stickiness:** Grace duration is **NOT** sticky. The state reverts to the last "Real" duration immediately after the grace note.
* *Example:* `c4:4 d:grace e` -> `e` is inferred as `:4`, not `:grace`.

### 5.5 Rests

Rests are treated as silent events.

* **Literal:** `r`
* **Syntax:** `r:4`, `r:1*8`

### 5.6 Resolution & Tick Rate

To ensure deterministic MIDI and audio rendering, a compliant compiler **MUST** operate on an internal resolution of at least **1920 PPQ** (Pulses Per Quarter Note).

---

## 6. The Pitch Engine

This section defines how frequency data is encoded for instruments using the `style=standard` engine. Tenuto utilizes a modified **Scientific Pitch Notation (SPN)** combined with stateful inference to define pitch.

### 6.1 Pitch Syntax

A valid pitch token consists of three components in strict order:

**Syntax:** `[Step] [Accidental?] [Octave?]`

1. **Step:** Case-insensitive letter `A` through `G`. These correspond to the diatonic steps of the standard Western 12-Tone Equal Temperament (12-TET) scale.
2. **Accidental (Optional):** Modifier suffix altering the pitch by chromatic steps.

* **Standard:** `#` (Sharp, +1), `b` (Flat, -1), `x` (Double Sharp, +2), `bb` (Double Flat, -2), `n` (Natural, 0).
* **Microtonal:** `qs` (Quarter Sharp, +0.5), `qf` (Quarter Flat, -0.5), `tqs` (Three-quarter Sharp, +1.5), `tqf` (Three-quarter Flat, -1.5).

3. **Octave (Optional):** Integer `0` through `9`.

### 6.2 The Reference Standard (Physical Grounding)

To ensure this specification remains decipherable over millennia, the pitch ontology is grounded in acoustic physics, not software protocols.

* **Reference Octave:** The integer `4` (as in `A4`) defines the octave containing the tuning reference.
* **Reference Frequency:** Unless overridden in `meta`, the token `a4` is normatively defined as **440 Hz**.
* **Mathematical Derivation:** All other pitches are derived logarithmically relative to this constant:

Where  is the distance in semitones from A4.

* **Archival Note:** While `C4` is culturally termed "Middle C", its physical definition is ~261.63 Hz relative to A4=440.

### 6.3 Sticky Octaves (State Persistence)

To reduce verbosity, Tenuto employs **Sticky Octave** logic.

* **Rule:** If the Octave integer is omitted from a pitch token, the parser **MUST** infer it from the immediately preceding pitch event in the same Voice/Staff context.
* **Logic:** The inference is **Absolute** (State Copy), not Relative (Interval).
* *Example:* `c4 b a g` resolves to `c4 b4 a4 g4`.
* **Initialization:** If the first note of a staff has no octave, the compiler **SHOULD** default to `4` (The Reference Octave).

### 6.4 Accidental Logic & Statelessness

To prevent ambiguity caused by "Measure Rules" (which vary by century and style), Tenuto adopts a **Stateless Accidental** model for the raw code.

1. **Explicit Mode:** An accidental in the code (`f#4`) **ALWAYS** sets the pitch to that specific chromatic value.
2. **Implicit Mode:** A note without an accidental (`f4`) inherits the accidental defined by the current **Key Signature** (Global State).
3. **Statelessness:** Accidentals do **NOT** persist strictly through the measure in the code logic. Every token is evaluated independently against the Key Signature.

* *Durability Note:* This ensures that if a single measure is extracted or a bar line is moved, the pitch data remains mathematically correct without needing to "scan back" for previous accidentals in the measure.

### 6.5 Chords (Vertical Polyphony)

Multiple pitches played simultaneously by a single voice are enclosed in square brackets `[]`.

**Syntax:** `[ pitch1 pitch2 ... ] :duration`

* **Sticky Duration:** The duration applies to the entire chord object.
* **Internal State:** Sticky Octaves apply sequentially *within* the chord from left to right.
* *Example:* `[c4 e g]` resolves to `[c4 e4 g4]`.

### 6.6 Ties

Ties extend the duration of a pitch by connecting it to a subsequent note of the same pitch.

**Syntax:** Append `~` to the pitch token.

* **Single Note:** `c4:4~ c4:8` (Total duration 1.5 beats).
* **Chord Tying:** Ties **MAY** be applied to individual notes within a chord structure, allowing for complex polyphonic suspensions within a single voice.
* *Example:* `[c4~ e4 g4] [c4 f4 a4]` (C is tied; E and G move to F and A).

### 6.7 Data Integrity Recommendations

For files intended for long-term archival (100+ years), it is **RECOMMENDED** to use **Strict Explicit Mode**, where every note includes an explicit Octave and Duration. This immunizes the data against corruption of the "Sticky State" chain.

```tenuto
%% Archival Safe
vln: c4:4 e4:4 g4:4 c5:4


```

---

## 7. Notational Attributes

Attributes are metadata attached to events that modify their semantic meaning (Amplitude, Envelope, Timbre) or visual presentation. Tenuto utilizes a **Dot Notation** syntax to chain these modifiers.

### 7.1 Attribute Grammar

Attributes are appended to the Event token, immediately following the duration (if present).

**Syntax:** `Event (:Duration)? (.Modifier)*`

* **Chaining:** Multiple modifiers **MAY** be chained on a single event.
* *Example:* `c4:4.stacc.acc.ff`
* **Arguments:** Modifiers **MAY** accept arguments enclosed in parentheses. Arguments generally support Integers, Floats, or String Literals.
* *Example:* `.text("dolce")`, `.finger(3)`
* **Commutativity:** The order of modifiers is **Commutative** regarding their semantic effect (`.stacc.acc` is identical to `.acc.stacc`). However, for visual rendering, the order **SHOULD** define the Z-stacking order moving outward from the notehead.

### 7.2 Category A: Dynamics (Amplitude)

Dynamics control the energy (loudness) of the event.

* **Tokens:** `pppp`, `ppp`, `pp`, `p`, `mp`, `mf`, `f`, `ff`, `fff`, `ffff`, `sfz`, `fp`, `rfz`.
* **State Behavior:** Dynamics are **Sticky**. A dynamic token sets the `CurrentAmplitude` state for the Staff. This state persists across bar lines and applies to all subsequent events until a new Dynamic token is encountered.
* *Logic:* `c4.ff d e` -> `d` and `e` are also `ff`.

### 7.3 Category B: Articulations (Envelope)

Articulations modify the temporal envelope (Attack, Decay, Sustain, Release) of the specific event they are attached to.

| Token | Name | Audio Semantics | State Behavior |
| --- | --- | --- | --- |
| `.stacc` | Staccato | Gate Time  50% | **Transient** |
| `.stacciss` | Staccatissimo | Gate Time  25% | **Transient** |
| `.ten` | Tenuto | Gate Time = 100% (Legato) | **Transient** |
| `.acc` | Accent | Attack Energy +15% | **Transient** |
| `.marc` | Marcato | Attack Energy +25%, Fast Decay | **Transient** |

* **Transient Logic:** Unlike dynamics, an articulation applies **ONLY** to the event it decorates. It does not persist.
* *Logic:* `c4.stacc d e` -> Only `c4` is staccato. `d` and `e` are normal.

#### 7.3.1 The Fermata Exception

The `.fermata` token is unique. While syntactically an articulation, semantically it acts as a **Global Flow Control** instruction.

* **Behavior:** It halts the **Global Clock** (defined in Section 5) for all staves simultaneously.
* **Duration:** The hold time is undefined (interpretive) unless specified via a `meta` override, but the standard requires the clock to pause, creating a gap in the linear stream.

### 7.4 Category C: Technique Instructions (Timbre)

Technique attributes modify the physical method of sound production.

* **Tokens:** `.pizz` (Pizzicato), `.arco` (Arco), `.mute` (Con Sordino), `.open` (Senza Sordino), `.harm` (Natural Harmonic), `.harm_art` (Artificial Harmonic).
* **State Behavior:** Technique instructions are **Sticky**. They represent a physical change in the instrument state (e.g., putting on a mute) that persists until explicitly reversed.
* *Logic:* `c4.pizz d e` -> The section plays pizzicato until an `.arco` token is seen.

### 7.5 Category D: Text & Physical Hints

These attributes provide visual instructions for the performer but do not necessarily alter the audio synthesis unless a specific Keyswitch Map (Section 4.6) is defined.

* **Generic Text:** `.text("String")`. Renders text above the staff.
* **Placement:** `.text_below("String")`, `.text_above("String")`.
* **Fingering:** `.finger(1)` through `.finger(5)`.
* **String Indicator:** `.str(1)` through `.str(N)`.
* **State Behavior:** These are **Transient**.

### 7.6 Extension Mechanism (User-Defined)

To ensure the specification remains durable as musical styles evolve, Tenuto reserves a namespace for custom, user-defined attributes.

* **Syntax:** `.x_[Identifier]`
* **Behavior:** Compliant parsers **MUST** ignore unknown attributes prefixed with `x_` during rendering/playback but **MUST** preserve them in the Abstract Syntax Tree (AST). This allows custom tools or future plugins to utilize data without breaking standard compilers.
* *Example:* `.x_bowScrape`

---

## 8. The Tablature Engine

This section defines the grammar for instruments using the `style=tab` engine. Unlike the Standard engine which encodes *Resultant Pitch* (`c4`), the Tablature engine encodes *Physical Action* (Fret/String coordinates) and *Mechanical Manipulation* (Bends, Slides).

### 8.1 Coordinate Grammar

The fundamental unit of data is the **Tab Coordinate**, representing the intersection of a string vector and a fret position.

**Syntax:** `Fret-String`

1. **Fret:** Integer `0` through `N`.

* `0`: Open String.
* `x` or `X`: Dead Note (Percussive mute with indeterminate pitch).

2. **Hyphen:** Mandatory separator token.
3. **String:** Integer `1` through `N`.

* **Normative Mapping (The Inverse Rule):** String `1` corresponds to the **Highest Pitched String** (physically thinnest). String `N` corresponds to the **Lowest Pitched String**.
* **Index Resolution:** In the `tuning` array defined in Section 4 (ordered Low to High), String `1` maps to `tuning[Length - 1]`. String `N` maps to `tuning[0]`.

*Example:* `0-6` represents the Open Low E string on a standard guitar.

### 8.2 Pitch Resolution Algorithm

To support audio playback, synthesis, and Standard Notation conversion, the compiler **MUST** be able to derive absolute frequency from tab coordinates.

**Formula:** 

* **Validation:** If the provided String ID exceeds the number of strings defined in the instrument's `tuning` array, the compiler **MUST** throw a **Range Error (E801)**.
* **Tuning Integrity:** If the `tuning` array is missing from the `def` block, the compiler **SHOULD** default to Standard Guitar Tuning (`E2` to `E4`) and emit a **Warning**.

### 8.3 Mechanical Techniques (Legato & Timbre)

Techniques specific to the mechanics of fretted instruments are applied as dot modifiers.

| Token | Name | Semantics | State Behavior |
| --- | --- | --- | --- |
| `.h` | Hammer-on | Legato attack (No pluck) from lower to higher fret. | **Transient** |
| `.p` | Pull-off | Legato attack (No pluck) from higher to lower fret. | **Transient** |
| `.t` | Tap | Right-hand fret strike (Percussive attack). | **Transient** |
| `.sl` | Slide | Continuous pitch glissando between coordinates. | **Transient** |
| `.pm` | Palm Mute | Timbre filter (Low-pass) + Decay reduction. | **Sticky** |
| `.letring` | Let Ring | Disables "Note Off" messages (Laissez vibrer). | **Sticky** |
| `.harm` | Nat. Harmonic | Renders `<N>`, plays overtone series. | **Transient** |
| `.ph` | Pinch Harmonic | Artificial harmonic (High frequency squeal). | **Transient** |

### 8.4 Pitch Modification (Bends)

Bends represent continuous mechanical alteration of string tension, resulting in microtonal pitch shifts.

**Syntax:** `.bu(Target)` (Bend Up), `.bd(Target)` (Bend Down/Release).

* **Target Values:** `quarter` (1/4 tone), `half` (1 semitone), `full` (1 tone), `1.5` (Minor 3rd), `2` (Major 3rd).
* **Envelope Logic:**
* **Standard Bend:** `10-2:4.bu(full)` starts at the fretted pitch and ramps linearly to the target over the duration.
* **Pre-Bend:** `10-2:4.pb(full).bd(0)` starts at the target pitch (string tension already increased) and releases to the fretted pitch.
* **Hold:** `10-2:4.bu(full).hold` maintains the target pitch for the duration.

### 8.5 Strums & Chords

Simultaneous coordinates are grouped in brackets `[]` to form a Chord Object.

**Syntax:** `[ Coord1 Coord2 ... ] :Duration .Direction`

* **Direction:** `.down` (Downstroke, Low strings to High) or `.up` (Upstroke, High strings to Low).
* *Audio Semantics:* Directions impose a slight millisecond delay (strum speed) between the onset of each note in the chord, rather than perfect simultaneity.
* **Ghost Strums:** `[x-6 x-5 x-4]` represents a percussive rake across multiple strings.

### 8.6 Rhythmic Continuity

Tablature events adhere to the **Sticky Duration** rules defined in Section 5.

* *Example:* `0-6:8 3-6 5-6` indicates three eighth notes.
* *Constraint:* Unlike ASCII tab, which is often spatially proportional but rhythmically ambiguous, Tenuto Tablature **MUST** have a strictly defined rhythmic grid. A coordinate without a discernible duration context is a **Syntax Error**.

---

## 9. The Percussion Engine

This section defines the grammar for instruments using the `style=grid` engine. Unlike pitched instruments, the Percussion Engine operates on a **Mapped Token System**, where arbitrary alphanumeric keys correspond to specific instruments (e.g., Snare, Kick) defined in the Staff's `map` attribute.

### 9.1 Token Grammar

The fundamental unit of data is the **Mapped Key**.

**Syntax:** `Key (:Duration)? (.Modifier)*`

1. **Key:** An alphanumeric string that **MUST** exist as a key in the instrument's `map` dictionary (defined in Section 4).

* *Validation:* Usage of a key not present in the map is a **Lookup Error (E901)**.

2. **Duration:** Adheres to the standard **Sticky State** logic defined in Section 5.

*Example:* Given a map `{ k: Kick, s: Snare }`, the stream `k:4 s k s` produces a standard rock beat.

### 9.2 Polyphony (Chords vs. Voices)

Percussion notation frequently requires simultaneous events (e.g., Kick drum and Crash cymbal).

#### 9.2.1 Simultaneity (Chords)

To trigger multiple mapped instruments at the exact same tick within a single voice, use square brackets `[]`.

* **Syntax:** `[ k c ] :4` (Kick and Crash together).

#### 9.2.2 Voice Layering

Standard drum notation typically separates limbs into distinct logical voices to preserve clarity (e.g., Cymbals on Voice 1, Drums on Voice 2). Users **SHOULD** utilize Voice Groups (Section 10) for complex drum set notation.

### 9.3 Rudiments & Articulations

Percussion-specific techniques are applied as dot modifiers. These attributes modify the velocity envelope and attack characteristics.

| Token | Name | Audio Semantics | State Behavior |
| --- | --- | --- | --- |
| `.ghost` | Ghost Note | Velocity  40%. **Overrides** Sticky Dynamic. | **Transient** |
| `.flam` | Flam | Single grace note () before impact. | **Transient** |
| `.drag` | Drag | Double grace note () before impact. | **Transient** |
| `.ruff` | Ruff | Triple grace note before impact. | **Transient** |
| `.roll` | Tremolo | Repeated re-triggering (Buzz/Press). | **Transient** |
| `.choke` | Choke | Note Off triggered immediately (). | **Transient** |

* **Roll Specifics:**
* **Tremolo:** `.roll(3)` indicates an unmeasured press roll (rendered with 3 slashes on stem).
* **Measured:** `.roll(1)` indicates exact subdivision (8th or 16th depending on context).
* **Ties:** The Tilde `~` operator **MAY** be used to extend a roll across bar lines. `s:1.roll ~ s:1` results in a continuous roll of 2 measures.

### 9.4 Sticking (Hand Assignment)

For rudimental analysis, sticking is defined via attributes.

* **Tokens:** `.R` (Right), `.L` (Left), `.B` (Both).
* **State Behavior:** Sticking is **Transient**.
* *Example:* `s:16.R s.L s.R s.L` (Paradiddle).

### 9.5 The "Rim" Modifier

Many percussion instruments have multiple strike zones (Head vs. Rim). Rather than mandating separate mapped keys for "Snare Head" and "Snare Rim" (which breaks semantic grouping), Tenuto supports a generic `.rim` modifier.

* **Syntax:** `Key.rim`
* **Semantics:** The renderer looks for a specific "Rim" variant in the sound patch, or alters the notehead (e.g., X notehead) if defined in the Theme.
* *Example:* `s.rim` (Rimshot or Cross-stick, depending on dynamic context).

---

## 10. Advanced Polyphony

Tenuto supports **Multi-Threaded Logic** within a single staff, allowing for independent rhythmic streams (e.g., a Pianist playing a melody and accompaniment in the same hand, or a Drummer playing independent limb patterns). This is achieved through **Voice Groups**.

### 10.1 Voice Group Syntax

Polyphonic regions are enclosed in curly braces `{}`. Within this block, specific **Voice Identifiers** separate the logic streams.

**Syntax:**

```tenuto
Staff_ID: {
  Voice_ID: Events... |
  Voice_ID: Events... |
}


```

* **State Inheritance (Entry):** The Voice Group creates a branching scope.
* **`v1` (Primary):** Inherits the Sticky State (Octave, Duration) from the event immediately preceding the block.
* **`v2`...`v4` (Secondary):** Reset to defaults (Octave 4, Quarter Note) upon entry. This isolation prevents the "Melody's" previous duration from accidentally applying to a new "Bass" line entering the texture.
* **State Inheritance (Exit):** Upon closing the block `}`, the global Sticky State is restored to the state of the last event in **`v1`** (The Primary Voice).

### 10.2 Voice Identifiers & Semantics

Tenuto defines four normative voice layers per staff.

| Identifier | Role | Default Stem Direction |
| --- | --- | --- |
| **`v1`** | Primary / Melody | Up |
| **`v2`** | Secondary / Bass | Down |
| **`v3`** | Tertiary / Inner | Up |
| **`v4`** | Quaternary | Down |

### 10.3 The Synchronization Constraint (Time Integrity)

To ensure the measure remains mathematically valid, the Tenuto compiler enforces strict **Temporal Alignment**.

* **Rule:** The total duration of events in **every** declared voice within a group **MUST** be identical.
* **Padding:** If a voice requires silence to fill the measure, explicit Rests (`r`) **MUST** be used.
* **Validation:**
* If `v1` contains 1920 ticks (Half Note), `v2` must also contain 1920 ticks.
* **Error:** Failure to balance durations results in a **Synchronization Error (E1001)**.

```tenuto
%% Valid Polyphony (Total: 4 beats)
vln: {
  v1: c5:2 d5:2 |
  v2: a4:1      |
}


```

### 10.4 Cross-Staff Notation (Grand Staff)

For instruments defined as a `group` (e.g., Piano, Harp), voices may visually cross between staves while remaining logically attached to their source stream.

* **Attribute:** `.cross(Target_Staff_ID)`
* **Behavior:** The event belongs to the logical stream of the *current* staff (for playback and timekeeping) but is rendered on the *target* staff.
* *Example:* `pno_rh: c4.cross(pno_lh)` draws a Middle C on the bass clef staff, but it remains stemmed to the treble clef voice.

### 10.5 Collision Handling

When multiple voices occupy the same pitch/time coordinate:

1. **Unisons:** Voices with identical duration merge into a single notehead with dual stems.
2. **Offset:** Voices with differing durations (e.g., Half Note vs Quarter Note) are horizontally offset to preserve visual clarity.
3. **Seconds:** Intervals of a second (e.g., F and G) are automatically offset to prevent overlapping noteheads.


## 11. Structure & Flow Control

Musical time is rarely strictly linear. Repetitions, alternative endings, and structural jumps (e.g., Da Capo) require a dedicated grammar to control both the **Playback Cursor** (for audio) and the **Visual Layout** (for reading).

### 11.1 Bar Lines (Terminals)

Explicit Bar Line tokens denote structural boundaries. While a `measure` block implies a standard bar line at its end, explicit tokens override this default.

| Token | Name | Semantics |
| --- | --- | --- |
| ` | ` | Single Bar |
| ` |  | ` |
| ` | ] ` | Final Bar |
| ` | : ` | Start Repeat |
| ` : | ` | End Repeat |
| `: | :` | Double Repeat |

* **Global Synchronization:** Structural tokens are **System-Global**. If `vln` defines a Repeat Sign `|:`, the compiler enforces this repeat on **ALL** staves in the system for that tick.
* **Conflict:** If `vln` defines `|:` and `vlc` defines `|` at the same tick, the compiler **MUST** throw a **Structure Mismatch Error (E1101)**.

### 11.2 Voltas (Alternative Endings)

Repeated sections often require different endings on subsequent passes. Tenuto uses a Bracket Syntax to define these regions.

**Syntax:** `[ N. Events... ]`

* **N:** Integer or list (e.g., `1.` or `1,3.`). Indicates which iteration(s) this block is active for.
* **Logic:**
* On Pass , the cursor enters the bracket.
* On other passes, the cursor **skips** the bracket content entirely.
* **Validation:** All staves active in the measure **MUST** define the same Volta brackets to ensure the system stays aligned.

```tenuto
measure 5 {
  meta { volta: "1." }  %% Preferred: Define in Meta for clarity
  vln: g4 a b c :| 
  vlc: g2    c3  :|
}

```

### 11.3 Navigation Markers (Jumps)

Jumps allow for non-linear movement across the score (e.g., D.S. al Coda).

#### 11.3.1 Anchor Points

Anchors mark a specific tick location in the score.

* `.segno`: Renders the Sign () and registers the timestamp as `Target_Segno`.
* `.coda`: Renders the Coda symbol () and registers the timestamp as `Target_Coda`.
* `.fine`: Marks the potential end of the piece.

#### 11.3.2 Jump Instructions

Instructions trigger the cursor movement.

* `.dc_al_fine`: Jump to Start. Play until `.fine`.
* `.ds_al_fine`: Jump to `.segno`. Play until `.fine`.
* `.dc_al_coda`: Jump to Start. Play until `.to_coda` (To Coda), then jump to `.coda`.
* `.ds_al_coda`: Jump to `.segno`. Play until `.to_coda`, then jump to `.coda`.

### 11.4 Rehearsal Marks

Rehearsal marks serve as human-readable checkpoints.

**Syntax:** `.mark("Label")` attached to a bar line or the first event of a measure.

* **Auto-Increment:** `.mark` (without arguments) instructs the renderer to increment the sequence (A, B, C... or 1, 2, 3...) automatically based on the previous mark.

### 11.5 Anacrusis (Pickup Measures)

To handle pieces that begin before the first downbeat:

**Syntax:** `meta { pickup: Duration }`

* **Location:** This attribute is valid ONLY in the first `measure` block of a file.
* **Behavior:** The measure is treated as "Measure 0" for numbering purposes. The compiler validates that the content length equals the pickup duration, not the full Time Signature.

```tenuto
measure 0 {
  meta { time: 4/4, pickup: :8 } %% Pickup is one 8th note
  vln: g4:8 |
}

```

---

## 12. The Lyric Engine

Lyrics are handled as a **Parallel Data Stream** mapped to a specific Staff or Voice ID. This separation ensures that the musical logic remains uncluttered while allowing the textual content to be read naturally in the source code.

### 12.1 The Lyric Stream

Lyrics are assigned using the `.lyric` suffix.

**Syntax:** `Target_ID.lyric: "Text String"`

* **Targeting:**
* `vln.lyric`: Implicitly maps to the **Primary Voice (`v1`)** of the staff.
* `vln:v2.lyric`: Maps specifically to Voice 2.
* **Mapping Logic:** The text string is tokenized into syllables based on the grammar defined in 12.2. These tokens are mapped **1-to-1** onto the pitch events of the target voice.
* **Skip Rules:** Rests (`r`) and Grace Notes (`:grace`) are **skipped** automatically by the engine. The lyrics map only to "Real" rhythmic events with positive duration.

### 12.2 Syllabification Grammar

The text string is parsed using specific delimiters to determine syllable boundaries and visual rendering.

| Token | Name | Mapping Behavior | Visual Result |
| --- | --- | --- | --- |
| `          ` (Space) | Word Break | Advance to next note. | Standard word spacing. |
| `-` | Hyphen | Advance to next note. | Centered dash between notes. |
| `_` | Melisma | Advance to next note. | Continuous extension line (underscore). |
| `~` | Elision | **Stay on current note.** | Lyric slur (undertie) joining two words. |
| `*` | Skip | Advance to next note. | No text rendered (Empty slot). |

* **Usage Example:**
* *Code:* `"Glo- ~ ria __ in ex- cel- sis * De- o"`
* *Mapping:*

1. `Glo` + `ria` (Elision)  Note 1.
2. `__` (Melisma extension)  Note 2.
3. `in`  Note 3.
4. `ex`  Note 4.
5. `cel`  Note 5.
6. `sis`  Note 6.
7. `*` (Skip)  Note 7 (e.g., an instrumental passing tone).
8. `De`  Note 8.
9. `o`  Note 9.

### 12.3 Multiple Stanzas (Verses)

To notate multiple verses (e.g., Hymns), append an integer index to the keyword.

**Syntax:** `.lyric_N` (where N is an integer starting at 1).

```tenuto
measure 1 {
  vox: c4 d e f |
  vox.lyric_1: "1. Joy to the world"
  vox.lyric_2: "2. No more let sins"
}

```

* **Synchronization:** The compiler **SHOULD** validate that the syllable count of the text string matches the event count of the target voice. Mismatches **SHOULD** generate a **Sync Warning (W1201)** but must not halt compilation (rendering stops at the mismatch).

### 12.4 Non-Western Scripts (Logographic)

Tenuto is UTF-8 native. However, for logographic languages (e.g., Chinese, Japanese) which typically do not use spaces, the **Space Delimiter** is still **REQUIRED** within the source code to define the 1-to-1 mapping.

* *Code:* `"桜 (Sa) 桜 (ku) ra (ra)"`
* *Rendering:* The engine renders the characters without spacing (based on the Language metadata), but uses the code spaces to align them to the three distinct notes.

### 12.5 Chorus & Section Labels

To indicate the start of a structural section (e.g., "Chorus:") inside the lyrics block without consuming a note mapping:

* **Syntax:** Enclose the label in angle brackets `< >`.
* *Example:* `"<Chorus:> Hal- le- lu- jah"`
* The `<Chorus:>` tag is attached to the *same* note as "Hal", but is rendered to the left (typically bold/italic), serving as a margin label.

---

## 13. Layout Directives

Tenuto documents are **Reflowable**. Like HTML, the final visual presentation depends on the target medium (e.g., A4 Paper, iPad Screen, Scrolling Web View). However, specific engraving scenarios require manual overrides to force structural layouts. These are handled via **Layout Directives** contained within `meta` blocks.

### 13.1 Break Directives

Breaks serve as "Hard Returns" for the rendering engine. They instruct the compiler on how to flow the measure stream onto the visual canvas.

**Syntax:** `meta { break: "Type" }`

* **`break: "system"`**: Forces a new musical system (line) to begin *immediately after* the current measure.
* *Justification:* The renderer **SHOULD** spread the current system to fill the available width (Full Justification) before breaking.
* **`break: "page"`**: Forces the content to move to the top of the next page *immediately after* the current measure.
* **`break: "none"`**: Explicitly forbids a break after this measure (Glue), ensuring the next measure stays on the same system if physically possible.

### 13.2 Horizontal Spacing (Stretch)

To adjust the visual density of a specific measure (e.g., to compress a measure full of whole notes or expand a crowded measure):

**Syntax:** `meta { stretch: Float }`

* **Default:** `1.0` (Calculated natural width based on the font's glyph metrics).
* **Behavior:** The engine multiplies the calculated width of the measure content by this factor.
* `> 1.0`: Expands whitespace (Looser).
* `< 1.0`: Condenses whitespace (Tighter).

### 13.3 Vertical Spacing & Indentation

These attributes control the positioning of the system *containing* the current measure.

**Syntax:** `meta { key: Float }`

* **`spacer: Float`**: Adds extra vertical whitespace (in Staff Spaces) *below* the current system.
* *Usage:* Separating distinct exercises or movements on a single page.
* **`indent: Float`**: Adds horizontal whitespace (in Staff Spaces) to the *start* of the system.
* *Usage:* Standard practice for the first system of a piece, or to visually offset a Coda section.

### 13.4 Visibility Controls

Specific elements can be hidden for layout clarity (e.g., Cadenzas, Cutaway Scores).

**Syntax:** `meta { key: Boolean | Float }`

* **`hide_empty: Boolean`**: If `true`, staves in this system containing only Multi-Measure Rests are concealed (French Scoring). This is typically set globally but can be toggled per-system.
* **`numbering: Boolean`**: If `false`, the measure number is hidden for this measure.
* **`staff_scale: Float`**: Scales the entire staff size relative to the global staff size.
* *Example:* `staff_scale: 0.7` is used for **Ossia** staves or **Cue** lines.

---

## 14. Playback Control (The Synth Engine)

While Tenuto is primarily a notation format, it includes a robust set of directives to drive audio synthesis. To ensure longevity, these controls are defined as **Abstract Mathematical Values** rather than hardware-specific byte codes. It is the responsibility of the Compiler/Renderer to map these values to the target protocol (MIDI 1.0, MIDI 2.0, OSC, or Internal Synthesis).

### 14.1 The Abstract Mixer

Mixer parameters are properties of the **Staff State**. They can be set globally in the `def` block or modified dynamically via `meta` injection within the logic stream.

#### 14.1.1 Gain & Panning

* **`vol: Float`**: Normalized Gain.
* **Range:** `0.0` (Silence) to `1.0` (Unity Gain / Maximum Velocity).
* *Default:* `1.0`.
* **`pan: Float`**: Bipolar Stereo position.
* **Range:** `-1.0` (Hard Left) to `+1.0` (Hard Right). `0.0` is Center.
* *Default:* `0.0`.

#### 14.1.2 Effects Sends

To support spatialization and timbre modification without defining specific plugin architectures:

* **`reverb: Float`**: Amount of signal sent to the global Reverb/Space bus (`0.0` to `1.0`).
* **`chorus: Float`**: Amount of signal sent to the Modulation bus (`0.0` to `1.0`).

#### 14.1.3 Channel State

* **`mute: Boolean`**: If `true`, the staff produces no audio but preserves its logical state.
* **`solo: Boolean`**: If `true`, *only* this staff (and other soloed staves) produce audio.

```tenuto
measure 1 {
  %% Fade out violin, pan left, increase reverb over the measure
  meta { vln.vol: [1.0, 0.0], vln.pan: -0.5, vln.reverb: 0.8 }
}

```

### 14.2 Patch Resolution (Timbre)

The `patch` attribute (from Section 4) accepts a **Uniform Resource Name (URN)** to identify sound sources. This allows the specification to remain agnostic regarding the synthesis technology.

* **Standard:** `patch: "gm:Violin"` (General MIDI Mapping).
* **Precise:** `patch: "msb:0,lsb:0,pc:40"` (MIDI Bank/Program Change).
* **SoundFont:** `patch: "sf2:UserBank.sf2:PresetName"`
* **Waveform:** `patch: "wave:sawtooth"` (Basic Synthesis).

### 14.3 Tempo Geometry (The Time Map)

Tempo controls the rate of the "Tick" counter relative to wall-clock time.

**Syntax:** `meta { tempo: Value, curve: "Type" }`

* **Static Tempo:** `tempo: 120`. Sets the immediate BPM.
* **Ramped Tempo:** `tempo: [Start, End]`. Defines a transition strictly over the duration of the **Current Measure**.
* **Curve Types:**
* `"step"`: Immediate jump (Default).
* `"linear"`: Constant rate of change ().
* `"exp"`: Exponential curve (). Recommended for natural-sounding *Accelerando*.
* `"log"`: Logarithmic curve. Recommended for natural-sounding *Ritardando*.

### 14.4 Micro-Timing (Swing)

Swing alters the playback start time of off-beat notes without changing their notated duration or metric position.

**Syntax:** `meta { swing: Percentage }`

* **Definition:** The percentage of the beat allocated to the first subdivision.
* **Standard Values:**
* `50`: Straight (No swing). 50/50 split.
* `66`: Triplet Swing (Standard Jazz). 67/33 split.
* `75`: Hard Swing (Funk/Dotted). 75/25 split.
* **Grid Resolution:** The engine **SHOULD** auto-detect the quantization level based on the tempo (typically swinging 8th notes below 100 BPM, and 16th notes below 60 BPM). Explicit overrides are handled via `swing_grid: Duration`.

### 14.5 Humanization

To prevent the "Machine Gun Effect" inherent in digital playback, the engine supports algorithmic randomization.

**Syntax:** `meta { humanize: Float }`

* **Value:** A percentage (`0.0` to `1.0`) of variance.
* **Behavior:** The engine applies a random offset of  to both **Velocity** and **Tick Start Time** for every event.
* *Example:* `humanize: 0.05` adds  organic jitter.

---

## 15. Macros & Variables

To adhere to the design principle of **Inference Over Redundancy**, Tenuto supports a robust Pre-Processor that handles variable substitution and macro expansion *before* the Linearization phase. This allows composers to define constants for global settings and reusable logic blocks for repetitive musical patterns.

### 15.1 Variables (Constants)

Variables store primitive data types (Integers, Floats, Strings, TabCoords) for reuse. Once defined, a variable is immutable within its scope (effectively a constant).

**Syntax:** `var Name = Value`

* **Naming:** Case-sensitive alphanumeric identifier starting with a letter.
* **Usage:** Prefix with `$`.
* **Scope:**
* **Global:** Defined in the Root block or `tenuto` header. Visible to all subsequent blocks.
* **Local:** Variables are generally **NOT** supported inside `measure` blocks to prevent ambiguity regarding state mutation. They are configuration tools, not dynamic state cursors.

```tenuto
var FortePlus = 115
var ThemeName = "Main Motif"

vln: c4.vel($FortePlus)

```

### 15.2 Macros (Event Blocks)

Macros act as reusable containers for musical patterns. They function as **Compile-Time Text Substitutions**, meaning the compiler injects the macro's body into the stream before parsing the events.

**Syntax:** `macro Name(Arg1, Arg2=Default) = { Events... }`

* **Definition:**

```tenuto
%% A drum pattern taking a Hi-Hat type and a Velocity
macro RockBeat(hat, v=90) = { k:4 $hat.vel($v) s:4 $hat }

```

* **Invocation:**

```tenuto
drm: $RockBeat(h_closed)        %% Uses default v=90
drm: $RockBeat(h_open, 110)     %% Overrides v with 110

```

* **Argument Substitution:** The compiler replaces instances of `$ArgName` within the macro body with the provided values.

### 15.3 Macro Modifiers (Transposition)

Macros containing pitch data can be transposed dynamically upon invocation, allowing a single melodic pattern to be reused across different harmonic contexts.

**Syntax:** `$Name + Semitones` or `$Name - Semitones`

* **Behavior:** The compiler iterates through the expanded token stream. Every **Pitch Literal** found is shifted by  semitones.
* **Exclusions:** Transposition ignores Rests (`r`), Percussion Keys (`k`), and Attributes (`.stacc`).
* **Tablature Constraint:** Transposition on Tablature macros is **Undefined Behavior** unless the compiler implements specific logic to recalculate string/fret coordinates (which may result in impossible fingerings). The compiler **SHOULD** emit a warning if transposing tab data.

### 15.4 Recursion & Safety

To ensure the compiler remains deterministic and halt-able:

1. **Circular Reference:** A macro **MUST NOT** call itself, directly or indirectly. The compiler **MUST** detect dependency cycles (e.g., A calls B, B calls A) and emit a **Fatal Error (E5001)**.
2. **Expansion Limits:** The compiler **SHOULD** enforce a maximum recursion depth (Recommended: 64) and a maximum token count per measure to prevent "Zip Bomb" style attacks or memory exhaustion from exponentially expanding macros.

### 15.5 Conditional Logic (Build Targets)

Macros allow for conditional compilation based on external flags, useful for generating different editions (e.g., "Score" vs "Parts" or "MIDI" vs "PDF") from the same source.

**Syntax:** `if (Condition) { ... }`

```tenuto
measure 1 {
  if (target == "audio") {
    %% Only compiled when generating audio
    keys: c2:1.vol(40) %% Sub-bass reinforcement
  }
}

```

---

## 16. File Organization

To manage complexity in large-scale works (e.g., Symphonies, Operas) and facilitate collaboration, Tenuto supports a modular architecture. This allows the separation of **Definitions** (Physics) from **Logic** (Notes), and the separation of distinct instrument families into their own source files.

### 16.1 The Import Directive

The `import` statement instructs the compiler to read and process the contents of an external file at the current position.

**Syntax:** `import "Filepath"`

* **Path Resolution:** Paths are resolved relative to the directory of the **current file** (the file containing the import statement). Forward slashes `/` are the mandatory directory separator for cross-platform compatibility.
* **Idempotency (Duplicate Guard):** The compiler **MUST** maintain a registry of processed file paths. If a file is imported multiple times (e.g., `A` imports `Lib`, `B` imports `Lib`), the compiler **MUST** process it only once (the first time) and ignore subsequent imports. This prevents "Duplicate Definition" errors for shared libraries.
* **Cycle Detection:** If the compiler detects a circular dependency (File A imports File B, File B imports File A), it **SHOULD** emit a **Cyclic Import Warning** and break the cycle by ignoring the redundant import.

### 16.2 The Additive Merge Model (The "Open Measure")

Unlike imperative programming languages where re-defining a function overwrites the previous definition, Tenuto utilizes an **Additive Merge Strategy** for temporal logic.

* **Principle:** A `measure` block is an "Open Container" indexed by an integer (Time Slice).
* **Behavior:** If `measure 1` is defined in `strings.ten` and also in `winds.ten`, the compiler **MERGES** the contents into a single internal Measure Object at `Index 1`.
* **Conflict Resolution:**
* **Event Data:** Merged additively. Content from `strings` and `winds` will coexist in the final system.
* **Metadata:** Must be consistent. If `strings.ten` declares `time: 4/4` and `winds.ten` declares `time: 3/4` for the *same measure index*, the compiler **MUST** throw a **Meta Mismatch Error (E1601)**.

### 16.3 Scope & Visibility

* **Definitions:** `def` statements inside an imported file populate the **Global Symbol Table**.
* *Constraint:* The `def` must be processed before any `measure` block attempts to use that Staff ID. Therefore, `import "setup.ten"` should typically appear at the top of the Master Linker.
* **Variables:** Variables defined in the root of an imported file become **Global**.
* *Best Practice:* To avoid namespace collisions, reusable libraries **SHOULD** use unique prefixes for their variables (e.g., `$std_drum_vol`).

### 16.4 Project Structure Standards

For interoperability and version control (Git), the following directory structure is the **Normative Standard** for complex projects:

* **`score.ten`**: The Master Linker. Contains `meta` (Global settings) and `import` statements. No musical logic.
* **`def/`**: Directory for definition files (e.g., `instruments.ten`).
* **`src/`**: Directory for logic files (e.g., `strings.ten`, `winds.ten`).
* **`lib/`**: Directory for shared macros and variables.

```tenuto
%% score.ten
tenuto {
  meta { title: "Symphony No. 1" }
  
  %% 1. Load Physics
  import "def/orchestra.ten"
  
  %% 2. Load Logic (Merged Additively)
  import "src/movement_1.ten"
  import "src/movement_2.ten"
}

```

---

## 17. Advanced Engraving Controls

Tenuto relies on a sophisticated **Inference Engine** to handle standard engraving rules (adhering to conventions like Gould's *Behind Bars*). However, professional scores often require manual overrides to solve visual collisions, articulate phrasing, or notate contemporary techniques. These overrides are applied as **Attributes**.

### 17.1 Manual Beaming

By default, the engine groups beams based on the Time Signature and Metric Grid. Manual overrides take precedence over automatic grouping.

* **`.bm` (Beam Start):** Begins a beam group at this event.
* **`.bme` (Beam End):** Terminates the beam group *after* this event.
* **`.bmb` (Beam Break/Sub-beam):** Forces a secondary beam break (e.g., splitting a group of 16ths into 2+2) without breaking the primary beam.

**Syntax:**

```tenuto
%% Beam across a rest (Rest is included in the beam group)
c8.bm d8 r8 e8.bme

```

#### 17.1.1 Feathered Beams (Contemporary)

To indicate distinct accelerando or ritardando within a beamed group:

* **`.bm(feather_accel)`**: Beams fan outward (wider to narrower separation).
* **`.bm(feather_rit)`**: Beams fan inward (narrower to wider separation).

#### 17.1.2 Stemlets

To draw short stems on rests within a beam group (facilitating rhythmic reading in complex syncopation):

* **`.stemlet`**: Applied to a Rest event (`r`) inside a beam group.

### 17.2 Stem Direction & Length

Stem direction is calculated based on the note's position on the staff and its Voice ID.

* **Direction:** `.up`, `.down`, `.auto`.
* **Length:** `.stem_len(Float)`.
* *Value:* Relative to default length (1.0 = standard 3.5 spaces).
* *Usage:* Essential for avoiding collisions in dense polyphonic passages or extending stems to meet cross-staff beams.

### 17.3 Curve Geometry (Slurs & Ties)

While the engine attempts to place curves to avoid collisions, complex voicing requires manual hints.

* **Direction:** `.slur_above`, `.slur_below`, `.tie_above`, `.tie_below`.
* **Shape:** `.slur(flat)` (Bracket style) or `.slur(dotted)` (Editorial/Phrasing variant).

### 17.4 Cross-Staff Beaming Interaction

When using the `.cross(Target_Staff_ID)` attribute (defined in Section 10) within a beamed group:

1. **Beam Ownership:** The beam belongs to the **Source Staff** (where the logic is defined). All properties (color, visibility) are inherited from the source.
2. **Rendering:** The renderer **MUST** calculate a beam slope that connects the noteheads across the visual distance of the two staves.
3. **Knee Beams:** If the interval is large, the renderer **SHOULD** automatically employ "Knee Beams" (changing slope direction at the staff crossing point) unless overridden by a manual slope directive.

### 17.5 Invisible Elements (Spacers)

To adjust spacing without printing ink (e.g., to align lyrics or create gap placeholders in educational worksheets):

* **`.hide`**: The event takes up time and horizontal space but renders no ink (Invisible Note/Rest).
* **`.null`**: The event takes up time but **NO** horizontal space (Zero-width). Useful for anchoring text at a specific time without disrupting the layout.

---

## 18. Ornamentation & Lines

This section defines symbols that modify the pitch content (Ornaments), articulate chords (Arpeggios), or visually connect sequential events (Lines). Unlike Section 17 (Layout), these attributes have significant **Audio Semantics**—they alter the synthesized output (e.g., adding notes, sliding pitch, or repeating attacks).

### 18.1 Decorators (Atomic Ornaments)

Decorators are attributes attached to a single event. They imply a specific alteration of the performed pitch or rhythm.

| Token | Name | Audio Semantics | Arguments |
| --- | --- | --- | --- |
| `.tr` | Trill | Rapid alternation ( 2nd) with upper neighbor. | `.tr(flat)`, `.tr(sharp)` |
| `.mord` | Upper Mordent | Main  Upper  Main. |  |
| `.mord_inv` | Lower Mordent | Main  Lower  Main. |  |
| `.turn` | Turn | Upper  Main  Lower  Main. | `.turn(sharp, flat)` |
| `.prall` | Pralltriller | Short trill / Snap. |  |

* **Trill Extension:** To draw a wavy extension line (spanner) after the symbol, use `.tr_ext`.
* *Example:* `c1.tr.tr_ext` implies the trill continues for the full duration of the whole note.

### 18.2 Arpeggiation & Tremolo

Modifications to the attack envelope or repetition rate of a chord or note.

* **`.arp`**: Standard wavy vertical line. Notes in the chord are played sequentially (Strum) from bottom to top.
* **`.arp(down)`**: Arpeggiate top-to-bottom.
* **`.trem(N)`**: Single-note tremolo. `N` = number of slashes (e.g., 3 for unmeasured/32nd notes).
* *Audio:* Repeats the note at the specified subdivision rate.

### 18.3 Connective Spanners (Glissando)

Connective spanners are attributes applied to a **Source Event**. The engine automatically draws the line to the **Target Event** (the immediate next note in the same voice).

**Syntax:** `Source.Attribute Target`

* **`.gliss`**: Straight line. Continuous pitch slide (MIDI Pitch Bend).
* **`.port`**: Portamento. Often curved. Nuanced slide characteristic of voice or strings.
* **`.fall`**: Jazz fall-off. No specific target pitch (Target is implicit silence/release).
* **`.doit`**: Jazz slide-up. No specific target pitch.
* **`.fingered_trem`**: Tremolo between two distinct pitches. Applied to the first note.
* *Example:* `c2.fingered_trem e2` renders as two whole notes with tremolo bars connecting them. The total duration is shared between the two (they are not played sequentially).

### 18.4 State Lines (Ottava & Pedal)

For lines that span arbitrary durations (potentially across measures), Tenuto uses **State Toggles**.

#### 18.4.1 Ottava (Octave Shift)

Modifies both the visual display (bracket) and the playback pitch.

* **Start:** `.8va` (Octave Up), `.8vb` (Octave Down), `.15ma` (Two Octaves Up).
* **End:** `.loco` (Return to written pitch).
* **Scope:** Applied to the Staff context. The shift persists until the `.loco` token is encountered.

#### 18.4.2 Piano Pedal

* **Start:** `.ped` (Pedal Down / Sustain On).
* **Change:** `.ped_change` (Lift and re-press instantly).
* **End:** `.ped_up` (Pedal Up / Sustain Off).

---

## 19. Microtonality & Tuning Systems

Tenuto supports non-12TET (Equal Temperament) tuning systems as native elements of the language. This allows for the precise notation of Maqam, Just Intonation, Xenharmonic scales, and historic temperaments.

### 19.1 Extended Accidental Syntax

For systems based on subdivisions of the tone (Quarter-tones), Tenuto extends the standard accidental grammar defined in Section 6.

| Token | Name | Interval Deviation | Visual Symbol |
| --- | --- | --- | --- |
| `qs` | Quarter Sharp | +50 cents | Sharp with one vertical stroke (𝄲) |
| `qf` | Quarter Flat | -50 cents | Backwards flat (𝄳) |
| `tqs` | Three-Quarter Sharp | +150 cents | Sharp with three strokes (𝄰) |
| `tqf` | Three-Quarter Flat | -150 cents | Backwards double flat (𝄱) |

* **Usage:** `c4qs` (C Quarter-Sharp).
* **Audio Semantics:** The playback engine maps these symbols to the exact logarithmic midpoint between standard semitones relative to the current tuning system.
* **Stickiness:** Like standard accidentals, these reset at the barline unless the Key Signature dictates otherwise.

### 19.2 Cent Deviation (Physics Override)

For precise retuning that does not align with standard symbols (e.g., spectralism, tuning to the 5th partial, or fine-tuning samples), pitch can be modified by specific **Cent** values ( semitone).

**Syntax:** `Pitch + Cents` or `Pitch - Cents`

* **Granularity:** Integers or Floats.
* **Additive Logic:** Deviations are additive to the base pitch *and* its accidental.
* *Example:* `c#4+10` means "Start at C#, then add 10 cents."
* **Rendering:** The renderer **SHOULD** place the cent deviation value as a small number above the note, or use an arrow if the deviation is small (< 33 cents).

### 19.3 Tuning Arrows (Helmholtz-Ellis / Just Intonation)

For systems relying on commatic shifts (Syntonic comma, Pythagorean comma):

* **`.arrow_up`**: Raises pitch by one syntonic comma ( cents).
* **`.arrow_down`**: Lowers pitch by one syntonic comma.
* **`.slash_sharp`**: Turkish/Maqam sharp (approx +1/9 tone).

### 19.4 Absolute Frequency Literals

For electronic music, acoustics testing, or drone music where "Note Names" are irrelevant abstractions, Tenuto supports raw frequency input.

**Syntax:** `hz(Frequency)`

* **Example:** `hz(440):1 hz(442):1` creates a 2Hz beating effect over whole notes.
* **Rendering:** Renders as a notehead without a specific staff position (or on a single-line staff) labeled with the frequency value, unless a specific `clef` allows mapping frequency to vertical position.

### 19.5 External Tuning Maps (.scl / .kbm)

To support complex arbitrary tuning systems (19-TET, Slendro, Pelog, Partch), Tenuto adopts the industry-standard **Scala** format.

**Syntax:** `meta { tuning_file: "Path/To/Scale.scl", tuning_root: Pitch }`

* **Behavior:** The compiler maps the linear diatonic steps of the staff (C, D, E...) to the steps defined in the SCL file, anchored at `tuning_root`.
* **Semantic Separation:** When a custom tuning map is active:

1. **Visual:** The notation remains "nominal" (what the player reads/fingers).
2. **Audio:** The frequency is determined strictly by the map.

* *Note:* This decouples the "Written Note" from the "Sounding Pitch," essential for instruments with fixed but non-standard intonation (e.g., a prepared piano or a specific Gamelan metallophone).

---

## 20. Visual Styling (The Theme Engine)

Tenuto strictly enforces the separation of **Musical Data** (Pitch/Rhythm) from **Visual Presentation** (Ink/Fonts). The **Theme Engine** controls the rendering layer, allowing the same logical score to be displayed in drastically different visual styles (e.g., Classical vs. Jazz) without altering a single line of the source code.

### 20.1 Theme Profiles

Global visual settings are applied via the `theme` key in the `meta` block.

**Syntax:** `meta { theme: "ProfileID" }`

* **`"standard"`**: Traditional engraving (e.g., Bravura-style). High contrast, serif fonts, straight beams.
* **`"jazz"`**: Handwritten appearance (e.g., Petaluma-style). Ink-pen aesthetic, "hand" font for text, slightly irregular line widths.
* **`"educational"`**: Larger noteheads, simplified symbols, color-friendly spacing.
* **`"dark"`**: Inverted colors (White ink on black background) optimized for digital screen reading in low-light environments.

### 20.2 Color Syntax

Events can be colored for educational purposes (e.g., Boomwhackers, Kodály) or analytical highlighting.

**Syntax:** `Event.color("Hex")` or `Event.ColorToken`

* **Tokens:** `.red`, `.blue`, `.green`, `.orange`, `.purple`, `.black`, `.white`, `.grey`.
* **Hex:** `.color("#FF0000")`.
* **Scope:** The color attribute applies to the **Notehead, Stem, and Beam** associated with the event. It does **NOT** propagate to attached Lyrics or Dynamics unless explicitly applied to those objects.

### 20.3 Notehead Overrides

The notehead shape conveys performance semantics (e.g., percussion technique, harmonics) or pedagogical information.

**Syntax:** `Event.head("ShapeID")`

| Shape ID | Visual | Semantic Meaning |
| --- | --- | --- |
| `"normal"` | Oval | Standard pitched note. |
| `"x"` | Cross | Percussive hit, Spoken word, Ghost note, or Dead note. |
| `"diamond"` | Diamond | Harmonic (Natural or Artificial). |
| `"triangle"` | Triangle | Percussion instrument (e.g., Triangle) or emphatic accent. |
| `"slash"` | Slash | Rhythmic notation (Pitch is indeterminate or irrelevant). |
| `"none"` | Empty | Invisible notehead (Stem and Beam remain visible). |

### 20.4 Text & Font Families

To ensure portability across operating systems and eras, Tenuto uses **Generic Font Families** rather than specific file references.

**Syntax:** `meta { font_face: "Family" }`

* **Families:**
* `"serif"`: Formal, traditional (e.g., Times New Roman).
* `"sans"`: Modern, clean (e.g., Helvetica).
* `"mono"`: Fixed-width (e.g., Courier).
* `"hand"`: Handwritten style (e.g., JazzText).
* **Behavior:** The renderer is responsible for mapping these generics to the best available font on the host system.

### 20.5 Visibility & Cue Notes

Attributes to control the rendering presence and scale of objects.

* **`.hidden`**: The object is effectively invisible (Opacity 0) but still occupies layout space.
* *Usage:* "Fill in the blank" worksheets or aligning lyrics to a rhythm that shouldn't be seen.
* **`.cue`**: Renders the event at a reduced size (normatively **70%** of the global staff size).
* *Usage:* Cue notes, Ossia bars, or grace notes that are measured (not timeless).

## 21. Advanced MIDI & Automation

For high-fidelity playback, Tenuto exposes raw control over the synthesis engine. While `vol` and `pan` (Section 14) are high-level abstractions, this section deals with the low-level protocols (MIDI 1.0, MIDI 2.0, or OSC) used to drive virtual instruments. This allows for the precise manipulation of timbre, expression, and synthesizer parameters directly from the score logic.

### 21.1 Control Change (CC) Messages

Discrete MIDI CC messages are attached to events using the `.cc()` modifier.

**Syntax:** `Event.cc(ControllerNumber, Value)`

* **ControllerNumber:** Integer (0-127).
* **Value:** Integer (0-127).
* **Timing:** The message is sent immediately *before* the Note On event.

### 21.2 Automation Curves (Ramps)

To create smooth changes (e.g., a crescendo swell via Expression) over the duration of a note or region, use an array of values `[Start, End]`.

**Syntax:** `Event.cc(Number, [Start, End], CurveType?)`

* **CurveType:**
* `"linear"` (Default): Constant rate of change.
* `"exp"`: Exponential. Recommended for Volume/Expression swells to match human hearing (Decibels).
* `"log"`: Logarithmic.
* **Duration:** The ramp lasts for the exact duration of the host event.
* **Density:** The resolution of the generated data is implementation-dependent (Recommended: 1 event per 10ms).

```tenuto
%% Expression (CC 11) swells exponentially from 0 to 100
c4:1.cc(11, [0, 100], "exp")


```

### 21.3 Keyswitches (Articulation Mapping)

Keyswitches are silent notes used to trigger specific sample layers in libraries (e.g., Kontakt, VSL). To keep the score clean, these are defined in the Instrument Definition.

**1. Definition:**
In the `def` block, map a custom attribute name to a MIDI Note Number.

```tenuto
def vln "Violin" keyswitch={
  arco: 24,  %% C0
  pizz: 25   %% C#0
}


```

**2. Usage:**
The defined keys become valid attributes for that staff.

```tenuto
vln: c4:4.pizz  %% Triggers Note 25, then plays C4


```

### 21.4 Pitch Bend & Aftertouch

To control pitch and pressure dynamically:

* **`.bend(Cents)`**: Sends Pitch Bend data.
* *Range:* `+/-` Cents (e.g., `.bend(200)` is +2 semitones).
* *Ramp:* `.bend([0, 200])` bends up over the note's duration.
* **`.press(Value)`**: Channel Pressure (Aftertouch). Affects the whole channel.
* **`.polypress(Value)`**: Polyphonic Aftertouch. Affects only this specific note (if supported by the synth/MPE).

### 21.5 Program Changes

To switch instrument patches mid-stream:

**Syntax:** `Event.pc(ProgramNumber)` or `Event.bank(MSB, LSB).pc(ProgramNumber)`

* *Example:* `c4.pc(10)` switches to the 10th patch before playing C4.
* *Note:* For large orchestral templates, it is **RECOMMENDED** to use Keyswitches (21.3) or separate Staves rather than Program Changes, as PC messages can cause audio glitches in some samplers.

---

## 22. Compiler Directives & Debugging

These directives control how the Tenuto compiler processes the source text, validates logic, and reports issues. They are essential for maintaining large scores, ensuring archival stability across software versions, and debugging complex logic.

### 22.1 Language Versioning

To ensure "Deep Time" durability, a file **MUST** declare the version of the specification it adheres to. This prevents future compilers from misinterpreting syntax that may be deprecated or redefined in later epochs.

**Syntax:** `meta { tenuto_version: "Major.Minor" }`

* **Location:** This attribute **SHOULD** appear in the first `meta` block of the root scope.
* **Validation:** If the compiler supports version 3.0 but encounters `tenuto_version: "2.0"`, it **MUST** activate its "Legacy 2.x Parser" mode or emit a fatal incompatibility error if backward compatibility is not supported.

### 22.2 Strict Mode

By default, Tenuto is **Lenient**. It attempts to auto-correct common mistakes (e.g., auto-closing beams at barlines, inferring missing durations). However, for archival quality or library development, **Strict Mode** forces explicit definitions.

**Syntax:** `meta { strict: true }`

* **Constraints Enforced:**

1. **Beaming:** Beams must be explicitly closed (`.bme`) before barlines.
2. **Stickiness:** "Sticky" attributes (Duration, Octave) do **NOT** persist across measure boundaries. Every measure must initialize its state explicitly.
3. **Sync:** Voice Group durations must match exactly (no auto-padding with rests).

### 22.3 Warning Control

Specific compiler warnings can be suppressed for intentional deviations (e.g., a deliberate rhythm clash or lyric mismatch).

**Syntax:** `meta { suppress: ["Code1", "Code2"] }`

* **Scope:** Lexical. Applies to the current block and its children.
* **Usage:** `meta { suppress: ["W1201"] }` silences Lyric Mismatch warnings for a specific measure.

### 22.4 Conditional Compilation

To support multi-target rendering (e.g., generating a Conductor's Score vs. a Violin Part vs. an MP3) from a single source file.

**Syntax:** `if (Condition) { ... }`

* **Environment Variables:** The compiler provides standard variables for inspection:
* `target`: `"score"` (PDF), `"audio"` (MIDI/WAV), `"part"` (Individual Part).
* `part_id`: The ID of the staff currently being rendered (e.g., `"vln"`).
* `debug`: Boolean.

```tenuto
measure 1 {
  vln: c4 d e f |
  
  if (target == "audio") {
    %% Reinforce bass only in audio render; invisible in PDF
    vln: c2:1.vol(0.5).hidden | 
  }
}


```

### 22.5 Debugging Tools

For analyzing compiler state during logic development.

* **`.trace`**: Attribute. Dumps the current **State Vector** (Tick, Octave, Duration, Velocity) of the event to the compiler log/console.
* *Example:* `c4.trace` prints `[Tick: 1920, Pitch: C4, Dur: :4, Vel: 100]`.
* **`meta { error: "Message" }`**: Halts compilation immediately with a custom user message. Useful for validating macro arguments.

---

## 23. The Standard Library

To ensure interoperability and reduce boilerplate code, every Tenuto compiler **MUST** implement the following **Standard Library** of constants. These identifiers are implicitly available in the Global Scope of every document and do not require import statements.

### 23.1 Standard Clefs

Supported values for the `clef` attribute (used in definitions).

| Identifier | Symbol | Center Line (from bottom, 1-5) |
| --- | --- | --- |
| `treble` | G Clef | 2 |
| `bass` | F Clef | 4 |
| `alto` | C Clef | 3 |
| `tenor` | C Clef | 4 |
| `perc` | Neutral | 3 |
| `tab` | Tablature | N/A (Lines determined by String Count) |

### 23.2 Standard Tunings

Pre-defined Pitch Arrays for the `tuning` attribute (used in `style=tab`).

| Identifier | Value (Low to High) | Description |
| --- | --- | --- |
| `guitar_std` | `[E2, A2, D3, G3, B3, E4]` | Standard 6-string Guitar |
| `guitar_drop_d` | `[D2, A2, D3, G3, B3, E4]` | Drop D Guitar |
| `bass_std` | `[E1, A1, D2, G2]` | Standard 4-string Bass |
| `bass_5` | `[B0, E1, A1, D2, G2]` | Standard 5-string Bass |
| `uke_std` | `[G4, C4, E4, A4]` | Ukulele (High G / Re-entrant) |
| `violin_std` | `[G3, D4, A4, E5]` | Violin |
| `cello_std` | `[C2, G2, D3, A3]` | Cello |

### 23.3 Percussion Maps

Pre-defined Key Maps for `style=grid`.

**`gm_kit`** (General MIDI Standard mapping)

* **Drums:** `k` (Kick), `s` (Snare), `ss` (Side Stick), `t1`/`t2`/`t3` (Toms High/Mid/Low).
* **Cymbals:** `h` (Closed Hat), `ho` (Open Hat), `ph` (Pedal Hat), `c` (Crash 1), `r` (Ride 1), `rb` (Ride Bell).

### 23.4 Color Constants

Standard web-safe colors for use with the `.color()` attribute.

* **Values:** `red` (#FF0000), `blue` (#0000FF), `green` (#008000), `orange` (#FFA500), `purple` (#800080), `black` (#000000), `white` (#FFFFFF), `grey` (#808080).

### 23.5 General MIDI Patch Constants

String constants mapping to the standard General MIDI Sound Set (Program Numbers 0-127). Using these aliases is preferred over raw integers for readability.

* `gm_piano` (0: "Acoustic Grand Piano")
* `gm_epiano` (4: "Electric Piano 1")
* `gm_organ` (16: "Drawbar Organ")
* `gm_guitar` (24: "Acoustic Guitar (Nylon)")
* `gm_bass` (32: "Acoustic Bass")
* `gm_violin` (40: "Violin")
* `gm_strings` (48: "String Ensemble 1")
* `gm_choir` (52: "Choir Aahs")
* `gm_trumpet` (56: "Trumpet")
* `gm_sax` (65: "Alto Sax")
* `gm_flute` (73: "Flute")
* `gm_kit` (Channel 10 Default)

---

## 24. Error Reference

The Tenuto compiler emits specific codes to aid debugging. Implementations **MUST** use these standardized codes to ensure that error messages are searchable, consistent across different tools, and machine-parsable by IDEs and CI/CD pipelines.

### 24.1 Severity Levels

1. **Fatal (F):** Compilation halts immediately. The file cannot be parsed.
2. **Error (E):** Compilation fails to produce a valid artifact. The parser may attempt to continue scanning to report additional errors, but the output is unusable.
3. **Warning (W):** Compilation proceeds. The compiler has applied an **Auto-Correction**. The output is valid but may not match the user's intent.

### 24.2 1000-Series: Lexical & Meta Errors

* **E1001: Malformed Token.** The parser encountered a character sequence that violates the grammar (e.g., illegal symbols inside a pitch token).
* **E1002: Unbalanced Delimiter.** A block `{`, `[`, or `(` was opened but never closed.
* **E1004: Version Incompatible.** The file requests a specification version (`tenuto_version`) higher than the compiler supports.
* **E1005: Encoding Error.** The source file is not valid UTF-8.

### 24.3 2000-Series: Definition & Import Errors

* **E2001: Undefined Identifier.** Attempting to use a Staff ID or Variable (`$var`) that has not been defined in the current scope.
* **E2002: Duplicate Definition.** Attempting to register a Staff ID or Variable Name that already exists.
* **E2003: Import Failure.** The referenced file path could not be resolved or read.
* **E2004: Circular Import.** A dependency loop (e.g., A imports B, B imports A) was detected.

### 24.4 3000-Series: Time & Structure Errors

* **E3001: Time Overflow.** The total duration of events in a voice exceeds the capacity of the Time Signature.
* **E3002: Voice Sync Failure.** The total duration of voices within a Voice Group (`v1`, `v2`) do not match.
* **E3003: Tuplet Ratio Error.** The contents of a tuplet block cannot mathematically fit into the declared ratio (e.g., trying to fit 5 quarters into a `3:2` bracket of 8th notes).
* **E3004: Structure Mismatch.** Different staves define conflicting structural markers (e.g., `vln` has `|:` while `vlc` has `|`) at the same absolute tick.
* **W3005: Pickup Mismatch.** The duration of the anacrusis measure does not match the declared `pickup` metadata.
* **W3006: Lyric Count Mismatch.** The number of lyric syllables defined in the `lyrics` block does not match the number of valid note events in the target measure.

### 24.5 4000-Series: Attribute & Value Errors

* **W4001: Open Beam.** A beam started with `.bm` was not closed before a barline. (Compiler auto-closes it).
* **E4002: Invalid Type Cast.** Passing an incompatible type (e.g., String) to a numeric parameter.
* **W4003: Value Out of Range.** A value exceeded its allowed bounds (e.g., `vol: 1.5` or `midi: 128`) and was clamped.
* **E4004: Invalid Percussion Key.** Using a key character (e.g., `x`) that is not defined in the instrument's `map`.

### 24.6 5000-Series: Macro & Pre-Processor Errors

* **E5001: Circular Reference.** A macro definition calls itself recursively.
* **E5002: Recursion Limit Exceeded.** Macro expansion depth exceeded the safety limit (Standard: 64).
* **E5003: Argument Mismatch.** A macro was invoked with an incorrect number of arguments.

### 24.7 9000-Series: System & Implementation Errors

These codes are reserved for the compiler environment itself.

* **F9001: IO Error.** Write permission denied or disk full.
* **F9002: Internal Error.** The compiler encountered an unrecoverable state (crash/panic).
* **F9003: Memory Limit Exceeded.** The score logic structure is too large for the host system RAM.

---

## 25. Implementation Guidelines

To ensure that valid Tenuto files produce identical logical and auditory output across different software implementations (CLIs, DAWs, Web Libraries) and operating systems, compilers **MUST** adhere to the following architectural and mathematical standards.

### 25.1 File Standards

* **Extension:** Source files **SHOULD** use the `.ten` extension.
* **Encoding:** Files **MUST** be encoded in **UTF-8**.
* **Normalization:** The compiler **MUST** normalize all identifiers and string literals to **Unicode NFC** (Normalization Form C) before parsing. This prevents "Variable Not Found" errors caused by visually identical but byte-distinct characters (e.g., on macOS vs. Windows filesystems).

### 25.2 Pitch Standards (The "Middle C" Rule)

To resolve the historical ambiguity between hardware manufacturers (e.g., Yamaha C3 vs. Roland C4):

* **Standard:** **C4** is Normatively Defined as **MIDI Note Number 60** (approx. 261.63 Hz).
* **Reference:** **A4** is Normatively Defined as **MIDI Note Number 69** (440.0 Hz default).
* **Compliance:** All internal pitch calculations must be relative to these anchors.

### 25.3 Temporal Logic (Rational Arithmetic)

Musical time relies on precise subdivisions (e.g., Triplets = 1/3) that cannot be accurately represented by standard IEEE 754 binary floating-point numbers (where 1/3  0.333333...).

* **Requirement:** The compiler **MUST** use **Rational Arithmetic** (Fraction structures storing explicit Numerator/Denominator integers) for all internal time, duration, and position calculations.
* **Rendering:** Conversion to Floating Point (for Audio buffers) or Integer Ticks (for MIDI export) **MUST** occur only at the final **Rendering Stage** to minimize cumulative rounding errors.

### 25.4 Forward Compatibility (Graceful Degradation)

To ensure that older compilers can robustly handle files created by newer versions of the specification:

* **Unknown Attributes:** If the parser encounters an event attribute it does not recognize (e.g., `.future_feature`), it **SHOULD** ignore the attribute, emit a **Warning**, and continue processing the host event. It **MUST NOT** treat this as a Fatal Syntax Error.
* **Unknown Metadata:** Unknown keys in `meta` blocks **SHOULD** be ignored (read-only) or preserved (read-write).

### 25.5 The Compilation Pipeline

A compliant compiler **SHOULD** follow this logical execution flow:

1. **Lexing:** Convert UTF-8 Stream  Token Stream.
2. **Expansion (Pre-Process):** Execute `import` directives and expand `macro` calls. (Recursion limits enforced here).
3. **Definition (Context):** Scan `def` and `var` blocks to populate the Global Symbol Table.
4. **Linearization:** Convert the hierarchical `measure` / `repeat` structures into a linear timeline of absolute events.
5. **Validation:** Check for Voice Sync (E3002), Range Constraints, and Logic consistency.
6. **Rendering:** Transpile the validated linear stream to the target format (PDF, MIDI, MusicXML).

---

## 26. Formal Grammar (EBNF)

This section provides the **Normative Syntax** of Tenuto v2.0 using Extended Backus-Naur Form (EBNF). In the event of a contradiction between the prose description in previous sections and this grammar, this grammar takes precedence as the authority for parser implementation.

### 26.1 Lexical Tokens (Terminals)

These patterns define the primitive tokens generated by the Lexer.

```ebnf
/* Primitives */
IDENTIFIER  ::= [a-zA-Z_] [a-zA-Z0-9_]*
INTEGER     ::= [0-9]+
FLOAT       ::= [0-9]+ "." [0-9]+
STRING      ::= '"' [^"]* '"'

/* Music Literals */
/* Matches c4, c#4, cqs4 (quarter sharp), etc. */
PITCH_LIT   ::= [a-g] [qs|qf|tqs|tqf|#|b]* [0-9]?  

/* Matches :4 (quarter), :8. (dotted eighth) */
DURATION    ::= ":" [0-9]+ ("." [0-9]+)?           

/* Ignored Tokens */
WHITESPACE  ::= [ \t\r\n]+
COMMENT     ::= "%%" [^\r\n]*


```

### 26.2 High-Level Structure

```ebnf
Score       ::= Header? TopLevel*

Header      ::= "tenuto" STRING? /* Version Declaration */

TopLevel    ::= Import
              | Definition
              | VariableDecl
              | MacroDef
              | Block
              | MetaBlock

Import      ::= "import" STRING

Definition  ::= "def" IDENTIFIER STRING? AttributeList?

VariableDecl::= "var" IDENTIFIER "=" Value

/* Macro Definition: macro Name(Args) = { Body } */
MacroDef    ::= "macro" IDENTIFIER "(" ParamList? ")" "=" "{" Voice "}"

Block       ::= Measure | Repeat | Volta

Measure     ::= "measure" (INTEGER | IDENTIFIER)? AttributeList? "{" Logic* "}"

Repeat      ::= "repeat" INTEGER? "{" Logic* "}"

Volta       ::= "volta" Range "{" Logic* "}"


```

### 26.3 Logic & Events

```ebnf
Logic       ::= Assignment
              | MetaBlock
              | Conditional

/* Assignment: vln: c4 d e | */
Assignment  ::= IDENTIFIER ":" VoiceGroup

/* Voice Group must end with a pipe to confirm sync */
VoiceGroup  ::= Voice ("|" Voice)* "|"

Voice       ::= (Event | Tuplet | MacroCall)*

Event       ::= (Note | Chord | Rest | Space | Percussion) Duration? Attribute*

Note        ::= PITCH_LIT
Chord       ::= "[" PITCH_LIT+ "]"
Rest        ::= "r"
Space       ::= "s"
Percussion  ::= IDENTIFIER

Tuplet      ::= "tuplet" "(" INTEGER ":" INTEGER ")" "{" Voice "}"

/* Macro Call: $Name(Args) or $Name+2 */
MacroCall   ::= "$" IDENTIFIER ("(" ArgList ")")? Transposition?

Transposition ::= ("+" | "-") INTEGER


```

### 26.4 Attributes & Data Types

```ebnf
/* Attribute: .vol(80) or .stacc */
Attribute   ::= "." IDENTIFIER ("(" ArgList ")")?

MetaBlock   ::= "meta" "{" KeyValueList "}"

Conditional ::= "if" "(" Expression ")" "{" Logic* "}"

/* Data Structures */
ArgList     ::= Value ("," Value)*
KeyValueList::= (IDENTIFIER ":" Value ","?)*

Value       ::= INTEGER | FLOAT | STRING | IDENTIFIER | Array | Map

Array       ::= "[" Value ("," Value)* "]"
Map         ::= "{" KeyValueList "}"


```

---

## 27. Interoperability & Exchange

Tenuto is designed to sit in the center of the modern music toolchain, serving as a high-level abstraction that can be compiled down to presentation formats (PDF/SVG) or interchange formats (MusicXML, MIDI). This section defines the **Normative Mapping** rules to ensure consistent export behavior across different compilers.

### 27.1 MusicXML 4.0 Mapping

When exporting to MusicXML, the compiler **MUST** map Tenuto structures as follows to ensure visual fidelity in notation software (e.g., Dorico, Finale, MuseScore).

1. **Root Structure:**

* The `tenuto` block maps to the root `<score-partwise>` element.
* Metadata (`title`, `composer`) maps to `<work><work-title>` and `<identification><creator>`.

2. **Definitions:**

* Each `def` statement maps to a `<score-part>` element in the `<part-list>`.
* `group` blocks map to `<part-group type="start">` and `<part-group type="stop">`.

3. **Logic & Time:**

* `measure` blocks map to sequential `<measure>` elements.
* **Voices:** Tenuto voice IDs (`v1`, `v2`) map to MusicXML `<voice>` integers (1, 2).

4. **Event Data:**

* `style=standard` events map to `<note><pitch>`.
* `style=tab` events map to `<note><notation><technical><fret>` and `<string>`.
* `microtonality` (`qs`, `qf`) maps to `<pitch><alter>` (decimal values, e.g., 0.5) and `<accidental>` tags.

### 27.2 MIDI 1.0 / 2.0 Mapping

When exporting to Standard MIDI Files (SMF), the compiler **MUST** adhere to these resolutions to ensure consistent playback.

1. **Timing & Resolution:**

* Files **SHOULD** use a resolution of **480 PPQ** (or higher) to accurately capture complex tuplets.
* Duration Multipliers (`:1 * 4`) must be unrolled into actual time.

2. **Track Layout:**

* Each `def` becomes a dedicated MIDI Track.
* `style=grid` (Percussion) **MUST** default to MIDI Channel 10 unless the `channel` attribute is explicitly defined.

3. **Articulation Mapping (Gate Times):**

* To ensure articulation is audible:
* `.stacc`  Reduce Note-On duration to 50% of the notated value.
* `.ten`  Maintain 100% duration (Legato).
* *Default:* 90% duration (to simulate natural phrasing/breath).

4. **Dynamics:**

* Maps `pppp` (Velocity 16) through `ffff` (Velocity 127). Standard `mf` should map to 80.

---

## 28. Reference Example (The "Kitchen Sink")

The following example demonstrates the integration of the V2 engines (Standard, Tab, Percussion, Macros, and Logic) into a single valid document. It serves as a validation test for compliant compilers.

```tenuto
tenuto {
  meta { 
    title: "Tenuto V2 Reference", 
    tempo: 130, 
    style: "jazz",
    tenuto_version: "2.0"
  }

  %% 1. DEFINITIONS (The Physics)
  group "Rhythm Section" symbol=bracket {
    def sax "Tenor Sax" style=standard clef=treble transpose=-14 keyswitch={ growl: 30 }
    def gtr "Guitar"    style=tab      tuning=guitar_std
    def drm "Drum Kit"  style=grid     map=gm_kit
  }

  %% 2. MACROS (The Pre-Processor)
  %% A generic riff macro that accepts a root pitch argument
  macro LickA(root) = { $root:16 d eb f g f eb d }

  %% 3. LOGIC (The Flow)
  measure 1 {
    meta { time: 4/4 }
    
    %% SAX: Sticky duration + Articulation + Microtonality
    sax: c4:4.acc dqs e c5.stacc |

    %% GUITAR: Tablature + Strumming + Ghost Notes
    gtr: [0-6 2-5 2-4]:4.down [x-6 x-5 x-4].up r:2 |

    %% DRUMS: Linear drumming pattern with Roll
    drm: k:8. s:16 k:8 s k s k s:4.roll(3) |
  }

  measure 2 {
    %% SAX: Polyphonic split
    sax: {
      v1: $LickA(c5) c5:2.fermata |
      v2: g4:1.fermata            |
    }
    
    %% GUITAR: Pitch bend technique (Quarter tone bend)
    gtr: 10-2:2.bu(quarter) 10-2.bd(0) |
    
    %% DRUMS: Fill
    drm: s:16 s s s t1 t1 t2 t2 c:1 |
  }
}


```

# Addendum A: Advanced Implementation & Extensions

**Version:** 1.1 (Extension to Tenuto 2.0)

**Status:** Normative

**Scope:** Real-Time Protocols, Binary Serialization, Cryptography, and Collaboration.

---

## A.1 Live Execution Model (REPL & Daemon)

To support live coding and interactive performance, Tenuto defines a standard runtime environment that persists state between compilation events.

### A.1.1 The Runtime Daemon (`tenutod`)

Implementations **SHOULD** provide a daemon process that exposes two primary interfaces:

1. **REPL Socket:** Accepts Tenuto code chunks (text or binary) via WebSocket or Unix Socket.
2. **Control API:** REST/OSC interface for querying state (e.g., `GET /v1/state/tempo`) without injecting logic.

### A.1.2 State Mutation & Timing

The `@sync` directive is expanded to the `@at` directive for precise scheduling.

**Syntax:** `@at(TimeSpec) Block`

* **Relative Timing:** `@at(+2beats)` queues execution for 2 beats from the current cursor.
* **Absolute Timing:** `@at(measure 12)` queues execution for the downbeat of Measure 12.
* **Timecode:** `@at(01:30.500)` queues execution for a specific SMPTE/Wall-clock time.

```tenuto
%% Queue a key change at the start of the next phrase
@at(measure 17) meta { key: "D" }


```

### A.1.3 Delta Updates

For efficient network transmission, the daemon accepts **Change Sets** rather than full file re-uploads.

```json
{
  "type": "delta",
  "target": "measure 5",
  "logic": "vln: c4 d e f |",
  "version": 3,
  "parent_hash": "a1b2c3..."
}


```

---

## A.2 Binary Format (.tenb)

For high-performance parsing and network transmission, Tenuto defines a canonical **Binary Encoding**.

* **Extension:** `.tenb`
* **MIME:** `application/x-tenuto-binary`
* **Endianness:** Little-Endian

### A.2.1 Chunk Types

The body consists of Type-Length-Value chunks.

| Hex ID | Name | Description |
| --- | --- | --- |
| `0x01` | **META** | JSON metadata (UTF-8). |
| `0x02` | **DEFS** | Binary instrument definitions struct. |
| `0x03` | **LOGIC** | Compressed Event Stream (See A.2.2). |
| `0x04` | **MACROS** | Macro definitions tree. |
| `0x05` | **THEME** | Embedded theme/font data. |
| `0x06` | **SAMPLE** | Embedded audio snippets (Optional). |
| `0x07` | **HASH** | Cryptographic integrity signatures. |

### A.2.2 Logic Chunk Structure

The logic stream is highly optimized for sequential reading.

**Header (12 bytes):**

* `Start Tick` (uint64)
* `End Tick` (uint64)
* `Staff ID` (uint16)

**Event Array:**
Each event is packed: `[Type][Flags][TickOffset][Data...]`

* **TickOffset:** Varint delta from the previous event (enables highly efficient packing).
* **Compression:** Logic chunks **MUST** be compressed using **Zstandard (zstd)** with a standard dictionary trained on the Tenuto corpus.

---

## A.3 Cryptographic Integrity & Archival

To serve as a "Deep Time" format, files must be verifiable against bit-rot and tampering.

### A.3.1 The Canonical Form Algorithm

Before hashing, the source must be normalized to ensure consistent signatures regardless of formatting.

1. **Strip:** Comments and whitespace (reduce to single spaces).
2. **Normalize:** Lowercase all identifiers; sort `meta` keys alphabetically.
3. **Expand:** Inline all imports and macros.
4. **Convert:** All durations to fractional representation (`:4.`  `:3/8`).

### A.3.2 The Integrity Block

Files **MAY** include a hash of the canonical form.

```tenuto
meta {
  integrity: {
    algorithm: "sha256",
    hash: "e3b0c44298fc1c149afbf4c8996fb924...",
    canonical_version: "2.0",
    signature: "gpg-signature-string"
  }
}


```

### A.3.3 Merkle Tree Structure

For large works (e.g., Operas), the integrity block **SHOULD** implement a Merkle Tree where each Measure or Movement is hashed individually. This allows a parser to identify exactly *where* corruption occurred without discarding the entire file.

---

## A.4 Feature Degradation Matrix

Renderers have varying capabilities (e.g., a simple MIDI player vs. a pro notation suite). Implementations **MUST** declare their Tier and adhere to the normative fallback rules.

| Tier | Capability | Fallback Behavior |
| --- | --- | --- |
| **Tier 1 (Basic)** | MIDI/Text Only | **Microtonality:** Round to nearest semitone.<br>

 |



**Noteheads:** All map to `normal`.





**Techniques:** Ignored. |
| **Tier 2 (Standard)** | Notation Editors | **Microtonality:** Pitch Bend or Accidentals.





**Noteheads:** Support `x`, `diamond`, `triangle`.





**Techniques:** Text labels. |
| **Tier 3 (Reference)** | Full Engine | **Microtonality:** Exact frequency synthesis.





**Noteheads:** Full SVG shape support.





**Techniques:** Sample switching / Modeling. |

---

## A.5 Error Correction (Leniency)

To ensure consistent behavior across "Lenient" compilers, auto-correction follows a defined **Leniency Ladder**.

### A.5.1 Severity Levels

* **Level 0 (Strict):** No correction. Fatal errors only.
* **Level 1 (Soft):** Auto-close beams at barlines; infer missing Octave 4 on initialization.
* **Level 2 (Aggressive):** Fix obvious typos (e.g., `:5` interpreted as `:4` if time sig is 4/4).
* **Level 3 (Creative):** Algorithmic gap-filling (e.g., Markov generation for empty measures). *Experimental.*

### A.5.2 The Correction Log

When a compiler applies a correction, it **MUST** generate a machine-readable log entry:

```json
{
  "level": "warning",
  "rule": "infer_duration_sync",
  "location": { "measure": 5, "tick": 960 },
  "original": "c4 d e |",
  "corrected": "c4:4 d:4 e:4 |",
  "confidence": 0.85
}


```

---

## A.6 Real-Time Collaboration Protocol

For multi-user editing, Tenuto defines the **OmniScore Synchronization Protocol (OSP)**.

### A.6.1 CRDT Model

The document state is managed as a **Conflict-Free Replicated Data Type (CRDT)**.

* **Registers:** Each Measure is a discrete register.
* **Lists:** Voices within measures are ordered lists with unique UUIDs.
* **Clocks:** State changes are ordered via Vector Clocks.

### A.6.2 Protocol Buffers Definition

```protobuf
message OSPMessage {
  enum Type {
    PATCH = 0;
    CURSOR_UPDATE = 1;
    STATE_SYNC = 2;
    CONFLICT_RESOLUTION = 3;
  }
  
  string client_id = 1;
  uint64 timestamp = 2;
  VectorClock vector_clock = 3;
  repeated Operation operations = 4;
}


```

---

## A.7 Implementation Checklist

To claim full compliance with Tenuto 2.0 + Addendum A, an implementation must:

* [ ] Parse and Generate valid `.ten` text and `.tenb` binary files.
* [ ] Validate Cryptographic Hashes (SHA-256) on load.
* [ ] Implement the `@at` scheduling directive.
* [ ] Expose the Correction Log via API or Console.
* [ ] Declare a Renderer Tier (1-3) and strictly follow degradation rules.
* [ ] Support WebSocket connections for the `tenutod` protocol.