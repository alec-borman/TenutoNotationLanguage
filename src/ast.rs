//! # Tenuto Abstract Syntax Tree (AST)
//! 
//! Defines the logical structure of a Tenuto 3.0 Document. 
//! This module represents the pure, parsed text *before* variables are expanded, 
//! tuplets are mathematically resolved, or pitches are algorithmically spelled.

use std::collections::HashMap;

// ============================================================================
// 1. ROOT & GLOBAL SCOPE
// ============================================================================

/// The Root Document (Spec 3.1)
/// Encapsulates the entire parsed Tenuto file, including versioning and global declarations.
#[derive(Debug, Clone, PartialEq)]
pub struct Score {
    pub header_version: Option<String>,
    pub items: Vec<TopLevel>,
}

/// Global Scope Blocks (Spec 3.2 & 26.2)
/// Defines the structural primitives that can exist outside of a measure timeline.
#[derive(Debug, Clone, PartialEq)]
pub enum TopLevel {
    /// E.g., `import "file.ten"` (Spec 16.1)
    Import(String),
    
    /// Global metadata block parsed from `meta @{ ... }` (Spec 3.3)
    Meta(HashMap<String, Value>),
    
    /// Instrument definition parsed from `def id label @{ ... }` (Spec 4.1)
    Def { 
        id: String, 
        label: Option<String>, 
        attributes: HashMap<String, Value> 
    },
    
    /// Staves grouping (Spec 4.5) for orchestral brackets and systemic organization
    Group { 
        label: Option<String>, 
        attributes: HashMap<String, Value>, 
        items: Vec<TopLevel> 
    },
    
    /// Constant declaration (Spec 15.1) e.g., `var base_vol = 90`
    VariableDecl { 
        name: String, 
        value: Value 
    },
    
    /// Reusable logic block with optional default args (Spec 15.2)
    MacroDef { 
        name: String, 
        args: Vec<(String, Option<Value>)>, 
        body: Voice 
    },
    
    /// The primary time-slice container (Spec 3.5)
    /// Holds the explicit musical logic, bound to an absolute temporal grid.
    Measure { 
        range: MeasureRange, 
        attributes: HashMap<String, Value>, 
        content: Vec<Logic> 
    },
    
    /// Top-level repeat wrapper (Spec 11.1)
    Repeat { 
        count: Option<u32>, 
        content: Vec<Logic> 
    },
    
    /// Alternative endings (Spec 11.2)
    Volta { 
        range: String, 
        content: Vec<Logic> 
    },
    
    /// Build-target conditional logic (Spec 22.4) e.g., `if ($target == "audio")`
    Condition { 
        expression: Expression, 
        content: Vec<TopLevel> 
    },
    
    /// Real-time scheduling (Addendum A.1.2)
    AtDirective { 
        time_spec: String, 
        block: Box<TopLevel> 
    },
}

/// Defines which time-slices a measure block populates (Spec 3.5.1)
#[derive(Debug, Clone, PartialEq)]
pub enum MeasureRange {
    /// `measure { ... }` (Implicitly follows the previous measure index)
    Implicit,          
    /// `measure 5 { ... }`
    Single(i64),       
    /// `measure Chorus { ... }`
    Identifier(String),
    /// `measure 1-4 { ... }`
    Range(i64, i64),   
    /// `measure 1, 3, 5 { ... }`
    List(Vec<i64>),    
}

// ============================================================================
// 2. LOGIC & EVENT STREAM
// ============================================================================

/// Logical statements inside a measure (EBNF 26.3)
#[derive(Debug, Clone, PartialEq)]
pub enum Logic {
    /// Parsed from `staff: <[ v1: ... | v2: ... ]>` (Spec 10.1)
    /// Maps a collection of polyphonic voices to a specific physical instrument target.
    Assignment { 
        staff_id: String, 
        voices: Vec<Voice> 
    },
    
    /// Parsed from `meta @{ ... }` inside a measure (Spec 3.3)
    LocalMeta(HashMap<String, Value>),
    
    /// Measure-level conditional logic
    Condition { 
        expression: Expression, 
        content: Vec<Logic> 
    },

    /// V3.0 MASTER SLICE: Parallel Lyric Mapping (Spec 16.1)
    LyricAssignment {
        staff_id: String,
        text: String,
    },
}

/// A single monophonic or polyphonic thread (Spec 10)
#[derive(Debug, Clone, PartialEq)]
pub struct Voice {
    pub voice_id: Option<String>,
    pub events: Vec<Event>,
}

/// The exhaustive set of temporal and physical events (EBNF 26.3)
/// Note: Optional durations signify reliance on the Inference Engine's Stateful Cursor.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Standard Acoustic Pitch (Spec 6) e.g., `c#4:4.stacc`
    Note { 
        pitch: String, 
        cents: Option<i32>, 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32>, 
        is_tied: bool, 
        attributes: Vec<Attribute> 
    },
    
    /// Simultaneous Acoustic Pitches e.g., `[c4 e4 g4]:2`
    Chord { 
        notes: Vec<Event>, 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32>, 
        is_tied: bool, 
        attributes: Vec<Attribute> 
    },
    
    /// Tablature Physical Coordinate (Spec 8) e.g., `0-6:8`
    Tab { 
        fret: String, 
        string: u8, 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32>, 
        attributes: Vec<Attribute> 
    },
    
    /// Simultaneous Tablature Coordinates
    TabChord { 
        tabs: Vec<Event>, 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32>, 
        attributes: Vec<Attribute> 
    },
    
    /// Mapped Percussion or Sampler Trigger (Spec 9) e.g., `k:4.roll(3)`
    Percussion { 
        key: String, 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32>, 
        attributes: Vec<Attribute> 
    },
    
    /// Simultaneous Percussion Triggers
    PercussionChord { 
        keys: Vec<String>, 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32>, 
        attributes: Vec<Attribute> 
    },
    
    /// Explicit silence. Renders as ink in Sheet Music.
    Rest { 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32> 
    },
    
    /// Action Notation Spacer. Consumes time, but renders absolutely no ink.
    /// Used for drawing pure automation (CC) curves in independent control lanes.
    Space { 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32>, 
        attributes: Vec<Attribute> 
    },
    
    /// Absolute Raw Frequency e.g., `hz(440.5):1`
    Frequency { 
        hz: String, 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32>, 
        attributes: Vec<Attribute> 
    },
    
    /// Standard Polyrhythmic Tuplet e.g., `(c4:8 d e):3/2`
    /// Condenses or expands a sequence of multiple events by a Rational fraction (P/Q).
    Tuplet { 
        content: Voice, 
        p: u64, 
        q: u64 
    },

    /// V3.0 Euclidean Rhythm Generator e.g., `(k):3/8` (Spec 13.2)
    /// Disburses a single event K times over an N-slot subdivision matrix.
    /// Evaluated entirely by the Bresenham line-drawing math in the IR Cursor.
    Euclidean {
        content: Box<Event>,
        k: u64,
        n: u64
    },
    
    /// Macro Invocation e.g., `$Motif(c4)+2:16`
    /// Holds arguments, transpositions, and universal overrides to apply during expansion.
    MacroCall { 
        name: String, 
        args: Vec<Value>, 
        transpose: Option<i32>,
        duration: Option<String>,
        dots: u8,
        multiplier: Option<u32>,
        attributes: Vec<Attribute>,
    },
    
    /// Explicit structural barlines (e.g., `||`, `|:`)
    Barline(BarlineType), 
}

#[derive(Debug, Clone, PartialEq)]
pub enum BarlineType { 
    Single,         // `|`
    Double,         // `||`
    Final,          // `|]`
    RepeatStart,    // `|:`
    RepeatEnd,      // `:|`
    RepeatDouble    // `:|:`
}

// ============================================================================
// 3. DATA STRUCTURES & PRIMITIVES
// ============================================================================

/// Parsed from `.attribute(args)` (Spec 7.1)
/// Captures sequential DSP modifiers, articulations, and CC instructions.
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<Value>,
}

/// Primitive Data Types (Spec 2.5 & 26.4)
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Num(i64),
    Float(f64),
    Bool(bool),
    Id(String),
    Array(Vec<Value>),
    
    /// Native Map Sigil payload `@{ key: val }`
    Map(HashMap<String, Value>),
    
    /// V3.0 SLICE 4: Absolute Physical Time e.g., `15ms`, `10ticks`
    TimeVal(String),
}

/// Conditional compilation expressions evaluated by the Preprocessor.
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub left: Box<Value>,
    pub operator: String,
    pub right: Box<Value>,
}