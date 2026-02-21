# Tenuto 2.1.0 Release Notes

**Date:** February 21, 2026  
**Version:** 2.1.0  
**Status:** Normative / Final Update  

## Executive Summary

Tenuto 2.1.0 is a critical syntax update designed to resolve **LL(k) parsing ambiguity conflicts** identified in the 2.0.0 specification. 

In version 2.0.0, the compiler relied heavily on standard curly braces (`{ }`) for both structural code blocks (like `measure` or `macro`) and internal data structures (like polyphonic voices and metadata maps). This created a "Metadata Trap" where recursive descent parsers required expensive, deep backtracking to differentiate between a structural block and a key-value data map.

Version 2.1.0 solves this by introducing **unique compound sigils** for data structures and polyphonic blocks. This allows parsers to remain strictly deterministic, improving compilation speed and reducing memory overhead, especially for live-coding and REPL environments (`tenutod`).

---

## ⚠️ Breaking Changes

Tenuto 2.1.0 introduces breaking lexical changes to the V2.0.0 syntax. Compilers supporting 2.1.0 **MUST** enforce the new compound sigils. Files written in 2.0.0 will require a syntax migration to compile under 2.1.0 Strict Mode.

### 1. The Map Sigil (`@{ }`)
All Key-Value data maps, including `meta` blocks and instrument dictionary attributes, must now use the Map Sigil (`@{ }`) instead of standard braces (`{ }`).

* **Affected Areas:** Global `meta`, local `measure` metadata, instrument `map` dictionaries (Percussion), `keyswitch` arrays, and tuning/theme maps.
* **Why:** The parser instantly recognizes `@` as the start of a data structure, preventing confusion with structural scopes.

### 2. Voice Brackets (`<[ ]>`)
Polyphonic multi-voice blocks within a single staff must now be enclosed in Voice Brackets (`<[ ]>`) instead of standard braces (`{ }`).

* **Affected Areas:** Any staff assignment utilizing `v1`, `v2`, etc.
* **Why:** Resolves the ambiguity between an event block and a staff assignment branching into simultaneous time streams.

*(Note: High-level structural scopes like `tenuto`, `measure`, `group`, and `macro` bodies **continue** to use standard curly braces `{ }`.)*

---

## Syntax Migration Guide (Before & After)

### Metadata & Configuration Maps

**❌ V2.0.0 (Deprecated)**
```tenuto
meta { 
  title: "Symphony No. 1", 
  tempo: 120 
}

def drm "Drum Kit" style=grid map={ k: [0, 36], s: [2, 38] }
```

**✅ V2.1.0 (Current)**
```tenuto
meta @{ 
  title: "Symphony No. 1", 
  tempo: 120 
}

def drm "Drum Kit" style=grid map=@{ k: [0, 36], s: [2, 38] }
```

### Advanced Polyphony (Voice Groups)

**❌ V2.0.0 (Deprecated)**
```tenuto
measure 1 {
  pno: {
    v1: c4:4 d e f |
    v2: c2:2        |
  }
}
```

**✅ V2.1.0 (Current)**
```tenuto
measure 1 {
  pno: <[
    v1: c4:4 d e f |
    v2: c2:2        |
  ]>
}
```

---

## EBNF Grammar Updates

For compiler and tooling authors, the Formal Grammar (Section 26) has been updated with the following normative rules:

1. **Tokens (26.1):** Added `<[`, `]>`, and `@{` as atomic multi-character operators.
2. **Assignments (26.3):**
   ```ebnf
   Assignment      ::= IDENTIFIER ":" (VoiceGroup | MultiVoiceBlock)
   MultiVoiceBlock ::= "<[" Voice ("|" Voice)* "]" ">"
   ```
3. **Data Types (26.4):**
   ```ebnf
   MetaBlock ::= "meta" "@{" KeyValueList "}"
   Map       ::= "@{" KeyValueList "}"
   ```

---

## Additional Refinements

* **Version Declaration:** The `tenuto_version` metadata key (or `tenuto "2.1" {` string header) should be updated to target `"2.1"` to ensure the compiler activates the correct lexer logic.
* **Reference Example:** Section 28 ("The Kitchen Sink") has been fully rewritten to demonstrate V2.1.0 compliant syntax. 
* **Addendum A (Live Execution):** The `@sync` scheduling directive has been renamed to `@at` to better reflect temporal anchoring (e.g., `@at(measure 17) meta @{ key: "D" }`).
