# PROJECT TENUTO: The Semantic Standard for Music

## 1. The Industry Problem: The "Semantic Gap"
Digital music infrastructure is fundamentally fractured. For 40 years, the industry has relied on two disconnected paradigms:
1.  **Visual Formats (MusicXML):** Bloated, coordinate-based XML designed strictly for printing sheet music. It is virtually unreadable to humans, hostile to version control, and overwhelmingly token-heavy for modern AI to generate.
2.  **Execution Formats (MIDI / DAW Projects):** Mechanical, low-level hardware protocols. MIDI records keystrokes but suffers from "structural amnesia"—it forgets what a measure, a key signature, or a tuplet actually is. DAW project files are proprietary black boxes that trap composer data.

**The Breaking Point:** Generative AI for music (like Suno or Udio) currently outputs raw, uneditable audio waveforms. To generate *symbolic, editable* music, AI models need a text representation. MusicXML routinely exceeds LLM context windows (thousands of tokens per measure), and MIDI lacks the semantic logic required for high-level compositional reasoning.

## 2. What Tenuto IS
**Tenuto is a declarative, domain-specific programming language (DSL) and high-performance Rust compiler for music.** 

It serves as a "Single Source of Truth." A composer or an AI model writes highly compressed, human-readable logic. The Tenuto compiler mathematically derives both the **visual typography** (Sheet Music) and the **mechanical execution** (MIDI/Audio) deterministically.

*   **It is Token-Efficient:** By using "Sticky State" inference (inheriting previous durations and octaves natively), Tenuto compresses musical logic by up to 90% compared to XML. It is the perfect, native language for LLM music generation.
*   **It is Mathematically Pure:** Tenuto abandons floating-point time (which causes drift) in favor of Rational Arithmetic (exact fractions), cleanly solving complex polyrhythms and Euclidean beat distributions.
*   **It is Archival & Portable:** A `.ten` file is plain UTF-8 text. It can be version-controlled via Git, diffed, and compiled anywhere, immune to the decay of proprietary software.

## 3. What Tenuto is NOT
To maintain a tight, capital-efficient scope, we enforce strict boundaries on what the core project represents:
*   **It is NOT a GUI DAW.** We are not building a heavy Electron app with a piano roll or trying to immediately displace Ableton Live's user interface. We are building the *infrastructure* that connects to those tools.
*   **It is NOT a black-box AI audio generator.** Tenuto is the structured, deterministic pipeline that generative AI will use to output *editable* compositions.
*   **It is NOT a raw audio editor.** While the `style=concrete` engine maps and slices samples, Tenuto does not encode binary `.wav` data. It orchestrates it.

## 4. Current Traction & Technical Moat
We are not pitching an idea; we are pitching a proven architecture. We have successfully engineered the **Tenuto v2.2 Reference Compiler (`tenutoc`) in Rust**.
*   **The Engine Works:** It features a blazingly fast, deterministic LL(1) parser built on `chumsky` and `logos`.
*   **Dual-Backend Delivery:** The compiler natively executes the "Sticky State" logic and exports to both **Standard MIDI** and **MusicXML 4.0**.
*   **Advanced Specifications Written:** The incredibly difficult theoretical work for the next phase—the *Tenuto Engraving Architecture Specification (TEAS)*—is already mapped out, utilizing Entity-Component-System (ECS) memory and Cassowary linear constraint solvers.
*   **100% Test Coverage:** The core engine is mathematically verified and ready to scale.

## 5. The Strategic Execution Roadmap
With strategic funding and a dedicated core engineering team, we will scale Tenuto from a CLI compiler into an omnipresent industry standard across three targeted phases.

### Phase 1: The Developer Ecosystem (Months 1–6)
*Goal: Drive early adoption by making Tenuto feel like a first-class programming language.*
*   **`tenuto-lsp`:** Build the Language Server Protocol. Offer official VS Code and Neovim extensions with real-time syntax checking, macro hover-documentation, and auto-formatting.
*   **The Web Playground:** Deploy a Wasm-compiled, client-side web editor (similar to the TypeScript or Rust playgrounds) where users can write Tenuto, hear MIDI playback, and see MusicXML sheet music instantly in their browser.

### Phase 2: The "Typst of Music" (Months 6–18)
*Goal: Bypass legacy notation software entirely by rendering our own mathematically perfect sheet music.*
*   **`tenuto-engrave`:** Execute the TEAS specification. Build a pure-Rust, native SVG layout engine. 
*   **Incremental Compilation:** Utilize the `salsa` framework so that modifying a single note in a 100-page symphony updates the SVG render in under 50 milliseconds.
*   **Accessibility Standards:** Natively generate WAI-ARIA compliant Semantic SVGs and Braille Music (BRF) formats directly from the compiler's intermediate representation.

### Phase 3: AI Partnerships & The Web Runtime (Months 18–24)
*Goal: Establish Tenuto as the default format for machine learning and interactive web audio.*
*   **The `<tenuto>` Web Component:** Finalize the WebAudio API bindings so developers can drop a `<tenuto src="song.ten">` tag into any HTML document to generate interactive, responsive sheet music and audio without external plugins.
*   **AI Model Fine-Tuning:** Partner with AI research labs (e.g., OpenAI, Anthropic, HuggingFace) to fine-tune foundation models on a corpus of Tenuto code, creating an "AI Copilot for Music" that generates flawless, mathematically valid `.ten` logic from text prompts.

## 6. The Value Proposition & Business Model
Tenuto will operate on an **Open Core** model (similar to Linux, Docker, or Vercel). The compiler and language specification remain strictly open-source (MIT License) to guarantee global adoption and archival trust.

Revenue and enterprise value are captured via:
1.  **AI Licensing & APIs:** Providing the data-pipelines, sanitization tools, and programmatic infrastructure for AI companies building symbolic music generators.
2.  **Enterprise Tooling:** Selling proprietary VST3/CLAP plugins that allow commercial studios to execute Tenuto code natively inside existing DAWs.
3.  **Cloud Rendering Infrastructure:** Offering high-volume, headless PDF engraving and audio compilation APIs for educational platforms, publishers, and sheet music distributors.

Tenuto is ready to transition from a technological breakthrough into the foundational infrastructure of the next generation of music software.
