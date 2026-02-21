# Tenuto 2.0: The Exhaustive Lexer/Parser Blueprint

**Target:** 100% EBNF & Prose Compliance (EBNF Override Applied)

## 1. The Exhaustive Lexical Definitions (`logos`)

Every token required by EBNF 26.1 and Addendum A. Order matters for priority.

```rust
use logos::Logos;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(skip r"[ \t\r\n\f]+")] 
#[logos(skip r"%%.*")]
pub enum Token {
    // 1. KEYWORDS (Case-Insensitive)
    #[regex("(?i)tenuto")] KwTenuto,
    #[regex("(?i)meta")] KwMeta,
    #[regex("(?i)def")] KwDef,
    #[regex("(?i)measure")] KwMeasure,
    #[regex("(?i)group")] KwGroup,
    #[regex("(?i)import")] KwImport,
    #[regex("(?i)macro")] KwMacro,
    #[regex("(?i)var")] KwVar,
    #[regex("(?i)if")] KwIf,
    #[regex("(?i)else")] KwElse,
    #[regex("(?i)repeat")] KwRepeat,
    #[regex("(?i)volta")] KwVolta,
    #[regex("(?i)tuplet")] KwTuplet, // Added from EBNF 26.3

    // 2. RUNTIME DIRECTIVES (Addendum A)
    #[token("@at")] AtDirective,

    // 3. PUNCTUATION & OPERATORS
    #[token("{")] LBrace,
    #[token("}")] RBrace,
    #[token("[")] LBracket,
    #[token("]")] RBracket,
    #[token("(")] LParen,
    #[token(")")] RParen,
    #[token(":")] Colon,
    #[token("|")] Pipe,
    #[token("~")] Tilde,
    #[token("=")] Equals,
    #[token(",")] Comma,
    #[token(".")] Dot,
    #[token("$")] Dollar,
    #[token("*")] Star,
    #[token("+")] Plus,
    #[token("-")] Minus,
    #[token("/")] Slash,

    // 4. STRUCTURAL BARLINES
    #[token("|:")] RepeatStart,
    #[token(":|")] RepeatEnd,
    #[token(":|:")] RepeatDouble,
    #[token("||")] DoubleBar,
    #[token("|]")] FinalBar,

    // 5. LITERALS & DATA TYPES
    #[regex(r"[0-9]+")] Integer,
    #[regex(r"[0-9]+\.[0-9]+")] Float,
    #[regex("(?i)true|false")] Boolean,
    
    // String matching EBNF 26.1 exactly
    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#)] StringLit,

    // 6. DOMAIN-SPECIFIC PRIMITIVES
    // Absolute Frequency (Spec 19.4)
    #[regex(r"(?i)hz\([0-9]+(\.[0-9]+)?\)")] FreqLit,

    // Duration (Spec 5.1 & 5.4) - Excludes dots, parsed separately
    #[regex(r":(grace(\.slash|\.noSlash)?|[0-9]+)")] DurationLit,

    // Tablature (Spec 8.1)
    #[regex(r"(?i)[0-9xX]+-[0-9]+")] TabLit,

    // Pitch (Spec 6 & 19) - Captures accidental, octave, and cents offset
    #[regex(r"(?i)[a-g](qs|qf|tqs|tqf|bb|x|#|b|n)?[0-9]?([+-][0-9]+)?")] PitchLit,

    // Attribute (Spec 18/21) - Bypasses identifier restrictions for `.8va`
    #[regex(r"\.[a-zA-Z0-9_]+")] AttributeLit,

    // 7. IDENTIFIERS
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")] Identifier,
}

```

## 2. The Exhaustive Abstract Syntax Tree (AST)

This matches the exact hierarchy of EBNF Section 26, structurally accommodating the missing measure attributes, multipliers, and macro transpositions.

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Score {
    pub header_version: Option<String>,
    pub items: Vec<TopLevel>,
}

#[derive(Debug, Clone)]
pub enum TopLevel {
    Import(String),
    Meta(HashMap<String, Value>),
    Def { 
        id: String, 
        label: Option<String>, 
        attributes: HashMap<String, Value> 
    },
    Group { 
        label: Option<String>, 
        attributes: HashMap<String, Value>, // Captures symbol=bracket 
        items: Vec<TopLevel> 
    },
    VariableDecl { name: String, value: Value },
    MacroDef { 
        name: String, 
        args: Vec<(String, Option<Value>)>, 
        body: Voice 
    },
    // CRITICAL FIX: Measure can have attributes applied to it (EBNF 26.2)
    Measure { 
        id: MeasureId, 
        attributes: Vec<Attribute>, 
        content: Vec<Logic> 
    },
    Repeat { 
        count: Option<u32>, 
        content: Vec<Logic> 
    },
    // CRITICAL FIX: Volta uses EBNF syntax `volta Range { Logic* }`
    Volta { 
        range: String, 
        content: Vec<Logic> 
    },
    Condition { 
        expression: Expression, 
        content: Vec<TopLevel> 
    },
    // ADDENDUM A
    AtDirective { 
        time_spec: String, 
        block: Box<TopLevel> 
    },
}

#[derive(Debug, Clone)]
pub enum MeasureId {
    Implicit,
    Single(i64),
    Identifier(String), // EBNF 26.2 allows IDENTIFIER for measures
    Range(i64, i64),
    List(Vec<i64>),
}

// Logic maps to `Logic` in EBNF 26.3
#[derive(Debug, Clone)]
pub enum Logic {
    Assignment { staff_id: String, voices: Vec<Voice> },
    LocalMeta(HashMap<String, Value>),
    Condition { expression: Expression, content: Vec<Logic> },
}

#[derive(Debug, Clone)]
pub struct Voice {
    pub voice_id: Option<String>, // Compensates for EBNF/Prose mismatch
    pub events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub enum Event {
    // Note: dots and multiplier applied to ALL standard events
    Note { 
        pitch: String, 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32>, 
        is_tied: bool, 
        attributes: Vec<Attribute> 
    },
    Chord { 
        notes: Vec<Event>, // Inner notes hold their own pitches/ties
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32>, 
        is_tied: bool, 
        attributes: Vec<Attribute> 
    },
    Tab { 
        fret: String, 
        string: u8, 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32>, 
        attributes: Vec<Attribute> 
    },
    TabChord { 
        tabs: Vec<Event>, 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32>, 
        attributes: Vec<Attribute> 
    },
    Percussion { 
        key: String, 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32>, 
        attributes: Vec<Attribute> 
    },
    PercussionChord { 
        keys: Vec<String>, 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32>, 
        attributes: Vec<Attribute> 
    },
    Rest { duration: Option<String>, dots: u8, multiplier: Option<u32> },
    Space { duration: Option<String>, dots: u8, multiplier: Option<u32> },
    Frequency { 
        hz: String, 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32>, 
        attributes: Vec<Attribute> 
    },
    Tuplet { content: Voice, p: u64, q: u64 },
    MacroCall { 
        name: String, 
        args: Vec<Value>, 
        transpose: Option<i32> // e.g., $Macro + 2
    },
    Barline(Token), // Passes the specific barline token directly
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<Value>,
}

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
    pub operator: String,
    pub right: Box<Value>,
}

```

## 3. Mandatory Parser Combinator Resolutions (`chumsky`)

If you build the parser, these specific combinatorial behaviors are required to correctly populate the AST above:

1. **The Master Event Combinator:** Must parse an event payload (Pitch, Tab, Percussion, Rest), followed *optionally* by a Duration, followed *optionally* by `.repeated().count()` for Dots, followed *optionally* by `* Integer` for the Multiplier, followed *optionally* by `~` for the Tie, followed *optionally* by `.repeated()` for Attributes.
2. **Lyric Parallelism:** Because `vln.lyric: "Text"` assigns to an identifier with a dot extension, the parser for `Logic::Assignment` must check if the `IDENTIFIER` contains `.lyric` or `.lyric_N`. If it does, it routes to a specific `Statement::Lyric` (or `LocalMeta` handling) rather than a standard `VoiceGroup`.
3. **Macro Transposition:** The `Event::MacroCall` parser must consume the optional `Token::Plus` or `Token::Minus` followed by `Token::Integer` immediately after the macro invocation.
4. **Error Recovery:** Wrap `Logic` parsing in `.recover_with(skip_then_retry_until(any(), choice((just(Token::Pipe), just(Token::RBrace)))))` to ensure that a syntax error in one voice does not halt the compilation of the entire measure.
