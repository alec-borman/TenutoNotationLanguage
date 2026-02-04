markdown
# OmniScore Reference Compiler (`omnic`)

![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange)

**`omnic`** is the official reference compiler for the OmniScore v2.0 specification.

OmniScore is a declarative, domain-specific language (DSL) for musical intent. Unlike traditional formats that focus on visual coordinates (like where a note sits on a page), OmniScore focuses on **musical logic**. It uses a deterministic inference engine to calculate timing, octaves, and layout at render-time.

## ✨ Key Features

- **Sticky State Inference**: Stop repeating yourself. Durations and octaves persist until you change them, drastically reducing verbosity.
- **Physics-Based Definitions**: Define your instruments (physics) separately from your notes (logic). Re-use musical patterns across different instruments.
- **Rational Time Engine**: Built-in support for complex tuplets and perfect rhythmic alignment using fraction-based math, eliminating floating-point rounding errors.
- **Multi-Phase Compilation Pipeline**:
    1.  **Lexing**: High-speed tokenization via [Logos](https://crates.io/crates/logos).
    2.  **Parsing**: Recursive-descent AST generation via [Chumsky](https://crates.io/crates/chumsky).
    3.  **Linearization**: Resolution of the "Sticky State" into a deterministic, flat timeline of events.

## 🚀 Getting Started

### Prerequisites
-   **Rust** (version 1.70 or higher) and **Cargo** (included with Rust).

### Installation
1.  Clone the repository:
    ```bash
    git clone https://github.com/alec-borman/OmniScoreNotationLanguage.git
    cd OmniScoreNotationLanguage
    ```
2.  Build the release binary:
    ```bash
    cargo build --release
    ```
    The executable will be located at `./target/release/omnic` (or `omnic.exe` on Windows).

## 🛠 Usage

To compile an OmniScore source file and view the inferred timeline:

```bash
./omnic --input your_score.omni
```

### Example: A Simple Scale

Create a file named `test.omni` with the following content:

```omniscore
omniscore {
    meta { title: "Simple Scale", tempo: 120 }

    def vln "Violin" style=standard

    measure 1 {
        vln: c4:4 d e f |
    }
}
```

Run the compiler:
```bash
./omnic --input test.omni
```

The compiler will parse the file, resolve all sticky states, and output a linearized sequence of events with their absolute timings.

## 📖 Documentation

-   **Language Specification**: The complete OmniScore v2.0 language definition, including formal grammar and semantics, is detailed in the **[OmniScore Specification](omniscore-specification.md)**.
-   **API Documentation**: Generate local documentation for the compiler internals:
    ```bash
    cargo doc --open
    ```

## 🗺 Project Roadmap

- [x] **Phase 1:** Lexical Analysis (Tokenizer)
- [x] **Phase 2:** AST Parsing (Grammar & Parser)
- [x] **Phase 3:** Time Inference Engine (Linearization)
- [ ] **Phase 4:** MIDI Backend Export
- [ ] **Phase 5:** SVG/PDF Engraving Engine
- [ ] **Phase 6:** Language Server Protocol (LSP) Support

## 👥 Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

1.  Fork the repository.
2.  Create a feature branch (`git checkout -b feature/amazing-feature`).
3.  Commit your changes (`git commit -m 'Add some amazing feature'`).
4.  Push to the branch (`git push origin feature/amazing-feature`).
5.  Open a Pull Request.

## 📄 License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.

---

**Created and maintained by [Alec Borman](https://github.com/alec-borman).**
```
You can simply copy this text, create a new file named `README.md` in your repository's root folder, and paste it in. This will give your project an immediate, professional presence on GitHub.

If you have a logo or a more detailed build/install process, we can easily add those sections as well.
