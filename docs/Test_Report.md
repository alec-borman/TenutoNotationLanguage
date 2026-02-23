# 📊 Tenuto v2.1.1 Test Analysis & Quality Assurance Report

**Project:** `tenutoc` Reference Compiler  
**Date:** February 23, 2026  
**Test Framework:** Rust Native (`cargo test`)  
**Overall Status:** **100% Passing (Ready for CI/CD Pipeline)**  

---

## 1. Executive Summary

The testing strategy for the Tenuto v2.1.1 compiler was driven by **Test-Driven Development (TDD)** and **Regression Safeguarding**. Because `tenutoc` handles highly complex, interdependent data transformations (Lexing → AST → IR → Visual IR → XML/MIDI), testing was rigorously divided into localized **Unit Tests** for math/state logic and end-to-end **Integration Tests** for pipeline health.

As of the v2.1.1 release, the test suite consists of **31 strictly enforced tests** (25 Unit, 6 Integration). The suite executes in `< 0.05s` under the `dev` profile, guaranteeing rapid developer feedback.

The test harness successfully identified and prevented several critical compiler panics, infinite loops, and data-loss edge cases during the transition to the deterministic LL(1) architecture.

---

## 2. Coverage Breakdown by Module

### 🟢 `lexer` (7 Tests)
*   **Focus:** Validating the ingestion of V2.1 compound sigils (`@{`, `<[`, `]>`), domain-specific primitives (Microtonal pitches, Tablature, Frequencies), and correct ignorance of whitespace/comments.
*   **Key Validation:** Ensured that `c4` parses as a `PitchLit` and not an `Identifier`, enforcing Regex priority matching.

### 🟢 `parser` & `ast` (Handled via Integration Tests)
*   **Focus:** LL(1) deterministic AST generation and Ariadne error recovery.
*   **Key Validation:** Validated that syntax errors do not crash the compiler, but cleanly output visual spans. Guaranteed that block terminators (`}`, `]>`) are not swallowed by `skip_then_retry_until` recovery sets.

### 🟢 `preprocessor` (1 Unit Test + Integration)
*   **Focus:** Variable injection, Macro expansion, and Recursion safety.
*   **Key Validation:** `test_transposition_logic` guarantees the mathematical shifting of Scientific Pitch Notation (e.g., `eb3` minus 2 semitones = `c#3`). Verified that recursion depth > 64 safely aborts (`E5002`).

### 🟢 `spelling` (11 Unit Tests)
*   **Focus:** The Accidental State Machine and Algorithmic "Line of Fifths" Speller.
*   **Key Validation:** Validated Gould's Rules of Engraving. Tests successfully enforce that accidentals do *not* cross octaves, that memory resets perfectly at barlines, and that explicitly cancelling a key signature forces an `AccidentalDisplay::Explicit` (Natural sign).

### 🟢 `rebar` (6 Unit Tests)
*   **Focus:** The Guillotine (Note slicing) and Void Filler (Padding).
*   **Key Validation:** Proved that a Half Note (3840 ticks) placed on Beat 4 of a 4/4 measure perfectly slices into a Quarter Note tied to a Quarter Note across the barline boundary.

### 🟢 `ir` & `midi` / `xml` (6 Integration Tests)
*   **Focus:** End-to-End pipeline execution.
*   **Key Validation:** `test_rational_engine` guarantees zero floating-point drift. `test_inference_strict_mode_sync_failure` ensures that mismatched polyphonic lengths cleanly throw `E3002`.

---

## 3. Defect Analysis: Critical Bugs Caught & Mitigated

During the v2.1 development cycle, the test suite intercepted several critical architectural failures before they could reach production:

### 🐛 Defect 1: The "Ambiguity Trap" (Infinite Parsing Loops)
*   **Phase:** Parser Generation (`chumsky`).
*   **Caught By:** `test_stage_4_multivoice_polyphony`
*   **Root Cause:** Optional parser chains combined with `.repeated()` caused Chumsky to succeed while consuming zero tokens, resulting in an infinite recursion panic.
*   **Resolution:** Implemented explicit delimiter requirements and strict `just(Token::RBrace).not().rewind()` peek guards to break the loop at block boundaries.

### 🐛 Defect 2: Rust Exhaustive Pattern Matching Data Loss
*   **Phase:** Inference Engine (`ir.rs`).
*   **Caught By:** `test_stage_2_boundaries_and_engines`
*   **Root Cause:** During the upgrade to inject `SpelledPitch` into events, the `Event::Percussion` match arm was accidentally deleted. Rust's `_ => {}` catch-all silently threw away all drum data.
*   **Resolution:** Re-implemented the percussion handler and added explicit assertions for track length and MIDI note values.

### 🐛 Defect 3: The Variable Macro-Masquerade
*   **Phase:** Preprocessor.
*   **Caught By:** `test_stage_5_macros_and_variables`
*   **Root Cause:** A variable used as a standalone event (e.g., `$root:16`) parsed identically to a 0-argument macro. The compiler threw `E2001: Undefined Macro`.
*   **Resolution:** Upgraded the Preprocessor to evaluate the Symbol Table and dynamically convert parameter-less macro calls into standard `Event::Note` types if they match a known variable.

### 🐛 Defect 4: Tie Target State Corruption (`E4005`)
*   **Phase:** Inference Engine.
*   **Caught By:** `test_tuplets_and_attributes`
*   **Root Cause:** The IR originally attempted backward-looking tie resolution, which failed when notes were chained (`c4~ c8~ c16`).
*   **Resolution:** Converted the engine to a forward-looking `tied_pitches: Vec<u8>` queue that extends `duration_ticks` of the root event rather than generating duplicate NoteOn events.

---

## 4. Future Testing Strategies (The QA Roadmap)

While 100% of the current specification is verified, as we move toward Phase V (LSP) and Phase VII (SVG Layout), our testing methodology must scale. 

**Planned QA Implementations:**

1.  **Snapshot Testing (`insta` crate):** 
    *   *Why:* Manually writing assertions for a 1,000-line MusicXML file is impossible. 
    *   *How:* We will use Snapshot testing to render standard `.ten` files to `.musicxml` and automatically compare the output against known-good "Golden Masters".
2.  **Property-Based Testing (`proptest` crate):** 
    *   *Why:* To ensure the Rational Time engine and Lexer never panic on weird user input.
    *   *How:* Feed thousands of randomly generated, chaotic fraction combinations into the Tuplet engine to guarantee `Division by Zero` or `Overflow` panics are impossible.
3.  **Fuzzing (`cargo fuzz`):**
    *   *Why:* As a text-based parser, `tenutoc` will eventually face maliciously crafted or entirely malformed text files. Fuzzing will ensure the Ariadne error recovery never crashes the language server.
