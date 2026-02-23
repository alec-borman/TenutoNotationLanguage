# Tenuto API Reference

**Version:** 2.1.0 | **Crate:** `tenutoc` | **Status:** Deterministic LL(1)

---

## 📦 Crate Overview

The `tenutoc` crate serves as the reference implementation for the Tenuto v2.1.0 language. It is designed as a **modular, multi-stage pipeline**, allowing developers to hook into different stages of compilation:

| Module | Purpose | Transformation |
|--------|---------|----------------|
| `lexer` | High-performance tokenization | Source string → Token stream |
| `parser` | Deterministic LL(1) AST generation | Token stream → Score (AST) |
| `preprocessor` | Macro & Variable expansion | Score → Expanded Score |
| `ir` | The Inference Engine (Rational Time) | Expanded Score → Timeline |
| `midi` | Backend export | Timeline → MIDI Bytes |

---

## 📥 Installation

Add `tenutoc` to your `Cargo.toml`:

```toml
[dependencies]
tenutoc = { git = "https://github.com/alec-borman/TenutoNotationLanguage" }
```

---

## 🔡 Module: `tenutoc::lexer`

The Lexer module is the entry point for the compiler. It utilizes the **`logos`** crate to generate a high-speed, regex-based state machine for tokenization. In V2.1.0, the lexer handles compound sigils natively, enabling downstream deterministic parsing.

### Usage Example

```rust
use tenutoc::lexer::Token;
use logos::Logos;

let source = "pno: <[ v1: c4:4 ]>";
let mut lexer = Token::lexer(source);

assert_eq!(lexer.next(), Some(Ok(Token::Identifier("pno".into()))));
assert_eq!(lexer.next(), Some(Ok(Token::Colon)));
assert_eq!(lexer.next(), Some(Ok(Token::VoiceBracketStart)));
// ...
```

### Enum: `Token`

Represents the smallest semantic units of the Tenuto language.

#### Variant Categories

##### 1. Keywords (Case-Insensitive)

| Variant | Corresponds To | Description |
|---------|---------------|-------------|
| `KwTenuto` | `tenuto` | The root block keyword |
| `KwMeta` | `meta` | Metadata block keyword |
| `KwDef` | `def` | Instrument definition keyword |
| `KwMeasure` | `measure` | Measure block keyword |
| `KwGroup` | `group` | Staff grouping keyword |
| `KwMacro` | `macro` | Macro definition |
| `KwVar` | `var` | Variable declaration |

##### 2. Structure & Punctuation (V2.1 Upgrades)

| Variant | Symbol | Description |
|---------|--------|-------------|
| `MapStart` | `@{` | **[V2.1]** Map Sigil (Data structures) |
| `VoiceBracketStart`| `<[` | **[V2.1]** Polyphonic block start |
| `VoiceBracketEnd` | `]>` | **[V2.1]** Polyphonic block end |
| `LBrace` / `RBrace`| `{` `}` | Structural scope delimiters |
| `LBracket` / `RBracket`| `[` `]` | Chord delimiters |
| `LParen` / `RParen`| `(` `)` | Tuplet grouping / Arguments |
| `Colon` | `:` | Assignment / Duration separator |
| `Pipe` | `|` | Bar line / Voice separator |
| `Dot` | `.` | Attribute chaining (e.g., `.stacc`) |
| `Dollar` | `$` | Macro/Variable invocation prefix |

##### 3. Literals

| Variant | Type | Example | Notes |
|---------|------|---------|-------|
| `Integer` | `i64` | `120` | Whole numbers for BPM, counts |
| `Float` | `String` | `1.5` | Stored as string to preserve Hash/Eq traits |
| `StringLit` | `String` | `"Violin I"` | Double-quoted text with escape sequences |

##### 4. Domain-Specific Primitives (Prioritized)

| Variant | Type | Example | Regex Priority Notes |
|---------|------|---------|---------------|
| `FreqLit` | `String` | `hz(440)` | Absolute frequency mapping |
| `PitchLit` | `String` | `c4`, `F#5`, `Abqs2`| Resolves standard and microtonal accidentals |
| `DurationLit` | `String` | `:4`, `:grace` | Excludes trailing dots (handled via `Token::Dot`) |
| `TabLit` | `String` | `0-6`, `12-2` | Resolves string-fret coordinates |
| `AttributeLit`| `String` | `.stacc`, `.8va` | Leading dot bypasses standard Identifier regex |

### Error Handling

The lexer is **fault-tolerant** regarding comments:

```rust
Token::InvalidComment
```

This special token variant traps C-style comments (`//`) which are explicitly invalid in Tenuto (which uses `%%`). This allows the parser to emit a highly specific "Invalid Comment Syntax" Ariadne diagnostic rather than failing on an unknown character.

---

## 📋 Lexer Public API

```rust
// Stream-based processing via Logos trait implementation
pub struct Lexer<'source, Token>;

impl<'source> Lexer<'source, Token> {
    pub fn spanned(&self) -> SpannedIter<'source, Token>;
    pub fn slice(&self) -> &'source str;
}
```

### Performance Characteristics

| Operation | Time Complexity | Memory Usage |
|-----------|----------------|--------------|
| Tokenization | O(n) | ~1.5x source size |
| Compound Sigils | O(1) via DFA | Zero-backtracking |
| Error Recovery | O(1) per error | Minimal overhead |


## 🌲 Module: `tenutoc::parser`

The Parser module converts a flat stream of Tokens into a hierarchical **Abstract Syntax Tree (AST)**. In V2.1.0, the parser is implemented using the **Chumsky** crate as a strictly **Deterministic LL(1)** parser, eliminating the infinite loops and deep backtracking overhead found in V2.0.

### Usage

The parser expects a Chumsky `Stream` of tokens and returns a `Score` object with a list of parsing errors (if any). It leverages Chumsky's robust error recovery to ensure the compiler can report multiple syntax errors at once.

```rust
use chumsky::prelude::*;
use chumsky::Stream;
use tenutoc::lexer::Token;
use tenutoc::parser::parser;

// Assume 'tokens' is a Vec<(Token, Span)> from the Lexer
let len = source_len;
let stream = Stream::from_iter(len..len + 1, tokens.into_iter());

let (ast_opt, parse_errs) = parser().parse_recovery(stream);

if let Some(score) = ast_opt {
    println!("Parsed {} top-level blocks", score.items.len());
}
```

### Deterministic Features & Recovery (V2.1)

*   **Compound Sigil Resolution:** The parser relies on `Token::MapStart` (`@{`) and `Token::VoiceBracketStart` (`<[`) to unambiguously distinguish data maps and polyphonic blocks from standard code blocks.
*   **Peek Guards:** Uses Chumsky's `not().rewind()` to implement lookahead guards, ensuring that block terminators (`}`, `]>`) are not accidentally swallowed by error recovery systems (`skip_then_retry_until`).

---

## 🏗️ Module: `tenutoc::ast`

The Abstract Syntax Tree represents the logical structure of a Tenuto file. In V2.1.0, the AST has been updated to natively map the new compound sigils into Rust `HashMap`s.

### 1. Root: `Score`
The top-level container for a single compilation unit.

```rust
pub struct Score {
    pub header_version: Option<String>, // e.g., "2.1"
    pub items: Vec<TopLevel>,           // The sequence of blocks
}
```

### 2. High-Level Blocks: `TopLevel`
Represents the major sections of the file. Note the heavy use of `HashMap<String, Value>` to store data parsed from the V2.1 `@{}` map sigils.

```rust
pub enum TopLevel {
    /// Global Metadata: `meta @{ key: val }`
    Meta(HashMap<String, Value>),

    /// Instrument Definition: `def vln "Violin" attributes=@{...}`
    Def {
        id: String,
        label: Option<String>,
        attributes: HashMap<String, Value>,
    },

    /// Musical Content: `measure 1 { ... }`
    Measure {
        range: MeasureRange,
        attributes: HashMap<String, Value>, // Stores local @{} meta
        content: Vec<Logic>,
    },

    /// Variable/Constant: `var my_vol = 80`
    VariableDecl {
        name: String,
        value: Value,
    },
    
    // ... Macros, Groups, Imports, Conditionals
}
```

### 3. Logic Flow: `Logic`
Instructions inside a measure block.

```rust
pub enum Logic {
    /// Voice Assignment: `vln: <[ v1: ... ]>` or `vln: c4 |`
    Assignment {
        staff_id: String,
        voices: Vec<Voice>,
    },
    
    /// Local Metadata: `meta @{ ... }` inside a measure
    LocalMeta(HashMap<String, Value>),
}
```

### 4. Event Containers: `Voice` & `Event`
The atomic units of musical data.

```rust
pub struct Voice {
    pub voice_id: Option<String>, // e.g., "v1"
    pub events: Vec<Event>,
}

pub enum Event {
    /// Standard Note: `c4:4.stacc`
    Note {
        pitch: String,
        cents: Option<i32>,
        duration: Option<String>,
        dots: u8,
        multiplier: Option<u32>,
        is_tied: bool,
        attributes: Vec<Attribute>,
    },

    /// Tuplet (Recursive): `(c d e):3/2`
    Tuplet {
        content: Voice,
        p: u64, // "Play P notes..."
        q: u64, // "...in the time of Q"
    },

    /// Macro Invocation: `$Lick(c4)+2:16.stacc`
    MacroCall {
        name: String,
        args: Vec<Value>,
        transpose: Option<i32>,
        duration: Option<String>,
        dots: u8,
        multiplier: Option<u32>,
        attributes: Vec<Attribute>,
    },
    
    // ... Chords, Tabs, Percussion, Rests, Frequencies, Barlines
}
```

### 5. Primitive Types: `Value` & `Attribute`

```rust
pub enum Value {
    Str(String),
    Num(i64),
    Float(f64),
    Bool(bool),
    Id(String),
    Array(Vec<Value>),
    Map(HashMap<String, Value>), // Maps the V2.1 `@{}` structure
}

pub struct Attribute {
    pub name: String,
    pub args: Vec<Value>,
}
```

---

## 🛠️ Module: `tenutoc::preprocessor`

The Preprocessor (introduced formally in V2.1) sits between the Parser and the Inference Engine. It traverses the AST to expand macros, resolve variables, and execute conditional compilation flags.

### Usage

```rust
use tenutoc::preprocessor::Preprocessor;
use std::collections::HashMap;

// Initialize with environment variables (e.g., target="audio")
let mut initial_env = HashMap::new();
let mut preprocessor = Preprocessor::new(initial_env);

// Expand the AST
let expanded_score = preprocessor.expand(raw_ast)?;
```

### Core Responsibilities

#### 1. Variable Resolution
The preprocessor recursively scans down into `Value::Map` and `Value::Array` structures, as well as `Event` attributes, replacing `Value::Id("$var_name")` with the actual stored value.

*Example AST Transformation:*
`c4.vol($my_var)` → `c4.vol(80)`

#### 2. Macro Expansion
When encountering an `Event::MacroCall`, the engine injects the contents of the `TopLevel::MacroDef` body into the voice stream.
*   **Context Passing:** Attributes (`.stacc`), durations (`:16`), and dots attached to the macro call are uniformly applied to *all* resulting events generated by the macro.
*   **Transposition:** If a transposition integer (`+2`) is provided, all `Event::Note` and `Event::Chord` pitches inside the macro are shifted accordingly using the `shift_spn` algorithm.

#### 3. Recursion Safety
The preprocessor tracks expansion depth to prevent malicious or accidental infinite loops (e.g., Macro A calls Macro B calls Macro A).
*   `MAX_RECURSION_DEPTH` is strictly enforced at **64**. Exceeding this triggers an `E5002` fatal error.



## 🧠 Module: `tenutoc::ir` (The Inference Engine)

The Inference Engine is the core logic processor of Tenuto. It performs **Linearization**: transforming the hierarchical, relative structure of the AST into a flat, absolute-time structure called the **Timeline**. 

This is where the "Sticky State" logic (contextual duration/octave inference), Rational Arithmetic (tuplet calculations), and multi-engine abstractions (Tablature, Percussion Maps) are permanently resolved into explicit MIDI Note properties.

### Usage

The engine exposes a single public entry point: `compile()`. It requires the fully expanded AST from the Preprocessor and a boolean flag for Strict Mode.

```rust
use tenutoc::ir::{self, Timeline};

// Assume 'expanded_ast' is a Score object processed by the Preprocessor
let strict_mode = false;

match ir::compile(expanded_ast, strict_mode) {
    Ok(timeline) => {
        println!("Compiled successfully!");
        println!("Title: {}", timeline.title);
        println!("Total Tracks: {}", timeline.tracks.len());
    },
    Err(e) => eprintln!("Logic Error: {}", e),
}
```

---

## 🎞️ The Timeline Structure

The Timeline is the "Resolved" state of the music—ideal for rendering, playback, or analysis because all contextual ambiguity has been baked into explicit integer values.

### 1. Root: `Timeline`

```rust
pub struct Timeline {
    pub title: String,
    pub tempo: u32,                     // Global BPM
    pub ppq: u32,                       // Pulses Per Quarter Note (1920)
    pub tracks: HashMap<String, Track>, // Map of Staff ID → Track Data
}
```

### 2. Instrument Stream: `Track`

Represents a single instrument's physical capabilities and its sequential events. **[V2.1]** Introduces native `keyswitches` and `perc_map` dictionaries extracted from the `@{}` AST nodes.

```rust
pub struct Track {
    pub label: String,
    pub patch: String,
    pub tuning: Vec<u8>,                  // Open strings for Tablature
    pub keyswitches: HashMap<String, u8>, // Attribute → MIDI trigger
    pub perc_map: HashMap<String, u8>,    // String Key → MIDI note
    pub events: Vec<AtomicEvent>,         // Chronologically sorted events
}
```

### 3. The Atom: `AtomicEvent`

An event positioned in absolute time.

```rust
pub struct AtomicEvent {
    pub tick: u64,           // Absolute start time
    pub duration_ticks: u64, // Logical duration (for visual spacing)
    pub gate_ticks: u64,     // Physical duration (for audio playback length)
    pub kind: EventKind,
}

pub enum EventKind {
    Note { 
        pitch_midi: u8, // MIDI Note Number (Middle C = 60)
        cents: i32,     // Microtonal Pitch Bend
        velocity: u8    // Dynamics 
    },
    Frequency { 
        hz: f64,        // Absolute frequency literal
        velocity: u8 
    },
    Rest,
}
```

---

## 🧮 Logic & Math

### Rational Time (Eliminating Drift)

To prevent floating-point drift in complex polyrhythms (where `1/3 + 1/3 + 1/3 = 0.99999`), the engine uses **Rational Numbers** (fractions) for all duration and tuplet calculations until the final conversion to integer ticks.

```rust
pub struct Rational {
    pub num: u64,
    pub den: u64,
}

impl Rational {
    /// Reduces the fraction automatically via Greatest Common Divisor (GCD)
    pub fn new(num: u64, den: u64) -> Self;
    
    /// Converts the fraction into absolute PPQ ticks
    pub fn to_ticks(&self, ppq: u32) -> u64;
}
```

### Tuplet Scalar Logic

When entering a tuplet `(events):P/Q` (e.g., play 3 notes in the time of 2), the engine multiplies the active `time_scalar` by the ratio `Q / P`.

*   **Standard 8th Note:** 960 ticks
*   **Inside a 3:2 Tuplet:** `960 * (2/3) = 640 ticks` per note.

---

## 🕹️ The Cursor Model (Sticky State)

The engine uses an internal `Cursor` to traverse the logic blocks. The cursor maintains the state of the active staff, automatically filling in missing information.

```rust
struct Cursor {
    current_tick: u64,
    last_duration: Rational,
    last_octave: u8,
    last_velocity: u8,
    time_scalar: Rational,
    ppq: u32,
    tied_pitches: Vec<u8>, // [V2.1] Forward-looking tie queue
}
```

### V2.1 Sticky State Rules

1.  **True Cross-Measure Stickiness:** The engine maintains a `HashMap<String, Cursor>`. When `measure 2` begins, the primary voice (`v1`) automatically inherits the exact `last_octave` and `last_duration` from the end of `measure 1`.
2.  **Strict Mode Isolation:** If `--strict` mode is enabled, `last_duration` and `last_octave` are forcefully reset to defaults (`:4` and `Octave 4`) at every barline, enforcing explicit coding practices.
3.  **Polyphonic Isolation:** Inside a Voice Bracket (`<[ ]>`), Voice 1 inherits the global staff cursor, while Voices 2–4 generate fresh, isolated cursors to prevent the melody's state from corrupting the bassline.

### V2.1 Forward-Looking Ties (`~`)

In V2.1.0, the tie operator (`c4~`) adds the MIDI pitch `60` to the `tied_pitches` queue. When the next `c4` event is parsed, the engine detects it in the queue, finds the original event in the Track history, and *extends* its `duration_ticks` and `gate_ticks` seamlessly, instead of emitting a second NoteOn event.

---

## 🔄 Multi-Engine Abstractions

The `ir.rs` module dynamically translates different notation styles into a unified MIDI structure.

1.  **Standard Engine (`parse_pitch`):**
    Translates strings like `c#5` or `dbqs4` into a base MIDI integer and a microtonal cent deviation.
2.  **Tablature Engine (`style=tab`):**
    Parses `0-6` using the **Inverse Rule**. It takes `String 6`, references the track's `tuning` array from bottom to top, finds the open pitch (e.g., E2 / Midi 40), and adds the fret integer (`0`) to output Midi `40`.
3.  **Percussion Engine (`style=grid`):**
    Bypasses standard pitch parsing entirely. It references the `perc_map` (e.g., `k: [0, 36]`) and directly emits Midi `36` when the token `k` is encountered.


## 📤 Module: `tenutoc::midi` (The Backend Layer)

Once the Inference Engine has produced a deterministic `Timeline`, the final stage of the compiler is to **Export** that data into a standard binary format. 

The V2.1.0 reference implementation includes a native **Standard MIDI File (SMF) Export Engine** built on top of the highly-optimized `midly` crate.

### Usage

The engine exposes a simple export function that takes a reference to the `Timeline` and returns a vector of bytes (`Vec<u8>`).

```rust
use tenutoc::midi;
use std::fs;

// 'timeline' is the output from tenutoc::ir::compile()
match midi::export(&timeline) {
    Ok(bytes) => {
        fs::write("output.mid", bytes).expect("Failed to save MIDI");
        println!("Export successful!");
    },
    Err(e) => eprintln!("Export failed: {}", e),
}
```

---

## 🔄 MIDI Transformation Logic (V2.1)

The export process performs four critical transformations to map Tenuto's internal logic to the MIDI 1.0 specification.

### 1. Absolute → Delta Time Conversion
Tenuto uses **Absolute Ticks** (e.g., Event A at tick 0, Event B at tick 480). MIDI uses **Delta Ticks** (time elapsed since the previous event). The exporter collects all events for a track, sorts them chronologically, and calculates the exact delta sequence.

*Note on V2.1 PPQ:* To avoid compilation panics with `midly`, the 1920 PPQ integer is safely cast using `midly::num::u15::from_int_lossy(timeline.ppq as u16)`.

### 2. Duration Explosion (NoteOn / NoteOff)
Tenuto represents notes as single atomic events. The exporter splits these into pairs based on the `gate_ticks` (influenced by articulations like `.stacc`).

| Event | Created At | Purpose |
|-------|------------|---------|
| **Note On** | `event.tick` | Strike the note (Velocity > 0) |
| **Note Off** | `event.tick + event.gate_ticks` | Release the note (Velocity = 0) |

### 3. Native Channel 10 Percussion Routing
In V2.1, the engine inspects the track's patch name. If it matches `gm_kit` or contains the word `"drum"`, the exporter **forces the track onto MIDI Channel 10** (Index 9). The melodic channel allocator automatically skips Channel 10 to prevent accidental piano notes playing as drum hits.

### 4. Microtonal Pitch Bends
If a `Note` event has a non-zero `cents` value (e.g., parsed from `c4qs`), the exporter emits a 14-bit `PitchBend` message *immediately before* the `NoteOn` event, and a `PitchBend` reset message (Center = 8192) *immediately after* the `NoteOff` event to prevent smearing into the next note.

---

## 🖥️ The CLI Orchestrator (`main.rs`)

The `main.rs` binary serves as the orchestrator of the entire pipeline. It utilizes the **Command Pattern** (via `clap`) to chain the modules together and `ariadne` to print beautiful, contextual error messages.

### The Pipeline Flow

```rust
// 1. Lexing (Generates Token Stream)
let lexer = Token::lexer(&source_code);

// 2. Parsing (Deterministic LL(1) AST Generation)
let stream = Stream::from_iter(eoi_span, token_stream.into_iter());
let (ast_opt, parse_errs) = parser().parse_recovery(stream);

// 3. Preprocessing (Variable & Macro Expansion)
let mut preprocessor = Preprocessor::new(HashMap::new());
let expanded_score = preprocessor.expand(ast_opt.unwrap())?;

// 4. Inference (IR - Sticky State & Rational Time)
let timeline = ir::compile(expanded_score, cli.strict)?;

// 5. Backend Export (MIDI Serialization)
let midi_bytes = midi::export(&timeline)?;
std::fs::write(&cli.output, midi_bytes)?;
```

### CLI Usage

```bash
# Basic compilation to MIDI
tenutoc --input my_song.ten --output render.mid

# Strict Mode (Halts on parser warnings or synchronization errors)
tenutoc --input my_song.ten --strict
```

---

## 🛑 Error Taxonomy (`tenutoc::TenutoError`)

V2.1.0 standardizes compiler errors into specific error codes (Spec Section 24). This ensures errors are searchable and machine-parsable for IDE integrations.

```rust
pub enum TenutoError {
    // --- 1000-Series: Lexical & Meta Errors ---
    #[error("E1001: Malformed Token at position {0}")] 
    MalformedToken(usize),
    
    #[error("E1002: Syntax Error - {0}")] 
    SyntaxError(String),
    
    #[error("E1004: Version Incompatible - Requested {0}")] 
    VersionIncompatible(String),
    
    // --- 2000-Series: Definition & Import Errors ---
    #[error("E2001: Undefined Identifier - '{0}'")] 
    UndefinedIdentifier(String),
    
    #[error("E2002: Duplicate Definition - '{0}'")] 
    DuplicateDefinition(String),
    
    // --- 3000-Series: Time & Structure Errors ---
    #[error("E3002: Voice Sync Failure in staff '{0}'. Lengths: {1:?}")] 
    VoiceSyncFailure(String, Vec<u64>),
    
    // --- 5000-Series: Macro & Pre-Processor Errors ---
    #[error("E5001: Circular Reference detected in macro '{0}'")] 
    CircularReference(String),
    
    #[error("E5002: Recursion Limit Exceeded (>{1}) in macro '{0}'")] 
    RecursionLimitExceeded(String, usize),
    
    #[error("E5003: Argument Mismatch in '{0}' - {1}")] 
    ArgumentMismatch(String, String),
    
    // --- 9000-Series: System Errors ---
    #[error("F9001: IO Error - {0}")] 
    IoError(#[from] std::io::Error),
}
```

### Strict Mode (`--strict`) vs. Lenient Mode

*   **Lenient Mode (Default):** The compiler will attempt to auto-recover from missing semicolons, missing brackets, or slightly desynced polyphonic voices. It will emit the generated MIDI based on its "best guess" AST.
*   **Strict Mode:** If the `parse_recovery()` returns *any* errors, or if Voice 1 is `3840` ticks long but Voice 2 is only `1920` ticks long (`E3002: Voice Sync Failure`), the compiler instantly aborts. This is required for creating archival-grade `.ten` files.
