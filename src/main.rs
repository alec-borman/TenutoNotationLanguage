use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{Parser as ChumskyParser, Stream};
use clap::Parser;
use logos::Logos;
use std::collections::HashMap;
use std::path::PathBuf;

use tenutoc::lexer::Token;
use tenutoc::parser::parser;
use tenutoc::preprocessor::Preprocessor; 
use tenutoc::ir;                         
use tenutoc::midi;                       
use tenutoc::rebar::VisualScore;
use tenutoc::xml;

/// Reference Compiler for Tenuto v2.1.0
#[derive(Parser, Debug)]
#[command(name = "tenutoc")]
#[command(version = "2.2.0")]
#[command(about = "Compiles Tenuto v2.1 DSL into MIDI and MusicXML.", long_about = None)]
struct Cli {
    /// Input source file (.ten)
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    /// Output file (.mid, .xml, .musicxml)
    #[arg(short, long, value_name = "OUT")]
    output: Option<PathBuf>,

    /// Enable Strict Mode (Halts on warnings, disables auto-correction)
    #[arg(short, long)]
    strict: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    println!("🎵 tenutoc v2.1.0 (Deterministic Engine)");
    println!("Reading {:?}", cli.input);

    // 1. Read Source File
    let filename = cli.input.file_name().unwrap().to_string_lossy().to_string();
    let source_code = std::fs::read_to_string(&cli.input).map_err(|e| {
        format!("F9001: IO Error - Could not read file {:?}: {}", cli.input, e)
    })?;

    // 2. Phase 1: Lexical Analysis
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

    // 3. Phase 2: Parsing (Token Stream -> AST)
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
        
        // 4. Phase 2.5: Macro Pre-Processor
        let mut preprocessor = Preprocessor::new(HashMap::new());
        let expanded_score = preprocessor.expand(score)
            .map_err(|e| format!("E5001: Preprocessor Error - {}", e))?;
        println!("✅ Phase 2.5: Macros & Variables Expanded.");

        // 5. Phase 3: Inference Engine (IR)
        let timeline = ir::compile(expanded_score, cli.strict)
            .map_err(|e| format!("E3001: Inference Error - {}", e))?;
        println!("✅ Phase 3: Absolute Timeline Generated.");

        // 6. Output Routing
        let output_path = cli.output.unwrap_or_else(|| cli.input.with_extension("mid"));
        let ext = output_path.extension().unwrap_or_default().to_string_lossy().to_lowercase();

        if ext == "xml" || ext == "musicxml" {
            // Phase 3.5 & 4: Visual IR and XML
            println!("✅ Phase 3.5: Rebarring & Slicing Timeline.");
            let visual_score = VisualScore::build(&timeline);
            
            println!("✅ Phase 4: Exporting MusicXML.");
            let xml_string = xml::export(&visual_score, timeline.ppq)
                .map_err(|e| format!("F9002: MusicXML Export Error - {}", e))?;
            
            std::fs::write(&output_path, xml_string)?;
            println!("🚀 Successfully compiled to Sheet Music: {:?}", output_path);
            
        } else {
            // Phase 4: Audio/MIDI
            println!("✅ Phase 4: Exporting MIDI 1.0.");
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