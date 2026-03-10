use logos::Logos;

/// The Exhaustive Token Enum for Tenuto 3.0.0
#[derive(Logos, Debug, PartialEq, Eq, Hash, Clone)]
#[logos(skip r"[ \t\r\n\f]+")] 
#[logos(skip r"%%.*")]         
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
    // 3. PUNCTUATION & OPERATORS
    // ========================================================================
    #[token("{")] LBrace,
    #[token("}")] RBrace,
    
    // V2.1 Compound Sigils
    #[token("@{")] MapStart,           
    #[token("<[")] VoiceBracketStart,  
    #[token("]>")] VoiceBracketEnd,    
    
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

    // ========================================================================
    // 4. STRUCTURAL BARLINES
    // ========================================================================
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
    
    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string() 
    })]
    StringLit(String),

    // ========================================================================
    // 6. DOMAIN-SPECIFIC PRIMITIVES (High Priority)
    // ========================================================================
    
    // --- V3.0 SLICE 4: PHYSICAL TIME DOMAIN ---
    // e.g., 15ms, 1.5s, 10ticks
    #[regex(r"(?i)[0-9]+(\.[0-9]+)?(ms|s|ticks)", |lex| lex.slice().to_string(), priority=4)]
    TimeVal(String),

    #[regex(r"(?i)hz\([0-9]+(\.[0-9]+)?\)", |lex| lex.slice().to_string())]
    FreqLit(String),

    #[regex(r":(grace(\.slash|\.noSlash)?|[0-9]+)", |lex| lex.slice().to_string())]
    DurationLit(String),

    #[regex(r"(?i)[0-9xX]+-[1-9][0-9]*", |lex| lex.slice().to_string())]
    TabLit(String),

    #[regex(r"(?i)[a-g](qs|qf|tqs|tqf|bb|x|#|b|n)*[0-9]?([+-][0-9]+)?", |lex| lex.slice().to_string(), priority=3)]
    PitchLit(String),

    #[regex(r"\.[a-zA-Z0-9_]+", |lex| lex.slice().to_string())]
    AttributeLit(String),

    // ========================================================================
    // 7. IDENTIFIERS (Lowest Priority)
    // ========================================================================
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string(), priority=1)]
    Identifier(String),

    // ========================================================================
    // 8. ERROR RECOVERY
    // ========================================================================
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
    fn test_v3_physical_time_val() {
        let tokens = lex("15ms 1.5s 10ticks");
        assert_eq!(tokens, vec![
            Token::TimeVal("15ms".into()),
            Token::TimeVal("1.5s".into()),
            Token::TimeVal("10ticks".into()),
        ]);
    }

    #[test]
    fn test_v2_1_compound_sigils() {
        let tokens = lex("@{ <[ ]>");
        assert_eq!(tokens, vec![
            Token::MapStart,
            Token::VoiceBracketStart,
            Token::VoiceBracketEnd,
        ]);
    }

    #[test]
    fn test_domain_primitives() {
        let tokens = lex("c4 f#5 ebqs2 a4+10");
        assert_eq!(tokens, vec![
            Token::PitchLit("c4".into()),
            Token::PitchLit("f#5".into()),
            Token::PitchLit("ebqs2".into()),
            Token::PitchLit("a4+10".into()),
        ]);

        let tokens = lex(":grace.slash :4. :16");
        assert_eq!(tokens, vec![
            Token::DurationLit(":grace.slash".into()),
            Token::DurationLit(":4".into()),
            Token::Dot,
            Token::DurationLit(":16".into()),
        ]);

        let tokens = lex("0-6 x-5");
        assert_eq!(tokens, vec![
            Token::TabLit("0-6".into()),
            Token::TabLit("x-5".into()),
        ]);

        let tokens = lex("hz(440) hz(432.5)");
        assert_eq!(tokens, vec![
            Token::FreqLit("hz(440)".into()),
            Token::FreqLit("hz(432.5)".into()),
        ]);
    }

    #[test]
    fn test_attributes_vs_identifiers() {
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