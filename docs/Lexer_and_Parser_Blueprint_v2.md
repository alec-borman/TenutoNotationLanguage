# Tenuto 2.0 Front-End Architecture Blueprint

**Target Stack:** Rust, `logos`, `chumsky`
**Phase:** Text Input  Token Stream  Stateless Abstract Syntax Tree (AST)

## 1. Architectural Directives

1. **Zero-Copy Lexing:** The lexer MUST utilize string slices (`&str` or `String` references) to avoid heap allocations until the AST construction phase.
2. **Stateless AST Construction:** The parser MUST NOT evaluate the "Sticky State" (e.g., inferring missing octaves or durations). The AST must remain a purely syntactic, 1:1 representation of the source text. Context resolution is strictly the responsibility of the downstream Linearization engine.
3. **Strict vs. Lenient Error Recovery:** The parser MUST utilize error-recovery combinators (synchronizing on boundaries like `|` or `}`) to collect multiple syntax errors in a single pass, aligning with the spec's "Lenient Mode" degradation requirements.

---

## 2. Lexical Specification (`logos`)

The `Token` enum must explicitly define every terminal required by the EBNF grammar. Patterns must be strictly ordered by priority to prevent shadowing.

### 2.1 Keywords (Case-Insensitive)

* `KwTenuto`, `KwMeta`, `KwDef`, `KwMeasure`, `KwGroup`, `KwImport`, `KwMacro`, `KwVar`, `KwIf`, `KwElse`, `KwRepeat`, `KwVolta`
* *Implementation:* `#[regex("(?i)keyword")]`

### 2.2 Operators & Punctuation

* **Enclosures:** `{` `}`, `[` `]`, `(` `)`
* **Separators:** `:` (Colon), `,` (Comma), `|` (Pipe), `=` (Equals)
* **Operators:** `~` (Tie), `+` (Plus), `-` (Minus), `*` (Star), `/` (Slash)
* **Structural Barlines:** `|:` (RepeatStart), `:|` (RepeatEnd), `:|:` (RepeatDouble), `||` (DoubleBar), `|]` (FinalBar)
* **Addendum A Directives:** `@at` (Runtime Schedule)

### 2.3 Primitives

* **Integer:** `[0-9]+`
* **Float:** `[0-9]+\.[0-9]+`
* **Boolean:** `(?i)true|false`
* **StringLiteral:** `"([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*"`
* **Identifier:** `[a-zA-Z_][a-zA-Z0-9_]*`

### 2.4 Domain-Specific Primitives (High Priority)

To satisfy the complex physics and timing engines, these regex patterns must capture musical data:

1. **PitchLiteral:** Captures standard pitches, microtonal accidentals, octaves, and optional cent deviations.
* *Regex Target:* `(?i)[a-g](qs|qf|tqs|tqf|bb|x|#|b|n)?[0-9]?([+-][0-9]+)?`
* *Examples:* `c4`, `f#5`, `ebqs2`, `a4+10`


2. **DurationLiteral:** Captures base time fractions and standard text flags. (Dots are handled separately by the `Dot` token to allow the parser to count them).
* *Regex Target:* `:(grace(\.slash|\.noSlash)?|[0-9]+)`
* *Examples:* `:4`, `:16`, `:grace`, `:grace.slash`


3. **TabLiteral:** Captures coordinate intersections (Fret-String), including dead notes.
* *Regex Target:* `(?i)[0-9x]+-[0-9]+`
* *Examples:* `0-6`, `12-2`, `x-5`


4. **AbsoluteFrequency:** Captures raw Hertz values.
* *Regex Target:* `(?i)hz\([0-9]+(\.[0-9]+)?\)`


5. **Attribute:** Captures dot-prefixed identifiers, explicitly allowing leading numbers (e.g., `.8va`, `.15ma`).
* *Regex Target:* `\.[a-zA-Z0-9_]+`



---

## 3. Abstract Syntax Tree (AST) Specification

The Rust structures below represent the exhaustive, 100% complete state of a parsed Tenuto 2.0 file. Every syntax branch leads to one of these nodes.

### 3.1 The Global Document

```rust
#[derive(Debug, Clone)]
pub struct Score {
    pub version: Option<String>,
    pub items: Vec<TopLevel>,
}

#[derive(Debug, Clone)]
pub enum TopLevel {
    Meta(HashMap<String, Value>),
    Def { id: String, label: String, attributes: HashMap<String, Value> },
    Group { label: String, symbol: Option<String>, items: Vec<TopLevel> },
    Import(String),
    VariableDecl { name: String, value: Value },
    MacroDef { name: String, args: Vec<(String, Option<Value>)>, body: Voice },
    Measure { range: MeasureRange, content: Vec<Statement> },
    Condition { expression: Expression, content: Vec<TopLevel> },
    AtDirective { time_spec: String, block: Box<TopLevel> },
}

#[derive(Debug, Clone)]
pub enum MeasureRange {
    Implicit,          // e.g., "measure {"
    Single(i64),       // e.g., "measure 1 {"
    Range(i64, i64),   // e.g., "measure 1-4 {"
    List(Vec<i64>),    // e.g., "measure 1, 3, 5 {"
}

```

### 3.2 Statements & Control Flow

Statements occur inside `measure` blocks and dictate the logic for specific logical ticks.

```rust
#[derive(Debug, Clone)]
pub enum Statement {
    Assignment { staff_id: String, voices: Vec<Voice> },
    LocalMeta(HashMap<String, Value>),
    IfBlock { condition: Expression, content: Vec<Statement> },
    Repeat { times: Option<u32>, content: Vec<Statement> },
    Volta { passes: Vec<u32>, content: Vec<Statement> },
    Lyric { target_id: String, stanza: Option<u32>, text: String },
}

#[derive(Debug, Clone)]
pub struct Voice {
    pub events: Vec<Event>,
}

```

### 3.3 The Event Stream (Temporal Logic)

This exhaustive enum captures every physical, auditory, or structural event that consumes time or decorates a staff.

```rust
#[derive(Debug, Clone)]
pub enum Event {
    // 1. Standard Pitch Engine
    Note { 
        pitch: String, 
        cents: Option<i32>, 
        duration: Option<String>, 
        dots: u8, 
        is_tied: bool, 
        attributes: Vec<Attribute> 
    },
    Chord { 
        notes: Vec<Event>, // Inner events contain ties/pitches
        duration: Option<String>, 
        dots: u8, 
        is_tied: bool, 
        attributes: Vec<Attribute> 
    },
    
    // 2. Tablature Engine
    Tab { 
        fret: String, 
        string: u8, 
        duration: Option<String>, 
        dots: u8, 
        attributes: Vec<Attribute> 
    },
    TabChord { 
        tabs: Vec<Event>, 
        duration: Option<String>, 
        dots: u8, 
        attributes: Vec<Attribute> 
    },
    
    // 3. Percussion Engine
    Percussion { 
        key: String, 
        duration: Option<String>, 
        dots: u8, 
        attributes: Vec<Attribute> 
    },
    PercussionChord { 
        keys: Vec<String>, 
        duration: Option<String>, 
        dots: u8, 
        attributes: Vec<Attribute> 
    },
    
    // 4. Time Management & Silence
    Rest { duration: Option<String>, dots: u8, multiplier: Option<u32> },
    Space { duration: Option<String>, dots: u8 }, // "s" literal
    
    // 5. Acoustics & Polyphony
    Frequency { hz: f64, duration: Option<String>, dots: u8, attributes: Vec<Attribute> },
    Tuplet { content: Voice, p: u64, q: u64 },
    
    // 6. Pre-Processor & Structure
    MacroCall { name: String, args: Vec<Value>, transpose: Option<i32> },
    Barline(BarlineType), 
}

#[derive(Debug, Clone)]
pub enum BarlineType { Single, Double, Final, RepeatStart, RepeatEnd, RepeatDouble }

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<Value>,
}

```

### 3.4 Values & Expressions

Used for metadata, instrument definitions, and conditional branching.

```rust
#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Num(i64),
    Float(f64),
    Bool(bool),
    Id(String),
    Array(Vec<Value>),
    Map(HashMap<String, Value>), 
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub left: Box<Value>,
    pub operator: String, // "==", "!=", ">", etc.
    pub right: Box<Value>,
}

```

---

## 4. Parser Combinator Strategies (`chumsky`)

To assemble the AST accurately, the parser must implement the following specific logic paths.

### 4.1 Postfix Chaining

* **Attributes:** Attributes are parsed using `.repeated()`. The parser must eagerly consume all trailing attributes for an event and pack them into the `Vec<Attribute>`.
* **Ties (`~`):** The tie is a postfix operator. The parser must look for `just(Token::Tilde).or_not()` immediately following a `PitchLit` (for single notes) or immediately following a `RBracket` (for whole chords), resolving it into the `is_tied: bool` flag.
* **Duration Dots:** Parsed via `just(Token::Dot).repeated().count()` immediately following a `DurationLit` and stored as `dots: u8`.

### 4.2 Left-Recursion (Tuplets & Nested Groups)

Tuplets `( c d e ):3/2` and Groups `group "Strings" { def ... }` are recursive. The parser must use `chumsky`'s `recursive()` combinator. Inner `Voice` and `TopLevel` vectors must be constructed safely to prevent stack overflows during parsing.

### 4.3 Multi-Type Event Resolution

The `event` parser choice must follow this strict fallback order to prevent ambiguous syntax hijacking:

1. `Tuplet` (Looks for `(`)
2. `Chord` (Looks for `[`)
3. `Rest` / `Space` (Looks for `r` or `s` followed by duration)
4. `Frequency` (Looks for `hz()`)
5. `MacroCall` (Looks for `$`)
6. `Tab` (Looks for Tab Coordinate)
7. `Note` (Looks for Pitch Literal)
8. `Percussion` (Looks for standard Identifier)

### 4.4 Macro Arguments & Default Values

When parsing `MacroDef`, the parser must handle the argument list EBNF `IDENTIFIER ("=" Value)?`. It returns a `Vec<(String, Option<Value>)>` where the `Option` represents the default value if provided.

---
