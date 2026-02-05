use tenutoc::lexer::Token;
use tenutoc::parser::{self, Score, TopLevel, Statement, Event, Value};
use tenutoc::ir::{self, Timeline, EventKind};
use tenutoc::Rational;
use logos::Logos;
use chumsky::Parser;
use chumsky::Stream;

// ========================================================================
// 1. RATIONAL ARITHMETIC TESTS
// ========================================================================

#[test]
fn test_rational_reduction() {
    // 2/4 should reduce to 1/2
    let r = Rational::new(2, 4);
    assert_eq!(r.num, 1);
    assert_eq!(r.den, 2);
}

#[test]
fn test_rational_to_ticks() {
    let ppq = 1920;
    // Quarter note (1/4) = 1920 ticks
    let r = Rational::new(1, 4);
    assert_eq!(r.to_ticks(ppq), 1920);

    // Dotted Quarter (3/8) = 2880 ticks
    let r_dotted = Rational::new(3, 8);
    assert_eq!(r_dotted.to_ticks(ppq), 2880);
}

#[test]
#[should_panic(expected = "Division by Zero")]
fn test_rational_panic() {
    Rational::new(1, 0);
}

// ========================================================================
// 2. LEXER TESTS
// ========================================================================

#[test]
fn test_lexer_keywords_and_literals() {
    let src = "tenuto meta def measure c4 :4. \"String\"";
    let mut lex = Token::lexer(src);

    assert_eq!(lex.next(), Some(Ok(Token::KwTenuto)));
    assert_eq!(lex.next(), Some(Ok(Token::KwMeta)));
    assert_eq!(lex.next(), Some(Ok(Token::KwDef)));
    assert_eq!(lex.next(), Some(Ok(Token::KwMeasure)));
    assert_eq!(lex.next(), Some(Ok(Token::PitchLit("c4".into()))));
    assert_eq!(lex.next(), Some(Ok(Token::DurationLit(":4.".into()))));
    assert_eq!(lex.next(), Some(Ok(Token::StringLit("String".into()))));
}

#[test]
fn test_lexer_comments_are_ignored() {
    let src = "c4 %% This is a comment\n d4";
    let mut lex = Token::lexer(src);

    assert_eq!(lex.next(), Some(Ok(Token::PitchLit("c4".into()))));
    // The comment should be skipped entirely
    assert_eq!(lex.next(), Some(Ok(Token::PitchLit("d4".into()))));
}

#[test]
fn test_lexer_operators() {
    let src = "{ } [ ] : | = , .";
    let mut lex = Token::lexer(src);
    
    assert_eq!(lex.next(), Some(Ok(Token::LBrace)));
    assert_eq!(lex.next(), Some(Ok(Token::RBrace)));
    assert_eq!(lex.next(), Some(Ok(Token::LBracket)));
    assert_eq!(lex.next(), Some(Ok(Token::RBracket)));
    assert_eq!(lex.next(), Some(Ok(Token::Colon)));
    assert_eq!(lex.next(), Some(Ok(Token::Pipe)));
    assert_eq!(lex.next(), Some(Ok(Token::Equals)));
    assert_eq!(lex.next(), Some(Ok(Token::Comma)));
    assert_eq!(lex.next(), Some(Ok(Token::Dot)));
}

// ========================================================================
// 3. PARSER TESTS
// ========================================================================

fn parse_str(src: &str) -> Option<Score> {
    let lexer = Token::lexer(src);
    let token_stream: Vec<(Token, std::ops::Range<usize>)> = lexer.spanned()
        .map(|(tok, span)| (tok.unwrap(), span))
        .filter(|(tok, _)| *tok != Token::InvalidComment)
        .collect();

    let len = src.chars().count();
    let stream = Stream::from_iter(len..len + 1, token_stream.into_iter());
    
    let (ast, _errs) = parser::parser().parse_recovery(stream);
    ast
}

#[test]
fn test_parser_structure() {
    let src = r#"
    tenuto {
        meta { title: "Test" }
        def vln "Violin"
        measure 1 {
            vln: c4 |
        }
    }
    "#;
    
    let ast = parse_str(src).expect("Failed to parse valid score");
    assert_eq!(ast.items.len(), 3); // Meta, Def, Measure
}

#[test]
fn test_parser_attributes() {
    let src = r#"tenuto { measure 1 { vln: c4.stacc.vol(80) | } }"#;
    let ast = parse_str(src).unwrap();
    
    if let TopLevel::Measure { content, .. } = &ast.items[0] {
        if let Statement::Assignment { voices, .. } = &content[0] {
            if let Event::Note { attributes, .. } = &voices[0].events[0] {
                assert_eq!(attributes[0].name, "stacc");
                assert_eq!(attributes[1].name, "vol");
                if let Value::Num(n) = &attributes[1].args[0] {
                    assert_eq!(*n, 80);
                } else { panic!("Expected number arg"); }
            }
        }
    }
}

// ========================================================================
// 4. INFERENCE ENGINE (LINEARIZATION) TESTS
// ========================================================================

#[test]
fn test_inference_sticky_state() {
    // Tests if duration and octave stickiness works
    // c4:4 -> sets Octave 4, Duration 1/4 (1920 ticks)
    // d    -> inherits Octave 4, Duration 1/4
    // e    -> inherits Octave 4, Duration 1/4
    let src = r#"
    tenuto {
        def vln "Violin"
        measure 1 {
            vln: c4:4 d e |
        }
    }
    "#;

    let ast = parse_str(src).unwrap();
    let timeline = ir::compile(ast).expect("Compilation failed");
    let track = timeline.tracks.get("vln").unwrap();

    assert_eq!(track.events.len(), 3);
    
    // Check Event 1 (c4:4)
    assert_eq!(track.events[0].duration_ticks, 1920);
    if let EventKind::Note { pitch, .. } = track.events[0].kind {
        assert_eq!(pitch, 60); // Middle C
    }

    // Check Event 2 (d - sticky)
    assert_eq!(track.events[1].duration_ticks, 1920);
    if let EventKind::Note { pitch, .. } = track.events[1].kind {
        assert_eq!(pitch, 62); // D4
    }
}

#[test]
fn test_inference_dotted_rhythm() {
    // :4. should be 1.5x length of :4 (1920 * 1.5 = 2880)
    let src = r#"
    tenuto {
        def vln "Violin"
        measure 1 {
            vln: c4:4. |
        }
    }
    "#;

    let ast = parse_str(src).unwrap();
    let timeline = ir::compile(ast).unwrap();
    let track = timeline.tracks.get("vln").unwrap();

    assert_eq!(track.events[0].duration_ticks, 2880);
}

#[test]
fn test_inference_accidental_parsing() {
    // c#4 -> 61, db4 -> 61, c4 -> 60
    let src = r#"
    tenuto {
        def vln "Violin"
        measure 1 {
            vln: c#4:4 db4 c4 |
        }
    }
    "#;

    let ast = parse_str(src).unwrap();
    let timeline = ir::compile(ast).unwrap();
    let track = timeline.tracks.get("vln").unwrap();

    // C#4
    if let EventKind::Note { pitch, .. } = track.events[0].kind { assert_eq!(pitch, 61); }
    // Db4
    if let EventKind::Note { pitch, .. } = track.events[1].kind { assert_eq!(pitch, 61); }
    // C4
    if let EventKind::Note { pitch, .. } = track.events[2].kind { assert_eq!(pitch, 60); }
}

#[test]
fn test_inference_rest_handling() {
    // Rests should advance time but produce no Note events in this basic IR implementation
    // (Note: The IR in ir.rs currently skips pushing events for Rests, but advances the cursor)
    // Wait, looking at ir.rs: 
    // AstEvent::Rest ... cursor.current_tick += dur_ticks; (No push to track.events)
    // So we check if the NEXT note is at the correct time.
    
    let src = r#"
    tenuto {
        def vln "Violin"
        measure 1 {
            vln: c4:4 r:4 c4 |
        }
    }
    "#;

    let ast = parse_str(src).unwrap();
    let timeline = ir::compile(ast).unwrap();
    let track = timeline.tracks.get("vln").unwrap();

    // Event 0: c4 at tick 0
    assert_eq!(track.events[0].tick, 0);
    
    // Event 1: c4 at tick 3840 (1920 + 1920 skipped)
    assert_eq!(track.events[1].tick, 3840);
}

#[test]
fn test_metadata_extraction() {
    let src = r#"
    tenuto {
        meta { title: "Heroic Symphony", tempo: 150 }
    }
    "#;
    let ast = parse_str(src).unwrap();
    let timeline = ir::compile(ast).unwrap();

    assert_eq!(timeline.title, "Heroic Symphony");
    assert_eq!(timeline.tempo, 150);
}

#[test]
fn test_def_attributes() {
    let src = r#"
    tenuto {
        def pno "Piano" patch="Acoustic Grand"
    }
    "#;
    let ast = parse_str(src).unwrap();
    let timeline = ir::compile(ast).unwrap();
    
    let track = timeline.tracks.get("pno").unwrap();
    assert_eq!(track.label, "Piano");
    assert_eq!(track.patch, "Acoustic Grand");
}