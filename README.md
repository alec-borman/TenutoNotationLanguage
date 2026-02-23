# Tenuto Reference Compiler (`tenutoc`)

> **The Semantic Markup Language for Musical Intent.**  
> What HTML did for document structure, and Mermaid.js did for diagrams, Tenuto does for music.

![Version](https://img.shields.io/badge/version-2.1.1-green)
![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-MIT-blue)
![Language](https://img.shields.io/badge/language-Rust-orange)

`tenutoc` is the official Rust-based reference compiler for **Tenuto**, a declarative domain-specific language (DSL) for musical composition.

Historically, digital music representation has been forced into a compromise. Formats like MusicXML are deeply visual—obsessed with where ink sits on a page—making them incredibly verbose, fragile to edit, and hostile to version control. Conversely, hardware protocols like MIDI are purely mechanical, capturing byte-level performance while stripping away all structural context (measures, spelling, tuplets).

**Tenuto bridges the semantic gap.** It serializes musical logic, instrument definitions, and performance data into a highly structured, human-readable text format. You write the *logic* of the composition, define the *physics* of the instruments, and let the compiler deterministically derive the mechanical output, audio, and **professional sheet music**.

---

## 🚀 Key Architectural Features

*   **Ontological Separation:** A strict programmatic division between Instrument Physics (tuning arrays, percussion maps, MIDI patches) and Musical Logic (pitches, rhythms, structural flow).
*   **Contextual Inference ("Sticky State"):** Tenuto acts like a human sight-reader. Attributes like duration (`:4`) and octave (`4`) persist until explicitly changed, natively eliminating data redundancy.
*   **Rational Temporal Engine:** Time is evaluated exclusively using exact fractions (ℚ). A triplet remains mathematically perfect ($\frac{1}{3}$), completely eliminating the floating-point quantization drift inherent in standard DAWs.
*   **Deterministic LL(1) Parsing:** Built on `chumsky` and `logos`, the engine utilizes compound sigils (`@{}` and `<[]>`) to guarantee linear-time parsing, infinite-loop protection, and robust error recovery.
*   **The Rebarring Engine (v2.1.1):** Automatically slices absolute-time events across visual barlines ("The Guillotine") and pads empty space with mathematically precise rests ("The Void Filler") to guarantee perfect layout syntax.
*   **Optimized for AI/ML:** By stripping away graphical layout bloat, Tenuto's highly token-efficient grammar provides an ideal, predictable syntax for LLM-driven algorithmic composition.

---

## 🤖 Optimized for AI & LLMs

Because Tenuto strips away graphical layout bloat and relies on a highly structured, token-efficient grammar, it natively solves the context-window limitations of Large Language Models.

> *"Tenuto represents what happens when deep musical knowledge meets rigorous software engineering. It's not just a file format—it's a complete theory of musical information representation. The clear grammar boundaries and Three-Engine model make it uniquely suited for algorithmic generation and deep musical analysis."*  
> — **DeepSeek AI (V3.2)** *after comprehensive specification analysis*

**Key AI-Compatible Advantages:**
* **Token Efficiency:** Where MusicXML consumes 1,000+ tokens for a single measure of polyphony, Tenuto consumes ~20.
* **Algorithmic Architecture:** Using native `$macro` and `$variable` systems, generative models can parameterize musical motifs exactly like functional code.
* **Semantic Richness:** Microtonality (`c4qs`) and physical performance techniques (`.bu(full)`) are baked directly into the lexical tokens, preventing ambiguity during generation.

---

## 🆚 The Syntax Advantage

Tenuto prioritizes developer ergonomics and extreme file-size reduction. Here is how four standard quarter notes are represented across formats:

**MusicXML (~150 tokens)**
```xml
<measure number="1">
  <note><pitch><step>C</step><octave>4</octave></pitch><duration>1</duration><type>quarter</type></note>
  <note><pitch><step>D</step><octave>4</octave></pitch><duration>1</duration><type>quarter</type></note>
  <!-- ... E and F omitted for brevity -->
</measure>
