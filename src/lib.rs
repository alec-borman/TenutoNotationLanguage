//! # Tenuto Reference Compiler (tenutoc) Core Library
//! 
//! Implements the **Tenuto 2.1.0 Specification**.
//! 
//! **V2.1.0 Milestone:** Introduces Deterministic LL(1) Parsing via compound sigils:
//! - `@{ }` (Map Sigil): Used for Key-Value Data Maps, Global Metadata, and Attributes.
//! - `<[ ]>` (Voice Brackets): Used for Polyphonic Multi-Voice Blocks.
//! 
//! This eliminates the "Metadata Trap" of V2.0 and guarantees linear-time parsing.

pub mod ast;          // Defines the 2.1 Logical Structure (Score, TopLevel, Event)
pub mod lexer;        // Emits V2.1 Tokens (MapStart, VoiceBracketStart, etc.)
pub mod parser;       // Deterministic Chumsky Parser
pub mod preprocessor; // Macro and Variable Resolution Engine
pub mod spelling;     // FIXED: Added the Spelling Engine to the module tree!
pub mod ir;           // Rational Temporal Engine (Sticky State)
pub mod midi;         // Standard MIDI File 1.0/2.0 Exporter
pub mod rebar;
pub mod xml;

pub use crate::ast::*; 

use logos::Logos;
use chumsky::Parser;
use thiserror::Error;

/// Exhaustive Compiler Error Reference (Spec Section 24)
#[derive(Error, Debug)]
pub enum TenutoError {
    // 1000-Series: Lexical & Meta Errors
    #[error("E1001: Malformed Token at position {0}")] MalformedToken(usize),
    #[error("E1002: Unbalanced Delimiter or Syntax Error - {0}")] SyntaxError(String),
    #[error("E1004: Version Incompatible - Requested {0}")] VersionIncompatible(String),
    
    // 2000-Series: Definition & Import Errors
    #[error("E2001: Undefined Identifier - '{0}'")] UndefinedIdentifier(String),
    #[error("E2002: Duplicate Definition - '{0}'")] DuplicateDefinition(String),
    #[error("E2003: Import Failure - Could not resolve '{0}'")] ImportFailure(String),
    #[error("E2004: Circular Import detected - '{0}'")] CircularImport(String),
    
    // 3000-Series: Time & Structure Errors
    #[error("E3002: Voice Sync Failure in staff '{0}'. Lengths: {1:?}")] VoiceSyncFailure(String, Vec<u64>),
    #[error("E3004: Structure Mismatch at tick {0}")] StructureMismatch(u64),
    
    // 4000-Series: Attribute & Value Errors
    #[error("E4002: Invalid Type Cast - {0}")] InvalidTypeCast(String),
    #[error("E4003: Value Out of Range - {0}")] ValueOutOfRange(String),
    
    // 5000-Series: Macro & Pre-Processor Errors
    #[error("E5001: Circular Reference detected in macro '{0}'")] CircularReference(String),
    #[error("E5002: Recursion Limit Exceeded (>{1}) in macro '{0}'")] RecursionLimitExceeded(String, usize),
    #[error("E5003: Argument Mismatch in '{0}' - {1}")] ArgumentMismatch(String, String),
    
    // 9000-Series: System & Implementation Errors
    #[error("F9001: IO Error - {0}")] IoError(#[from] std::io::Error),
    #[error("F9002: Internal Compiler Error - {0}")] InternalError(String),
}

/// The main compilation orchestrator
pub struct Compiler {
    pub strict_mode: bool,
}

impl Default for Compiler {
    fn default() -> Self { Self::new() }
}

impl Compiler {
    /// Initializes a new compiler instance in Lenient mode (default)
    pub fn new() -> Self { Self { strict_mode: false } }

    /// Builder pattern to enable Strict Mode (Spec 22.2)
    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Orchestrates the entire compilation pipeline from raw text to MIDI bytes.
    pub fn compile_to_midi(&self, source_code: &str) -> Result<Vec<u8>, TenutoError> {
        
        // Phase 1: Lexical Analysis
        let lexer = lexer::Token::lexer(source_code);
        let tokens: Vec<_> = lexer.spanned()
            .filter_map(|(res, span)| match res {
                Ok(token) if token != lexer::Token::InvalidComment => Some((token, span)),
                _ => None, // Invalid tokens are dropped here; rigorous handling exists in `main.rs` CLI
            })
            .collect();

        // Phase 2: Parsing (Deterministic LL(1) via V2.1 Sigils)
        let stream = chumsky::Stream::from_iter(source_code.len()..source_code.len() + 1, tokens.into_iter());
        let (ast_opt, parse_errs) = parser::parser().parse_recovery(stream);

        // Enforce Strict Mode compilation halt on parser warnings/errors
        if self.strict_mode && !parse_errs.is_empty() {
            return Err(TenutoError::SyntaxError("Parsing failed due to syntax errors in Strict Mode.".into()));
        }

        let raw_ast = ast_opt.ok_or_else(|| TenutoError::SyntaxError("Could not generate valid AST.".into()))?;
        
        // Version Check (Spec 22.1)
        if let Some(ver) = &raw_ast.header_version {
            if ver == "3.0" {
                return Err(TenutoError::VersionIncompatible(ver.clone()));
            }
        }
        
        // Phase 2.5: Preprocessor Expansion (Macros, Variables, Maps)
        let mut preprocessor = preprocessor::Preprocessor::new(std::collections::HashMap::new());
        let expanded_ast = preprocessor.expand(raw_ast)
            .map_err(|e| TenutoError::InternalError(format!("Preprocessor failed: {}", e)))?;

        // Phase 3: Inference Engine (Sticky State & Rational Time)
        let timeline = ir::compile(expanded_ast, self.strict_mode)
            .map_err(|e| TenutoError::InternalError(format!("Inference engine failed: {}", e)))?;

        // Phase 4: Backend Emission (MIDI 1.0/2.0)
        let midi_bytes = midi::export(&timeline)
            .map_err(|e| TenutoError::InternalError(format!("MIDI export failed: {}", e)))?;

        Ok(midi_bytes)
    }
}