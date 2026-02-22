use tenutoc::lexer::Token;
use tenutoc::ir::{self, EventKind, Rational, Timeline};
use tenutoc::preprocessor::Preprocessor;
use logos::Logos;
use chumsky::Parser;
use chumsky::Stream;
use std::collections::HashMap;

// ========================================================================
// CORE TEST RUNNER HELPER
// ========================================================================
fn compile_source(src: &str, strict_mode: bool) -> Result<Timeline, String> {
    let lexer = Token::lexer(src);
    let token_stream: Vec<_> = lexer.spanned()
        .filter_map(|(res, span)| res.ok().map(|t| (t, span)))
        .filter(|(tok, _)| *tok != Token::InvalidComment)
        .collect();

    let len = src.chars().count();
    let stream = Stream::from_iter(len..len + 1, token_stream.into_iter());
    
    let (ast_opt, errs) = tenutoc::parser::parser().parse_recovery(stream);
    
    if !errs.is_empty() {
        return Err(format!("Parse errors: {:?}", errs));
    }
    
    let ast = ast_opt.expect("Failed to produce AST");
    let mut preprocessor = Preprocessor::new(HashMap::new());
    let expanded_ast = preprocessor.expand(ast)?;
    
    ir::compile(expanded_ast, strict_mode)
}

// ========================================================================
// STAGE 1: MATH & RATIONALS
// ========================================================================
#[test]
fn test_rational_engine() {
    assert_eq!(Rational::new(2, 4).num, 1);
    assert_eq!(Rational::new(1, 4).to_ticks(1920), 1920); // Quarter
    assert_eq!(Rational::new(3, 8).to_ticks(1920), 2880); // Dotted Quarter
}

// ========================================================================
// STAGE 2: LEXER & SIGILS
// ========================================================================
#[test]
fn test_lexer_v2_1_sigils() {
    let mut lex = Token::lexer("tenuto @{ <[ ]> c4 :4.");
    assert_eq!(lex.next(), Some(Ok(Token::KwTenuto)));
    assert_eq!(lex.next(), Some(Ok(Token::MapStart)));
    assert_eq!(lex.next(), Some(Ok(Token::VoiceBracketStart)));
    assert_eq!(lex.next(), Some(Ok(Token::VoiceBracketEnd)));
    assert_eq!(lex.next(), Some(Ok(Token::PitchLit("c4".into()))));
    assert_eq!(lex.next(), Some(Ok(Token::DurationLit(":4".into()))));
    assert_eq!(lex.next(), Some(Ok(Token::Dot)));
}

// ========================================================================
// STAGE 3: PARSER STRUCTURE 
// ========================================================================
#[test]
fn test_stage_1_structural() {
    let src = r#"
    tenuto "2.1" {
        meta @{ title: "Basic" }
        group "Section" {
            def sax "Sax" attributes=@{ patch: "piano" }
        }
        measure 1 {
            sax: c4 d4 e4 |
        }
    }
    "#;
    let timeline = compile_source(src, false).unwrap();
    assert_eq!(timeline.title, "Basic");
}

#[test]
fn test_stage_2_boundaries_and_engines() {
    let src = r#"
    tenuto "2.1" {
        def sax "Sax"
        def gtr "Gtr" style=tab tuning=[40, 45, 50, 55, 59, 64]
        def drm "Kit" map=@{ sn: [0, 38] }
        
        measure 1 {
            sax: c4 :2 |
            gtr: 0-6 :4 2-5 |
            drm: sn :2 |
        }
    }
    "#;
    let timeline = compile_source(src, false).unwrap();
    
    // Check Tab Engine Inverse string parsing
    let gtr_track = timeline.tracks.get("gtr").unwrap();
    if let EventKind::Note { pitch_midi, .. } = gtr_track.events[0].kind { assert_eq!(pitch_midi, 40); }
    if let EventKind::Note { pitch_midi, .. } = gtr_track.events[1].kind { assert_eq!(pitch_midi, 47); }
    
    // Check Percussion mapping
    let drm_track = timeline.tracks.get("drm").unwrap();
    if let EventKind::Note { pitch_midi, .. } = drm_track.events[0].kind { assert_eq!(pitch_midi, 38); }
}

#[test]
fn test_stage_4_multivoice_polyphony() {
    let src = r#"
    tenuto "2.1" {
        def sax "Sax"
        measure 1 {
            sax: <[
                v1: c4 d4 |
                v2: g3 :2 |
            ]>
        }
    }
    "#;
    // Tests strict_mode=true to ensure durations perfectly align
    assert!(compile_source(src, true).is_ok()); 
}

// ========================================================================
// STAGE 4: MACROS & ATTRIBUTES
// =========================================================