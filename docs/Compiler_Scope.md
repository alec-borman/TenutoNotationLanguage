# Tenuto Language Specification & Implementation State

**Version:** 2.2.0 (The Performance Engine)  
**Status:** Normative / Tier 3 Reference Compliance  
**Compiler:** `tenutoc` (Rust)  
**License:** MIT  

---

## 1. Introduction & Implementation Summary

Tenuto is a declarative, domain-specific language (DSL) designed to serialize musical logic, instrument physics, and performance data into a highly structured, human-readable text format. It bridges the "Semantic Gap" between visual typography (Sheet Music) and mechanical execution (MIDI).

### 1.1 Reference Compiler Status (`tenutoc` v2.2.0)
The official Rust reference compiler (`tenutoc`) has achieved **Tier 3 (Full Audio Performance & Interchange) Compliance**.
*   **Lexical & Syntactic Layer:** 100% Compliant (Deterministic LL(1)).
*   **Semantic & Logical Layer:** 100% Compliant (Rational Time, Sticky State, Polyphonic Sync).
*   **Visual Interchange Layer:** 100% Compliant (MusicXML 4.0, Algorithmic Spelling, Rebarring).
*   **Performance Layer:** 95% Compliant (CC Automation, Bends, Tremolos, Ghost Notes implemented. Structural graph unrolling for repeats is currently stubbed).

### 1.2 Design Philosophy
1.  **Ontological Separation:** Instrument physics (tuning, capabilities) are strictly separated from musical logic.
2.  **Contextual Inference ("Sticky State"):** Attributes like duration and octave persist until explicitly changed, mirroring how human musicians read sight-music, and reducing file sizes by up to 90% over XML.
3.  **Absolute Mathematical Truth:** Time is evaluated using exact fractions (Rational Arithmetic), eliminating the floating-point quantization drift found in standard DAWs.

### 1.3 The Compilation Pipeline
The reference compiler implements a 6-stage transformation pipeline:
1.  **Lexing (`logos`):** O(n) tokenization with atomic compound sigils.
2.  **Parsing (`chumsky`):** LL(1) deterministic AST generation with `ariadne` error recovery.
3.  **Preprocessing:** Macro expansion and deep variable substitution.
4.  **Inference (IR):** Resolves the "Sticky State" into an absolute-time timeline.
5.  **Visual Translation:** Rebarring (slicing time into measures) and Diatonic Spelling.
6.  **Export:** Serialization to MIDI 1.0 and MusicXML 4.0.

---

## 2. Lexical Structure & Grammar
*Implementation Status: 🟢 100% Complete*

Tenuto enforces strict rules regarding case sensitivity and unique sigils to guarantee $O(1)$ linear-time parsing without ambiguity or deep backtracking.

### 2.1 Character Set & Case Sensitivity
*   Files **MUST** be encoded in UTF-8.
*   **Keywords** (`def`, `measure`, `meta`) and **Note Names** (`c4`, `F#5`) are Case-Insensitive.
*   **Identifiers** (Staff IDs, Macros, Variables) and **String Literals** are Case-Sensitive.
*   **Comments** are denoted by `%%` and ignore all subsequent text on the line.

### 2.2 Compound Sigils (The v2.1 Deterministic Update)
To differentiate between structural code blocks and internal data arrays, Tenuto utilizes unique compound operators.
*   `@{ ... }` **(The Map Sigil):** Encloses Key-Value data structures (Metadata blocks, Instrument attributes).
*   `<[ ... ]>` **(Voice Brackets):** Encloses multi-voice polyphonic blocks within a single staff assignment.
*   `{ ... }` **(Structural Braces):** Encloses high-level scopes (`tenuto`, `measure`, `group`, `macro`).

### 2.3 Domain-Specific Primitives
The lexer natively captures musical data types before they reach the parser:
*   **PitchLit:** `[a-gA-G][#|b|x|n|qs|qf...]*[0-9]?` (e.g., `c#4`, `ebqs2`).
*   **DurationLit:** `:[0-9]+` or `:grace` (e.g., `:4`, `:16`).
*   **TabLit:** `[0-9xX]+-[0-9]+` (e.g., `0-6`, `12-2`).
*   **AttributeLit:** `\.[a-zA-Z0-9_]+` (e.g., `.stacc`, `.vol(80)`).

---

## 3. Document Structure & Merging
*Implementation Status: 🟢 100% Complete (Single-File) | 🔴 External `import` Stubbed*

A valid Tenuto document is a self-contained unit of musical logic enforcing a **Declaration-Before-Use** policy across three phases.

### 3.1 Phase 1: Configuration (`meta`)
Establishes the global environment. Uses the Map Sigil (`@{}`).
```tenuto
meta @{ title: "Symphony", tempo: 120, time: "4/4", key: "D" }
```

### 3.2 Phase 2: Definition (`def`)
Registers Instrument IDs into the Global Symbol Table.
```tenuto
def vln "Violin I" style=standard patch="gm_violin"
```
*   *Constraint:* An ID must be defined before it is referenced in a `measure`.

### 3.3 Phase 3: Logic (`measure`)
The container for temporal events.
```tenuto
measure 1 { vln: c4:4 d e f | }
```

### 3.4 Dynamic Additive Merging (The IR Absolute Grid)
*Implementation Status: 🟢 100% Complete (v2.2.0)*

Tenuto utilizes an **Additive Merge Strategy**. A `measure` block is an open container indexed by a time-slice. 
*   **The Grid:** The compiler dynamically calculates a global "Measure Grid" based on the active Time Signature (e.g., `4/4` = 7680 ticks; `3/4` = 5760 ticks).
*   **The Merge:** If `measure 1` is declared for the Violin, and later `measure 1` is declared for the Cello, the compiler mathematically maps both sets of logic to the exact same absolute start-tick in the IR. 

### 3.5 Preprocessor: Variables and Macros
*Implementation Status: 🟢 100% Complete*

*   **Variables:** `var my_vol = 80`. The preprocessor recursively injects variables into standard events and nested data maps.
*   **Macros:** `macro Motif(root) = { $root:8 d e f }`. Macros act as compile-time text substitutions.
*   **Transposition:** `$Motif(c4)+2` iterates through the macro body, mathematically shifting all Scientific Pitch Notation events up by 2 semitones (to `d4`) before passing them to the IR. 
*   *Safety:* Recursion depth is strictly capped at `64` to prevent malicious infinite expansions (`E5002`).


## 4. Instrument Definitions (The Physics)
*Implementation Status: 🟢 100% Complete*

Tenuto enforces a strict ontological separation between the physical parameters of an instrument and the musical notes played on it. The `def` statement registers a new Staff ID and configures how the parsing engine translates input into MIDI/XML data.

**Syntax:** `def [ID] [Label] [Attributes]`

### 4.1 Core Attributes
Attributes are defined using space-separated `key=value` pairs. Complex values utilize the V2.1 Map Sigil (`@{}`).

| Attribute | Valid Styles | Type | Description | Compiler Implementation |
| :--- | :--- | :--- | :--- | :--- |
| **`style`** | All | Enum | `standard`, `tab`, or `grid`. Dictates the parsing engine. | 🟢 Executed in `ir.rs`. |
| **`patch`** | All | String | GM standard name (`gm_piano`) or patch ID. | 🟢 Resolves to MIDI Program Change. |
| **`tuning`** | `tab` | Array | Open string pitches (Low to High). e.g., `[40, 45, 50, 55, 59, 64]`. | 🟢 Mathematical base for Tab offsets. |
| **`map`** | `grid` | Map | Token-to-MIDI mapping. e.g., `@{ k: [0, 36] }`. | 🟢 Native mapping in `ir.rs`. |
| **`keyswitch`**| `standard` | Map | Articulation-to-MIDI mapping. e.g., `@{ pizz: 25 }`. | 🟢 Emits silent 1-tick trigger notes. |

---

## 5. The Cognitive Input Engines
*Implementation Status: 🟢 100% Complete*

Because a pianist, a guitarist, and a drummer conceptualize music differently, the compiler routes events through three distinct cognitive engines based on the instrument's `style`.

### 5.1 Standard Engine (`style=standard`)
*   **Input Data:** Scientific Pitch Notation (`c4`, `F#5`).
*   **Chords:** Multiple pitches played simultaneously are enclosed in brackets `[c4 e4 g4]:2`. The compiler unrolls these into parallel `AtomicEvent`s.
*   **Ties (`~`):** Appending a tilde (`c4~`) places the MIDI pitch into a forward-looking memory queue. When the next `c4` is encountered, the IR compiler does *not* strike a new note; it extends the `duration_ticks` and `gate_ticks` of the original event, ensuring seamless MIDI playback and valid XML ties.

### 5.2 Tablature Engine (`style=tab`)
*   **Input Data:** Tab Coordinates formatted as `Fret-String` (e.g., `0-6`, `12-2`).
*   **The Inverse String Rule:** String `1` always represents the highest-pitched (physically thinnest) string. The compiler mathematically derives absolute pitch by indexing the instrument's `tuning` array in reverse:
    `Pitch = tuning[tuning.len() - String] + Fret`
*   *Note on Visuals:* The engine calculates the sounding MIDI pitch for audio, but preserves the original string/fret data for XML `<technical>` tags.

### 5.3 Percussion Engine (`style=grid`)
*   **Input Data:** Arbitrary alphanumeric keys (`k`, `sn`, `hh`).
*   **The Mapping Rule:** The compiler bypasses standard pitch math entirely. It intercepts the key, queries the staff's `perc_map` dictionary, and emits the configured MIDI integer (e.g., `36` for kick drum).

---

## 6. Pitch Spelling & Microtonality
*Implementation Status: 🟢 100% Complete (via `tenutoc::spelling`)*

Translating an absolute MIDI integer (e.g., `61`) into graphical sheet music requires complex contextual algorithms to determine if the note should be rendered as a **C♯** or a **D♭**.

### 6.1 The Algorithmic Speller (Line of Fifths)
When an event originates from `style=tab` or `style=grid`, the compiler only knows its mathematical MIDI value. To generate MusicXML, the engine algorithmic derives the diatonic spelling.
1.  **Diatonic Match:** It checks the active Key Signature (`meta @{ key: "D" }`). If the MIDI note natively exists in the key, it spells it accordingly.
2.  **Chromatic Fallback:** If the note is non-diatonic, it references the Line of Fifths. In sharp keys (G, D, A), it prefers sharp spellings; in flat keys (F, Bb, Eb), it prefers flats.

### 6.2 The Accidental State Machine (Gould's Rules)
To determine if a note requires explicit ink on the page (an accidental symbol), the compiler passes every `SpelledPitch` through an internal State Machine that strictly adheres to professional engraving rules.
*   **Measure Resets:** Accidental memory is wiped completely clean at every absolute barline.
*   **Octave Isolation:** A written C♯4 does *not* apply to a C5 in the same measure. The state machine tracks memory independently per-octave.
*   **Cancellation (Natural Signs):** If the Key Signature dictates F♯, and the compiler encounters an F natural, the state machine registers a deviation from the baseline and actively forces an `AccidentalDisplay::Explicit` instruction, ensuring the `<accidental>natural</accidental>` tag is printed in the XML.

### 6.3 Microtonality
Microtonal intent is natively integrated into the Lexer and AST.
*   **Syntax:** Suffixes `qs` (+50 cents), `qf` (-50 cents), `tqs` (+150 cents), `tqf` (-150 cents).
*   **Execution (MIDI):** The IR translates these suffixes into precise 14-bit MIDI Pitch Bend wrapper events. It applies the bend *immediately before* the NoteOn, and issues a center-reset (8192) *immediately after* the NoteOff to prevent smearing.
*   **Execution (Visual):** The XML exporter translates the modifiers into exact floating-point `<alter>` tags (e.g., `<alter>1.5</alter>`) and their corresponding `<accidental>` visual strings.


## 7. Rhythm & The Event Engine
*Implementation Status: 🟢 100% Complete*

The core of Tenuto's efficiency is its handling of time. Unlike coordinate-based formats that require explicit start positions for every event, Tenuto treats music as a **Linear Stream of Durations**. The absolute start time of an event is deterministically calculated as the sum of all preceding durations in that voice.

### 7.1 Rational Temporal Arithmetic (Defeating Drift)
Standard digital audio workstations (DAWs) operate on an integer grid (e.g., 960 PPQ). When evaluating irrational rhythms like tuplets (e.g., playing 3 notes in the space of 2), dividing $960$ by $3$ works, but dividing $960$ by $7$ results in floating-point truncation, causing temporal drift over long symphonies.

*   **The Rational Engine:** The Tenuto IR compiler represents all durations internally as exact `Rational` structures (fractions with explicit numerators and denominators).
*   **The Math:** A triplet eighth note is not evaluated as `0.33333` beats. It is stored as $\frac{1}{2} \times \frac{2}{3} = \frac{1}{3}$. The engine only converts these perfect fractions into PPQ integer ticks (`u64`) at the final instant of event instantiation, mathematically preventing drift.

### 7.2 The Sticky State Cursor
To minimize file size, Tenuto employs a stateful `Cursor`. If a duration or octave is omitted from an event, the compiler infers it from the cursor's memory.

*   **Syntax:** Duration is denoted by a colon `c4:4` (Quarter note) or `:8.` (Dotted eighth).
*   **Persistence:** `c4:4 d e f`  The compiler remembers `:4` and applies it to `d`, `e`, and `f`.
*   **Barline Boundaries:** By default, the sticky state flows continuously across barlines. However, if `--strict` mode is enabled, the compiler forcefully resets the cursor's duration (`:4`) and octave (`4`) at every new measure block, enforcing explicit, archival-safe coding practices.

### 7.3 Tuplets & Grace Notes
*   **Tuplets `(events):P/Q`:** The compiler enters a temporary scope where the cursor's `time_scalar` is multiplied by the ratio $\frac{Q}{P}$. The IR attaches a `TupletState` to the resulting events so the MusicXML exporter knows exactly where to draw `<tuplet>` brackets and `<time-modification>` tags.
*   **Grace Notes `:grace`:** Evaluated as atemporal events. They consume `0` logical ticks (so they do not break the visual measure grid) but steal `ppq/4` physical gate ticks to ensure they play back audibly in the MIDI stream. Grace durations bypass the sticky state entirely.

---

## 8. Advanced Polyphony & Synchronization
*Implementation Status: 🟢 100% Complete*

Tenuto supports multi-threaded logic within a single staff, allowing independent rhythmic streams (e.g., a Pianist playing a melody and accompaniment simultaneously).

### 8.1 Voice Brackets (`<[ ]>`)
Polyphonic regions are enclosed in V2.1 Voice Brackets. Distinct voices (`v1`, `v2`) are separated by the pipe character `|`.

```tenuto
pno: <[
  v1: c5:4 d e f |
  v2: c3:1       |
]>
```

### 8.2 Scope Isolation
To prevent logical corruption between parallel threads, the compiler manages multiple cursors simultaneously:
*   **Primary Voice (`v1`):** Inherits the global sticky state from the events preceding the block, and exports its final state back to the main staff line when the block closes.
*   **Secondary Voices (`v2..v4`):** The compiler generates isolated, clean cursors (defaulting to `:4` and `Octave 4`) upon entry.

### 8.3 The Synchronization Constraint (Strict Mode)
To ensure the measure remains mathematically valid, the compiler tracks the total duration consumed by each voice. If `--strict` mode is enabled, the compiler enforces that the total duration of events in *every* declared voice must be identical. If `v1` lasts 3840 ticks and `v2` lasts 1920 ticks, the compiler halts and throws `E3002: Voice Sync Failure`.

---

## 9. Visual Typography: The Rebarring Engine
*Implementation Status: 🟢 100% Complete (via `tenutoc::rebar`)*

Because the `Timeline` IR is an absolute, continuous 1-dimensional stream of ticks, it cannot be directly rendered into sheet music. The Rebarring Engine translates this continuous tape into a 2D layout consisting of rigid visual boxes called `VisualMeasures`.

### 9.1 The Measure Grid
The engine scans the global metadata for Time Signatures (`meta @{ time: "3/4" }`). It calculates the absolute tick capacity of every measure in the piece, building a continuous grid array (e.g., `[0, 5760, 11520]`).

### 9.2 The Guillotine (Note Slicing)
The engine iterates through the timeline. If an `AtomicEvent` straddles a measure boundary (e.g., a 4-beat note starting on beat 3), it executes **The Guillotine Algorithm**:
1.  It mathematically slices the note into two distinct `VisualEvent`s precisely at the barline tick.
2.  It distributes the `gate_ticks` proportionally between the slices for accurate audio-playback tracking.
3.  It sets the `tie_start` flag on the first slice, and `tie_stop` on the second, signaling the XML exporter to draw `<notations><tied type="start"/></notations>`.
4.  *Safety:* The algorithm identifies `EventKind::Rest` and slices them mathematically, but deliberately suppresses the visual tie flags.

### 9.3 The Void Filler
Visual layout requires that every measure add up perfectly to its time signature. The engine utilizes a **Void Filler Algorithm** to scan each `VisualMeasure`. If it detects chronological gaps between events, or if a measure is completely empty, it automatically instantiates and injects explicit `<rest/>` events to fill the exact tick deficit.


## 10. Continuous Control & Automation (v2.2.0)
*Implementation Status: 🟢 100% Complete*

Tenuto 2.2.0 transforms the language from a static notation formatter into a high-fidelity performance engine. The compiler parses text-based automation curves and dynamically translates them into sub-measure physical events.

### 10.1 MIDI Control Change (CC) Sweeps
*   **Syntax:** `Event.cc(Controller,[StartVal, EndVal], "CurveType")`
*   **Execution:** The Inference Engine evaluates the logical duration of the event. It divides the duration into discrete 48-tick "steps" (approx 10ms at 120 BPM) and generates a dense array of `TrackEventKind::Midi(Controller)` bytes.
*   *Example:* `c4:1.cc(11, [0, 127], "linear")` swells the expression pedal smoothly over a whole note.

### 10.2 Tablature Bends
*   **Syntax:** `.bu(Target)` (Bend Up) and `.bd(Target)` (Bend Down). Targets accept absolute float steps (`"quarter"`, `"half"`, `"full"`).
*   **Execution:** The engine leverages the standard 14-bit MIDI Pitch Wheel (`0` to `16383`, centered at `8192`). A "full" bend maps to an exact upward ramp to `16383` across the duration of the note.

### 10.3 Micro-Timing & Percussion Rudiments
*   **Tremolo Rolls (`.roll(N)`):** The engine mathematically slices a single event into multiple rapid-fire MIDI strikes (e.g., `s:4.roll(3)` divides the snare hit into 8 distinct $32^{nd}$ notes).
*   **Ghost Notes (`.ghost`):** Applies an immediate $0.4\times$ scalar to the current sticky velocity for realistic drum grooves.

---

## 11. Output Routing & Execution (`main.rs`)

The `tenutoc` reference implementation utilizes a command-pattern orchestrator. It inspects the requested `--output` file extension to determine the required level of inference.

### 11.1 The MIDI Backend (`.mid`)
*   **Target:** Physical Audio Synthesizers / DAWs.
*   **Process:** Takes the `Timeline` IR, sorts all events chronologically, and converts absolute ticks into sequential "Delta Ticks."
*   **Resolution:** Operates at the natively declared `<divisions>`, defaulting to the high-resolution 1920 PPQ.

### 11.2 The MusicXML 4.0 Backend (`.xml`, `.musicxml`)
*   **Target:** Visual Engraving Software (MuseScore, Dorico, Sibelius).
*   **Process:** Requires the `spelling` and `rebar` engines. It bypasses heavy DOM manipulation, utilizing a high-speed, pre-allocated UTF-8 String Builder to stream the data.
*   **Features:**
    *   Automatically generates `<backup>` and `<forward>` tags to synchronize voices inside `<[ ]>` brackets.
    *   Reverse-engineers the tick durations to output accurate visual `<type>` (e.g., "quarter") and `<dot/>` tags.
    *   Automatically generates `<time-modification>` and bracket tags for Tuplets based on the IR state.

---

## 12. Error Taxonomy & Diagnostics (`tenutoc::TenutoError`)
*Implementation Status: 🟢 100% Complete*

Tenuto explicitly avoids vague compiler panics. Powered by the `ariadne` crate, it emits context-aware, syntax-highlighted terminal outputs pinpointing the exact byte-span of the failure. Errors are grouped into standardized codes for CI/CD tracking.

### 12.1 1000-Series: Lexical & Syntax Errors
*   **`E1001: Malformed Token`** - Unrecognized characters. (Includes trapping C-style `//` comments to remind users to use `%%`).
*   **`E1002: Syntax Error`** - Unbalanced delimiters (`{`, `[`, `@{`, `<[`).

### 12.2 2000-Series: Scope & Definition Errors
*   **`E2001: Undefined Identifier`** - Attempting to reference a `$variable` or `Staff ID` that was not declared in the `meta` or `def` blocks.
*   **`E2002: Duplicate Definition`** - Two macros or definitions sharing the same identifier.

### 12.3 3000-Series: Logical & Temporal Errors
*   **`E3002: Voice Sync Failure`** - Triggered exclusively in `--strict` mode. Indicates that the physical duration of `v1` does not perfectly match `v2` within a Polyphonic Voice Bracket.

### 12.4 4000-Series: Type & Value Errors
*   **`E4002: Invalid Type Cast`** - E.g., providing a String where an Integer duration is expected.
*   **`E4005: Tie Target Not Found`** - A structural failure in the `rebar` engine where a note requested a tie (`~`) but no valid forward-looking pitch existed to receive it.

### 12.5 5000-Series: Preprocessor Limits
*   **`E5002: Recursion Limit Exceeded`** - A `$macro` called itself iteratively, triggering the hardcapped depth limit of 64.

