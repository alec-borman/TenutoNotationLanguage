use clap::Parser;
use std::path::PathBuf;
use logos::Logos;
use chumsky::Parser as ChumskyParser; 
use chumsky::Stream;
use tenutoc::lexer::Token; // Changed from omnic
use tenutoc::parser::parser; 
use tenutoc::ir; 

#[derive(Parser)]
#[command(name = "tenutoc")]
#[command(version = "2.0.0")]
#[command(about = "Reference Compiler for Tenuto v2.0", long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,
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
        // FILTER: Remove InvalidComments/Lexer Errors so the Parser doesn't panic
        .filter(|(tok, _)| *tok != Token::InvalidComment) 
        .collect();

    println!("✅ Phase 1: Lexing Complete ({} tokens)", token_stream.len());

    // 3. Parsing
    let len = source.chars().count();
    let eoi = len..len + 1; 
    let stream = Stream::from_iter(eoi, token_stream.into_iter());

    let (ast, parse_errs) = parser().parse_recovery(stream);
    
    for err in parse_errs {
        println!("❌ Parse Error: {:?}", err);
    }

    if let Some(score) = ast {
        println!("✅ Phase 2: Parsing Complete.");
        
        // 4. Linearization (Phase 3)
        println!("--- Starting Inference Engine ---");
        
        match ir::compile(score) {
            Ok(timeline) => {
                println!("✅ Phase 3: Linearization Complete.");
                println!("Score Title: {}", timeline.title);
                println!("Global Tempo: {} BPM", timeline.tempo);
                
                for (id, track) in timeline.tracks {
                    println!("Track [{}]: {} events", id, track.events.len());
                    for (i, e) in track.events.iter().enumerate() {
                        println!("  {}: Tick {:4} -> Dur {:3} | {:?}", i, e.tick, e.duration_ticks, e.kind);
                    }
                }
            },
            Err(e) => eprintln!("🔥 Logic Error: {}", e),
        }
    }

    Ok(())
}