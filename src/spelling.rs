//! # Tenuto Spelling & Accidental Engine
//! 
//! This module resolves absolute MIDI integers into context-aware graphical pitches
//! suitable for SVG layout and MusicXML export. It implements the standard rules of 
//! Western music notation (Gould's *Behind Bars*), including the Line of Fifths, 
//! measure-local accidental memory, and strict octave isolation.

use std::fmt;
use std::collections::HashMap;

// ============================================================================
// 1. CORE TYPES & DATA STRUCTURES
// ============================================================================

/// Represents the seven diatonic steps of the Western musical alphabet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Step {
    C, D, E, F, G, A, B
}

impl Step {
    /// Maps the step to a 0-indexed integer (C = 0, B = 6) for rapid array lookups.
    #[inline(always)]
    pub fn to_index(self) -> usize {
        match self {
            Step::C => 0, Step::D => 1, Step::E => 2, Step::F => 3,
            Step::G => 4, Step::A => 5, Step::B => 6,
        }
    }

    /// Inverse mapping from array index back to Step.
    pub fn from_index(idx: usize) -> Self {
        match idx % 7 {
            0 => Step::C, 1 => Step::D, 2 => Step::E, 3 => Step::F,
            4 => Step::G, 5 => Step::A, 6 => Step::B,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Step::C => 'C', Step::D => 'D', Step::E => 'E', Step::F => 'F',
            Step::G => 'G', Step::A => 'A', Step::B => 'B',
        };
        write!(f, "{}", c)
    }
}

/// Dictates the graphical rendering state of the accidental.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccidentalDisplay {
    /// No symbol is drawn (implied by key signature or measure memory).
    Implicit, 
    /// The symbol MUST be drawn (overrides key signature or previous state).
    Explicit, 
    /// Drawn in parentheses (editorial or cautionary reminder).
    Courtesy, 
}

/// The fully resolved, context-aware pitch ready for visual layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SpelledPitch {
    pub step: Step,
    /// Chromatic alteration: -2 (bb), -1 (b), 0 (natural), 1 (#), 2 (x)
    pub alter: i8,
    /// Microtonal fractional alteration (e.g., 0.5 for Quarter-Sharp)
    pub micro_alter: f32,
    /// Standard SPN Octave (Middle C = 4)
    pub octave: i8,
    /// The layout engine instruction for drawing the ink
    pub display: AccidentalDisplay,
}

// ============================================================================
// 2. KEY SIGNATURE MATHEMATICS
// ============================================================================

/// A highly optimized mathematical representation of the Circle of Fifths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeySignature {
    /// The raw number of sharps (positive) or flats (negative). Range: -7 to +7.
    pub sharps_or_flats: i8,
    /// An O(1) lookup table storing the active alteration (-1, 0, or 1) for each Step (C to B).
    alterations: [i8; 7],
}

impl Default for KeySignature {
    /// Defaults to C Major / A Minor (0 sharps/flats).
    fn default() -> Self {
        Self { sharps_or_flats: 0, alterations: [0; 7] }
    }
}

impl KeySignature {
    /// Parses a standard Key string (e.g., "D", "F#m", "Eb Major") into a mathematical state.
    pub fn parse(key_str: &str) -> Result<Self, String> {
        let normalized = key_str.replace(" ", "").to_lowercase();
        
        // Lookup table mapping normalized string to (sharps_or_flats)
        let sf = match normalized.as_str() {
            // Major Keys
            "cb" | "cbmajor" | "cbmaj" => -7,
            "gb" | "gbmajor" | "gbmaj" => -6,
            "db" | "dbmajor" | "dbmaj" => -5,
            "ab" | "abmajor" | "abmaj" => -4,
            "eb" | "ebmajor" | "ebmaj" => -3,
            "bb" | "bbmajor" | "bbmaj" => -2,
            "f"  | "fmajor"  | "fmaj"  => -1,
            "c"  | "cmajor"  | "cmaj"  => 0,
            "g"  | "gmajor"  | "gmaj"  => 1,
            "d"  | "dmajor"  | "dmaj"  => 2,
            "a"  | "amajor"  | "amaj"  => 3,
            "e"  | "emajor"  | "emaj"  => 4,
            "b"  | "bmajor"  | "bmaj"  => 5,
            "f#" | "f#major" | "f#maj" => 6,
            "c#" | "c#major" | "c#maj" => 7,

            // Minor Keys
            "abm" | "abminor" | "abmin" => -7,
            "ebm" | "ebminor" | "ebmin" => -6,
            "bbm" | "bbminor" | "bbmin" => -5,
            "fm"  | "fminor"  | "fmin"  => -4,
            "cm"  | "cminor"  | "cmin"  => -3,
            "gm"  | "gminor"  | "gmin"  => -2,
            "dm"  | "dminor"  | "dmin"  => -1,
            "am"  | "aminor"  | "amin"  => 0,
            "em"  | "eminor"  | "emin"  => 1,
            "bm"  | "bminor"  | "bmin"  => 2,
            "f#m" | "f#minor" | "f#min" => 3,
            "c#m" | "c#minor" | "c#min" => 4,
            "g#m" | "g#minor" | "g#min" => 5,
            "d#m" | "d#minor" | "d#min" => 6,
            "a#m" | "a#minor" | "a#min" => 7,
            
            _ => return Err(format!("E4003: Invalid Key Signature String: '{}'", key_str)),
        };

        Ok(Self::from_sharps_flats(sf))
    }

    /// Constructs the O(1) alteration array strictly following the Order of Sharps/Flats.
    pub fn from_sharps_flats(sf: i8) -> Self {
        let mut alterations = [0; 7];
        
        if sf > 0 {
            // Order of Sharps: F, C, G, D, A, E, B
            let order = [Step::F, Step::C, Step::G, Step::D, Step::A, Step::E, Step::B];
            for i in 0..(sf as usize) {
                alterations[order[i].to_index()] = 1;
            }
        } else if sf < 0 {
            // Order of Flats: B, E, A, D, G, C, F
            let order = [Step::B, Step::E, Step::A, Step::D, Step::G, Step::C, Step::F];
            for i in 0..(-sf as usize) {
                alterations[order[i].to_index()] = -1;
            }
        }

        Self { sharps_or_flats: sf, alterations }
    }

    /// Returns the global alteration for a given step (e.g., F in D Major returns 1).
    #[inline(always)]
    pub fn alteration_for(&self, step: Step) -> i8 {
        self.alterations[step.to_index()]
    }
}

// ============================================================================
// 3. THE ALGORITHMIC SPELLER (MIDI → PITCH)
// ============================================================================

impl SpelledPitch {
    /// Derives the musically correct spelling for a raw MIDI integer based on the Key Signature.
    /// Used for `style=tab` and `style=grid` where absolute frequency must be translated to visual space.
    pub fn from_midi(midi_pitch: u8, cents: i32, key: &KeySignature) -> Self {
        let pc = (midi_pitch % 12) as i8;
        
        // Step 1: Diatonic Matcher
        let base_pcs = [0, 2, 4, 5, 7, 9, 11]; // MIDI pitch classes for C, D, E, F, G, A, B
        let mut diatonic_match = None;

        for step_idx in 0..7 {
            let step = Step::from_index(step_idx);
            let alt = key.alteration_for(step);
            let expected_pc = (base_pcs[step_idx] + alt).rem_euclid(12);
            
            if expected_pc == pc {
                diatonic_match = Some((step, alt));
                break;
            }
        }

        // Step 2: Chromatic Fallback (Line of Fifths proximity)
        let (step, alter) = if let Some((s, a)) = diatonic_match {
            (s, a)
        } else {
            let prefer_sharps = key.sharps_or_flats >= 0;
            
            // Lookup tables mapping Pitch Class (0-11) to (Step, Alteration)
            let sharp_spellings = [
                (Step::C, 0), (Step::C, 1), (Step::D, 0), (Step::D, 1), 
                (Step::E, 0), (Step::F, 0), (Step::F, 1), (Step::G, 0), 
                (Step::G, 1), (Step::A, 0), (Step::A, 1), (Step::B, 0)
            ];
            let flat_spellings = [
                (Step::C, 0), (Step::D, -1), (Step::D, 0), (Step::E, -1), 
                (Step::E, 0), (Step::F, 0), (Step::G, -1), (Step::G, 0), 
                (Step::A, -1), (Step::A, 0), (Step::B, -1), (Step::B, 0)
            ];

            if prefer_sharps {
                sharp_spellings[pc as usize]
            } else {
                flat_spellings[pc as usize]
            }
        };

        // Step 3: Octave Calculation
        let base_pc = base_pcs[step.to_index()];
        let raw_octave = (midi_pitch as i8 / 12) - 1;
        
        let mut correct_octave = raw_octave;
        let mut raw_natural_midi = (correct_octave + 1) * 12 + base_pc;
        
        while (midi_pitch as i8) < raw_natural_midi + alter {
            correct_octave -= 1;
            raw_natural_midi -= 12;
        }
        while (midi_pitch as i8) > raw_natural_midi + alter {
            correct_octave += 1;
            raw_natural_midi += 12;
        }

        // Step 4: Microtonal Resolution
        let micro_alter = cents as f32 / 100.0;

        Self {
            step,
            alter,
            micro_alter,
            octave: correct_octave,
            display: AccidentalDisplay::Implicit, // Will be overridden by the State Machine
        }
    }

    /// Preserves the exact visual spelling written by the composer in `style=standard`.
    pub fn from_string(pitch_str: &str, fallback_octave: i8) -> Result<Self, String> {
        let chars: Vec<char> = pitch_str.to_ascii_lowercase().chars().collect();
        if chars.is_empty() {
            return Err("E4002: Empty pitch string".into());
        }

        let step = match chars[0] {
            'c' => Step::C, 'd' => Step::D, 'e' => Step::E, 'f' => Step::F,
            'g' => Step::G, 'a' => Step::A, 'b' => Step::B,
            _ => return Err(format!("E4002: Invalid step char '{}'", chars[0])),
        };

        let mut alter = 0;
        let mut micro_alter = 0.0;
        let mut i = 1;

        // Parse accidental modifiers
        while i < chars.len() && !chars[i].is_ascii_digit() {
            match chars[i] {
                '#' => alter += 1,
                'b' => alter -= 1,
                'x' => alter += 2,
                'n' => alter = 0, // Explicit natural
                'q' => {
                    if i + 1 < chars.len() && chars[i+1] == 's' { micro_alter = 0.5; i += 1; }
                    else if i + 1 < chars.len() && chars[i+1] == 'f' { micro_alter = -0.5; i += 1; }
                },
                't' => {
                    if i + 2 < chars.len() && chars[i+1] == 'q' && chars[i+2] == 's' { micro_alter = 1.5; i += 2; }
                    else if i + 2 < chars.len() && chars[i+1] == 'q' && chars[i+2] == 'f' { micro_alter = -1.5; i += 2; }
                },
                _ => {}
            }
            i += 1;
        }

        // Parse explicit octave if provided, otherwise use the inferred IR fallback
        let octave = if i < chars.len() {
            chars[i..].iter().collect::<String>().parse::<i8>().unwrap_or(fallback_octave)
        } else {
            fallback_octave
        };

        Ok(Self {
            step,
            alter,
            micro_alter,
            octave,
            display: AccidentalDisplay::Implicit, // Will be overridden by the State Machine
        })
    }
}

// ============================================================================
// 4. THE ACCIDENTAL STATE MACHINE (GOULD'S RULES)
// ============================================================================

/// Tracks the active accidentals within a single visual measure.
/// Enforces strict octave isolation and resets at every barline.
#[derive(Debug, Clone)]
pub struct MeasureSpellingState {
    key_signature: KeySignature,
    /// Maps (Step, Octave) to the currently active (alteration, micro_alteration).
    active_accidentals: HashMap<(Step, i8), (i8, f32)>,
}

impl MeasureSpellingState {
    /// Initializes a new measure state bounded by a specific Key Signature.
    pub fn new(key_signature: KeySignature) -> Self {
        Self {
            key_signature,
            active_accidentals: HashMap::new(),
        }
    }

    /// Evaluates a SpelledPitch, determines its correct graphical display (Implicit vs Explicit), 
    /// and updates the measure's internal memory.
    pub fn process_pitch(&mut self, mut pitch: SpelledPitch) -> SpelledPitch {
        let state_key = (pitch.step, pitch.octave);
        
        let current_active = self.active_accidentals
            .get(&state_key)
            .copied()
            .unwrap_or_else(|| (self.key_signature.alteration_for(pitch.step), 0.0));

        let note_accidental = (pitch.alter, pitch.micro_alter);

        if note_accidental == current_active {
            pitch.display = AccidentalDisplay::Implicit;
        } else {
            pitch.display = AccidentalDisplay::Explicit;
            self.active_accidentals.insert(state_key, note_accidental);
        }

        pitch
    }

    /// Clears the state, simulating a barline reset. Retains the active key signature.
    pub fn reset_at_barline(&mut self) {
        self.active_accidentals.clear();
    }

    /// Changes the active key signature (e.g., at a mid-piece key change).
    pub fn set_key_signature(&mut self, new_key: KeySignature) {
        self.key_signature = new_key;
        self.active_accidentals.clear();
    }
}

// ============================================================================
// UNIT TESTS 
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_indexing() {
        assert_eq!(Step::C.to_index(), 0);
        assert_eq!(Step::B.to_index(), 6);
        assert_eq!(Step::from_index(3), Step::F);
    }

    #[test]
    fn test_key_signature_parsing() {
        let d_maj = KeySignature::parse("D Major").unwrap();
        assert_eq!(d_maj.sharps_or_flats, 2);
        assert_eq!(d_maj.alteration_for(Step::F), 1);
        assert_eq!(d_maj.alteration_for(Step::C), 1);
        assert_eq!(d_maj.alteration_for(Step::G), 0); // Natural

        let eb_min = KeySignature::parse("ebm").unwrap();
        assert_eq!(eb_min.sharps_or_flats, -6);
        assert_eq!(eb_min.alteration_for(Step::B), -1);
        assert_eq!(eb_min.alteration_for(Step::C), -1);
        assert_eq!(eb_min.alteration_for(Step::F), 0); // Natural
        
        let c_maj = KeySignature::parse("C").unwrap();
        assert_eq!(c_maj.sharps_or_flats, 0);
        assert_eq!(c_maj.alteration_for(Step::B), 0);
    }

    #[test]
    fn test_key_signature_invalid() {
        assert!(KeySignature::parse("H Major").is_err());
    }

    #[test]
    fn test_algorithmic_speller_diatonic() {
        let c_maj = KeySignature::parse("C").unwrap();
        let p1 = SpelledPitch::from_midi(65, 0, &c_maj);
        assert_eq!(p1.step, Step::F);
        assert_eq!(p1.alter, 0);
        
        let g_maj = KeySignature::parse("G").unwrap();
        let p2 = SpelledPitch::from_midi(66, 0, &g_maj);
        assert_eq!(p2.step, Step::F);
        assert_eq!(p2.alter, 1);
    }

    #[test]
    fn test_algorithmic_speller_chromatic_fallback() {
        let d_maj = KeySignature::parse("D").unwrap(); // Prefers sharps
        let p1 = SpelledPitch::from_midi(63, 0, &d_maj);
        assert_eq!(p1.step, Step::D);
        assert_eq!(p1.alter, 1);

        let f_maj = KeySignature::parse("F").unwrap(); // Prefers flats
        let p2 = SpelledPitch::from_midi(61, 0, &f_maj);
        assert_eq!(p2.step, Step::D);
        assert_eq!(p2.alter, -1);
    }

    #[test]
    fn test_octave_boundaries() {
        let c_maj = KeySignature::parse("C").unwrap();
        let p1 = SpelledPitch::from_midi(71, 0, &c_maj);
        assert_eq!(p1.step, Step::B);
        assert_eq!(p1.octave, 4);

        let p2 = SpelledPitch::from_midi(72, 0, &c_maj);
        assert_eq!(p2.step, Step::C);
        assert_eq!(p2.octave, 5);
    }

    #[test]
    fn test_explicit_string_parsing() {
        let p1 = SpelledPitch::from_string("f#5", 4).unwrap();
        assert_eq!(p1.step, Step::F);
        assert_eq!(p1.alter, 1);
        assert_eq!(p1.octave, 5);

        let p2 = SpelledPitch::from_string("dbqs", 3).unwrap(); // Fallback octave 3
        assert_eq!(p2.step, Step::D);
        assert_eq!(p2.alter, -1);
        assert_eq!(p2.micro_alter, 0.5);
        assert_eq!(p2.octave, 3);
    }

    #[test]
    fn test_state_machine_implicit_vs_explicit() {
        let d_maj = KeySignature::parse("D").unwrap();
        let mut state = MeasureSpellingState::new(d_maj);

        let n1 = SpelledPitch { step: Step::F, alter: 1, micro_alter: 0.0, octave: 4, display: AccidentalDisplay::Implicit };
        let out1 = state.process_pitch(n1);
        assert_eq!(out1.display, AccidentalDisplay::Implicit);

        let n2 = SpelledPitch { step: Step::F, alter: 0, micro_alter: 0.0, octave: 4, display: AccidentalDisplay::Implicit };
        let out2 = state.process_pitch(n2);
        assert_eq!(out2.display, AccidentalDisplay::Explicit);

        let n3 = SpelledPitch { step: Step::F, alter: 0, micro_alter: 0.0, octave: 4, display: AccidentalDisplay::Implicit };
        let out3 = state.process_pitch(n3);
        assert_eq!(out3.display, AccidentalDisplay::Implicit);
    }

    #[test]
    fn test_state_machine_octave_isolation() {
        let c_maj = KeySignature::parse("C").unwrap();
        let mut state = MeasureSpellingState::new(c_maj);

        let n1 = SpelledPitch { step: Step::F, alter: 1, micro_alter: 0.0, octave: 4, display: AccidentalDisplay::Implicit };
        assert_eq!(state.process_pitch(n1).display, AccidentalDisplay::Explicit);

        let n2 = SpelledPitch { step: Step::F, alter: 1, micro_alter: 0.0, octave: 5, display: AccidentalDisplay::Implicit };
        assert_eq!(state.process_pitch(n2).display, AccidentalDisplay::Explicit);
    }

    #[test]
    fn test_state_machine_barline_reset() {
        let c_maj = KeySignature::parse("C").unwrap();
        let mut state = MeasureSpellingState::new(c_maj);

        let n1 = SpelledPitch { step: Step::C, alter: 1, micro_alter: 0.0, octave: 4, display: AccidentalDisplay::Implicit };
        assert_eq!(state.process_pitch(n1.clone()).display, AccidentalDisplay::Explicit);
        
        assert_eq!(state.process_pitch(n1.clone()).display, AccidentalDisplay::Implicit);

        state.reset_at_barline();

        assert_eq!(state.process_pitch(n1).display, AccidentalDisplay::Explicit);
    }
}