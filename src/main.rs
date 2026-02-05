use clap::Parser;
use std::path::PathBuf;
use logos::Logos;
use chumsky::Parser as ChumskyParser; 
use chumsky::Stream;
use tenutoc::lexer::Token;
use tenutoc::parser::parser; 
use tenutoc::ir; 
use tenutoc::midi; // <--- Import MIDI

#[derive(Parser)]
#[command(name = "tenutoc")]
#[command(version = "2.0.0")]
#[command(about = "Reference Compiler for Tenuto v2.0", long_about = None)]
struct Cli {
    /// Input source file (.ten)
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    /// Output MIDI file (.mid)
    #[arg(short, long, value_name = "OUT")]
    output: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    println!("🎵 tenutoc v2.0.0");
    println!("Reading {:?}", cli.input);

    // 1. Read Source
    let source = std::fs::read_to_string(&cli.input)
        .map_err(|e| format!("F9001: Could not read file {:?}: {}", cli.input, e))?;

    // 2. Lexical Analysis
    let lexer = Token::lexer(source.as_str());
    let token_stream: Vec<(Token, std::ops::Range<usize>)> = lexer.spanned()
        .map(|(tok, span)| match tok {
            Ok(t) => (t, span),
            Err(_) => (Token::InvalidComment, span), 
        })
        .filter(|(tok, _)| *tok != Token::InvalidComment) 
        .collect();

    println!("✅ Phase 1: Lexing Complete ({} tokens)", token_stream.len());

    // 3. Parsing
    let len = source.chars().count();
    let eoi = len..len + 1; 
    let stream = Stream::from_iter(eoi, token_stream.into_iter());

    let (ast, parse_errs) = parser().parse_recovery(stream);
    for err in parse_errs { println!("❌ Parse Error: {:?}", err); }

    if let Some(score) = ast {
        println!("✅ Phase 2: Parsing Complete.");
        
        // 4. Linearization
        println!("--- Starting Inference Engine ---");
        match ir::compile(score) {
            Ok(timeline) => {
                println!("✅ Phase 3: Linearization Complete.");
                println!("    Title: {}", timeline.title);
                println!("    Tempo: {} BPM", timeline.tempo);
                
                // 5. MIDI Export
                if let Some(out_path) = cli.output {
                    println!("--- Starting MIDI Encoder ---");
                    let bytes = midi::export(&timeline)?;
                    std::fs::write(&out_path, bytes)?;
                    println!("🎹 Saved MIDI to {:?}", out_path);
                } else {
                    println!("ℹ️  No output file specified. Use --output <FILE.mid> to save.");
                }
            },
            Err(e) => eprintln!("🔥 Logic Error: {}", e),
        }
    }

    Ok(())
}