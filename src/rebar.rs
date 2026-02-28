//! # Tenuto Re-Barring Engine
//!
//! Converts the absolute, continuous `Timeline` IR into a discrete `VisualScore` 
//! organized by rigid measure boundaries. Handles note slicing (ties across barlines)
//! and automatic rest padding.

use std::collections::HashMap;
use crate::ir::{Timeline, AtomicEvent, TabArticulation};

// ============================================================================
// 1. VISUAL IR DATA STRUCTURES
// ============================================================================

/// The Root of the Visual Layout AST. Ready for MusicXML export or SVG rendering.
#[derive(Debug, Clone)]
pub struct VisualScore {
    pub title: String,
    pub staves: HashMap<String, VisualStaff>,
}

/// A single instrument staff mapped into rigid measures.
#[derive(Debug, Clone)]
pub struct VisualStaff {
    pub measures: Vec<VisualMeasure>,
}

/// A discrete box of time. Events inside MUST sum exactly to `end_tick - start_tick`.
#[derive(Debug, Clone)]
pub struct VisualMeasure {
    pub number: usize,
    pub time_signature: TimeSignature,
    pub start_tick: u64,
    pub end_tick: u64,
    pub events: Vec<VisualEvent>,
}

/// The visual representation of a musical event.
#[derive(Debug, Clone)]
pub struct VisualEvent {
    pub atomic: AtomicEvent,
    /// If true, draw a tie curving to the right ->
    pub tie_start: bool, 
    /// If true, draw a tie curving to the left <-
    pub tie_stop: bool,  
}

// ============================================================================
// 2. THE MEASURE GRID & TIME SIGNATURES
// ============================================================================

/// Mathematical representation of a meter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeSignature {
    pub numerator: u32,
    pub denominator: u32,
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self { numerator: 4, denominator: 4 }
    }
}

impl TimeSignature {
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(format!("E4002: Invalid time signature format: '{}'", s));
        }
        let num = parts[0].trim().parse().map_err(|_| format!("E4002: Invalid numerator: '{}'", parts[0]))?;
        let den = parts[1].trim().parse().map_err(|_| format!("E4002: Invalid denominator: '{}'", parts[1]))?;
        Ok(Self { numerator: num, denominator: den })
    }

    pub fn measure_capacity_ticks(&self, ppq: u32) -> u64 {
        (self.numerator as u64 * 4 * ppq as u64) / (self.denominator as u64)
    }
}

pub struct MeasureGrid {
    pub boundaries: Vec<(u64, u64, TimeSignature)>,
}

impl MeasureGrid {
    pub fn generate(
        initial_sig: TimeSignature,
        sig_changes: &HashMap<u64, TimeSignature>, 
        max_tick: u64,
        ppq: u32,
    ) -> Self {
        let mut boundaries = Vec::new();
        let mut current_tick = 0;
        let mut current_sig = initial_sig;
        let effective_max = std::cmp::max(max_tick, 1);

        while current_tick < effective_max {
            if let Some(&new_sig) = sig_changes.get(&current_tick) {
                current_sig = new_sig;
            }
            let capacity = current_sig.measure_capacity_ticks(ppq);
            let next_tick = current_tick + capacity;
            
            boundaries.push((current_tick, next_tick, current_sig));
            current_tick = next_tick;
        }

        Self { boundaries }
    }

    pub fn slice_event(&self, event: &AtomicEvent) -> Vec<(usize, VisualEvent)> {
        let mut slices = Vec::new();
        let mut remaining_duration = event.duration_ticks;
        let mut current_start_tick = event.tick;
        let mut is_first_slice = true;

        for (m_idx, &(_m_start, m_end, _)) in self.boundaries.iter().enumerate() {
            if m_end <= current_start_tick { continue; }
            if remaining_duration == 0 { break; }

            let available_in_measure = m_end - current_start_tick;
            let slice_duration = std::cmp::min(remaining_duration, available_in_measure);
            let is_last_slice = slice_duration == remaining_duration;

            let mut sliced_atomic = event.clone();
            sliced_atomic.tick = current_start_tick;
            sliced_atomic.duration_ticks = slice_duration;
            sliced_atomic.gate_ticks = ((event.gate_ticks as f64) 
                * (slice_duration as f64 / event.duration_ticks as f64)) as u64;

            let mut tie_stop = !is_first_slice;
            let mut tie_start = !is_last_slice;

            if matches!(event.kind, crate::ir::EventKind::Rest) {
                tie_stop = false;
                tie_start = false;
            }

            slices.push((m_idx, VisualEvent {
                atomic: sliced_atomic,
                tie_start,
                tie_stop,
            }));

            remaining_duration -= slice_duration;
            current_start_tick += slice_duration;
            is_first_slice = false;
        }

        slices
    }
}

// ============================================================================
// 3. THE VOID FILLER & STAFF BUILDER
// ============================================================================

impl VisualStaff {
    pub fn build(track: &crate::ir::Track, grid: &MeasureGrid) -> Self {
        let mut measures: Vec<VisualMeasure> = grid.boundaries.iter().enumerate().map(|(i, &(start, end, sig))| {
            VisualMeasure {
                number: i + 1,
                time_signature: sig,
                start_tick: start,
                end_tick: end,
                events: Vec::new(),
            }
        }).collect();

        for event in &track.events {
            // Grace notes do not consume logical time in measures.
            if event.is_grace { continue; }

            let slices = grid.slice_event(event);
            for (m_idx, vis_event) in slices {
                if m_idx < measures.len() {
                    measures[m_idx].events.push(vis_event);
                }
            }
        }

        for measure in &mut measures {
            measure.events.sort_by_key(|e| e.atomic.tick);

            let mut filled_events = Vec::new();
            let mut current_tick = measure.start_tick;

            for ev in &measure.events {
                if ev.atomic.tick > current_tick {
                    let gap_duration = ev.atomic.tick - current_tick;
                    filled_events.push(VisualEvent {
                        atomic: AtomicEvent {
                            tick: current_tick, duration_ticks: gap_duration, gate_ticks: gap_duration,
                            kind: crate::ir::EventKind::Rest,
                            tuplet_state: None, 
                            is_grace: false, 
                            is_ghost: false, 
                            tremolo_slashes: None,
                            cc_automations: vec![], // FIXED: Added missing CC field
                            tab_articulation: TabArticulation::None, // FIXED: Added missing Tab field
                        },
                        tie_start: false, tie_stop: false,
                    });
                }
                
                filled_events.push(ev.clone());
                current_tick = std::cmp::max(current_tick, ev.atomic.tick + ev.atomic.duration_ticks);
            }

            if current_tick < measure.end_tick {
                let gap_duration = measure.end_tick - current_tick;
                filled_events.push(VisualEvent {
                    atomic: AtomicEvent {
                        tick: current_tick, duration_ticks: gap_duration, gate_ticks: gap_duration,
                        kind: crate::ir::EventKind::Rest,
                        tuplet_state: None, 
                        is_grace: false, 
                        is_ghost: false, 
                        tremolo_slashes: None,
                        cc_automations: vec![], // FIXED: Added missing CC field
                        tab_articulation: TabArticulation::None, // FIXED: Added missing Tab field
                    },
                    tie_start: false, tie_stop: false,
                });
            }

            measure.events = filled_events;
        }

        Self { measures }
    }
}

// ============================================================================
// 4. THE VISUAL SCORE ORCHESTRATOR
// ============================================================================

impl VisualScore {
    pub fn build(timeline: &Timeline) -> Self {
        let max_tick = timeline.tracks.values()
            .flat_map(|t| t.events.iter())
            .map(|e| e.tick + e.duration_ticks)
            .max()
            .unwrap_or(0);

        let initial_sig = TimeSignature::default(); 
        let sig_changes = HashMap::new(); 

        let grid = MeasureGrid::generate(initial_sig, &sig_changes, max_tick, timeline.ppq);

        let mut staves = HashMap::new();
        for (staff_id, track) in &timeline.tracks {
            staves.insert(staff_id.clone(), VisualStaff::build(track, &grid));
        }

        Self {
            title: timeline.title.clone(),
            staves,
        }
    }
}

// ============================================================================
// UNIT TESTS 
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_signature_parsing() {
        let ts = TimeSignature::parse("3/4").unwrap();
        assert_eq!(ts.numerator, 3);
        assert_eq!(ts.denominator, 4);
    }

    #[test]
    fn test_measure_capacity_math() {
        let ts_4_4 = TimeSignature::parse("4/4").unwrap();
        assert_eq!(ts_4_4.measure_capacity_ticks(1920), 7680);
        let ts_3_4 = TimeSignature::parse("3/4").unwrap();
        assert_eq!(ts_3_4.measure_capacity_ticks(1920), 5760);
    }

    #[test]
    fn test_measure_grid_generation() {
        let ts_4_4 = TimeSignature::parse("4/4").unwrap();
        let grid = MeasureGrid::generate(ts_4_4, &HashMap::new(), 15000, 1920);
        assert_eq!(grid.boundaries[0], (0, 7680, ts_4_4));
        assert_eq!(grid.boundaries[1], (7680, 15360, ts_4_4));
    }

    #[test]
    fn test_guillotine_contained_note() {
        let ts = TimeSignature::parse("4/4").unwrap();
        let grid = MeasureGrid::generate(ts, &HashMap::new(), 7680, 1920);
        let event = AtomicEvent {
            tick: 0, duration_ticks: 1920, gate_ticks: 1920,
            kind: crate::ir::EventKind::Rest, 
            tuplet_state: None, 
            is_grace: false, 
            is_ghost: false, 
            tremolo_slashes: None,
            cc_automations: vec![],
            tab_articulation: TabArticulation::None,
        };
        let slices = grid.slice_event(&event);
        assert_eq!(slices.len(), 1);
        assert!(!slices[0].1.tie_start && !slices[0].1.tie_stop); 
    }

    #[test]
    fn test_guillotine_straddling_note() {
        let ts = TimeSignature::parse("4/4").unwrap();
        let grid = MeasureGrid::generate(ts, &HashMap::new(), 15360, 1920);
        let event = AtomicEvent {
            tick: 5760, duration_ticks: 3840, gate_ticks: 3840,
            kind: crate::ir::EventKind::Note { 
                pitch_midi: 60, cents: 0, velocity: 100, 
                spelling: crate::spelling::SpelledPitch::from_midi(60, 0, &crate::spelling::KeySignature::default()) 
            },
            tuplet_state: None, 
            is_grace: false, 
            is_ghost: false, 
            tremolo_slashes: None,
            cc_automations: vec![],
            tab_articulation: TabArticulation::None,
        };
        let slices = grid.slice_event(&event);
        assert_eq!(slices.len(), 2);
        assert_eq!(slices[0].1.atomic.duration_ticks, 1920);
        assert_eq!(slices[0].1.tie_start, true);
        assert_eq!(slices[1].1.atomic.tick, 7680);           
        assert_eq!(slices[1].1.tie_stop, true);              
    }

    #[test]
    fn test_void_filler_empty_measure() {
        let ts = TimeSignature::parse("4/4").unwrap();
        let grid = MeasureGrid::generate(ts, &HashMap::new(), 7680, 1920);
        let track = crate::ir::Track {
            label: "Vln".into(), patch: "violin".into(), tuning: vec![],
            keyswitches: HashMap::new(), perc_map: HashMap::new(), events: vec![],
            current_key: crate::spelling::KeySignature::default(),
            spelling_state: crate::spelling::MeasureSpellingState::new(crate::spelling::KeySignature::default()),
        };
        let visual_staff = VisualStaff::build(&track, &grid);
        assert_eq!(visual_staff.measures[0].events[0].atomic.duration_ticks, 7680); 
    }

    #[test]
    fn test_void_filler_partial_gaps() {
        let ts = TimeSignature::parse("4/4").unwrap();
        let grid = MeasureGrid::generate(ts, &HashMap::new(), 7680, 1920);
        let event = AtomicEvent {
            tick: 3840, duration_ticks: 1920, gate_ticks: 1920,
            kind: crate::ir::EventKind::Note { 
                pitch_midi: 60, cents: 0, velocity: 100, 
                spelling: crate::spelling::SpelledPitch::from_midi(60, 0, &crate::spelling::KeySignature::default()) 
            },
            tuplet_state: None, 
            is_grace: false, 
            is_ghost: false, 
            tremolo_slashes: None,
            cc_automations: vec![],
            tab_articulation: TabArticulation::None,
        };
        let track = crate::ir::Track {
            label: "Vln".into(), patch: "violin".into(), tuning: vec![],
            keyswitches: HashMap::new(), perc_map: HashMap::new(), events: vec![event],
            current_key: crate::spelling::KeySignature::default(),
            spelling_state: crate::spelling::MeasureSpellingState::new(crate::spelling::KeySignature::default()),
        };
        let visual_staff = VisualStaff::build(&track, &grid);
        let events = &visual_staff.measures[0].events;
        
        assert_eq!(events.len(), 3);
        assert!(matches!(events[0].atomic.kind, crate::ir::EventKind::Rest)); 
        assert!(matches!(events[1].atomic.kind, crate::ir::EventKind::Note{..})); 
        assert!(matches!(events[2].atomic.kind, crate::ir::EventKind::Rest)); 
    }

    #[test]
    fn test_visual_score_orchestrator() {
        let mut timeline = Timeline {
            title: "Rebar Test".into(), tempo: 120, ppq: 1920, tracks: HashMap::new(),
        };
        
        let track = crate::ir::Track {
            label: "Vln".into(), 
            patch: "violin".into(), 
            tuning: Vec::new(),
            keyswitches: HashMap::new(), 
            perc_map: HashMap::new(), 
            events: Vec::new(),
            current_key: crate::spelling::KeySignature::default(),
            spelling_state: crate::spelling::MeasureSpellingState::new(crate::spelling::KeySignature::default()),
        };
        
        timeline.tracks.insert("vln".into(), track);

        let v_score = VisualScore::build(&timeline);
        assert_eq!(v_score.title, "Rebar Test");
        assert!(v_score.staves.contains_key("vln"));
    }
}