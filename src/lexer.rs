use logos::Logos;

/// OmniScore Token Definitions
/// Implements Spec Section 26.1 (Lexical Tokens)
#[derive(Logos, Debug, PartialEq, Eq, Clone, Hash)]
#[logos(skip r"[ \t\r\n\f]+")] // CHANGED: Added \r to the skip list for Windows support
#[logos(skip r"%%.*")]          // Spec 2.3: Ignore Line Comments
pub enum Token {

    // ========================================================================
    // 1. KEYWORDS (Case-Insensitive)
    // ========================================================================
    #[regex("(?i)omniscore")]  KwOmniscore,
    #[regex("(?i)meta")]       KwMeta,
    #[regex("(?i)def")]        KwDef,
    #[regex("(?i)measure")]    KwMeasure,
    #[regex("(?i)group")]      KwGroup,
    #[regex("(?i)import")]     KwImport,
    #[regex("(?i)macro")]      KwMacro,
    #[regex("(?i)var")]        KwVar,
    #[regex("(?i)if")]         KwIf,
    #[regex("(?i)else")]       KwElse,

    // ========================================================================
    // 2. PUNCTUATION & OPERATORS (Spec 2.6)
    // ========================================================================
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

    // Structure Tokens
    #[token("|:")]  RepeatStart,
    #[token(":|")]  RepeatEnd,
    #[token(":|:")] RepeatDouble,
    #[token("||")]  DoubleBar,
    #[token("|]")]  FinalBar,

    // ========================================================================
    // 3. LITERALS & DATA TYPES (Spec 2.5)
    // ========================================================================
    
    // Integers: 1, 120
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),

    // Floats: 1.5, 0.75
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().to_string())]
    Float(String),

    // Strings: "Violin I" (Handles escaped quotes)
    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string() // Strip surrounding quotes
    })]
    StringLit(String),

    // ========================================================================
    // 4. MUSIC PRIMITIVES (High Priority)
    // ========================================================================

    // Duration: :4, :8., :16
    #[regex(r":[0-9]+(\.)*", |lex| lex.slice().to_string())]
    DurationLit(String),

    // Tab Coordinate: 0-6, 12-2
    #[regex(r"[0-9]+-[0-9]+", |lex| lex.slice().to_string())]
    TabLit(String),

    // Pitch: C4, f#5, Bb2, cqs4 (Quarter Sharp)
    #[regex(r"(?i)[a-g](qs|qf|tqs|tqf|bb|x|#|b|n)?[0-9]?", |lex| lex.slice().to_string(), priority=2)]
    PitchLit(String),

    // ========================================================================
    // 5. IDENTIFIERS (Lowest Priority)
    // ========================================================================
    
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string(), priority=1)]
    Identifier(String),

    // Trap C-style comments
    #[regex(r"//.*", |_| false)] 
    InvalidComment,
}