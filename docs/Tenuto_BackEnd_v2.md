# Tenuto 2.0 Back-End Architecture Blueprint

## 1. Pipeline Orchestration (`main.rs` & `lib.rs`)

**Role:** Coordinates the compilation passes, handles file I/O, and enforces the standard error taxonomy.

### Required Upgrades for 100% Compliance:

1. **The Missing Pre-Processor Pass:** Between Phase 2 (Parsing) and Phase 3 (Linearization), you must insert a **Pre-Processor** phase. The AST must be mutated to expand `$macros`, substitute `$variables`, evaluate `if` conditions, and inline `import` statements (Spec 15 & 16) *before* the `ir::compile` engine sees it.
2. **Strict Error Taxonomy (Spec 24):** Expand `TenutoError` in `lib.rs` to explicitly match the spec's error codes.
* `E2001` (Undefined Identifier)
* `E3002` (Voice Sync Failure)
* `F9002` (Internal Panic)


3. **Diagnostic Reporting:** Integrate the `ariadne` crate (already in your `Cargo.toml`) to catch errors from Chumsky and the IR engine and print beautiful, rustc-style console graphics pointing precisely to the exact line and character of the failure.

---

## 2. The Inference Engine (`ir.rs`)

**Role:** The "Brain" of Tenuto. It consumes the stateless AST, resolves the "Sticky State" context, tracks global metadata, and flattens nested hierarchies into a linear timeline of absolute ticks.

### Required Upgrades for 100% Compliance:

#### A. The Cursor State (Spec 5.2, 6.3, 7.2)

Your `Cursor` tracks octave and duration perfectly. It must be expanded to track:

* `current_velocity` (Sticky Dynamics: `mf`, `ff`).
* `current_tuning` (For Tablature calculations).
* `active_techniques` (Sticky techniques like `.pizz` or `.pm`).
* *Crucial Fix:* The cursor state must persist across standard bar lines, but the Spec's **Strict Mode** (Section 22.2) forces these to reset at measure boundaries. The `Cursor` needs a `.reset_sticky()` method invoked based on the pipeline's strictness setting.

#### B. Gate Time vs. Logical Duration (Spec 7.3)

Currently, your `AtomicEvent` has `duration_ticks`. To support articulation, it needs `gate_ticks`.

* `duration_ticks`: How much time the event consumes on the mathematical grid (advances the cursor).
* `gate_ticks`: How long the physical MIDI note actually sounds.
* *Implementation:* A `:4` note is 960 ticks. If the AST has a `.stacc` attribute, `duration_ticks` remains 960, but `gate_ticks` becomes 480 (50%).

#### C. Polyphonic Synchronization (Spec 10.3)

Your parallel voice loop is excellent. However, the Spec explicitly demands a **Synchronization Check**. After processing `v1`, `v2`, etc., the IR engine MUST assert that the total `cursor.current_tick` for all voices in that group is identical. If they mismatch, it must throw `E3002: Voice Sync Failure`.

#### D. Engine Routing (Spec 8 & 9)

The IR engine must handle the missing Event variants:

* **Tablature (`AstEvent::Tab`):** Retrieve the track's `tuning` array from the `Track` definition. Apply the Inverse Rule (Spec 8.1): `Pitch = Tuning[String] + Fret`. Yield a standard `EventKind::Note`.
* **Percussion (`AstEvent::Percussion`):** Look up the `key` string in the track's `map` dictionary. Yield a standard `EventKind::Note` but specifically tagged for Channel 10.
* **Grace Notes:** Must be processed as 0 logical `duration_ticks` (does not advance the cursor) but a fixed `gate_ticks` value (e.g., 60 ticks), flagged to "steal" from the previous/next event's gate time during MIDI encoding.

#### E. Tie Resolution (Spec 6.6)

If an `AstEvent::Note` has `is_tied: true`, the IR engine should **not** emit a new `AtomicEvent`. Instead, it should look backward in the `Track`'s event list, find the previous note of the exact same pitch, and simply add the new duration to its existing `duration_ticks`.

---

## 3. The Synthesizer Protocol (`midi.rs`)

**Role:** Translates the abstract `AtomicEvent` linear timeline into hardware-executable binary MIDI instructions.

### Required Upgrades for 100% Compliance:

#### A. Microtonality & Cents (Spec 19.2)

If an `AtomicEvent` comes through with a microtonal `cents` deviation (e.g., +50 cents for a Quarter Sharp), you cannot just emit a `NoteOn`.

* *Implementation:* You must emit a `MidiMessage::PitchBend` message immediately *before* the `NoteOn`, and then a `PitchBend` reset (center) immediately *after* the `NoteOff`.

#### B. CC Automation (Spec 21)

The IR engine will eventually pass through continuous control data. The MIDI encoder must translate abstract IR parameters into specific MIDI CCs:

* `vol`  `MidiMessage::Controller { controller: 7, value }`
* `pan`  `MidiMessage::Controller { controller: 10, value }`

#### C. Percussion Channel Enforcement

Your current auto-assignment `let channel = (idx % 16) as u8;` is dangerous for MIDI.

* Channel 10 (index 9) is hardcoded in General MIDI for percussion.
* *Implementation:* If the track `style=grid`, hardcode the channel to 9. For all pitched tracks, allocate channels dynamically from 0-8 and 10-15.

#### D. Keyswitch Articulations (Spec 21.3)

If an event contains an attribute mapped to a keyswitch (e.g., `.arco` mapped to MIDI Note 24), the MIDI encoder must emit a rapid `NoteOn`  `NoteOff` for Note 24 with a delta of `0` immediately *before* emitting the actual performance note.

---

### The Architecture Summary

By structuring your compiler this way, you maintain a strict, beautiful separation of concerns:

1. **Front-End (`lexer`, `parser`):** "What did the user write?" (Stateless)
2. **Pre-Processor:** "What is the final expanded text?"
3. **IR Engine (`ir.rs`):** "What is the mathematical and acoustic reality of this text?" (Stateful Inference)
4. **Back-End (`midi.rs`):** "How do I make a synthesizer play this reality?"
