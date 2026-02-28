use std::collections::HashMap;
use crate::ast::*;
use crate::spelling::{SpelledPitch, KeySignature, MeasureSpellingState};

// ============================================================================
// 1. DATA STRUCTURES & MATH (All marked PUB)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TupletState {
    pub actual_notes: u64,
    pub normal_notes: u64,
    pub is_start: bool,
    pub is_stop: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CCAutomation {
    pub controller: u8,
    pub start_val: u8,
    pub end_val: u8,
    pub curve: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TabArticulation {
    #[default]
    None,
    BendUp(f32),
    BendDown(f32),
    Slide,
}

#[derive(Debug, Clone)]
pub struct AtomicEvent {
    pub tick: u64,
    pub duration_ticks: u64,
    pub gate_ticks: u64,
    pub kind: EventKind,
    pub tuplet_state: Option<TupletState>,
    pub is_grace: bool,
    pub is_ghost: bool,
    pub tremolo_slashes: Option<u8>,
    pub cc_automations: Vec<CCAutomation>,
    pub tab_articulation: TabArticulation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventKind {
    Note { pitch_midi: u8, cents: i32, velocity: u8, spelling: SpelledPitch },
    Frequency { hz: f64, velocity: u8 },
    Rest,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub label: String,
    pub patch: String,
    pub tuning: Vec<u8>,
    pub keyswitches: HashMap<String, u8>,
    pub perc_map: HashMap<String, u8>,
    pub events: Vec<AtomicEvent>,
    pub current_key: KeySignature,
    pub spelling_state: MeasureSpellingState,
}

#[derive(Debug, Clone)]
pub struct Timeline {
    pub title: String,
    pub tempo: u32,
    pub ppq: u32,
    pub tracks: HashMap<String, Track>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational {
    pub num: u64,
    pub den: u64,
}

impl Rational {
    pub fn new(num: u64, den: u64) -> Self {
        if den == 0 { panic!("F9002: Division by Zero in Time Engine"); }
        let common = gcd(num, den);
        Self { num: num / common, den: den / common }
    }
    pub fn to_ticks(&self, ppq: u32) -> u64 {
        (self.num * 4 * ppq as u64) / self.den
    }
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 { a %= b; std::mem::swap(&mut a, &mut b); }
    a
}

// ============================================================================
// 2. THE STICKY STATE CURSOR & PARSERS
// ============================================================================

#[derive(Debug, Clone)]
struct Cursor {
    current_tick: u64,
    last_duration: Rational,
    last_octave: u8,
    last_velocity: u8,
    time_scalar: Rational,
    ppq: u32,
    tied_pitches: Vec<u8>,
    active_tuplet_ratio: Option<(u64, u64)>,
}

impl Cursor {
    fn new(start_tick: u64, ppq: u32) -> Self {
        Self {
            current_tick: start_tick, last_duration: Rational::new(1, 4),
            last_octave: 4, last_velocity: 80, time_scalar: Rational::new(1, 1),
            ppq, tied_pitches: Vec::new(), active_tuplet_ratio: None,
        }
    }

    fn parse_duration(&mut self, d_str: Option<&String>, dots: u8, multiplier: Option<u32>) -> (u64, u64, bool) {
        if let Some(s) = d_str {
            if s.starts_with(":grace") {
                let gate = self.ppq / 4;
                return (0, gate as u64, true);
            }
        }

        let base_rat = if let Some(s) = d_str {
            let denominator: u64 = s.trim_start_matches(':').parse().unwrap_or(4);
            let mut rat = Rational::new(1, denominator);
            if dots == 1 { rat = Rational::new(3, denominator * 2); }
            else if dots == 2 { rat = Rational::new(7, denominator * 4); }
            self.last_duration = rat; 
            rat
        } else {
            self.last_duration
        };

        let mut final_rat = Rational::new(base_rat.num * self.time_scalar.num, base_rat.den * self.time_scalar.den);
        if let Some(m) = multiplier { final_rat.num *= m as u64; }
        let ticks = final_rat.to_ticks(self.ppq);
        (ticks, ticks, false)
    }

    fn parse_pitch(&mut self, p_str: &str) -> u8 {
        let chars: Vec<char> = p_str.chars().collect();
        if chars.is_empty() { return 60; }
        let mut base = match chars[0].to_ascii_lowercase() {
            'c' => 0, 'd' => 2, 'e' => 4, 'f' => 5, 'g' => 7, 'a' => 9, 'b' => 11, _ => 0
        };
        let mut octave = self.last_octave;
        let mut has_octave = false;

        for i in 1..chars.len() {
            match chars[i] {
                '#' => base += 1, 'b' => base -= 1, 'x' => base += 2,
                c if c.is_ascii_digit() => { octave = c.to_digit(10).unwrap() as u8; has_octave = true; }
                _ => {}
            }
        }
        if has_octave { self.last_octave = octave; }
        (octave + 1) * 12 + base
    }

    fn parse_dynamics(&mut self, attributes: &[Attribute]) {
        for attr in attributes {
            match attr.name.as_str() {
                "pppp" => self.last_velocity = 16, "ppp"  => self.last_velocity = 32,
                "pp"   => self.last_velocity = 48, "p"    => self.last_velocity = 64,
                "mp"   => self.last_velocity = 72, "mf"   => self.last_velocity = 80,
                "f"    => self.last_velocity = 96, "ff"   => self.last_velocity = 112,
                "fff"  => self.last_velocity = 120, "ffff" => self.last_velocity = 127,
                _ => {} 
            }
        }
    }
}

struct ParsedAttributes {
    gate: u64, velocity: u8, is_ghost: bool, tremolo_slashes: Option<u8>,
    cc_automations: Vec<CCAutomation>, tab_articulation: TabArticulation,
}

fn apply_attributes(attributes: &[Attribute], mut gate: u64, mut velocity: u8) -> ParsedAttributes {
    let mut is_ghost = false;
    let mut tremolo_slashes = None;
    let mut cc_automations = Vec::new();
    let mut tab_articulation = TabArticulation::None;

    let parse_bend = |arg: Option<&Value>| -> f32 {
        match arg {
            Some(Value::Id(s)) | Some(Value::Str(s)) => match s.as_str() {
                "quarter" => 0.25, "half" => 0.5, "full" => 1.0, _ => 1.0,
            },
            Some(Value::Num(n)) => *n as f32, Some(Value::Float(f)) => *f as f32, _ => 1.0,
        }
    };

    for attr in attributes {
        match attr.name.as_str() {
            "stacc" => gate /= 2,
            "ghost" => { is_ghost = true; velocity = (velocity as f32 * 0.4) as u8; },
            "roll" => tremolo_slashes = Some(attr.args.first().and_then(|v| if let Value::Num(num) = v { Some(*num as u8) } else { None }).unwrap_or(3)),
            "bu" => tab_articulation = TabArticulation::BendUp(parse_bend(attr.args.first())),
            "bd" => tab_articulation = TabArticulation::BendDown(parse_bend(attr.args.first())),
            "sl" => tab_articulation = TabArticulation::Slide,
            "cc" => {
                if attr.args.len() >= 2 {
                    if let Value::Num(ctrl) = attr.args[0] {
                        let mut start_val = 0; let mut end_val = 0; let mut curve = "linear".to_string();
                        match &attr.args[1] {
                            Value::Num(v) => { start_val = *v as u8; end_val = *v as u8; },
                            Value::Array(arr) => {
                                if arr.len() >= 2 {
                                    if let Value::Num(s) = arr[0] { start_val = s as u8; }
                                    if let Value::Num(e) = arr[1] { end_val = e as u8; }
                                }
                            },
                            _ => {}
                        }
                        if attr.args.len() == 3 {
                            if let Value::Str(c) | Value::Id(c) = &attr.args[2] { curve = c.clone(); }
                        }
                        cc_automations.push(CCAutomation { controller: ctrl as u8, start_val, end_val, curve });
                    }
                }
            },
            _ => {}
        }
    }
    ParsedAttributes { gate, velocity, is_ghost, tremolo_slashes, cc_automations, tab_articulation }
}

// ============================================================================
// 3. THE COMPILER PIPELINE (Marked PUB)
// ============================================================================

pub fn compile(score: Score, strict_mode: bool) -> Result<Timeline, String> {
    let ppq = 1920;
    let mut timeline = Timeline { title: "Untitled".into(), tempo: 120, ppq, tracks: HashMap::new() };

    let mut initial_ts_ticks = (ppq * 4) as u64; // Default 4/4

    fn build_context(items: &[TopLevel], timeline: &mut Timeline, current_key: &mut KeySignature, initial_ts: &mut u64, ppq: u32) {
        for item in items {
            match item {
                TopLevel::Meta(kvs) => {
                    if let Some(Value::Str(t)) = kvs.get("title") { timeline.title = t.clone(); }
                    if let Some(Value::Num(t)) = kvs.get("tempo") { timeline.tempo = *t as u32; }
                    if let Some(Value::Str(k)) = kvs.get("key") { if let Ok(parsed) = KeySignature::parse(k) { *current_key = parsed; } }
                    
                    // FIXED: Read Global Time Signature!
                    if let Some(Value::Str(s)) = kvs.get("time") {
                        let parts: Vec<&str> = s.split('/').collect();
                        if parts.len() == 2 {
                            if let (Ok(num), Ok(den)) = (parts[0].trim().parse::<u64>(), parts[1].trim().parse::<u64>()) {
                                *initial_ts = (num * 4 * ppq as u64) / den;
                            }
                        }
                    }
                },
                TopLevel::Def { id, label, attributes } => {
                    let patch = attributes.get("patch").and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None }).unwrap_or("piano".into());
                    let mut keyswitches = HashMap::new();
                    if let Some(Value::Map(ks_map)) = attributes.get("keyswitch") {
                        for (k, v) in ks_map { if let Value::Num(n) = v { keyswitches.insert(k.clone(), *n as u8); } }
                    }
                    let mut perc_map = HashMap::new();
                    if let Some(Value::Map(pm)) = attributes.get("map") {
                        for (k, v) in pm {
                            if let Value::Array(arr) = v {
                                if arr.len() > 1 { if let Value::Num(midi) = arr[1] { perc_map.insert(k.clone(), midi as u8); } }
                            }
                        }
                    }
                    timeline.tracks.insert(id.clone(), Track {
                        label: label.clone().unwrap_or_else(|| id.clone()), patch, tuning: vec![40, 45, 50, 55, 59, 64], 
                        keyswitches, perc_map, events: Vec::new(), current_key: current_key.clone(), spelling_state: MeasureSpellingState::new(current_key.clone()),
                    });
                },
                TopLevel::Group { items: inner_items, .. } => build_context(inner_items, timeline, current_key, initial_ts, ppq),
                _ => {}
            }
        }
    }

    let mut initial_key = KeySignature::default();
    build_context(&score.items, &mut timeline, &mut initial_key, &mut initial_ts_ticks, ppq);

    let mut active_cursors: HashMap<String, Cursor> = HashMap::new();
    let mut measure_starts: HashMap<u64, u64> = HashMap::new();

    fn process_logic_stream(
        items: Vec<TopLevel>, timeline: &mut Timeline, cursors: &mut HashMap<String, Cursor>, ppq: u32, strict: bool, 
        mut current_ts_ticks: u64, measure_starts: &mut HashMap<u64, u64>, next_unassigned_index: &mut u64, next_unassigned_tick: &mut u64,
    ) -> Result<(), String> {
        for item in items {
            match item {
                TopLevel::Meta(kvs) => {
                    if let Some(Value::Str(s)) = kvs.get("time") {
                        let parts: Vec<&str> = s.split('/').collect();
                        if parts.len() == 2 {
                            if let (Ok(num), Ok(den)) = (parts[0].trim().parse::<u64>(), parts[1].trim().parse::<u64>()) {
                                current_ts_ticks = (num * 4 * ppq as u64) / den;
                            }
                        }
                    }
                },
                TopLevel::Measure { range, content, attributes } => {
                    let mut local_ts_ticks = current_ts_ticks;
                    if let Some(Value::Str(s)) = attributes.get("time") {
                        let parts: Vec<&str> = s.split('/').collect();
                        if parts.len() == 2 {
                            if let (Ok(num), Ok(den)) = (parts[0].trim().parse::<u64>(), parts[1].trim().parse::<u64>()) {
                                local_ts_ticks = (num * 4 * ppq as u64) / den;
                            }
                        }
                    }

                    // FIXED: Dynamic Absolute Time Grid
                    let m_idx = match range {
                        MeasureRange::Single(idx) => std::cmp::max(idx, 1) as u64,
                        MeasureRange::Implicit | _ => *next_unassigned_index,
                    };

                    let absolute_start_tick = if let Some(&t) = measure_starts.get(&m_idx) {
                        t // Additive merge: snap to previously evaluated time for this measure!
                    } else {
                        let t = *next_unassigned_tick;
                        measure_starts.insert(m_idx, t);
                        t
                    };

                    // Advance the global trackers
                    if m_idx >= *next_unassigned_index {
                        *next_unassigned_index = m_idx + 1;
                        *next_unassigned_tick = absolute_start_tick + local_ts_ticks;
                    }

                    measure_pass(content, timeline, cursors, absolute_start_tick, ppq, strict)?;
                    
                    for track in timeline.tracks.values_mut() { track.spelling_state.reset_at_barline(); }
                },
                TopLevel::Group { items: inner_items, .. } => { 
                    process_logic_stream(inner_items, timeline, cursors, ppq, strict, current_ts_ticks, measure_starts, next_unassigned_index, next_unassigned_tick)?; 
                },
                _ => {}
            }
        }
        Ok(())
    }

    let mut next_unassigned_index = 1;
    let mut next_unassigned_tick = 0;
    process_logic_stream(score.items, &mut timeline, &mut active_cursors, ppq, strict_mode, initial_ts_ticks, &mut measure_starts, &mut next_unassigned_index, &mut next_unassigned_tick)?;
    
    for track in timeline.tracks.values_mut() { track.events.sort_by_key(|e| e.tick); }
    Ok(timeline)
}

fn measure_pass(content: Vec<Logic>, timeline: &mut Timeline, active_cursors: &mut HashMap<String, Cursor>, start_tick: u64, ppq: u32, strict_mode: bool) -> Result<(), String> {
    for logic in content {
        if let Logic::Assignment { staff_id, voices } = logic {
            let track = timeline.tracks.get_mut(&staff_id).ok_or_else(|| format!("E2001: Undefined staff '{}'", staff_id))?;
            let mut voice_end_ticks = Vec::new();

            for (idx, voice) in voices.iter().enumerate() {
                let mut cursor = if idx == 0 {
                    let mut c = active_cursors.remove(&staff_id).unwrap_or_else(|| Cursor::new(start_tick, ppq));
                    c.current_tick = start_tick; // Snap to absolute grid
                    if strict_mode { c.last_duration = Rational::new(1, 4); c.last_octave = 4; }
                    c
                } else { Cursor::new(start_tick, ppq) };

                process_voice_events(voice, &mut cursor, track)?;
                voice_end_ticks.push(cursor.current_tick - start_tick);
                if idx == 0 { active_cursors.insert(staff_id.clone(), cursor); }
            }

            if strict_mode && voice_end_ticks.len() > 1 && !voice_end_ticks.iter().all(|&d| d == voice_end_ticks[0]) {
                return Err(format!("E3002: Voice Sync Failure in polyphonic block for staff '{}'", staff_id));
            }
        }
    }
    Ok(())
}

fn emit_keyswitches(attributes: &[Attribute], track: &mut Track, tick: u64) {
    for attr in attributes {
        if let Some(&ks_midi) = track.keyswitches.get(&attr.name) {
            let spelling = SpelledPitch::from_midi(ks_midi, 0, &track.current_key);
            track.events.push(AtomicEvent {
                tick, duration_ticks: 1, gate_ticks: 1,
                kind: EventKind::Note { pitch_midi: ks_midi, cents: 0, velocity: 1, spelling },
                tuplet_state: None, is_grace: false, is_ghost: false, tremolo_slashes: None,
                cc_automations: vec![], tab_articulation: TabArticulation::None,
            });
        }
    }
}

fn process_voice_events(voice: &Voice, cursor: &mut Cursor, track: &mut Track) -> Result<(), String> {
    for event in &voice.events {
        let t_state = cursor.active_tuplet_ratio.map(|(act, norm)| TupletState {
            actual_notes: act, normal_notes: norm, is_start: false, is_stop: false
        });

        match event {
            Event::Note { pitch, cents, duration, dots, multiplier, is_tied, attributes } => {
                let (log, gate_ticks, is_grace) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier);
                cursor.parse_dynamics(attributes); 
                let p_attrs = apply_attributes(attributes, gate_ticks, cursor.last_velocity);

                let midi = if let Some(&mapped) = track.perc_map.get(pitch) { mapped } else { cursor.parse_pitch(pitch) };
                let mut spelling = SpelledPitch::from_string(pitch, cursor.last_octave as i8).unwrap_or_else(|_| SpelledPitch::from_midi(midi, cents.unwrap_or(0), &track.current_key));
                spelling = track.spelling_state.process_pitch(spelling);

                emit_keyswitches(attributes, track, cursor.current_tick);

                if cursor.tied_pitches.contains(&midi) && !is_grace {
                    handle_tie(track, midi, log)?;
                    cursor.tied_pitches.retain(|&p| p != midi); 
                    if *is_tied { cursor.tied_pitches.push(midi); }
                } else {
                    track.events.push(AtomicEvent {
                        tick: cursor.current_tick, duration_ticks: log, gate_ticks: p_attrs.gate,
                        kind: EventKind::Note { pitch_midi: midi, cents: cents.unwrap_or(0), velocity: p_attrs.velocity, spelling },
                        tuplet_state: t_state, is_grace, is_ghost: p_attrs.is_ghost, tremolo_slashes: p_attrs.tremolo_slashes,
                        cc_automations: p_attrs.cc_automations, tab_articulation: p_attrs.tab_articulation,
                    });
                    if *is_tied && !is_grace { cursor.tied_pitches.push(midi); }
                }
                cursor.current_tick += log;
            },
            Event::Chord { notes, duration, dots, multiplier, is_tied: _is_tied, attributes: _attributes } => {
                let (log, gate_ticks, is_grace) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier);
                cursor.parse_dynamics(_attributes); 

                for note_event in notes {
                    if let Event::Note { pitch, cents, attributes: note_attrs, .. } = note_event {
                        let p_attrs = apply_attributes(note_attrs, gate_ticks, cursor.last_velocity);
                        let midi = cursor.parse_pitch(pitch);
                        let mut spelling = SpelledPitch::from_string(pitch, cursor.last_octave as i8).unwrap_or_else(|_| SpelledPitch::from_midi(midi, cents.unwrap_or(0), &track.current_key));
                        spelling = track.spelling_state.process_pitch(spelling);

                        emit_keyswitches(note_attrs, track, cursor.current_tick);

                        track.events.push(AtomicEvent {
                            tick: cursor.current_tick, duration_ticks: log, gate_ticks: p_attrs.gate,
                            kind: EventKind::Note { pitch_midi: midi, cents: cents.unwrap_or(0), velocity: p_attrs.velocity, spelling },
                            tuplet_state: t_state.clone(), is_grace, is_ghost: p_attrs.is_ghost, tremolo_slashes: p_attrs.tremolo_slashes,
                            cc_automations: p_attrs.cc_automations, tab_articulation: p_attrs.tab_articulation,
                        });
                    }
                }
                cursor.current_tick += log;
            },
            Event::Percussion { key, duration, dots, multiplier, attributes } => {
                let (log, gate_ticks, is_grace) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier);
                cursor.parse_dynamics(attributes); 
                let p_attrs = apply_attributes(attributes, gate_ticks, cursor.last_velocity);

                let midi = track.perc_map.get(key).copied().unwrap_or(60);
                let mut spelling = SpelledPitch::from_midi(midi, 0, &track.current_key);
                spelling = track.spelling_state.process_pitch(spelling);

                emit_keyswitches(attributes, track, cursor.current_tick);

                track.events.push(AtomicEvent {
                    tick: cursor.current_tick, duration_ticks: log, gate_ticks: p_attrs.gate,
                    kind: EventKind::Note { pitch_midi: midi, cents: 0, velocity: p_attrs.velocity, spelling },
                    tuplet_state: t_state, is_grace, is_ghost: p_attrs.is_ghost, tremolo_slashes: p_attrs.tremolo_slashes,
                    cc_automations: p_attrs.cc_automations, tab_articulation: p_attrs.tab_articulation,
                });
                cursor.current_tick += log;
            },
            Event::Tab { fret, string, duration, dots, multiplier, attributes } => {
                let (log, gate_ticks, is_grace) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier);
                cursor.parse_dynamics(attributes);
                let p_attrs = apply_attributes(attributes, gate_ticks, cursor.last_velocity);

                let str_idx = track.tuning.len().saturating_sub(*string as usize);
                let base_pitch = track.tuning.get(str_idx).cloned().unwrap_or(60);
                let midi = base_pitch + fret.parse::<u8>().unwrap_or(0);

                let mut spelling = SpelledPitch::from_midi(midi, 0, &track.current_key);
                spelling = track.spelling_state.process_pitch(spelling);

                track.events.push(AtomicEvent {
                    tick: cursor.current_tick, duration_ticks: log, gate_ticks: p_attrs.gate,
                    kind: EventKind::Note { pitch_midi: midi, cents: 0, velocity: p_attrs.velocity, spelling },
                    tuplet_state: t_state, is_grace, is_ghost: p_attrs.is_ghost, tremolo_slashes: p_attrs.tremolo_slashes,
                    cc_automations: p_attrs.cc_automations, tab_articulation: p_attrs.tab_articulation,
                });
                cursor.current_tick += log;
            },
            Event::Frequency { hz, duration, dots, multiplier, attributes } => {
                let (log, gate_ticks, is_grace) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier);
                cursor.parse_dynamics(attributes);
                let p_attrs = apply_attributes(attributes, gate_ticks, cursor.last_velocity);

                let freq: f64 = hz.parse().unwrap_or(440.0);
                track.events.push(AtomicEvent {
                    tick: cursor.current_tick, duration_ticks: log, gate_ticks: p_attrs.gate,
                    kind: EventKind::Frequency { hz: freq, velocity: p_attrs.velocity },
                    tuplet_state: t_state, is_grace, is_ghost: p_attrs.is_ghost, tremolo_slashes: p_attrs.tremolo_slashes,
                    cc_automations: p_attrs.cc_automations, tab_articulation: p_attrs.tab_articulation,
                });
                cursor.current_tick += log;
            },
            Event::Rest { duration, dots, multiplier } => {
                let (log, _, is_grace) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier);
                track.events.push(AtomicEvent {
                    tick: cursor.current_tick, duration_ticks: log, gate_ticks: log,
                    kind: EventKind::Rest, tuplet_state: t_state, is_grace, is_ghost: false, tremolo_slashes: None,
                    cc_automations: vec![], tab_articulation: TabArticulation::None,
                });
                cursor.current_tick += log;
            },
            Event::Tuplet { content, p, q } => {
                let old_scalar = cursor.time_scalar;
                cursor.time_scalar = Rational::new(old_scalar.num * *q, old_scalar.den * *p);
                cursor.active_tuplet_ratio = Some((*p, *q));
                
                let start_idx = track.events.len();
                process_voice_events(content, cursor, track)?;
                
                if start_idx < track.events.len() {
                    if let Some(ts) = &mut track.events[start_idx].tuplet_state { ts.is_start = true; }
                    let last_idx = track.events.len() - 1;
                    if let Some(ts) = &mut track.events[last_idx].tuplet_state { ts.is_stop = true; }
                }

                cursor.time_scalar = old_scalar;
                cursor.active_tuplet_ratio = None;
            },
            _ => {}
        }
    }
    Ok(())
}

fn handle_tie(track: &mut Track, target: u8, extra: u64) -> Result<(), String> {
    if let Some(ev) = track.events.iter_mut().rev().find(|e| matches!(e.kind, EventKind::Note { pitch_midi, .. } if pitch_midi == target)) {
        ev.duration_ticks += extra;
        ev.gate_ticks += extra;
        Ok(())
    } else {
        Err(format!("E4005: Tie target not found for pitch {}", target))
    }
}