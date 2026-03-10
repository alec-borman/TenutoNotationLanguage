//! # Tenuto Reference Compiler (tenutoc) CLI
//! 
//! The primary entry point for compiling `.ten` files into MIDI or MusicXML.
//! 
//! **V3.0.0 Updates (The Producer Update):**
//! Contains the 100% Core-Compliant Test Matrix for all 9 Architectural Slices,
//! validating Euclidean distribution, Action Notation, Generative Ergonomics,
//! Audio Sampling, Synth Physics, and Visual-Acoustic Demarcation.

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{Parser as ChumskyParser, Stream};
use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;
use logos::Logos;

use tenutoc::lexer::Token;
use tenutoc::parser::parser;
use tenutoc::preprocessor::Preprocessor; 
use tenutoc::ir;                         
use tenutoc::midi;                       
use tenutoc::rebar::VisualScore;
use tenutoc::xml;

/// Reference Compiler for Tenuto v3.0.0 (The Producer Update)
#[derive(Parser, Debug)]
#[command(name = "tenutoc")]
#[command(version = "3.0.0")]
#[command(about = "Compiles Tenuto v3.0 DSL into MIDI and MusicXML.", long_about = None)]
struct Cli {
    /// Input source file (.ten)
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    /// Output file (.mid, .xml, .musicxml)
    #[arg(short, long, value_name = "OUT")]
    output: Option<PathBuf>,

    /// Enable Strict Mode (Halts on warnings, enforces strict barline resets)
    #[arg(short, long)]
    strict: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    println!("🎵 tenutoc v3.0.0 (The Producer Update)");
    println!("Reading {:?}", cli.input);

    let filename = cli.input.file_name().unwrap().to_string_lossy().to_string();
    let source_code = std::fs::read_to_string(&cli.input).map_err(|e| {
        format!("F9001: IO Error - Could not read file {:?}: {}", cli.input, e)
    })?;

    let lexer = Token::lexer(&source_code);
    let mut token_stream = Vec::new();
    let mut has_lex_errors = false;

    for (res, span) in lexer.spanned() {
        match res {
            Ok(Token::InvalidComment) => {
                Report::build(ReportKind::Error, &filename, span.start)
                    .with_message("E1001: Invalid Comment Syntax")
                    .with_label(Label::new((&filename, span)).with_message("Tenuto uses `%%` for comments").with_color(Color::Red))
                    .finish().print((&filename, Source::from(&source_code))).unwrap();
                has_lex_errors = true;
            }
            Ok(token) => token_stream.push((token, span)),
            Err(_) => {
                Report::build(ReportKind::Error, &filename, span.start)
                    .with_message("E1001: Malformed Token")
                    .with_label(Label::new((&filename, span)).with_message("Unrecognized character sequence").with_color(Color::Red))
                    .finish().print((&filename, Source::from(&source_code))).unwrap();
                has_lex_errors = true;
            }
        }
    }

    if has_lex_errors && cli.strict {
        eprintln!("🔥 Compilation halted due to lexical errors (Strict Mode).");
        std::process::exit(1);
    }
    
    println!("✅ Phase 1: Lexical Analysis Complete.");

    let source_len = source_code.chars().count();
    let stream = Stream::from_iter(source_len..source_len + 1, token_stream.into_iter());
    let (ast_opt, parse_errs) = parser().parse_recovery(stream);

    let mut has_parse_errors = false;
    for err in parse_errs {
        has_parse_errors = true;
        Report::build(ReportKind::Error, &filename, err.span().start)
            .with_message("E1002: Syntax Error")
            .with_label(Label::new((&filename, err.span())).with_message(format!("{:?}", err.reason())).with_color(Color::Yellow))
            .finish().print((&filename, Source::from(&source_code))).unwrap();
    }

    if has_parse_errors && cli.strict {
        eprintln!("🔥 Compilation halted due to syntax errors (Strict Mode).");
        std::process::exit(1);
    }

    if let Some(score) = ast_opt {
        println!("✅ Phase 2: Deterministic LL(1) Parsing Complete.");
        
        let mut preprocessor = Preprocessor::new(HashMap::new());
        let expanded_score = preprocessor.expand(score)
            .map_err(|e| format!("E5001: Preprocessor Error - {}", e))?;
        println!("✅ Phase 2.5: Macros & Variables Expanded.");

        let timeline = ir::compile(expanded_score, cli.strict)
            .map_err(|e| format!("E3001: Inference Error - {}", e))?;
        println!("✅ Phase 3: Absolute Timeline Generated.");

        let output_path = cli.output.unwrap_or_else(|| cli.input.with_extension("mid"));
        let ext = output_path.extension().unwrap_or_default().to_string_lossy().to_lowercase();

        if ext == "xml" || ext == "musicxml" {
            println!("✅ Phase 3.5: Rebarring & Slicing Timeline.");
            let visual_score = VisualScore::build(&timeline);
            
            println!("✅ Phase 4: Exporting MusicXML.");
            let xml_string = xml::export(&visual_score, timeline.ppq)
                .map_err(|e| format!("F9002: MusicXML Export Error - {}", e))?;
            
            std::fs::write(&output_path, xml_string)?;
            println!("🚀 Successfully compiled to Sheet Music: {:?}", output_path);
            
        } else {
            println!("✅ Phase 4: Exporting MIDI.");
            let midi_bytes = midi::export(&timeline)
                .map_err(|e| format!("F9002: MIDI Export Error - {}", e))?;
            
            std::fs::write(&output_path, midi_bytes)?;
            println!("🚀 Successfully compiled to Audio/MIDI: {:?}", output_path);
        }

    } else {
        eprintln!("🔥 Fatal: Parser could not recover a valid AST.");
        std::process::exit(1);
    }

    Ok(())
}

// ============================================================================
// COMPILER INTEGRATION TESTS
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use tenutoc::ir::{EventKind, Rational, Timeline, LyricExtension};
    use tenutoc::spelling::{AccidentalDisplay, Step};

    fn compile_source(src: &str, strict_mode: bool) -> Result<Timeline, String> {
        let lexer = Token::lexer(src);
        let token_stream: Vec<_> = lexer.spanned()
            .filter_map(|(res, span)| res.ok().map(|t| (t, span)))
            .filter(|(tok, _)| *tok != Token::InvalidComment)
            .collect();

        let len = src.chars().count();
        let stream = Stream::from_iter(len..len + 1, token_stream.into_iter());
        
        let (ast_opt, errs) = tenutoc::parser::parser().parse_recovery(stream);
        if !errs.is_empty() { return Err(format!("Parse errors: {:?}", errs)); }
        
        let ast = ast_opt.expect("Failed to produce AST");
        let mut preprocessor = Preprocessor::new(HashMap::new());
        let expanded_ast = preprocessor.expand(ast)?;
        
        ir::compile(expanded_ast, strict_mode)
    }

    #[test]
    fn test_rational_engine() {
        assert_eq!(Rational::new(2, 4).unwrap().num, 1);
        assert_eq!(Rational::new(1, 4).unwrap().to_ticks(1920), 1920); 
    }

    #[test]
    fn test_stage_4_multivoice_polyphony() {
        let src = r#"
        tenuto "3.0" {
            def sax "Sax"
            measure 1 {
                sax: <[
                    v1: c4 d4 |
                    v2: g3 :2 |
                ]>
            }
        }
        "#;
        assert!(compile_source(src, true).is_ok()); 
    }

    #[test]
    fn test_spelling_engine_integration() {
        let src = r#"
        tenuto "3.0" {
            meta @{ key: "D" } 
            def vln "Violin" style=standard
            measure 1 {
                vln: f4:4 c#4:4 |
            }
        }
        "#;
        
        let timeline = compile_source(src, false).unwrap();
        let vln_track = timeline.tracks.get("vln").unwrap();

        if let EventKind::Note { spelling, .. } = &vln_track.events[0].kind {
            assert_eq!(spelling.step, Step::F);
            assert_eq!(spelling.alter, 0);
            assert_eq!(spelling.display, AccidentalDisplay::Explicit); 
        } else { panic!("Expected Note"); }

        if let EventKind::Note { spelling, .. } = &vln_track.events[1].kind {
            assert_eq!(spelling.step, Step::C);
            assert_eq!(spelling.alter, 1);
            assert_eq!(spelling.display, AccidentalDisplay::Implicit); 
        }
    }

    #[test]
    fn test_v3_euclidean_rhythms() {
        let src = r#"
        tenuto "3.0" {
            def drm "Kit" style=grid map=@{ k: 36 }
            measure 1 {
                drm: (k:2):3/8 | 
            }
        }
        "#;
        let timeline = compile_source(src, false).unwrap();
        let track = timeline.tracks.get("drm").unwrap();
        assert_eq!(track.events.len(), 8);
        assert!(matches!(track.events[0].kind, EventKind::Note { pitch_midi: 36, .. }));
        assert_eq!(track.events[0].duration_ticks, 480);
        assert!(matches!(track.events[1].kind, EventKind::Rest));
    }

    #[test]
    fn test_v3_action_notation_spacer() {
        let src = r#"
        tenuto "3.0" {
            def sub "808" style=synth
            measure 1 {
                sub: <[
                    v1: c2:1 |
                    v2: s:1.cc(7,[0, 127], "exp") |
                ]>
            }
        }
        "#;
        let timeline = compile_source(src, false).unwrap();
        let track = timeline.tracks.get("sub").unwrap();
        let spaces: Vec<_> = track.events.iter().filter(|e| e.kind == EventKind::Space).collect();
        assert_eq!(spaces.len(), 1);
        assert_eq!(spaces[0].duration_ticks, 7680);
        assert_eq!(spaces[0].cc_automations[0].controller, 7);
        assert_eq!(spaces[0].cc_automations[0].end_val, 127);
    }

    #[test]
    fn test_v3_auto_pad_voices() {
        let src = r#"
        tenuto "3.0" {
            meta @{ auto_pad_voices: true }
            def pno "Piano" style=standard
            measure 1 {
                pno: <[
                    v1: c4:4 d e f | 
                    v2: c3:2       | 
                ]>
            }
        }
        "#;
        let timeline = compile_source(src, true).unwrap(); 
        let track = timeline.tracks.get("pno").unwrap();
        assert_eq!(track.events.len(), 6);
    }

    #[test]
    fn test_v3_relative_pitch_heuristic() {
        let src = r#"
        tenuto "3.0" {
            def lead "Lead Synth" style=relative
            measure 1 {
                lead: b4:4 c f b |
            }
        }
        "#;
        let timeline = compile_source(src, false).unwrap();
        let track = timeline.tracks.get("lead").unwrap();
        if let EventKind::Note { pitch_midi, .. } = track.events[0].kind { assert_eq!(pitch_midi, 71); }
        if let EventKind::Note { pitch_midi, .. } = track.events[1].kind { assert_eq!(pitch_midi, 72); }
        if let EventKind::Note { pitch_midi, .. } = track.events[2].kind { assert_eq!(pitch_midi, 77); }
        if let EventKind::Note { pitch_midi, .. } = track.events[3].kind { assert_eq!(pitch_midi, 83); }
    }

    #[test]
    fn test_v3_micro_timing() {
        let src = r#"
        tenuto "3.0" {
            meta @{ tempo: 120 }
            def drm "Kit" style=grid map=@{ sn: 38 }
            measure 1 {
                drm: sn:4.pull(100ms) sn:4.push(100ms) | 
            }
        }
        "#;
        let timeline = compile_source(src, false).unwrap();
        let track = timeline.tracks.get("drm").unwrap();
        assert_eq!(track.events[0].physical_tick_offset, 384);
        assert_eq!(track.events[1].physical_tick_offset, -384);
    }
    
    #[test]
    fn test_v3_flam_rudiment() {
        let src = r#"
        tenuto "3.0" {
            def drm "Kit" style=grid map=@{ sn: 38 }
            measure 1 { drm: sn:4.flam | }
        }
        "#;
        let timeline = compile_source(src, false).unwrap();
        let track = timeline.tracks.get("drm").unwrap();
        assert_eq!(track.events.len(), 2);
        assert_eq!(track.events[0].is_grace, true);
        assert!(track.events[0].physical_tick_offset < 0); 
    }

    #[test]
    fn test_v3_swing_algorithm() {
        let src = r#"
        tenuto "3.0" {
            meta @{ swing: 66 } 
            def drm "Kit" style=grid map=@{ sn: 38 }
            measure 1 {
                drm: sn:16 sn:16 | 
            }
        }
        "#;
        let timeline = compile_source(src, false).unwrap();
        let track = timeline.tracks.get("drm").unwrap();
        assert_eq!(track.events[0].physical_tick_offset, 0);
        assert_eq!(track.events[1].physical_tick_offset, 154);
    }
    
    #[test]
    fn test_v3_humanization_determinism() {
        let src = r#"
        tenuto "3.0" {
            meta @{ humanize: 0.05 }
            def pno "Piano" style=standard
            measure 1 {
                pno: c4:4 c4:4 c4:4 c4:4 | 
            }
        }
        "#;
        let t1 = compile_source(src, false).unwrap();
        let t2 = compile_source(src, false).unwrap();
        let trk1 = t1.tracks.get("pno").unwrap();
        let trk2 = t2.tracks.get("pno").unwrap();
        assert_eq!(trk1.events[0].physical_tick_offset, trk2.events[0].physical_tick_offset);
        let has_shift = trk1.events.iter().any(|e| e.physical_tick_offset != 0);
        assert!(has_shift, "Humanize should have shifted at least one tick");
    }

    #[test]
    fn test_v3_concrete_slicing() {
        let src = r#"
        tenuto "3.0" {
            def vox "Vocal" style=concrete src="./vocals.wav" map=@{ a:[0.0s, 1.2s], b:[1.2s, 2.4s] }
            measure 1 {
                vox: a:2.slice(4).stretch.reverse b:2 |
            }
        }
        "#;
        let timeline = compile_source(src, false).unwrap();
        let track = timeline.tracks.get("vox").unwrap();
        assert_eq!(track.events.len(), 5);
        if let EventKind::Concrete { key, params } = &track.events[0].kind {
            assert_eq!(key, "a");
            assert_eq!(params.sample_start, 0);
            assert_eq!(params.sample_end, 300); 
            assert!(params.stretch);
            assert!(params.reverse);
        } else { panic!("Expected Concrete Event"); }
    }

    #[test]
    fn test_v3_monophonic_choke_groups() {
        let src = r#"
        tenuto "3.0" {
            def sub "808" style=synth cut_group=1
            def lead "Lead" style=synth cut_group=1
            measure 1 {
                sub: c2:1 |
                lead: r:2 c4:2 |
            }
        }
        "#;
        let timeline = compile_source(src, false).unwrap();
        assert_eq!(timeline.tracks.get("sub").unwrap().events[0].gate_ticks, 3840);
    }

    #[test]
    fn test_v3_synth_physics_parameters() {
        let src = r#"
        tenuto "3.0" {
            meta @{ tempo: 120 } 
            def sub "808" style=synth env=@{ a: 10ms, d: 500ms }
            measure 1 {
                sub: c2:4 c3:2.accelerate(-12) |
            }
        }
        "#;
        let timeline = compile_source(src, false).unwrap();
        let track = timeline.tracks.get("sub").unwrap();
        assert_eq!(track.events[1].synth_accelerate_semitones, Some(-12.0));
    }

    // ========================================================================
    // MASTER SLICE: LYRICS, REPEATS, AND DEMARCATION
    // ========================================================================

    #[test]
    fn test_v3_lyric_engine_mapping() {
        let src = r#"
        tenuto "3.0" {
            def vox "Singer" style=standard
            measure 1 {
                vox: c4:4 d e f g |
                vox.lyric: "Hal - le * lu _ jah"
            }
        }
        "#;
        let timeline = compile_source(src, false).unwrap();
        let track = timeline.tracks.get("vox").unwrap();
        
        // Note 0 (c4) -> "Hal" + Hyphen extension
        assert_eq!(track.events[0].lyric.as_deref(), Some("Hal"));
        assert_eq!(track.events[0].lyric_extension, LyricExtension::Hyphen);
        
        // Note 1 (d4) -> "le"
        assert_eq!(track.events[1].lyric.as_deref(), Some("le"));
        
        // Note 2 (e4) -> Skipped by asterisk
        assert!(track.events[2].lyric.is_none());
        
        // Note 3 (f4) -> "lu" + Melisma extension
        assert_eq!(track.events[3].lyric.as_deref(), Some("lu"));
        assert_eq!(track.events[3].lyric_extension, LyricExtension::Melisma);

        // Note 4 (g4) -> "jah"
        assert_eq!(track.events[4].lyric.as_deref(), Some("jah"));
    }

    #[test]
    fn test_v3_graph_unroller_repeats() {
        let src = r#"
        tenuto "3.0" {
            def pno "Piano" style=standard
            // FIX: repeat blocks contain standard logic
            repeat 3 {
                pno: c4:1 | 
            }
        }
        "#;
        let timeline = compile_source(src, false).unwrap();
        let track = timeline.tracks.get("pno").unwrap();
        
        // 1 Whole Note * 3 loops = 3 total events.
        assert_eq!(track.events.len(), 3);
        // Tick timeline should unroll sequentially (0, 7680, 15360)
        assert_eq!(track.events[0].tick, 0);
        assert_eq!(track.events[1].tick, 7680);
        assert_eq!(track.events[2].tick, 15360);
    }

    #[test]
    fn test_v3_visual_demarcation() {
        let src = r#"
        tenuto "3.0" {
            def vln "Violin" style=standard
            def sub "808" style=synth
            measure 1 {
                vln: c4:1 |
                sub: c2:1 |
            }
        }
        "#;
        let timeline = compile_source(src, false).unwrap();
        
        // Standard tracks are flagged to print
        assert_eq!(timeline.tracks.get("vln").unwrap().print, true);
        // Synth tracks are explicitly masked from MusicXML
        assert_eq!(timeline.tracks.get("sub").unwrap().print, false);
    }
}