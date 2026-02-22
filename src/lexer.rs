use logos::Logos;

/// The Exhaustive Token Enum for Tenuto 2.1.0
/// Maps 1:1 with EBNF Section 26.1 and Addendum A.
#[derive(Logos, Debug, PartialEq, Eq, Hash, Clone)]
#[logos(skip r"[ \t\r\n\f]+")] // Spec 2.2: Ignore all whitespace
#[logos(skip r"%%.*")]         // Spec 2.3: Ignore Line Comments
pub enum Token {
    // ========================================================================
    // 1. KEYWORDS (Case-Insensitive)
    // ========================================================================
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
    #[regex("(?i)tuplet")] KwTuplet,

    // ========================================================================
    // 2. RUNTIME DIRECTIVES (Addendum A.1.2)
    // ========================================================================
    #[token("@at")] AtDirective,

    // ========================================================================
    // 3. PUNCTUATION & OPERATORS (Updated for v2.1.0)
    // ========================================================================
    #[token("{")] LBrace,
    #[token("}")] RBrace,
    
    // V2.1 Compound Sigils (Spec 2.6)
    #[token("@{")] MapStart,           // Encloses Key-Value Data Maps
    #[token("<[")] VoiceBracketStart,  // Encloses Multi-voice Polyphony
    #[token("]>")] VoiceBracketEnd,    // Closes Multi-voice Polyphony
    
    #[token("[")] LBracket,
    #[token("]")] RBracket,
    #[token("(")] LParen,
    #[token(")")] RParen,
    #[token(":")] Colon,
    #[token("|")] Pipe,
    #[token("~")] Tilde,       // Spec 6.6: Ties
    #[token("=")] Equals,
    #[token(",")] Comma,
    #[token(".")] Dot,         // Used for Duration Dots (e.g., :4.)
    #[token("$")] Dollar,      // Macro/Variable invocation
    #[token("*")] Star,        // Multipliers (e.g., r:1 * 4)
    #[token("+")] Plus,        // Transposition / Microtonal
    #[token("-")] Minus,       // Transposition / Microtonal
    #[token("/")] Slash,       // Tuplet ratios (e.g., 3/2)

    // ========================================================================
    // 4. STRUCTURAL BARLINES (Spec 11.1)
    // ========================================================================
    // Logos automatically prioritizes longer matches, so `|:` is safely 
    // evaluated before `|` and `:`.
    #[token("|:")]  RepeatStart,
    #[token(":|")]  RepeatEnd,
    #[token(":|:")] RepeatDouble,
    #[token("||")]  DoubleBar,
    #[token("|]")]  FinalBar,

    // ========================================================================
    // 5. LITERALS & DATA TYPES
    // ========================================================================
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().unwrap_or(0))]
    Integer(i64),

    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().to_string())]
    Float(String),

    #[regex("(?i)true|false", |lex| lex.slice().to_lowercase() == "true")]
    Boolean(bool),
    
    // Spec 2.5: Handles escaped quotes
    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string() 
    })]
    StringLit(String),

    // ========================================================================
    // 6. DOMAIN-SPECIFIC PRIMITIVES (High Priority)
    // ========================================================================
    
    // Absolute Frequency (Spec 19.4) e.g., hz(440) or hz(442.5)
    #[regex(r"(?i)hz\([0-9]+(\.[0-9]+)?\)", |lex| lex.slice().to_string())]
    FreqLit(String),

    // Duration (Spec 5.1 & 5.4) - Base duration or Grace notes. 
    // DOES NOT consume trailing dots, leaving them for `Token::Dot`.
    #[regex(r":(grace(\.slash|\.noSlash)?|[0-9]+)", |lex| lex.slice().to_string())]
    DurationLit(String),

    // Tablature Coordinate (Spec 8.1) e.g., 0-6, x-5, 12-2
    #[regex(r"(?i)[0-9x]+-[0-9]+", |lex| lex.slice().to_string())]
    TabLit(String),

    // Pitch Literal (Spec 6 & 19) e.g., c4, f#5, ebqs2, a4+10
    // Priority 3 ensures 'c4' is parsed as Pitch, not Identifier.
    #[regex(r"(?i)[a-g](qs|qf|tqs|tqf|bb|x|#|b|n)*[0-9]?([+-][0-9]+)?", |lex| lex.slice().to_string(), priority=3)]
    PitchLit(String),

    // Attribute Literal (Spec 18/21) e.g., .stacc, .8va
    // Bypasses identifier restrictions to allow leading numbers after the dot.
    #[regex(r"\.[a-zA-Z0-9_]+", |lex| lex.slice().to_string())]
    AttributeLit(String),

    // ========================================================================
    // 7. IDENTIFIERS (Lowest Priority)
    // ========================================================================
    // User-defined staves, percussion keys, and variable names.
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string(), priority=1)]
    Identifier(String),

    // ========================================================================
    // 8. ERROR RECOVERY
    // ========================================================================
    // Trap C-style comments to fail gracefully if user confuses syntax.
    #[regex(r"//.*", |_| false)] 
    InvalidComment,
}

// ============================================================================
// UNIT TESTS
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn lex(input: &str) -> Vec<Token> {
        Token::lexer(input).filter_map(Result::ok).collect()
    }

    #[test]
    fn test_keywords() {
        let tokens = lex("tenuto META def MeAsUrE group");
        assert_eq!(tokens, vec![
            Token::KwTenuto, Token::KwMeta, Token::KwDef, 
            Token::KwMeasure, Token::KwGroup
        ]);
    }

    #[test]
    fn test_v2_1_compound_sigils() {
        // Spec 2.6: Ensures these are treated as atomic units
        let tokens = lex("@{ <[ ]>");
        assert_eq!(tokens, vec![
            Token::MapStart,
            Token::VoiceBracketStart,
            Token::VoiceBracketEnd,
        ]);
    }

    #[test]
    fn test_domain_primitives() {
        // Pitch with cents and microtones
        let tokens = lex("c4 f#5 ebqs2 a4+10");
        assert_eq!(tokens, vec![
            Token::PitchLit("c4".into()),
            Token::PitchLit("f#5".into()),
            Token::PitchLit("ebqs2".into()),
            Token::PitchLit("a4+10".into()),
        ]);

        // Grace notes and dotted durations
        let tokens = lex(":grace.slash :4. :16");
        assert_eq!(tokens, vec![
            Token::DurationLit(":grace.slash".into()),
            Token::DurationLit(":4".into()),
            Token::Dot,
            Token::DurationLit(":16".into()),
        ]);

        // Tablature
        let tokens = lex("0-6 x-5");
        assert_eq!(tokens, vec![
            Token::TabLit("0-6".into()),
            Token::TabLit("x-5".into()),
        ]);

        // Frequency
        let tokens = lex("hz(440) hz(432.5)");
        assert_eq!(tokens, vec![
            Token::FreqLit("hz(440)".into()),
            Token::FreqLit("hz(432.5)".into()),
        ]);
    }

    #[test]
    fn test_attributes_vs_identifiers() {
        // Ensure .8va is captured correctly, and vln is an identifier
        let tokens = lex("vln .8va .stacc");
        assert_eq!(tokens, vec![
            Token::Identifier("vln".into()),
            Token::AttributeLit(".8va".into()),
            Token::AttributeLit(".stacc".into()),
        ]);
    }

    #[test]
    fn test_structural_barlines() {
        let tokens = lex("| |: :| :|: || |]");
        assert_eq!(tokens, vec![
            Token::Pipe,
            Token::RepeatStart,
            Token::RepeatEnd,
            Token::RepeatDouble,
            Token::DoubleBar,
            Token::FinalBar,
        ]);
    }

    #[test]
    fn test_comments_and_whitespace() {
        let tokens = lex("c4 %% this is a comment\n d4");
        assert_eq!(tokens, vec![
            Token::PitchLit("c4".into()),
            Token::PitchLit("d4".into()),
        ]);
    }
}