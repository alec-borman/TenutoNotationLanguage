<div align="center">

# Tenuto Reference Compiler (tenutoc)

**The Declarative, Physics-Based Domain Specific Language for Musical Intent**

[Read the Specification](https://github.com/alec-borman/TenutoNotationLanguage/blob/main/tenuto-specification.md) • 
[Download Binary](https://github.com/alec-borman/TenutoNotationLanguage/releases) • 
[Contribute](https://github.com/alec-borman/TenutoNotationLanguage/blob/main/CONTRIBUTING.md)

</div>

## Table of Contents
1. [Introduction](#introduction)
2. [Core Philosophy](#core-philosophy)
3. [Key Features](#key-features)
4. [Installation](#installation)
5. [Getting Started](#getting-started)
6. [Architecture Overview](#architecture-overview)
7. [Syntax Reference](#syntax-reference)
8. [Development Roadmap](#development-roadmap)
9. [Contributing Guidelines](#contributing-guidelines)
10. [License](#license)

## Introduction

Tenuto is a domain-specific language designed to address the semantic gap in digital music representation. Traditional formats such as MusicXML and MIDI operate at either visual (layout, typography) or mechanical (event-based) abstraction levels, neither of which effectively captures musical intent.

Tenuto bridges this gap by treating musical composition as a declarative programming task. The language enables composers to define instrument physics (tuning, range, capabilities) separately from performance logic, employing a sophisticated inference engine to compute timing, beaming, and voice leading at compile time.

`tenutoc` is the official reference compiler implementing the Tenuto v2.0 specification, written in Rust for performance and reliability.

## Core Philosophy

### 1. Contextual Persistence (Sticky State)
Musical notation inherently relies on context—once established, parameters such as duration, octave, and articulation persist until explicitly changed. Tenuto formalizes this intuition through its "sticky state" system, significantly reducing verbosity compared to XML-based formats.

### 2. Rational Temporal Arithmetic
Rhythmic precision is fundamental to musical integrity. Tenuto employs rational numbers (fractions) throughout its timing engine, ensuring exact representation of tuplets, nested rhythms, and polyrhythms without floating-point approximation errors.

### 3. Separation of Physics and Logic
The language distinguishes between instrument definitions (physics) and musical content (logic). This separation enables musical patterns to be transposed between instruments with different physical constraints without rewriting the underlying musical ideas.

## Key Features

- **Deterministic Linearization**: Flattens complex nested structures into a single absolute timeline
- **Polyphonic Voice Management**: Native support for independent rhythmic threads within a single staff
- **Arbitrary Tuplet Nesting**: Recursive handling of complex rhythmic subdivisions
- **High-Performance Compilation**: Built on Logos lexer and Chumsky parser for rapid processing
- **Comprehensive Test Coverage**: Rigorous testing ensures reliability across edge cases
- **MIDI Export**: Native generation of Standard MIDI Files (SMF) for immediate playback

## Installation

### Prerequisites
- Rust 1.70 or later (for source compilation)

### Binary Distribution (Recommended)
Pre-compiled binaries for Windows, macOS, and Linux are available from the [Releases](https://github.com/alec-borman/TenutoNotationLanguage/releases) page. Download the appropriate binary for your platform and add it to your system PATH.

### Source Compilation
```bash
# Clone the repository
git clone https://github.com/alec-borman/TenutoNotationLanguage.git
cd TenutoNotationLanguage

# Build in release mode
cargo build --release

# Verify installation
./target/release/tenutoc --version
```

## Getting Started

### Example: Simple Melody
Create a file named `example.ten` with the following content:

```rust
tenuto {
    meta {
        title: "Example Composition",
        tempo: 120
    }

    // Instrument definition
    def violin "Violin" patch="Violin"

    // Musical content
    measure 1 {
        // Sticky state: Octave 4, Quarter notes
        violin: c4:4 d e f |
    }
    
    measure 2 {
        // Duration changes to Half note
        violin: g:2 a |
    }
}
```

### Compilation
```bash
tenutoc --input example.ten --output example.mid
```

### Output Interpretation
The compiler generates an intermediate representation showing the linearized timeline:

```
✅ Phase 1: Lexing Complete (24 tokens)
✅ Phase 2: Parsing Complete.
--- Starting Inference Engine ---
✅ Phase 3: Linearization Complete.
    Title: Example Composition
    Tempo: 120 BPM
--- Starting MIDI Encoder ---
🎹 Saved MIDI to "example.mid"
```

## Architecture Overview

### Compilation Pipeline
The compiler implements a three-phase transformation pipeline:

1. **Lexical Analysis** (`src/lexer.rs`)
   - Tokenization via Logos lexer
   - Comment stripping and whitespace normalization
   - UTF-8 text to typed token stream conversion

2. **Syntactic Analysis** (`src/parser.rs`)
   - Recursive descent parsing with error recovery
   - Abstract Syntax Tree (AST) construction
   - Structural validation

3. **Semantic Analysis & Linearization** (`src/ir.rs`)
   - Context-aware inference engine
   - Rational time scaling and tuplet resolution
   - Absolute timeline generation
   - MIDI pitch resolution

### Data Flow
```
Source Text → Tokens → AST → Intermediate Representation → MIDI Output
```

## Syntax Reference

### Basic Note Entry
```rust
piano: c4:4 d e f |        // Quarter notes C4, D4, E4, F4
```

### Chords
```rust
guitar: [c4 e g]:2 |       // C major triad, half note duration
```

### Tuplets
```rust
violin: (c d e):3/2 |      // Triplet: three notes in time of two
```

### Polyphonic Voices
```rust
measure 1 {
    piano: {
        v1: c5:2 e g |     // Upper voice
        v2: c3:4 e g c4 |  // Lower voice
    }
}
```

### Instrument Definition
```rust
def piano "Grand Piano" 
    range=[A0:C8]
    patch="Acoustic Grand Piano"
    style=standard
```

## Development Roadmap

| Phase | Component | Status | Target Release |
|-------|-----------|--------|----------------|
| I | Lexer & Parser | ✅ Complete | v2.0 |
| II | Inference Engine | ✅ Complete | v2.0 |
| III | MIDI Export | ✅ Complete | v2.0 |
| IV | MusicXML Export | ⏳ Planned | v2.2 |
| V | SVG Engraving | ⏳ Planned | v2.3 |
| VI | Language Server Protocol | ⏳ Planned | v2.4 |

## Contributing Guidelines

### Development Workflow
1. Fork the repository and create a feature branch
2. Implement changes with accompanying tests
3. Ensure all tests pass: `cargo test`
4. Format code: `cargo fmt`
5. Submit a pull request with clear description

### Code Standards
- Adhere to Rust best practices and idiomatic patterns
- Include comprehensive documentation for public APIs
- Write tests for new functionality in `tests/suite.rs`
- Follow the "parse, don't validate" principle

### Issue Reporting
When reporting issues, please include:
- Tenuto source file demonstrating the problem
- Expected vs. actual behavior
- Compiler version and platform information

## License

Copyright © 2024 Alec Borman and Tenuto Working Group.

This project is licensed under the MIT License. See [LICENSE](https://github.com/alec-borman/TenutoNotationLanguage/blob/main/LICENSE) for complete terms.

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions...

---

<div align="center">
<sub>Maintained by <a href="https://github.com/alec-borman">Alec Borman</a> and the Tenuto Working Group.</sub>
</div>
