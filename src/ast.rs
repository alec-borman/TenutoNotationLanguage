use std::collections::HashMap;

// ============================================================================
// 1. ROOT & GLOBAL SCOPE
// ============================================================================

/// The Root Document (Spec 3.1)
#[derive(Debug, Clone, PartialEq)]
pub struct Score {
    pub header_version: Option<String>,
    pub items: Vec<TopLevel>,
}

/// Global Scope Blocks (Spec 3.2 & 26.2)
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
    
    /// Staves grouping (Spec 4.5)
    Group { 
        label: Option<String>, 
        attributes: HashMap<String, Value>, 
        items: Vec<TopLevel> 
    },
    
    /// Constant declaration (Spec 15.1)
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
    /// V2.1: `attributes` uses a HashMap resolved from the `@{}` Map Sigil
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
    
    /// Build-target conditional logic (Spec 22.4)
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
    Implicit,          // `measure { ... }`
    Single(i64),       // `measure 1 { ... }`
    Identifier(String),// `measure Chorus { ... }`
    Range(i64, i64),   // `measure 1-4 { ... }`
    List(Vec<i64>),    // `measure 1, 3, 5 { ... }`
}

// ============================================================================
// 2. LOGIC & EVENT STREAM
// ============================================================================

/// Logical statements inside a measure (EBNF 26.3)
#[derive(Debug, Clone, PartialEq)]
pub enum Logic {
    /// Parsed from `staff: <[ v1: ... | v2: ... ]>` (Spec 10.1)
    Assignment { 
        staff_id: String, 
        voices: Vec<Voice> 
    },
    /// Parsed from `meta @{ ... }` inside a measure (Spec 3.3)
    LocalMeta(HashMap<String, Value>),
    
    Condition { 
        expression: Expression, 
        content: Vec<Logic> 
    },
}

/// A single monophonic or polyphonic thread (Spec 10)
#[derive(Debug, Clone, PartialEq)]
pub struct Voice {
    pub voice_id: Option<String>,
    pub events: Vec<Event>,
}

/// The exhaustive set of temporal and physical events (EBNF 26.3)
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Note { 
        pitch: String, 
        cents: Option<i32>, 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32>, 
        is_tied: bool, 
        attributes: Vec<Attribute> 
    },
    Chord { 
        notes: Vec<Event>, 
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
    Rest { 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32> 
    },
    Space { 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32> 
    },
    Frequency { 
        hz: String, 
        duration: Option<String>, 
        dots: u8, 
        multiplier: Option<u32>, 
        attributes: Vec<Attribute> 
    },
    Tuplet { 
        content: Voice, 
        p: u64, 
        q: u64 
    },
    /// V2.1 Preprocessor Target: Holds context to apply to the expanded body
    MacroCall { 
        name: String, 
        args: Vec<Value>, 
        transpose: Option<i32>,
        duration: Option<String>,
        dots: u8,
        multiplier: Option<u32>,
        attributes: Vec<Attribute>,
    },
    Barline(BarlineType), 
}

#[derive(Debug, Clone, PartialEq)]
pub enum BarlineType { 
    Single, 
    Double, 
    Final, 
    RepeatStart, 
    RepeatEnd, 
    RepeatDouble 
}

// ============================================================================
// 3. DATA STRUCTURES & PRIMITIVES
// ============================================================================

/// Parsed from `.attribute(args)` (Spec 7.1)
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
    /// V2.1 Map Sigil payload `@{ key: val }`
    Map(HashMap<String, Value>),
}

/// Conditional compilation expressions
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub left: Box<Value>,
    pub operator: String,
    pub right: Box<Value>,
}