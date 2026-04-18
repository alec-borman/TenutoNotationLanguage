𝄆 TENUTO 4.0.0 𝄇
The Declarative Protocol for Musical Physics & Logic
Canonical Language Specification
Version: 4.0.0 (The Declarative Protocol Edition) Status: Normative / Final License: MIT Maintainers: The Tenuto Working Group & The Teleportation Protocol Foundation Reference Implementations: Python 3.11+ (Native), JavaScript / Node.js 20+ (V8), WebAssembly (Browser/Edge)
Executive Abstract Tenuto 4.0.0 is an enterprise-grade, deterministic, declarative domain‑specific language (DSL) that mathematically unifies the discrete, topological universe of acoustic sheet music with the continuous, high-dimensional physics of digital signal processing (DSP). By implementing a Stateful Cursor, Rational Time evaluation, and an infinitely extensible Abstract Syntax Tree (AST), Tenuto compiles flawless MusicXML, sub-millisecond precise MIDI 2.0, and continuous audio buffers from a single, token-efficient, human-readable text file. It stands as the definitive programmatic interface for human composers, and the ultimate "Day Zero" execution environment for Artificial Intelligence orchestration.
--------------------------------------------------------------------------------
PART I: THE CORE ARCHITECTURE
1.1 The Four Core Axioms
Inference Over Redundancy (The Stateful Cursor): Musical notation is inherently sequential. XML‑based formats exhibit catastrophic verbosity. Tenuto defeats this utilizing a strict Sticky State cursor. Parameters such as duration (:4), octave (4), and amplitude (.ff) persist in the compiler’s memory until explicitly mutated, achieving up to a 90% reduction in token count—massively optimizing the format for LLM context windows.
Strict Ontological Separation: Tenuto rigorously isolates the Physics of an instrument (tuning matrices, ADSR envelopes, RAM sample allocations) from the Logic of the performance (rhythms, pitches, micro‑timing).
Absolute Mathematical Truth (Rational Time): Evaluating irrational rhythms on an arbitrary DAW integer grid inherently causes IEEE 754 floating‑point drift. Tenuto’s Inference Engine evaluates all time utilizing Rational Arithmetic (Q= 
Q
P
​
 ), coercing fractions into scalar physical ticks only at the final microsecond of rendering.
Electronic & Acoustic Parity: Abstract DSP manipulations—time‑stretching, sidechain ducking, granular slicing, and portamento glides—are elevated to native semantic primitives, accessible via the same unified syntax used for classical articulations (.stacc.vol(80).stretch).
1.2 The Deterministic Compilation Pipeline (T 
c
​
 )
A compliant Tenuto 4.0.0 compiler MUST execute a rigid, deterministic, six-stage architecture:
graph TD
    A[Raw Source Code .ten] -->|UTF-8 / NFC| B[1. Lexing O_n DFA]
    B --> C[2. Parsing Deterministic LL_1]
    C --> D[3. Preprocessor & Symbol Expansion]
    D --> E[4. Inference & IR Linearization]
    E --> F{5. Demarcation Pass}
    F -->|Visual Target| G[6a. Rebarring & Spelling Engine]
    F -->|Audio/Network Target| H[6b. DSP & Physics Unrolling]
    G --> I[MusicXML 4.0 / SVG / PDF]
    H --> J[Web Audio / OSC / MIDI 2.0]
1.3 Conformance Profiles
To foster a robust, open ecosystem, compilers MUST declare a Conformance Profile. However, ALL compilers MUST implement the universal LL(1) Lexer/Parser for the entire grammar; features excluded by a profile SHALL be gracefully bypassed (AST Pruning) rather than throwing lexical errors.
Profile A (Core Logic): Supports Stateful Cursor, Rational Time, and Polyphony. Emits MIDI/MusicXML.
Profile B (Native Audio): Profile A + ADSR Envelopes, Buffer Slicing, and Continuous Interpolation. Emits .wav or Web Audio API.
Profile C (Network/Delegation): Profiles A & B + Look-Ahead Scheduling and Synchronized Clock Protocols (OSC, sACN, Ableton Link).
--------------------------------------------------------------------------------
PART II: LANGUAGE MECHANICS & SYNTAX
Source files MUST be encoded in UTF‑8 and normalized to Unicode NFC. Keywords (def, measure) and Note Names (c4) are case-insensitive. Identifiers and strings are case-sensitive. Comments are denoted by %%.
2.1 Operators & Compound Sigils
Tenuto relies on strict compound sigils to mathematically differentiate structural scopes from internal data arrays, eliminating parser ambiguity.
Sigil
Formal Name
Compiler Application
{ }
Structural Braces
Defines compilation phases, global scopes, and macro definitions.
<[ ]>
Voice Brackets
Triggers the Polyphonic Parallel Engine and sandboxes state.
@{ }
Map Sigil
Triggers Key‑Value Dictionary parsing (Metadata, physics parameters).
[ ]
Chord/Array Brackets
Unrolls discrete pitches into simultaneous atomic events.
:
Assignment / Ratio
Binds logic to Staff IDs, explicit duration (:4), or Tuplet/Euclidean limits.
.
Attribute Dot
Accessor for chaining sequential DSP modifiers (.stacc.stretch).
$
Invocation Dollar
Signals the Preprocessor to evaluate the Symbol Table.
2.2 Domain‑Specific Primitives
Primitive Type
Regex Boundary Pattern
Semantic Evaluation
PitchLit
(?i)[a-g](qs|qf|tqs|tqf|x|#|b|n)*[0-9]?([+-][0-9]+)?
The fundamental unit of acoustic frequency.
TabCoord
(?i)[0-9xX]+-[1-9][0-9]*
Formatted strictly as Fret-String.
TimeVal
[0-9]+(\.[0-9]+)?(ms|s|ticks)
Absolute continuous physical time units.
Attribute
\.[a-zA-Z_]\w*
Chained method names (.roll, .ghost).
2.3 Document Topology & Additive Merging
A well‑formed Tenuto document represents a fully encapsulated unit enforcing Declaration‑Before‑Use.
tenuto "4.0" {
  %% Phase 1: Configuration
  meta @{ title: "The Billion Dollar Spec", tempo: 120, time: "4/4" }

  %% Phase 2: Definition (Physics)
  def pno "Grand Piano" style=standard patch="gm_piano"
  def vox "Lead Vocal"  style=concrete src="./vocals.wav" map=@{ a:[0s, 1.5s] }

  %% Phase 3: Logic (Additive Merging)
  measure 1 {
    pno: c4:4 d e f |
    vox: a:2.stretch r:2 |
  }
}
The Additive Merge Rule: If measure 1 is populated in a piano scope, and subsequently invoked in a vocal scope, the Inference Engine MUST mathematically snap the start‑tick of the vocal logic to the exact absolute timestamp of the piano logic.
--------------------------------------------------------------------------------
3. Instrument Definitions (The Cognitive Engines)
The def statement acts as a constructor, defining the internal Cognitive Engine via the style attribute.
Style Enum
The Cognitive Routing Engine
Expected Input
standard
The Helmholtz Model. Parses Scientific Pitch Notation.
c4, ebqs5
tab
The Physical Model. Parses spatial instrument coordinates.
0-6 (Fret-String)
grid
The Discrete Trigger Model. Parses arbitrary keys to MIDI.
k, s, h
concrete
The Schaefferian Model. Bypasses synthesis for raw audio.
Mapped string tokens
synth
The Continuous Frequency Model. Global ADSR/LFO targets.
PitchLit + curves
--------------------------------------------------------------------------------
4. The Temporal Engine: Rhythm & Micro-Timing
Tenuto strictly bifurcates time into two tracking vectors within the AtomicEvent IR: Logical Grid Time and Physical Playback Time.
Rational Math: Duration is appended via a colon (:4, :16.). Multipliers use an asterisk (* 4). Internally, a quarter note is stored strictly as 1/4.
.push(TimeVal) / .pull(TimeVal): Shifts the physical audio gate execution while leaving the Logical Grid mathematically perfect for clean sheet music export. .pull(10ms) shifts playback exactly 10 milliseconds late, regardless of track BPM.
Atemporal Events (Grace Notes): :grace consumes 0 logical metric capacity, stealing a physical fraction of the parent note’s gate_ticks for execution.
--------------------------------------------------------------------------------
5. The Pitch Engine (style=standard)
Anchored to the absolute physical constant of A 
4
​
 =440.0 Hz.
The Sticky Octave: c4 d e parses identically to c4 d4 e4.
Forward-Looking Ties (~): c4~ c:8 queues the initial event, mathematically extending its gate_ticks while suppressing the subsequent NoteOn trigger.
Gould’s Rules & The Accidental State Machine: While the Tenuto AST relies on strictly stateless accidentals (f#4 means exactly one F-sharp), Phase 6a (Visual Translation) routes pitches through a rigorous state machine enforcing Elaine Gould’s typesetting rules: independent per-octave memory, absolute barline resets, and explicit natural sign (♮) injection.
--------------------------------------------------------------------------------
6. Euclidean Topologies & Advanced Polyphony
Version 4.0.0 supercharges the polyphonic engine by bifurcating the Tuplet syntax to natively support algorithmic electronic rhythms.
Voice Isolation (<[ ]>): Voices are sandboxed. In --strict mode, every voice block MUST sum to the exact same total absolute duration (throws E3002).
Polyrhythm (Space-Separated): (c4:8 d e):3/2. Compresses three 8th notes into the duration of two.
Algorithmic Euclidean (Single Token): (k):3/8. Triggers the Bresenham line-drawing algorithm. It clones the single event 3 times and distributes it as evenly as possible across an 8-step grid, outputting the classic tresillo pattern in just 8 characters.
--------------------------------------------------------------------------------
7. Dynamic Signal Interaction & Automation
Tenuto abandons convoluted DAW routing graphs in favor of Action Notation. By utilizing Polyphonic Voice Brackets and the Spacer (s) token, composers can draw pure, invisible automation curves (LFOs).
sub: <[
  v1: c2:1 |                                  %% The 808 note, held for a whole note
  v2: s:4.cc(7, [0, 127], "exp") * 4 |        %% Sidechain: Volume ramps 0->127 exponentially every quarter note
]>
--------------------------------------------------------------------------------
8. Formal Grammar Specification (EBNF)
Score           ::= Header? TopLevel*
Header          ::= "tenuto" STRING? 
TopLevel        ::= Import | Definition | VariableDecl | MacroDef | Block | MetaBlock

Definition      ::= "def" IDENTIFIER STRING? DefAttr*
VariableDecl    ::= "var" IDENTIFIER "=" Value
MacroDef        ::= "macro" IDENTIFIER "(" ParamList? ")" "=" "{" Voice "}"

Block           ::= "measure" (INTEGER | Range | List)? MetaBlock? "{" Logic* "}"
Logic           ::= Assignment | MetaBlock | Conditional
Assignment      ::= IDENTIFIER ":" (VoiceGroup | MultiVoiceBlock)
MultiVoiceBlock ::= "<[" Voice ("|" Voice)* "]" ">"

Voice           ::= (Event | Tuplet | Euclidean | MacroCall)*
Event           ::= (PITCH_LIT | Chord | TAB_COORD | "r" | "s" | IDENTIFIER) DURATION? Modifiers*

Tuplet          ::= "(" Voice ")" ":" INTEGER "/" INTEGER
Euclidean       ::= "(" IDENTIFIER ")" ":" INTEGER "/" INTEGER
MacroCall       ::= "$" IDENTIFIER ("(" ArgList ")")? Transposition?

Modifiers       ::= ("." IDENTIFIER ("(" ArgList ")")? | "~")
Transposition   ::= ("+" | "-") INTEGER
--------------------------------------------------------------------------------
PART III: THE ENTERPRISE & SPATIAL ECOSYSTEM (ADDENDA A–M)
The following Normative Addenda elevate Tenuto 4.0.0 from a localized text compiler into a Tier-1 infrastructure protocol capable of orchestrating cloud AI, live theatrical arrays, LLM semantic indexing, and bi-directional UI mutation.
Addendum A: The Universal Semantic Conductor (Delegation)
To maintain token efficiency, tenutoc implements a Delegation Architecture. It packages linearized IR structures into NTP-timestamped OSC bundles, routing them predictively (Look-Ahead Scheduling) to external physics engines like SuperCollider (SuperDirt) or ChucK. Integration with Ableton Link overrides local PPQ clocks, locking AST boundaries to peer-to-peer network phases for live Algoraves.
Addendum B: The Zero-Friction Web Runtime
The tenutoc parser compiles to wasm32-unknown-unknown via Pyodide. The Web Runtime defaults to the browser's native Web Audio API, translating synth envelopes into AudioWorkletNode graphs. It exposes the <tenuto-score src="song.ten"> HTML Custom Element for zero-friction client-side DOM embedding.
Addendum C: Generative Ergonomics & Smart Compilation
To support linear AI-generative workflows without halting errors:
Polyphonic Auto-Padding: meta @{ auto_pad_voices: true } dynamically calculates maximum duration within a <[ ]> block and injects terminal rests, suppressing E3002 crashes.
Decoupled Control Lanes (pedal:): Isolates CC64 piano sustain logic from complex pitch arrays.
Relative Pitch Heuristics: style=relative overrides absolute Sticky Octaves, utilizing the "Closest Interval (Tritone) Rule" to smartly infer octaves and prevent runaway drift during AI arpeggio generation.
Addendum D: Deterministic Semantic Decompilation
A reverse inference pipeline that reverse-engineers explicit machine formats (MIDI/MusicXML) back into idiomatic Tenuto logic. It employs 15 strict algorithmic passes, including LZ77 Dictionary Coding (to extract $macros), Bresenham reversal (to deduce Euclidean (k):3/8 tuplets), and Tritone Smoothing.
Addendum E: Visual-Acoustic Demarcation
When compiling to visual targets (PDF/SVG), the AST executes an O(n) Demarcation Pass to prune Unprintable Physics (e.g., .slice, .pull). Tracks marked style=concrete are implicitly masked unless overridden with @print(true), which invokes a Graphic Notation Fallback (aleatoric bounding boxes and continuous Bezier pitch curves).
Addendum F: The REPL Architecture (Sketch Mode)
When invoked with --sketch, a normative pre-compilation layer seamlessly wraps raw text inputs (c4:4 d e) into a hidden compliant AST boilerplate. It preserves the "Sticky State" cursor across independent execution cycles, allowing zero-friction live coding that can later be ejected into fully formed archival files.
Addendum G: Signal Routing & Spatial Audio Matrix
Collapses the mixing console into the AST. The bus:// URI schema captures live audio streams for real-time granular manipulation without fatal circular dependency loops. Spatial primitives (.pan, .orbit) and abstract DSP chains (.fx(reverb, @{mix: 0.9})) are serialized as mathematical trajectories in the IR, guaranteeing cinematic spatial mixes render perfectly decades later independently of proprietary VSTs.
Addendum H: Theatrical Orchestration Layer
Transforms Tenuto into a live show-control protocol. Utilizing IEEE 802.1 TSN (Time-Sensitive Networking), the tenutod daemon acts as a gPTP Grandmaster. It predictively delegates lighting (style=sacn) and laser geometry (style=fb4) with microsecond phase-alignment, completely eliminating reactive latency and "inter-system smear."
Addendum I: Codebase Indexing (The RAG Blueprint)
For maintaining the Tenuto compiler via AI agents, this mandates AST-Aware Semantic Chunking using Tree-sitter. Code is vectorized and tagged by domain criticality (domain:compiler). This guarantees that LLMs accessing the repo via Retrieval-Augmented Generation (RAG) pull hyper-relevant, architecturally isolated logic blocks rather than fragmented text slices.
Addendum J: The Bi-Directional Projectional DAW
Tenuto formalizes Projectional Editing. The DAW graphical interface possesses zero proprietary binary state; it is strictly a Vite+React WebGL projection of the compiler's IR.
sequenceDiagram
    participant User
    participant DAW_WebGL as WebGL Projection
    participant Mutator as AST Mutator
    participant Code as Source Code
    participant Compiler as tenutoc
    
    User->>DAW_WebGL: Drags Note visually to C5
    DAW_WebGL->>Mutator: Emit `coord_delta(y: +2)`
    Mutator->>Code: Topological Regex Replace (a4 -> c5)
    Code->>Compiler: Trigger File-Watch Rebuild
    Compiler->>DAW_WebGL: Push New Intermediate Representation
Human graphical interactions (mouse drags, block resizing) are algorithmically translated into absolute topological text-mutations directly within the source code, triggering real-time sub-millisecond LSP re-compilations. The Code remains the absolute, singular Source of Truth.
Addendum K: Neural Acoustic Synthesis (Latent Audio Integration)
To ensure Tenuto operates as the ultimate standard for AI-assisted music production, Addendum K introduces native hooks for Latent Diffusion and Neural Audio models.
style=neural: Routes logical timeline data directly to a local or cloud-based neural audio API.
Semantic Injection: ai_choir: s:1.prompt("A haunting gregorian chant") constructs a secure API payload. The generated .wav buffer is deterministically mapped back to the timeline, treating the output identical to a style=concrete block.
Addendum L: Cryptographic Provenance & Zero-Trust Execution
Because Tenuto logic can execute external Python backends and hardware lasers, enterprise implementations MUST guarantee absolute security.
C2PA Credentials: meta @{ provenance: @{ author: "0x...", ai_agent: "Claude 3.5" } } automatically embeds cryptographic watermarks into the exported .wav ID3 tags.
The Sandbox Mandate: Untrusted URIs enforce Path Traversal Guards (E5001), disable hardware execution (style=sacn, style=fb4), and strictly confine OSC packets to localhost (127.0.0.1).
Addendum M: The Skybridge Protocol (Teleportation Ecosystem)
Tenuto 4.0 is a fully open-source, sovereign digital signal processing language. However, its core compiler was mathematically governed and generated utilizing the proprietary .tela Teleportation Protocol—a 1024-dimension vector geometry engine that governs LLM code determinism.
The two projects operate in a Tandem Skyscraper Architecture connected by the "Skybridge." When the core team architects a feature, it is proven in .tela and poured into Tenuto's Python codebase. For end-users and open-source contributors, the Skybridge is severed; Tenuto requires no proprietary tools.
The Decoupling Mandate: While Tenuto acts as the crucible for Tela’s genesis phase, the .tela protocol is fundamentally domain-agnostic. Once Tenuto reaches 4.0.0 LTS stability, the Skybridge Protocol will decouple, allowing the Teleportation Engine to govern arbitrary enterprise architectures—from distributed databases to operating systems—leaving Tenuto as its pristine, sovereign offspring, and the eternal gold standard of how human creativity and autonomous AI engineering achieve absolute parity.

𝄆 TENUTO STUDIO 4.0.0 𝄇
The Bi-Directional Projectional DAW
Canonical UI/UX & Architecture Specification
Version: 4.0.0 (The Projectional DAW Edition) Status: Normative / Final License: MIT Maintainers: The Tenuto Working Group & The Teleportation Protocol Foundation Reference Implementation Stack: React 19 (TypeScript), WebGPU / WebGL (Pixi.js v8+), Monaco Editor, Web Audio API (AudioWorklet), WebAssembly (Pyodide), LanceDB (WASM).
Executive Abstract Digital Audio Workstations (DAWs) have historically trapped musical logic inside opaque, proprietary, binary blobs (.als, .logicx, .flp). Tenuto Studio 4.0.0 abolishes the binary project file.
Tenuto Studio is not a standard audio editor; it is a Projectional Interface. The underlying human-readable Tenuto source code (.ten) is the absolute, singular Source of Truth. The graphical canvas—piano rolls, mixer faders, and automation curves—is a high-performance WebGPU projection of the compiler's Abstract Syntax Tree (AST). Every graphical drag, click, and resize is algorithmically translated into a surgical, deterministic text mutation in real-time. It provides the fluid, tactile ergonomics of a billion-dollar DAW, unified with the infinite archival stability and AI-native architecture of a text-based language.
--------------------------------------------------------------------------------
1. Architectural Philosophy & The Projectional Mandate
To achieve a zero-friction, production-grade environment, compliant Tenuto Studio implementations MUST adhere to four axiomatic principles:
Text as Absolute Truth (Bi-Directional Determinism): The Monaco text editor and the WebGPU canvas are two mathematically synchronized viewports of the exact same structural data. A change in the code instantaneously compiles to redraw the canvas; a physical drag on the canvas deterministically rewrites the code. The UI possesses absolute zero proprietary structural state.
Zero-Latency Perception: Audio and graphics MUST NEVER wait for the main React UI thread. Heavy computations (LL(1) parsing, AST linearization) execute in an isolated Pyodide Web Worker; audio evaluates in isolated AudioWorklet threads via SharedArrayBuffer; graphics render on the GPU. The UI maintains a locked 60–120 FPS.
Immutable Command History (Infinite Undo): A persistent, disk-backed command history supports thousands of undo steps with an O(1) memory footprint. Because state is merely text, the history leverages Git-style structural sharing rather than duplicating massive binary RAM states.
Semantic Accessibility (The Shadow DOM): The WebGPU canvas cannot be an opaque visual box. It must project a synchronized HTML "Shadow Tree" to ensure 100% WCAG 2.1 AA screen-reader compatibility and complete keyboard navigability.
1.1 UI Conformance Profiles
Implementations may vary in feature density to support devices ranging from mobile tablets to studio hardware arrays. UI developers MUST target one of the following profiles:
Profile
Required Subsystems
Target Audience & Environment
Profile A (Core)
Monaco Editor, Read-Only Canvas, Transport, Local Undo.
Web Embeds, <tenuto-score> Custom Elements, Lightweight REPLs.
Profile B (Professional)
Core + Bi-Directional Editing (Drag-to-Mutate), Mixer View, RAG Assistant, CC Automation Lanes.
Reference Standard. Producers, Composers, AI Prompt Engineers (Desktop/Electron).
Profile C (Enterprise)
Professional + Multi-User CRDT Sync, OSC Hardware Mapping, TSN Stage View.
Theatrical Directors, Studio LAN Collaboration.
--------------------------------------------------------------------------------
2. Application Shell & Topography
The UI topography abandons floating windows in favor of a responsive, split-pane tiling manager, utilizing CSS Grid/Flexbox for rigid frame boundaries and zero cumulative layout shift (CLS).
graph TD
    A[Tenuto Studio Application Shell] --> B(Global Command Bar & Transport)
    A --> C(Left Sidebar: Architecture)
    A --> D(Center Workspace: The Crucible)
    A --> E(Right Sidebar: Intelligence & Inspector)
    A --> F(Bottom Status Console)

    D --> D1[Left Hemisphere: Monaco Editor]
    D --> D2[Right Hemisphere: WebGPU Canvas]

    C --> C1[File Explorer & Dependencies]
    C --> C2[AST Symbol Outline]

    E --> E1[AST Node Inspector Panel]
    E --> E2[LanceDB RAG AI Copilot]

    F --> F1[WASM Compiler Diagnostics]
    F --> F2[AudioWorklet CPU Load]
    F --> F3[Ableton Link / Network Sync]
2.1 The Center Workspace (The Split-Brain View)
The core user experience revolves around a resizable vertical split pane:
The Left Hemisphere (Code): A fully language-aware Monaco Editor instance utilizing a custom Tenuto Language Server Protocol (LSP). Features sub-millisecond semantic highlighting, dynamic code-folding of measure blocks, and inline red-squiggly error diagnostics (E1000-series) injected directly by the compiler worker.
The Right Hemisphere (Graphics): A high-performance WebGPU viewport. Adapts dynamically based on the active AST style scope (e.g., switches to a continuous Piano Roll for style=standard, a physical Fretboard for style=tab, or a flattened trigger grid for style=grid).
2.2 Contextual Sidebars
Left Sidebar (Navigation): Houses the File Explorer (.ten, .scl tuning files, .wav assets) and the Symbol Outline, a hierarchical tree of def, macro, and measure blocks parsed directly from the AST. Clicking an outline node instantly scrolls both the Monaco Editor and the Canvas to the absolute execution tick.
Right Sidebar (Mutator & AI):
The Inspector: Context-sensitive data bindings. Clicking a note on the canvas populates the Inspector with its exact logical duration, pitch, and attached modifiers (e.g., .stacc). Adjusting a slider here executes a direct text-mutation in the code editor.
The AI Orchestrator: An AI copilot hooked into the local codebase vector database (See Section 7).
--------------------------------------------------------------------------------
3. The Projectional WebGPU Canvas
The Canvas is the graphical materialization of the Tenuto Intermediate Representation (IR). To render symphonic-scale scores (100,000+ concurrent events) smoothly, the engine MUST utilize Spatial Hash Grids for O(1) object retrieval and Frustum Culling (only rendering events inside the current viewport).
3.1 Mathematical Mapping (AST to Screen Space)
The visual projection strictly adheres to linear algebra mappings derived from the rational metrical grid:
X-Axis (Logical Time): X 
px
​
 =( 
PPQ
T 
abs
​
 
​
 )×Z 
scale
​
  (Where T 
abs
​
  is the absolute tick, PPQ is 1920, and Z 
scale
​
  is the zoom factor).
Y-Axis (Standard/Synth): Y 
px
​
 =(127−MIDI_Integer)×Row_Height. The background grid dynamically highlights the diatonic root dictated by meta @{ key: "..." }.
Y-Axis (Grid/Concrete): Flattened single rows mapped sequentially based on the map=@{} dictionary in the track's def block.
3.2 Semantic Visual Grammar
Tenuto uniquely unifies acoustic phrasing with electronic physics. Every valid text token has a strict, non-destructive visual counterpart:
Tenuto Source Token
Canvas Representation (WebGPU)
PitchLit + :Duration
A solid rounded rectangle. Width matches exact metric grid capacity.
:grace
Scaled to 60% height, pinned to the exact physical start-tick of the parent note.
.stacc / .accent
High-contrast articulation glyphs anchored above the bounding box.
.pull(15ms) / .push
A translucent ghost block locked to the rigid mathematical grid, with an arrow (« / ») pointing to the solid block delayed/advanced by the absolute physical offset.
~ (Tie)
A continuous, algorithmically smoothed Bezier curve traversing the gap between identical consecutive blocks.
s:4.cc(7, [0,127], "exp")
Automation Lane: Unfolds a discrete UI layer revealing draggable Bezier control points for continuous LFOs beneath the primary track.
3.3 Semantic Zooming (Level of Detail)
To prevent pixel clutter, the shader utilizes Level of Detail (LOD) culling:
100% Zoom: Full bounding boxes, lyric text (.lyric), precise velocity shading, and articulation icons.
50% Zoom: Text fades; blocks collapse into solid rhythmic line-segments.
10% Zoom: Individual tracks collapse into a continuous thermal heatmap representing density and RMS amplitude.
--------------------------------------------------------------------------------
4. The Topological Mutator (Bi-Directional Editing)
This is the core engineering achievement of the Tenuto UI. Moving a graphic on screen MUST result in a deterministic regex/AST transformation of the raw text, utilizing the (line, column) origin metadata attached to every IR node.
4.1 The Mutation Lexicon
Graphical Action
Mutator Target
Executed Text Algorithm (Regex / AST Replace)
Drag Right Edge (Resize)
Duration Token
Calculates absolute Δx against active PPQ grid. Converts to Rational Fraction. Replaces :4 with :8..
Drag Y-Axis (Vertical)
PitchLit Token
Maps pixel-Y to MIDI integer. Invokes Inverse Spelling Engine (preferring Key Signature). Replaces c4 with eb4.
Drag X-Axis (Grid Snap)
Topological Index
Calculates logical metrical displacement. Automatically injects or deletes preceding Rests (r:8) to physically shift logic while preserving measure capacity.
Alt/Option + X-Drag (Free)
Micro-Timing
Bypasses grid layout. Computes absolute microsecond displacement. Appends or mutates .push(15ms) or .pull(10ms).
Right-Click ➔ "Tenuto"
Dot Attributes
Physically appends .ten strictly to the end of the active token string.
4.2 The Lexical Lock (Macro Safety)
If a user attempts to graphically drag an event generated by a Preprocessor Expansion (e.g., $Motif(c4)), it renders with a Lock Icon 🔒 and the Mutator MUST reject the interaction.
Rationale: Macros define 1-to-many relationships. Dragging a single projection of a macro cannot safely mutate the root text without unpredictable, destructive cascading side-effects. The GUI forces the user to either edit the macro definition or execute an "Unroll Macro" command.
--------------------------------------------------------------------------------
5. State Management & The Deterministic Pipeline
Tenuto Studio completely bans circular data flows. The architecture guarantees the UI can never fall out of sync with the audio compiler.
sequenceDiagram
    participant User
    participant Canvas (React/Pixi)
    participant Mutator (Command OT)
    participant Monaco (Source Code)
    participant tenutoc (Wasm Worker)
    participant AudioContext (Worklet)

    User->>Canvas: Drags Graphical Event Block
    Canvas->>Mutator: Emit `coord_delta(x: +15px, y: -1)`
    Mutator->>Monaco: Execute `TextEdit(line: 4, "r:8 b3")`
    Monaco-->>Canvas: Optimistic Syntax Highlight
    Monaco->>tenutoc: Debounced AST Re-compilation Payload
    tenutoc->>Canvas: Emit new Intermediate Representation (IR)
    Canvas->>Canvas: Flush WebGPU Geometry Buffers (Redraw)
    tenutoc->>AudioContext: Update Execution Tape via SharedArrayBuffer
5.1 Debounced Commits & Optimistic UI
To prevent flooding the Language Server during continuous mouse drags (e.g., sweeping a note across 4 measures at 60 FPS):
onMouseDown: The Canvas detaches the target block, applying an Optimistic Transform to follow the mouse.
onMouseMove: The Mutator mathematically computes the prospective AST string but does not write to Monaco.
onMouseUp: The Mutator dispatches a single, atomic string-replacement payload to the editor buffer, triggering background Pyodide compilation.
--------------------------------------------------------------------------------
6. Immutable Command Architecture (Undo/Redo)
DAWs are notorious for massive RAM consumption due to history states. By treating the project as a pure text document, Tenuto Studio achieves an infinitely deep, memory-efficient undo/redo stack using the Operational Transform (OT) Command Pattern.
6.1 Command Implementation (TypeScript Reference)
Every interaction instantiates a MutationCommand object that interfaces purely with the Monaco Editor's model layer.
export interface MutationCommand {
  id: string;
  type: 'TEXT_MUTATION' | 'META_UPDATE' | 'TRACK_ADD';
  
  execute(editor: monaco.editor.IStandaloneCodeEditor): void;
  undo(editor: monaco.editor.IStandaloneCodeEditor): void;
  
  /** 
   * Enables the compression of continuous mouse-drag events (e.g. moving a fader)
   * into a single atomic history state to prevent flooding the undo stack.
   */
  canMerge(prev: MutationCommand): boolean;
  merge(prev: MutationCommand): MutationCommand;
}
Persistent Disk Backing: Because the history stack consists merely of text diffs, it is automatically serialized to browser IndexedDB or a local SQLite cache. A user can close the IDE, reopen it months later, and hit Ctrl+Z to seamlessly revert their last fader movement.
--------------------------------------------------------------------------------
7. The Stateless Mixer (Addendum G Projection)
Reflecting Addendum G of the Language Specification, Tenuto Studio's Mixer View is completely stateless. It does not save hidden fader parameters in a binary file; it exclusively projects and mutates the document's meta configuration block.
7.1 Fader Mutation Protocol
When a user moves the volume fader for track vox to -6dB:
The UI searches the active measure block for an automation spacer vox: <[ ... | s:1.vol(80) ]>.
If found, it edits the integer directly.
If not found, it seamlessly injects an invisible CC control lane into the active measure block, or updates the global variable in Phase 1 (var vox_vol = 80).
7.2 Dynamic FX Racks & Sidechain Routing
Effects: Clicking the [+] slot on a mixer channel presents a dropdown of Addendum G primitives. Selecting "Reverb" deterministically appends .fx(reverb, @{mix: 0.5}) to the active track's AST initialization.
Sidechaining: Selecting "Drum_Bus" as a sidechain source for "808_Sub" instantly rewrites the global configuration header (meta @{ sidechain: ... }) in the code editor, preserving the mix routing as archival text.
--------------------------------------------------------------------------------
8. Intelligence & The AI Orchestrator (Addendum I)
Implementing the directives of Language Spec Addendum I, the UI natively integrates a highly advanced Retrieval-Augmented Generation (RAG) copilot. It is deeply embedded into the compiler's memory space, bypassing the limitations of generic chat wrappers.
8.1 Architecture & Workflow
Local Vector DB: Uses LanceDB (compiled to WASM) to index the active project’s .ten files alongside the canonical Tenuto 4.0 Language Documentation.
Context-Aware Queries: When a user types "Refactor this melody into a Euclidean rhythm," the UI packages the query alongside the exact AST string slice of the active Monaco cursor selection.
The "Ghost Patch" Preview: The LLM streams back raw Tenuto text. The Studio compiles a temporary branch AST. The Canvas renders the AI's generated notes as Glowing, Translucent Ghost Blocks overlaid on the user's current music.
Patch Committing: If the user clicks [Accept Patch], it executes an ASTCommand text injection, instantly solidifying the ghost blocks into the permanent timeline.
--------------------------------------------------------------------------------
9. Aesthetic Ontology & Accessibility (A11y)
Tenuto Studio completely decouples data from presentation. UI styling relies on CSS Variables (Custom Properties) and Radix UI primitives, guaranteeing WCAG 2.1 AA accessibility and instant hot-swappable themes.
9.1 Normative Theme Profiles
Theme Target
UX Objective
Visual Execution
standard
Studio Reference
Dark mode (Slate/Zinc). High-contrast #3B82F6 base geometry. Strict, rectilinear bounding boxes.
jazz
Organic Composition
#FDF6E3 Solarized background. Petaluma cursive fonts. WebGPU shader applies subtle ink-bleed jitter to block geometry.
dyslexia
Cognitive Accessibility
Overrides UI fonts with OpenDyslexic. Multiplies all Canvas X/Y scaling by 1.5×. Polyphonic `<[ v1
9.2 The "Shadow Tree" for Screen Readers
A WebGPU canvas is inherently opaque to screen readers (VoiceOver, NVDA). Tenuto Studio maps the IR into an invisible, absolutely positioned DOM tree layered exactly behind the canvas.
<div class="sr-only" role="application" aria-label="Tenuto Canvas Workspace">
  <div role="list" aria-label="Piano Track">
    <button role="listitem" aria-label="Note C4, Duration Quarter, Start Beat 1">...</button>
  </div>
</div>
When a user Tabs through the UI, focus moves through the hidden DOM, triggering screen reader announcements ("Pitch C4. Duration Quarter Note. Micro-timing pushed 15 milliseconds. Measure 2, Beat 1"), while the WebGPU canvas dynamically draws a high-contrast focus ring around the corresponding geometry.
9.3 Keyboard Supremacy Matrix
Keyboard efficiency is non-negotiable for professional DAWs.
Command
Shortcut (Mac/Win)
Deterministic Semantic Action
Global Play/Pause
Space
Toggles execution of WASM / AudioWorklet clock.
Undo / Redo
Cmd/Ctrl + Z / Shift + Cmd/Ctrl + Z
Traverses the MutationCommand stack.
Quantize Grid
1 ... 9
Changes active Cartesian snap-grid (1 = Whole, 4 = Quarter).
Duplicate Selected
Cmd/Ctrl + D
Clones selected tokens, advancing absolute tick.
Octave Shift
Cmd/Ctrl + Up/Down
Mutates PitchLit integer by ±12.
Toggle Staccato
S
Appends/Removes .stacc to selected notes.
Replace with Rest
R
Swaps selected PitchLit token with r.
--------------------------------------------------------------------------------
10. System Performance Benchmarks (The Studio SLA)
A compliant Profile B (Professional) implementation MUST mathematically guarantee the following thresholds when executing a 10,000-event symphonic .ten file on mid-tier consumer silicon (e.g., Apple M1 / Snapdragon X):
Telemetry Vector
Strict Threshold
Engineering Mechanism
Cold Start to Interactive
<1.2 Seconds
Edge-cached WASM payloads; lazy-loading LanceDB indexes.
AST Compilation Delta
<50 ms
Rust-to-WASM / Pyodide delta-compilation inside a Web Worker.
Canvas Frame Pacing
16.66 ms (60 FPS)
WebGPU Vertex Instancing; spatial culling of off-screen IR nodes.
Audio Execution Latency
<10 ms
Bypassing React main thread; compiling IR directly to SharedArrayBuffer.
Interaction Latency
<8 ms
Optimistic UI transforms prior to debounced AST commits.
DOM Memory Footprint
<150 MB
React Virtualization for Editor; Pixi.js Garbage Collection for Canvas.
--------------------------------------------------------------------------------
Status Validation: This specification achieves absolute parity between the linguistic mathematics of Tenuto 4.0 and the spatial/temporal demands of human-computer interaction. It strips away the archaic bloat of legacy DAWs, ensuring that the creative interface remains eternally scalable, auditable, and mathematically flawless. Proceed to Actuation via the Declarative Engineering Protocol.

TENUTO STUDIO 4: ONBOARDING, ACOUSTIC PARITY & ADVANCED UX SPECIFICATION
Version: 1.0.0 (The Day-Zero Edition) Status: Normative / Final License: MIT Maintainer: The Tenuto Working Group Scope: Out-of-the-box (OOTB) acoustic engines, progressive disclosure UI, interactive documentation, and frictionless adoption workflows.
--------------------------------------------------------------------------------
1. Architectural Philosophy: "Zero-to-Symphony in 3 Seconds"
Historically, professional music software suffers from a catastrophic "Time-to-First-Sound" (TTFS). Users are greeted with empty, silent grids, forced to configure complex MIDI routing, locate missing VST plugins, and manually assign system audio drivers before hearing a single note. Furthermore, text-based music environments suffer from "Blank Canvas Anxiety."
Tenuto Studio 4 eliminates this paradigm.
The application mandates a Zero-Friction Genesis. Whether loaded via a web URL or opened as a local Desktop executable, the software MUST immediately synthesize high-fidelity audio upon the first user interaction. There are no "Missing VST" errors. There is no manual routing. The code, the visual canvas, and the acoustic engine are deterministically pre-linked.
This specification details the Tenuto Core Acoustic Library (TCAL), the First-Time User Experience (FTUE), and the Integrated Knowledge Engine, ensuring Tenuto Studio is simultaneously accessible to a novice beatmaker and mathematically rigorous enough for a procedural orchestration AI.
--------------------------------------------------------------------------------
2. The Tenuto Core Acoustic Library (TCAL)
Standard OS-level General MIDI (GM) sounds archaic and actively discourages professional producers. Tenuto Studio natively bundles and dynamically streams a curated suite of the highest-quality open-source SoundFonts (.sfz, .sf3) and physical models available.
2.1 The Core Acoustic Assets (Open Source)
When a user defines an instrument using standard naming conventions (e.g., patch="gm_piano"), the WebAssembly AudioWorklet intercepts the request and maps it to the TCAL.
Instrument Family
Curated Open-Source Asset
Quality / Characteristic
Acoustic Piano
Salamander Grand V3
Yamaha C5 Grand. Multi-velocity layers, rich string resonance and pedal noise.
Orchestral Strings
VSCO-2 CE (Versilian Studios)
Chamber orchestra. Intimate, highly articulated (pizzicato, tremolo natively mapped to .pizz and .roll).
Acoustic Drums
AVL Drumkits
Multi-sampled, humanized rock/jazz kits. Mapped perfectly to Tenuto's style=grid acoustic mappings.
Modern Electronic
Tenuto Urban Kit
High-impact curated 808s, transient-shaped kicks, and Foley-layered snares.
Synthesizers
Native WASM Physics
Generated in real-time via style=synth. Evaluates exact sine/triangle math to WebAudio primitives; zero megabytes required.
2.2 Incremental RAM Streaming (Web Optimization)
Loading 2GB of high-fidelity orchestra samples on a web-page load would destroy adoption. Tenuto Studio Web uses an AST-Aware JIT (Just-In-Time) Streaming Architecture.
Pre-Compilation Interrogation: When the user opens a .ten file, the App Shell scans the AST. If the file only uses vln: c4:4.pizz, the compiler identifies that only the C4 Pizzicato sample layer is required.
Byte-Range Fetching: The Web Worker uses HTTP Range requests to fetch only the necessary compressed audio chunks from the CDN, caching them persistently in the browser's IndexedDB.
The Micro-Synth Fallback: If the user hits "Play" before a heavy sample finishes downloading, the engine instantly falls back to a 10kb micro-synth (a basic sine wave matching the ADSR profile) for those specific notes, cross-fading into the HD sample the exact millisecond the buffer is populated. Audio never stutters or halts.
2.3 Implicit Instrument Routing (Magic Defaults)
To eliminate boilerplate, the tenutoc compiler utilizes Semantic Inference to automatically map instruments to high-quality patches if the user omits the patch= attribute. If a beginner simply types def pno "Piano" style=standard, the system dynamically injects patch="gm_piano" to guarantee a beautiful sound profile immediately.
--------------------------------------------------------------------------------
3. The "First 5 Minutes" (Onboarding Pipeline)
The First-Time User Experience (FTUE) is designed to alleviate "Blank Canvas Syndrome" and gently introduce the Projectional Editing paradigm.
3.1 The Welcome Matrix & Templates
Upon first launch, the user is presented with a cinematic welcome screen featuring one-click starting points:
Blank Sketch (Sketch Mode): Invokes the tenuto-light wrapper. The user is dropped into an un-opinionated track where typing c4:4 d e instantly generates sound and visual blocks without needing a def setup.
The Lo-Fi Beatmaker: Pre-loads a 4-bar loop featuring an 808 sub-bass, a trap hi-hat grid, and a style=concrete vocal chop, perfectly pre-routed for sidechain compression.
The String Quartet: Pre-loads Violin I, Violin II, Viola, and Cello, perfectly panned and routed through a Hall Reverb FX bus.
3.2 The "Projectional Ping" Tutorial
To implicitly teach the user that Code is UI and UI is Code, the app executes a frictionless, 10-second interactive guided tour:
Step 1: The app highlights a line of code: pno: c4:4.
Step 2: A floating tooltip prompts: "Change c4 to e4."
Step 3: When the user types e4, the Canvas block physically animates upward. A success chime plays.
Step 4: A tooltip points to the Canvas block: "Now, grab the right edge of the block and drag it."
Step 5: As the user drags the block, the text editor highlights the duration token as it magically rewrites itself from :4 to :8..
Outcome: The user implicitly understands bi-directional editing forever, without reading a manual.
--------------------------------------------------------------------------------
4. The Interactive Knowledge Engine (Built-In Help)
Traditional documentation requires context-switching to a PDF or a Wiki. Tenuto Studio embeds an interactive, AST-aware Help Guide & Lexicon directly into the Right Sidebar. This guide visually teaches users how to leverage Tenuto's most advanced features without memorizing syntax.
4.1 The Euclidean Tuplet Dial (Algorithmic Rhythms)
Writing (k):3/8 is incredibly token-efficient, but visualizing polyrhythms can be difficult for beginners.
The UI Interaction: If a user right-clicks a drum track and selects "Euclidean Builder," a circular "pie chart" dial appears as an overlay on the Canvas.
The Magic: The user drags the inner ring to set the "Hits" (K=3) and the outer ring to set the "Steps" (N=8). As the user rotates the dial, the UI instantly previews the rhythm visually as dots on the circle, plays the audio, and writes the (k):3/8 text into the editor in real-time.
4.2 Automation Shaping (The Curve Editor)
For .cc messages (volume swells) or .pan arrays.
The UI Interaction: The user clicks the chevron [v] on a track header to open its Z-Axis Automation Lane.
The Magic: A vector line graph appears. The user right-clicks a segment and chooses "Exponential". The line physically curves. Dragging the curve up/down alters the intensity. The UI seamlessly converts these mouse movements into s:4.cc(7, [0, 127], "exp") text on a decoupled invisible track in the editor.
4.3 The Semantic Lens (Hover X-Ray)
Every visual element in the Canvas and every token in the Code Editor possesses a deeply integrated "Lens" popover.
In the Code: Holding Alt/Option and hovering over the .slice(8) attribute pops up an animated 2-second GIF of an audio waveform being chopped into 8 pieces, providing instant visual comprehension of the abstract DSP command.
In the Canvas: Hovering over a tied note displays a tooltip showing its absolute rational duration (7/16) and its calculated physical execution time in milliseconds.
4.4 "Show Me" Macro-Injections
When a user reads a Codex entry about a complex feature, the documentation contains a glowing [Try it in my project] button.
Action: Clicking the button does not just copy text to the clipboard. The UI temporarily injects a new measure block into the user's active Monaco editor, immediately renders it on the Canvas, and loops playback.
Reversion: The injected block has a floating [Keep] / [Discard] overlay, allowing consequence-free experimentation.
--------------------------------------------------------------------------------
5. Ergonomic Polish & "Billion-Dollar" QoL
A true "billion-dollar" app doesn't just look good when things go right; it handles user mistakes with grace and intelligence.
5.1 Smart Error Resolution (Quick Fixes)
When the Tenuto compiler throws a strict mathematical error, the UI replaces traditional stack traces with Fail-Forward Diagnostics.
Scenario: The user writes a polyphonic block where Voice 1 is 4 beats long, but Voice 2 is 3.5 beats long.
Compiler Action: Throws E3002: Voice Sync Failure. The text gets a red squiggle.
The Polish: A floating lightbulb icon appears. Clicking it reveals: "Voice 2 is missing 0.5 beats. [Click here to auto-pad with Rests]". Clicking the button securely edits the AST, appending r:8 to Voice 2, instantly clearing the error and re-rendering the canvas.
5.2 The Command Palette (Cmd/Ctrl + K)
Power users hate moving their hands from the keyboard to the mouse. Tenuto Studio implements a universal Command Palette.
Press Cmd/Ctrl + K to open a fuzzy-search overlay.
Searchable Actions:
> Extract to Macro (Automatically cuts highlighted Canvas blocks and wraps them in a $macro definition at the top of the file).
> Theme: Dyslexia (Instantly applies OpenDyslexic fonts and wide staff spacing to both code and canvas).
> Export: WAV (Triggers background WebAudio offline rendering).
5.3 Graceful Degradation (The "Never Stop the Music" Rule)
In traditional live-coding environments, a syntax error crashes the audio engine, creating dead silence on stage.
Tenuto Protocol: If a user is actively typing and introduces an E1002 Syntax Error, the WebAudio thread MUST NOT crash. The UI flashes the specific token in red, but the compiler continues to loop the last known valid Intermediate Representation (IR) until the user fixes the typo. The music never stops.
5.4 Audio Scrubbing & "Needle Drop"
Action: Holding Shift and clicking anywhere on the visual timeline ruler.
The Polish: The UI executes a "Needle Drop". The AudioWorklet intelligently calculates all active sticky states, preceding ties, and ADSR envelopes from the start of the file up to that exact millisecond. This allows instantaneous, mathematically perfect playback from the middle of a 10-minute symphony without the "dropped note" phenomenon common in standard MIDI DAWs.
--------------------------------------------------------------------------------
6. Export, Collaboration & Interoperability
Adoption requires that users never feel trapped in a walled garden. Tenuto Studio 4.0 is engineered for absolute data portability.
6.1 AST URL Serialization ("Share as Link")
To foster a massive open-source community and viral adoption, Tenuto Studio Web supports AST URL Serialization.
Mechanism: Because Tenuto text is incredibly token-efficient, the entire AST of a song can be Base64 encoded and compressed into the URL parameters of the web app (e.g., studio.tenuto.org/?code=eJyrV...).
UX: A user clicks "Share". They receive a standard hyperlink. When a peer clicks the link, Tenuto Studio loads instantly in their browser, parses the URL, rebuilds the AST, fetches the required WASM audio samples, and plays the song—all without requiring a database query, file upload, or user login.
6.2 The Unified .zip Archive (For Concrete Audio)
If a user imports local .wav files for a style=concrete sampler track, sharing the .ten text file alone will break the file paths for the recipient.
Execution: Tenuto Studio dynamically packages the .ten source code AND all referenced local .wav files into a single standardized .zip archive. When the recipient drags this .zip into their browser, the UI virtually mounts the folder structure using the File System API, ensuring 100% flawless playback replication.
6.3 The Universal "Render" Dialog
Located in the top right Transport bar, the [Export] button opens the universal render matrix:
Audio (WAV/FLAC): The WebAudio/WASM engine performs an offline, faster-than-realtime bounce of the IR using the browser's OfflineAudioContext. A 5-minute track calculates in less than 3 seconds.
Stems Export: A single toggle renders every def track to its own isolated audio file, perfectly synced for mixing in Ableton or Pro Tools.
Sheet Music (PDF/MusicXML): Invokes the Visual Translation Layer. Unprintable physics (like .fx and .slice) are pruned, resolving skyline collisions, and triggering a native browser print dialog for publication-ready sheet music.

𝄆 TENUTO STUDIO 4.0 𝄇
Comprehensive UI Component & Interaction Architecture
Deployment Targets: Web Browser (PWA) & Local Desktop (Tauri/Electron)
To fully define the Tenuto Studio 4 user interface, we must construct a highly modular, deterministic component tree. Because the application must run flawlessly both in a standard web browser (via the File System Access API) and as a native local executable, the UI is entirely decoupled from the OS file system.
Below is the definitive visual, interaction, and component blueprint for Tenuto Studio 4.
--------------------------------------------------------------------------------
1. The Hybrid Application Shell
The App Shell is the root React component. It manages global state, window resizing, and the WebAssembly (WASM) compiler lifecycle.
1.1 Responsive Topology (The CSS Grid)
The UI relies on a strict 100vh / 100vw CSS Grid layout to eliminate scrolling outside of designated viewports.
Top Bar (Header): h-12. Contains the Project Name, Global File Menu, and Transport Controls.
Left Sidebar (Explorer): w-64 (Resizable). Contains the File Tree and AST Outline.
Center Workspace (The Crucible): flex-1. The split-pane dividing the Code Editor and the Canvas.
Right Sidebar (Inspector/AI): w-80 (Resizable, Collapsible). Contains the Context Inspector and LanceDB AI Copilot.
Bottom Bar (Status): h-8. Contains compiler diagnostics, CPU/DSP load, and network sync status.
1.2 Environment Parity
Web Mode: Uses the HTML5 showDirectoryPicker() to request permission to read/write a local folder. The UI renders a "Sync Status" icon indicating when the virtual memory buffer is flushed to the local disk.
Local Mode (Desktop): Utilizes native OS dialogs. Bypasses the browser sandbox to directly watch the file system. If an external text editor modifies the .ten file, the App Shell detects the change via file-watching APIs and instantly triggers a re-render.
--------------------------------------------------------------------------------
2. The Code Workspace (Left Hemisphere)
Powered by a deeply customized instance of the Monaco Editor, this is the primary input vector.
2.1 The Tenuto Language Server Protocol (LSP) UI
Semantic Highlighting:
Keywords (def, measure, meta): Purple (#C678DD).
Primitives (c4, 15ms, 0-6): Cyan (#56B6C2).
Attributes (.stacc, .pull): Yellow (#E5C07B).
Hover Cards: Hovering over a pitch (e.g., c4) invokes a floating popover displaying the absolute frequency (261.63 Hz) and its active sticky state (e.g., Current Velocity: 80, Duration: 1/4).
Inline Diagnostics: Mathematical errors (e.g., E3002: Voice Sync Failure) generate red squiggly underlines. Hovering displays the exact duration mismatch in rational fractions (e.g., Voice 1 = 3/4, Voice 2 = 7/8).
Ghost Text (AI Suggestions): When the RAG Assistant predicts the next measure, it appears as inline gray, italicized text. Pressing Tab accepts the code.
--------------------------------------------------------------------------------
3. The WebGPU Projectional Canvas (Right Hemisphere)
The Canvas is rendered via Pixi.js (WebGPU) to handle 100,000+ geometric objects without dropping below 60 FPS.
3.1 Track Headers (The Y-Axis Anchor)
Dynamically generated from Phase 2 def blocks. Each header is h-24 and fixed to the left of the canvas.
Visuals: Contains the Track Name, Style Icon (e.g., a waveform for concrete, a keyboard for standard), Mute (M) button, Solo (S) button, and a micro-volume slider.
Interaction: Clicking the Track Header selects the entire track in the AST, highlighting all corresponding code in the Monaco editor.
3.2 The Timeline & Playhead (The X-Axis)
The Grid Ruler: Fixed at the top of the canvas. Displays measure numbers and subdivision ticks.
The Playhead: A bright red, 1px vertical line. In Web Mode, its position is driven by the AudioContext.currentTime mapped against the global tempo map.
Smooth Scrolling: The canvas automatically pans to keep the playhead centered during playback.
3.3 Semantic Event Blocks
Every note or sample is a distinct, interactive WebGPU Polygon.
Standard Notes: Rendered with horizontal rounded corners.
Ties (~): Rendered as a glowing SVG path connecting two blocks. If the user drags the first block, the Bezier curve dynamically recalculates.
Ghost Blocks (Micro-Timing): If a note has .pull(15ms), the UI renders a dotted-outline block exactly on the grid, with a solid block shifted slightly to the right to represent the actual physical playback time.
3.4 Decoupled Automation Lanes (The Z-Axis)
When an automation attribute is detected (e.g., .cc(7, [0, 127], "exp")), an expansion chevron [v] appears on the Track Header.
Unfolding: Clicking it reveals a sub-lane dedicated to continuous controllers.
Curve Editing: The user sees a Bezier curve representing the exponential volume swell. Double-clicking the line adds a control point. Dragging the point dynamically updates the interpolation string in the text editor.
--------------------------------------------------------------------------------
4. The Global Mixer & DSP Console
Accessible via a floating overlay or a dedicated workspace tab, the Mixer projects the meta @{ sidechain: ... } and track-level .fx() attributes.
4.1 Channel Strips
Faders: Vertical sliders mapping 0 to 127 (or −∞ to +6 dB). Dragging the fader writes var track_vol = X into the source text.
FX Racks: A vertical stack of slots above the fader.
Empty Slot: Displays [ + ].
Filled Slot: Displays the active effect (e.g., [ Reverb ]). Clicking it opens a modal with virtual knobs mapped to the dictionary arguments (mix: 0.5, decay: 4s).
Routing Matrix: A dropdown at the bottom of the strip dictates the bus:// target (e.g., routing a Drum Kit to a Drum Bus).
--------------------------------------------------------------------------------
5. Right Sidebar: Inspector & AI Copilot
5.1 The Context Inspector
When an item is selected on the Canvas, the Inspector populates with deterministic input fields.
Selection: A PitchLit event.
UI Elements:
Duration Dropdown: [1/1, 1/2, 1/4, 1/8, 1/16]. Changing this instantly updates the text from :4 to :8.
Velocity Slider: Updates the dynamics.
Modifiers Checklist: Toggles for Staccato, Accent, Tenuto.
AST Origin Data: Read-only footer showing exact line/column of the source text.
5.2 The LanceDB Assistant
A chat interface that functions as the AI Studio Builder agent.
User Input: "Add a low-pass filter to the bass track."
UI Feedback: The assistant shows a loading spinner labeled [Querying Tenuto AST...].
Action: It outputs a suggested code diff. The user clicks Apply Patch, and the text editor and canvas simultaneously update.
--------------------------------------------------------------------------------
6. Design System & CSS Variables (Radix + Tailwind)
Tenuto Studio relies on a strict token system to ensure perfect contrast and themeability.
:root[data-theme="studio-dark"] {
  /* Layout & Surfaces */
  --bg-app-shell: #0F172A; /* Tailwind Slate 900 */
  --bg-workspace: #1E293B; /* Tailwind Slate 800 */
  --bg-canvas: #0B0F19;    /* Deep contrast for WebGPU */
  
  /* Typography */
  --font-mono: 'JetBrains Mono', monospace;
  --font-ui: 'Inter', sans-serif;
  
  /* Semantic Colors */
  --color-accent: #3B82F6; /* Active selections */
  --color-error:  #EF4444; /* E-Series Compiler Errors */
  --color-warn:   #F59E0B; /* W-Series Compiler Warnings */
  --color-success:#10B981; /* Tests Passed / Sync OK */
  
  /* Canvas Event Colors (Track Default Overrides) */
  --event-piano: #60A5FA;
  --event-drums: #F472B6;
  --event-synth: #A78BFA;
  
  /* Borders & Dividers */
  --border-subtle: #334155;
}
6.1 Component Library (Radix UI)
Context Menus: Right-clicking the canvas uses Radix ContextMenu to ensure it never clips off-screen and complies with ARIA accessibility standards.
Modals: Dialogs for exporting to MusicXML or WAV are rendered via Radix Dialog, utilizing a backdrop blur to maintain focus.
Split Panes: Implemented using react-resizable-panels, ensuring the user can collapse the text editor entirely if they only want to work visually, or collapse the canvas to code distraction-free.
--------------------------------------------------------------------------------
7. Interaction State Machine (The Data Flow)
To guarantee the "Zero-Latency Perception" mandate, the UI employs a highly specific React state architecture:
User Action: User clicks and drags a Note Block on the Canvas to the right (changing its time).
Optimistic Canvas State (requestAnimationFrame): The Canvas instantly detaches the graphic from the main WebGPU buffer and draws it under the mouse cursor. The UI feels infinitely responsive.
Command Generation: The drag triggers a MoveEventCommand containing the ASTNode_ID and the visual ΔX.
Debounced Commit (onMouseUp): When the user releases the mouse, the UI translates ΔX into a rational time value (e.g., +1/8th note). It executes a regex/AST text replacement in the hidden Monaco editor buffer.
Reconciliation: The text change triggers the WASM compiler. The compiler generates a fresh Intermediate Representation (IR). The Canvas receives the new IR, drops its optimistic state, and renders the true mathematical geometry. Because the translation was deterministic, the block visually remains exactly where the user dropped it.
