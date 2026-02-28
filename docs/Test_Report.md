# 📊 Tenuto v2.2.0 Test Analysis & Quality Assurance Report

**Project:** `tenutoc` Reference Compiler  
**Date:** February 28, 2026  
**Test Framework:** Rust Native (`cargo test`)  
**Overall Status:** **100% Passing (Ready for CI/CD Pipeline)**  

---

## 1. Executive Summary

The testing strategy for the Tenuto v2.2.0 compiler relies heavily on **Test-Driven Development (TDD)** and **Regression Safeguarding**. The v2.2.0 "Performance Engine" update required a massive expansion of the Intermediate Representation (IR) to support Continuous Control (CC) sweeps, Tablature bends, and Grace notes. 

Because `tenutoc` handles highly complex, interdependent data transformations, testing is rigorously divided into localized **Unit Tests** for math/state logic and end-to-end **Integration Tests** for pipeline health. The existing test harness proved its worth immediately by catching strict-typing and mock-data regressions during the `AtomicEvent` expansion, allowing us to implement DAW-level automation without breaking a single previously verified feature.

As of the v2.2.0 release, the test suite consists of **31 strictly enforced tests** (25 Unit, 6 Integration). The suite executes in `< 0.05s` under the `dev` profile, guaranteeing rapid developer feedback.

---

## 2. Coverage Breakdown by Module

### 🟢 `lexer` (7 Tests)
*   **Focus:** V2.1 compound sigils (`@{`, `<[`, `]>`), domain-specific primitives (Microtonal pitches, Tablature, Frequencies).
*   **Key Validation:** Ensured that `c4` parses as a `PitchLit` and not an `Identifier`, enforcing Regex priority matching.

### 🟢 `parser` & `ast` (Handled via Integration Tests)
*   **Focus:** LL(1) deterministic AST generation and Ariadne error recovery.
*   **Key Validation:** Validated that block terminators (`}`, `]>`) are not swallowed by `skip_then_retry_until` recovery sets. **[v2.2.0 Update]:** Verified that attributes chaining `.cc(11,[0,127])` and `.roll(3)` parse into the AST correctly.

### 🟢 `preprocessor` (1 Unit Test + Integration)
*   **Focus:** Variable injection, Macro expansion, and Recursion safety.
*   **Key Validation:** `test_transposition_logic` guarantees the mathematical shifting of Scientific Pitch Notation. Verified that recursion depth > 64 safely aborts (`E5002`).

### 🟢 `spelling` (11 Unit Tests)
*   **Focus:** The Accidental State Machine and Algorithmic "Line of Fifths" Speller.
*   **Key Validation:** Validated Gould's Rules of Engraving. Ensures accidentals do *not* cross octaves, memory resets perfectly at barlines, and cancelling a key signature forces an `AccidentalDisplay::Explicit` (Natural sign).

### 🟢 `rebar` (6 Unit Tests)
*   **Focus:** The Guillotine (Note slicing) and Void Filler (Padding).
*   **Key Validation:** Proved that a Half Note (3840 ticks) placed on Beat 4 of a 4/4 measure perfectly slices across the barline boundary. **[v2.2.0 Update]:** Verified that `:grace` notes bypass the Void Filler's logical time calculations, ensuring they do not pad or break the visual measure grid.

### 🟢 `ir` & `midi` / `xml` (6 Integration Tests)
*   **Focus:** End-to-End pipeline execution, Sticky Dynamics, and Additive Merging.
*   **Key Validation:** `test_rational_engine` guarantees zero floating-point drift. **[v2.2.0 Update]:** Validated that unrolling `.roll()` and `.bu(full)` generates dense arrays of CC and Pitch Bend events sorted perfectly by `tick` and `priority`.

---

## 3. Defect Analysis: Critical Bugs Caught & Mitigated

During the v2.1 and v2.2 development cycles, the test suite and Rust compiler intercepted several critical architectural failures before they could reach production:

### 🐛 Defect 1: The "Ambiguity Trap" (Infinite Parsing Loops) [v2.1]
*   **Phase:** Parser Generation (`chumsky`).
*   **Root Cause:** Optional parser chains combined with `.repeated()` caused Chumsky to succeed while consuming zero tokens, resulting in an infinite recursion panic.
*   **Resolution:** Implemented explicit delimiter requirements and strict `just(Token::RBrace).not().rewind()` peek guards to break the loop at block boundaries.

### 🐛 Defect 2: Rust Exhaustive Pattern Matching Data Loss [v2.1]
*   **Phase:** Inference Engine (`ir.rs`).
*   **Root Cause:** During the upgrade to inject `SpelledPitch` into events, the `Event::Percussion` match arm was accidentally deleted. Rust's `_ => {}` catch-all silently threw away all drum data.
*   **Resolution:** Re-implemented the percussion handler and added explicit assertions for track length and MIDI note values.

### 🐛 Defect 3: The "Spacer Rest" vs. "Snare Drum" Collision [v2.2]
*   **Phase:** Parser (`chumsky`).
*   **Caught By:** `automation.ten` manual integration test.
*   **Root Cause:** The letter `s` was hardcoded as a "spacer rest" (borrowed from LilyPond syntax). This blocked users from defining `s` as a percussion mapped key (e.g., Snare Drum), causing a fatal parser unexpected token error when attempting to attach `.roll(3)` to it.
*   **Resolution:** Removed `&& s != "s"` from the rest parser, freeing `s` to be used universally as a standard identifier in `style=grid`.

### 🐛 Defect 4: Hardcoded Additive Merging Grid [v2.2]
*   **Phase:** Inference Engine (`ir.rs`).
*   **Caught By:** Audio evaluation of non-4/4 time signatures.
*   **Root Cause:** The Additive Merging engine assumed all measures were exactly `ppq * 4` (4/4 time). If a user wrote in `3/4`, the engine played 3 beats, but snapped the *next* measure to the 4/4 grid, creating exactly 1 beat of dead silence between every measure.
*   **Resolution:** Upgraded the engine to dynamically parse the `time` KeyValue map from global and local `meta` blocks, pre-calculating accurate absolute start ticks for all measure boundaries before evaluating events.

### 🐛 Defect 5: Strict Typing on IR Expansion (`E0063`) [v2.2]
*   **Phase:** Rebarring Engine (`rebar.rs`).
*   **Caught By:** `cargo test` execution.
*   **Root Cause:** Expanding `AtomicEvent` to include `is_grace`, `is_ghost`, `tremolo_slashes`, and `cc_automations` immediately broke all mock instances in the `rebar.rs` unit tests.
*   **Resolution:** Rust's strict typing prevented incomplete IR data from reaching the visual layer. We successfully patched all test mocks with `Default`-equivalent initializers, guaranteeing data integrity.

---

## 4. Future Testing Strategies (The QA Roadmap)

While 100% of the current specification is verified, as we move toward Phase V (LSP) and Phase VII (SVG Layout), our testing methodology must scale to handle massive files and visual regressions.

**Planned QA Implementations:**

1.  **Snapshot Testing (`insta` crate):** 
    *   *Why:* Manually writing assertions for a 1,000-line MusicXML file or an SVG path is impossible. 
    *   *How:* We will use Snapshot testing to render standard `.ten` files to `.musicxml` and `.svg`, automatically comparing the output against known-good "Golden Masters".
2.  **Property-Based Testing (`proptest` crate):** 
    *   *Why:* To ensure the Rational Time engine and Lexer never panic on weird user input.
    *   *How:* Feed thousands of randomly generated, chaotic fraction combinations into the Tuplet engine to guarantee `Division by Zero` or `Overflow` panics are impossible.
3.  **Fuzzing (`cargo fuzz`):**
    *   *Why:* As a text-based parser, `tenutoc` will eventually face maliciously crafted or entirely malformed text files. Fuzzing will ensure the Ariadne error recovery never crashes the upcoming background Language Server.
