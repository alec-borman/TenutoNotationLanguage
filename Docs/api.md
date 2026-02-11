# Tenuto API Reference

**Version:** 2.0.0 | **Crate:** `tenutoc`

---

## 📦 Crate Overview

The `tenutoc` crate serves as the reference implementation for the Tenuto v2.0 language. It is designed as a **modular pipeline**, allowing developers to hook into different stages of compilation:

| Module | Purpose | Transformation |
|--------|---------|----------------|
| `lexer` | High-performance tokenization | Source string → Token stream |
| `parser` | Fault-tolerant AST generation | Token stream → Score struct |
| `ir` | The Inference Engine | Score → Timeline |
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

The Lexer module is the entry point for the compiler. It utilizes the **`logos`** crate to generate a high-speed, regex-based state machine for tokenization.

### Usage Example

```rust
use tenutoc::lexer::Token;
use logos::Logos;

let source = "vln: c4:4";
let mut lexer = Token::lexer(source);

assert_eq!(lexer.next(), Some(Ok(Token::Identifier("vln".into()))));
assert_eq!(lexer.next(), Some(Ok(Token::Colon)));
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
| `KwImport` | `import` | File import directive |
| `KwMacro` | `macro` | Macro definition |
| `KwVar` | `var` | Variable declaration |
| `KwIf` | `if` | Conditional compilation |
| `KwElse` | `else` | Conditional branch |

##### 2. Structure & Punctuation

| Variant | Symbol | Description |
|---------|--------|-------------|
| `LBrace` | `{` | Scope delimiter (start) |
| `RBrace` | `}` | Scope delimiter (end) |
| `LBracket` | `[` | Chord delimiter (start) |
| `RBracket` | `]` | Chord delimiter (end) |
| `LParen` | `(` | Tuplet grouping / Arguments (start) |
| `RParen` | `)` | Tuplet grouping / Arguments (end) |
| `Colon` | `:` | Assignment / Duration separator |
| `Pipe` | `|` | Bar line / Voice separator |
| `Slash` | `/` | Ratio separator (Time Signatures/Tuplets) |
| `Dot` | `.` | Attribute chaining (e.g., `.stacc`) |

##### 3. Literals

| Variant | Type | Example | Notes |
|---------|------|---------|-------|
| `Integer` | `i64` | `120` | Whole numbers for BPM, counts |
| `Float` | `String` | `1.5` | Stored as string to preserve precision |
| `StringLit` | `String` | `"Violin I"` | Double-quoted text with escape sequences |

##### 4. Musical Primitives

| Variant | Type | Example | Regex Pattern |
|---------|------|---------|---------------|
| `PitchLit` | `String` | `c4`, `F#5`, `Ab2` | `(?i)[a-g](qs\|qf\|tqs\|tqf\|bb\|x\|#\|b\|n)?[0-9]?` |
| `DurationLit` | `String` | `:4`, `:8.` | `:[0-9]+(\.)*` |
| `TabLit` | `String` | `0-6`, `12-2` | `[0-9]+-[0-9]+` |

### Error Handling

The lexer is **fault-tolerant** regarding comments:

```rust
Token::InvalidComment
```

This special token variant traps C-style comments (`//`) which are invalid in Tenuto (which uses `%%`). This allows the parser to emit helpful error messages rather than panicking when encountering unsupported comment syntax.

---

## 📋 Lexer Public API

```rust
// Primary entry point
pub fn tokenize(source: &str) -> Result<Vec<Token>, LexerError>

// Stream-based processing
pub struct TokenStream<'a> {
    pub source: &'a str,
    pub tokens: Vec<(Token, Span)>
}

impl<'a> TokenStream<'a> {
    pub fn new(source: &'a str) -> Self;
    pub fn peek(&self) -> Option<&Token>;
    pub fn next(&mut self) -> Option<Token>;
    pub fn span(&self) -> Span;
}
```

### Performance Characteristics

| Operation | Time Complexity | Memory Usage |
|-----------|----------------|--------------|
| Tokenization | O(n) | ~1.5x source size |
| Peak Throughput | ~50MB/s | <10MB working memory |
| Error Recovery | O(1) per error | Minimal overhead |

---

## 🔍 Token Examples

### Valid Token Sequences

```rust
// Instrument definition
[
    Token::KwDef,
    Token::Identifier("vln".into()),
    Token::StringLit("Violin I".into()),
    Token::LBrace,
    // ...
]

// Musical logic
[
    Token::Identifier("vln".into()),
    Token::Colon,
    Token::PitchLit("c4".into()),
    Token::DurationLit(":4".into()),
    Token::Dot,
    Token::Identifier("stacc".into()),
    Token::Pipe,
]
```

### Error Recovery Example

```rust
let source = "vln: c4:4 // This comment is invalid";
let tokens = tokenize(source);

// Returns:
// [
//   Ok(Token::Identifier("vln".into())),
//   Ok(Token::Colon),
//   Ok(Token::PitchLit("c4".into())),
//   Ok(Token::DurationLit(":4".into())),
//   Err(LexerError::InvalidComment("// This comment is invalid"))
// ]
```

---

## 📚 Related Modules

- **`tenutoc::parser`** → Builds AST from token stream
- **`tenutoc::span`** → Source location tracking
- **`tenutoc::error`** → Comprehensive error types

---

## 🧪 Testing the Lexer

```bash
# Run lexer-specific tests
cargo test lexer

# Test with sample files
cargo test --test lexer_integration

# Benchmark performance
cargo bench lexer_benchmarks
```


## 🌲 The Parser & AST

The Parser module converts a flat stream of Tokens (from the Lexer) into a hierarchical **Abstract Syntax Tree (AST)**. It utilizes the **Chumsky** library to provide robust, error-recovering recursive descent parsing.

### Usage

The parser expects a `Stream` of tokens and returns a `Score` object (the root of the AST) with a list of parsing errors (if any).

```rust
use chumsky::prelude::*;
use chumsky::Stream;
use tenutoc::lexer::Token;
use tenutoc::parser::parser;

// Assume 'tokens' is a Vec<(Token, Span)> from the Lexer
let len = source_len; // Total length of source string
let stream = Stream::from_iter(len..len + 1, tokens.into_iter());

let (ast, errors) = parser().parse_recovery(stream);

if let Some(score) = ast {
    println!("Successfully parsed {} items", score.items.len());
}
```

---

## 🏗 Abstract Syntax Tree (AST)

The AST represents the grammatical structure of a Tenuto file.

### 1. Root: `Score`
The top-level container for a single compilation unit.

```rust
pub struct Score {
    pub header: Option<String>, // e.g., "tenuto" version string
    pub items: Vec<TopLevel>,   // The sequence of blocks
}
```

### 2. High-Level Blocks: `TopLevel`
Represents the major sections of the file.

```rust
pub enum TopLevel {
    /// Global Metadata: `meta { key: val }`
    Meta(Vec<(String, Value)>),

    /// Instrument Definition: `def vln "Violin" ...`
    Def {
        id: String,
        label: String,
        attributes: Vec<(String, Value)>,
    },

    /// Musical Content: `measure 1 { ... }`
    Measure {
        id: Option<i64>, // Explicit measure number
        content: Vec<Statement>,
    },

    /// External File: `import "lib/drums.ten"`
    Import(String),
}
```

### 3. Logic Flow: `Statement`
Instructions inside a measure block.

```rust
pub enum Statement {
    /// Voice Assignment: `vln: c4 d e |`
    Assignment {
        staff_id: String,
        voices: Vec<Voice>,
    },
    
    /// Local Metadata: `meta { ... }` inside a measure
    LocalMeta(Vec<(String, Value)>),
}
```

### 4. Event Containers: `Voice` & `Event`
The atomic units of musical data.

```rust
pub struct Voice {
    pub events: Vec<Event>,
}

pub enum Event {
    /// Standard Note: `c4:4.stacc`
    Note {
        pitch: String,
        duration: Option<String>,
        attributes: Vec<Attribute>,
    },

    /// Chord: `[c4 e4 g4]:2`
    Chord {
        notes: Vec<String>,
        duration: Option<String>,
        attributes: Vec<Attribute>,
    },

    /// Rest: `r:4`
    Rest {
        duration: Option<String>,
    },

    /// Tuplet (Recursive): `(c d e):3/2`
    Tuplet {
        content: Voice,
        p: u64, // "Play P notes..."
        q: u64, // "...in the time of Q"
    },

    /// Tablature: `0-6`
    Tab {
        fret: u8,
        string: u8,
        duration: Option<String>,
        attributes: Vec<Attribute>,
    },

    /// Percussion/Grid: `k` or `sd`
    Percussion {
        key: String,
        duration: Option<String>,
        attributes: Vec<Attribute>,
    },
}
```

### 5. Primitive Types: `Value` & `Attribute`

#### `Value`
A polymorphic wrapper for literal data found in attributes or metadata.

```rust
pub enum Value {
    Str(String),
    Num(i64),
    Float(f64),
    Id(String), // Identifiers used as values, e.g., style=standard
    Array(Vec<Value>),
}
```

#### `Attribute`
Decorators attached to events (e.g., `.stacc`, `.vol(100)`).

```rust
pub struct Attribute {
    pub name: String,
    pub args: Vec<Value>,
}
```

---

## 🧩 The Parser Combinator

The `parser()` function returns a Chumsky parser composed of smaller, reusable parsers.

### Key Design Features

#### Recursion
The `Event` parser is defined recursively to handle nested structures:
- Tuplets within tuplets: `( ( c d e ):3/2 ):3/2`
- Chords with attributes: `[c4.stacc e4.acc]:2`
- Complex polyphonic structures

#### Error Recovery
The parser continues processing even when encountering malformed tokens, ensuring users get a comprehensive error list rather than stopping at the first issue.

### Parser Composition Example

```rust
fn event_parser() -> impl Parser<Token, Event, Error = Simple<Token>> + Clone {
    recursive(|event| {
        // Base events (notes, rests)
        let note = filter_map(|span, tok| match tok {
            Token::PitchLit(p) => Ok(Event::Note { 
                pitch: p, 
                duration: None, 
                attributes: vec!() 
            }),
            _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
        });
        
        // Tuplet parser (recursive)
        let tuplet = just(Token::LParen)
            .ignore_then(event.repeated())
            .then_ignore(just(Token::RParen))
            .then(tuplet_ratio_parser())
            .map(|(events, (p, q))| Event::Tuplet {
                content: Voice { events },
                p,
                q,
            });
        
        choice((note, tuplet))
    })
}
```

---

## 📊 AST Node Reference

### `TopLevel` Variants

| Variant | Syntax Example | Purpose |
|---------|---------------|---------|
| `Meta` | `meta { tempo: 120 }` | Global score metadata |
| `Def` | `def vln "Violin" style=standard` | Instrument registration |
| `Measure` | `measure 1 { vln: c4 \| }` | Container for musical events |
| `Import` | `import "strings.ten"` | File inclusion directive |

### `Event` Variants

| Variant | Syntax Example | Fields |
|---------|---------------|--------|
| `Note` | `c4:4.stacc` | `pitch`, `duration`, `attributes` |
| `Chord` | `[c4 e4 g4]:2` | `notes`, `duration`, `attributes` |
| `Rest` | `r:4` | `duration` |
| `Tuplet` | `(c d e):3/2` | `content`, `p`, `q` |
| `Tab` | `0-6:8.h` | `fret`, `string`, `duration`, `attributes` |
| `Percussion` | `k:4.ghost` | `key`, `duration`, `attributes` |

### Value Types in Attributes

| Type | Example | Use Case |
|------|---------|----------|
| `Str` | `"Violin I"` | Labels, text annotations |
| `Num` | `120` | BPM, MIDI values, counts |
| `Float` | `1.5` | Multipliers, ratios |
| `Id` | `standard` | Enum-style identifiers |
| `Array` | `[1, 2, 3]` | Lists, parameter curves |

---

## 🔧 Advanced Parser Features

### 1. Attribute Parsing
Attributes support both simple flags (`.stacc`) and parameterized forms (`.vol(100)`):

```rust
/// Parses: .stacc.vol(100).text("dolce")
fn attribute_parser() -> impl Parser<Token, Attribute, Error = Simple<Token>> {
    just(Token::Dot)
        .ignore_then(identifier_parser())
        .then(
            just(Token::LParen)
                .ignore_then(value_parser().separated_by(just(Token::Comma)))
                .then_ignore(just(Token::RParen))
                .or_not()
        )
        .map(|(name, args)| Attribute {
            name,
            args: args.unwrap_or_default(),
        })
}
```

### 2. Error Recovery Strategies
The parser implements multiple recovery points:

| Strategy | When Used | Example |
|----------|-----------|---------|
| **Skip Until** | Missing delimiter | Skip to next `}` on unmatched `{` |
| **Insert Missing** | Optional tokens | Insert missing `:` in durations |
| **Replace** | Invalid tokens | Replace `//` with `%%` comments |

### 3. Span Tracking
Every AST node includes source location information for precise error reporting:

```rust
pub struct Spanned<T> {
    pub node: T,
    pub span: Span, // (start_byte, end_byte)
}
```

---

## 🧪 Testing the Parser

```bash
# Run parser tests
cargo test parser

# Test specific grammar features
cargo test --test tuplet_parsing
cargo test --test attribute_parsing

# Generate parse trees for debugging
cargo run --bin debug-parse -- examples/test.ten
```

---

## 📈 Performance Metrics

| Operation | Time (μs) | Memory (KB) |
|-----------|-----------|-------------|
| Parse small file (1KB) | ~120 | ~50 |
| Parse orchestra score (100KB) | ~8,500 | ~4,000 |
| Error recovery overhead | +15% | +5% |
| Max recursion depth | 64 | - |



## 🧠 The Inference Engine (IR)

The Inference Engine is the core logic processor of Tenuto. It transforms the hierarchical, relative structure of the AST (`Score`) into a flat, absolute-time structure called the **Timeline**.

This process is called **Linearization** and is where the "Sticky State" logic (contextual duration/octave inference) and Rational Arithmetic (tuplet calculations) are resolved.

### Usage

The engine exposes a single public entry point: `compile()`.

```rust
use tenutoc::ir::{self, Timeline};

// Assume 'ast' is a valid Score object from the Parser
match ir::compile(ast) {
    Ok(timeline) => {
        println!("Compiled successfully!");
        println!("Title: {}", timeline.title);
        println!("Total Tracks: {}", timeline.tracks.len());
    },
    Err(e) => eprintln!("Logic Error: {}", e),
}
```

---

## 🎞 The Timeline Structure

The Timeline is the "Resolved" state of the music—ideal for rendering, playback, or analysis because all context has been baked into explicit values.

### 1. Root: `Timeline`

```rust
pub struct Timeline {
    pub title: String,
    pub tempo: u32,                    // Global BPM (e.g., 120)
    pub tracks: HashMap<String, Track>, // Map of Staff ID → Track Data
}
```

### 2. Instrument Stream: `Track`

Represents a single instrument or sound source.

```rust
pub struct Track {
    pub label: String,    // Display Name (e.g., "Violin I")
    pub patch: String,    // MIDI Patch Name (e.g., "Violin")
    pub channel: u8,      // Logic Channel (0-15)
    pub events: Vec<AtomicEvent>, // Sorted list of events
}
```

### 3. The Atom: `AtomicEvent`

An event positioned in absolute time.

- **Ticks**: Time measured in Ticks (standard resolution: 1920 PPQ)
- **Duration**: Also measured in Ticks

```rust
pub struct AtomicEvent {
    /// Absolute start time in ticks
    pub tick: u64,          
    
    /// Duration in ticks
    pub duration_ticks: u64,
    
    /// The specific action
    pub kind: EventKind,
}

pub enum EventKind {
    Note { 
        pitch: u8,    // MIDI Note Number (0-127). Middle C = 60.
        velocity: u8  // Dynamics (0-127). Default = 100.
    }, 
    Rest,
}
```

---

## 🧮 Logic & Math

### The Cursor Model

The engine uses a **Cursor** to traverse the AST. The cursor maintains the **Current State**:

| State Variable | Purpose | Example |
|----------------|---------|---------|
| `current_tick` | Current position in time | 1920 (1 beat into piece) |
| `last_duration` | "Sticky" duration for next event | `":4"` (quarter note) |
| `last_octave` | "Sticky" octave for next pitch | `4` (middle C octave) |
| `time_scalar` | Current time dilation factor (for Tuplets) | `2/3` (triplet scaling) |

### Rational Arithmetic

To prevent floating-point drift (where `1/3 + 1/3 + 1/3 = 0.99999`), the engine uses **Rational Numbers** (fractions) for all duration calculations until final conversion to integer ticks.

```rust
/// Rational number type used for exact duration math
pub struct Rational {
    pub numerator: u64,
    pub denominator: u64,
}

impl Rational {
    pub fn new(n: u64, d: u64) -> Self;
    pub fn reduce(&self) -> Self;
    pub fn to_ticks(&self, ppq: u64) -> u64;
}
```

### Tuplet Logic

When entering a tuplet `(events):P/Q` (play $P$ notes in the time of $Q$), the engine updates the `time_scalar`:

$$
\text{NewScalar} = \text{OldScalar} \times \frac{Q}{P}
$$

This ensures a triplet quarter note is mathematically exactly $2/3$ the length of a standard quarter note.

**Example:** `(c d e):3/2` in a 1920 PPQ timeline:
- Base duration: 8th note = 960 ticks
- Triplet scalar: $2/3$
- Result: $960 \times 2/3 = 640$ ticks per note

### Polyphony Processing

The engine processes voices in parallel:

1. **Voice Separation**: When a staff has multiple voices (`vln: { v1: ... | v2: ... }`), the engine spawns separate Cursors for each voice.
2. **Independent Advancement**: Each Cursor starts at the same tick but advances independently based on its events.
3. **Merging**: All events are merged into the Track and sorted by time.

```rust
// Internal processing for voice groups
struct VoiceCursor {
    tick: u64,
    last_duration: Rational,
    last_octave: u8,
    time_scalar: Rational,
}

impl VoiceCursor {
    fn process_voice(&mut self, voice: &Voice) -> Vec<AtomicEvent>;
}
```

---

## 🔍 Resolution Process

### Phase 1: State Initialization

```rust
fn initialize_cursors(score: &Score) -> HashMap<String, StaffState> {
    // For each staff, create initial cursor state
    // Set default durations, octaves, and time signatures
}
```

### Phase 2: Measure Linearization

```rust
fn linearize_measure(
    measure: &Measure,
    cursors: &mut HashMap<String, StaffState>
) -> Result<Vec<AtomicEvent>, CompileError> {
    // Process each statement in the measure
    // Apply "sticky state" inference
    // Handle time signature changes
}
```

### Phase 3: Post-Processing

```rust
fn finalize_timeline(
    events_by_staff: HashMap<String, Vec<AtomicEvent>>
) -> Timeline {
    // Sort events by tick
    // Merge polyphonic voices
    // Validate measure completeness
    // Apply swing quantization if specified
}
```

---

## ⚠️ Error Conditions

| Error Type | Condition | Example |
|------------|-----------|---------|
| `E3001: Time Overflow` | Voice duration > measure capacity | `vln: c4:1 d4:1` in 3/4 time |
| `E3002: Voice Sync Failure` | Polyphonic voices have different durations | `v1: c4:2 | v2: d4:1` |
| `E3003: Tuplet Ratio Error` | Tuplet content doesn't fit ratio | `(c4:1 d4:1):3/2` |
| `W3005: Pickup Mismatch` | Anacrusis ≠ declared pickup duration | Pickup of `:8` but content is `:4` |

---

## 📊 Performance Characteristics

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Linearization | O(n) events | Scales linearly with note count |
| Tuplet recursion | O(depth) | Max depth = 64 |
| Voice merging | O(m log m) | m = events per staff |
| Rational math | O(1) per op | GCD reductions cached |

---

## 🧪 Testing the Engine

```bash
# Run inference engine tests
cargo test ir

# Test specific inference rules
cargo test --test sticky_state
cargo test --test tuplet_math

# Generate timeline for debugging
cargo run --bin debug-timeline -- examples/test.ten
```


## 📤 The Backend Layer

Once the Inference Engine has produced a deterministic `Timeline`, the final stage of the compiler is to **Export** that data into a standard format.

The current reference implementation includes a native **MIDI Export Engine**.

---

## 🎹 MIDI Export Engine

The MIDI module translates Tenuto's internal **Absolute Time** model into MIDI's **Delta Time** model. It handles the low-level serialization of bytes compliant with the **Standard MIDI File (SMF)** specification.

### Usage

The engine exposes a simple export function that takes a reference to the timeline and returns a vector of bytes (`Vec<u8>`).

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

### Dependencies

This module relies on the **`midly`** crate for safe, strongly-typed MIDI serialization.

---

## 🔄 Transformation Logic

The export process performs three critical transformations:

### 1. Duration Explosion (NoteOn / NoteOff)

Tenuto represents notes as atomic events with a duration property (`Note { duration: 960 }`). MIDI represents notes as two distinct events in the stream.

The compiler splits every `AtomicEvent` into:

| Event | Created At | Purpose |
|-------|------------|---------|
| **Note On** | `event.tick` | Start the note |
| **Note Off** | `event.tick + event.duration_ticks` | End the note |

**Example:**
- Tenuto: `AtomicEvent { tick: 0, duration_ticks: 960, ... }`
- MIDI: `[NoteOn @ tick=0], [NoteOff @ tick=960]`

### 2. Time Conversion (Absolute → Delta)

Tenuto uses **Absolute Ticks** (e.g., Event A at tick 0, Event B at tick 480). MIDI uses **Delta Ticks** (time elapsed since the previous event).

**Algorithm:**
1. Collect all Note On/Off messages for a track
2. Sort them strictly by absolute tick
3. Iterate through the sorted list, calculating `delta = current_tick - previous_tick`

### 3. Channel Mapping

The compiler automatically assigns MIDI channels (0-15) to Tenuto Staves based on the track index.

**Note:** Future roadmap includes explicit channel assignment via `def vln channel=1`.

---

## 🖥️ CLI Integration (`main.rs`)

The `main.rs` binary serves as the orchestrator of the entire pipeline. It utilizes the **Command Pattern** to chain the modules together.

### The Pipeline Flow

```rust
// 1. Input
let source = fs::read_to_string(args.input)?;

// 2. Lexing
let tokens = Token::lexer(&source);

// 3. Parsing
let ast = parser().parse(tokens)?;

// 4. Inference (IR)
let timeline = ir::compile(ast)?;

// 5. Backend Selection
if let Some(path) = args.output {
    if path.extension() == Some("mid".as_ref()) {
        // 6. Export
        let bytes = midi::export(&timeline)?;
        fs::write(path, bytes)?;
    }
}
```

### CLI Usage

```bash
# Basic compilation to MIDI
tenutoc --input composition.ten --output composition.mid

# Verbose mode (shows compilation phases)
tenutoc -v -i composition.ten -o composition.mid

# Debug mode (dumps intermediate representations)
tenutoc --debug --input composition.ten
```

---

## 🔮 Future Backends

The modular architecture allows new backends to be added without modifying the Core Logic (Lexer/Parser/IR).

| Backend | Module | Status | Key Challenge |
|---------|--------|--------|---------------|
| **MusicXML** | `tenutoc::xml` | Planned v2.2 | Re-barring algorithm (linear timeline → measures) |
| **SVG Engraving** | `tenutoc::engrave` | Planned v2.3 | Layout algorithms, SMuFL font integration |
| **WAV/Audio** | `tenutoc::audio` | Future | Real-time synthesis or sampler integration |
| **PDF** | `tenutoc::pdf` | Future | Page layout, high-quality typography |

### MusicXML Backend (`tenutoc::xml`)

Will map `Timeline` events to `<note>`, `<measure>`, and `<part>` XML tags. Requires a "Re-Barring" algorithm to split the linear timeline back into measures based on Time Signatures.

### Scalable Vector Graphics (`tenutoc::engrave`)

A direct rendering engine to draw sheet music. Will use the `Timeline` for positioning and `Def` attributes for visual style.

---

## 📊 MIDI Export Details

### File Structure

| Section | MIDI Chunk | Contents |
|---------|------------|----------|
| **Header** | `MThd` | Format 1, 2 tracks, 1920 PPQ |
| **Conductor Track** | `MTrk` (Track 0) | Tempo changes, time signatures, key signatures |
| **Instrument Track** | `MTrk` (Track 1+) | Note events, program changes, controller messages |

### Event Resolution

| Tenuto Concept | MIDI Implementation |
|----------------|---------------------|
| Tempo (`meta { tempo: 120 }`) | `SetTempo` meta event (500,000 μs/quarter) |
| Time Signature (`meta { time: "4/4" }`) | `TimeSignature` meta event (4/4) |
| Key Signature (`meta { key: "C" }`) | `KeySignature` meta event (0 sharps/flats) |
| Note Velocity | `NoteOn` velocity byte (derived from dynamics) |
| Articulation | Gate time adjustment (`.stacc` = 50% duration) |

### Controller Messages

| Attribute | MIDI CC | Range | Default |
|-----------|---------|-------|---------|
| `vol` | CC7 (Volume) | 0-127 | 100 |
| `pan` | CC10 (Pan) | 0-127 | 64 (center) |
| `expression` | CC11 (Expression) | 0-127 | 127 |
| `reverb` | CC91 (Reverb) | 0-127 | 0 |

---

## 🧪 Testing the MIDI Export

```bash
# Run MIDI export tests
cargo test midi

# Generate and verify MIDI files
cargo test --test midi_roundtrip

# Benchmark export performance
cargo bench midi_export
```

### Example Output Structure

```
output.mid (Standard MIDI File)
├── MThd (Header)
│   ├── Format: 1
│   ├── Tracks: 2
│   └── Division: 1920 PPQ
├── MTrk (Conductor Track)
│   ├── SetTempo: 500,000 μs/qnote (120 BPM)
│   └── TimeSignature: 4/4
└── MTrk (Violin I)
    ├── ProgramChange: 40 (Violin)
    ├── NoteOn: C4 (tick=0, velocity=90)
    ├── NoteOff: C4 (tick=960)
    └── ...
```

---

## ⚠️ Limitations & Notes

1. **MIDI 1.0 Only**: Current implementation targets MIDI 1.0 specification
2. **Channel Pressure**: Polyphonic aftertouch not yet implemented
3. **System Exclusive**: Manufacturer-specific messages not supported
4. **RPN/NRPN**: Registered/Non-registered parameters not yet mapped

---
