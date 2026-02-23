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

#[derive(Debug, Clone)]
pub struct AtomicEvent {
    pub tick: u64,
    pub duration_ticks: u64,
    pub gate_ticks: u64,
    pub kind: EventKind,
    pub tuplet_state: Option<TupletState>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventKind {
    Note { 
        pitch_midi: u8, 
        cents: i32, 
        velocity: u8,
        spelling: SpelledPitch 
    },
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
// 2. THE STICKY STATE CURSOR (Internal)
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
            current_tick: start_tick,
            last_duration: Rational::new(1, 4),
            last_octave: 4,
            last_velocity: 100,
            time_scalar: Rational::new(1, 1),
            ppq,
            tied_pitches: Vec::new(),
            active_tuplet_ratio: None,
        }
    }

    fn parse_duration(&mut self, d_str: Option<&String>, dots: u8, multiplier: Option<u32>) -> (u64, u64) {
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

        let mut final_rat = Rational::new(
            base_rat.num * self.time_scalar.num,
            base_rat.den * self.time_scalar.den
        );

        if let Some(m) = multiplier { final_rat.num *= m as u64; }
        let ticks = final_rat.to_ticks(self.ppq);
        (ticks, ticks)
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
                c if c.is_ascii_digit() => {
                    octave = c.to_digit(10).unwrap() as u8;
                    has_octave = true;
                }
                _ => {}
            }
        }
        if has_octave { self.last_octave = octave; }
        (octave + 1) * 12 + base
    }
}

// ============================================================================
// 3. THE COMPILER PIPELINE (Marked PUB)
// ============================================================================

pub fn compile(score: Score, strict_mode: bool) -> Result<Timeline, String> {
    let ppq = 1920;
    let mut timeline = Timeline {
        title: "Untitled".into(), tempo: 120, ppq, tracks: HashMap::new(),
    };

    fn build_context(items: &[TopLevel], timeline: &mut Timeline, current_key: &mut KeySignature) {
        for item in items {
            match item {
                TopLevel::Meta(kvs) => {
                    if let Some(Value::Str(t)) = kvs.get("title") { timeline.title = t.clone(); }
                    if let Some(Value::Num(t)) = kvs.get("tempo") { timeline.tempo = *t as u32; }
                    if let Some(Value::Str(k)) = kvs.get("key") { 
                        if let Ok(parsed) = KeySignature::parse(k) { *current_key = parsed; }
                    }
                },
                TopLevel::Def { id, label, attributes } => {
                    let patch = attributes.get("patch").and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None }).unwrap_or("piano".into());

                    let mut keyswitches = HashMap::new();
                    if let Some(Value::Map(ks_map)) = attributes.get("keyswitch") {
                        for (k, v) in ks_map {
                            if let Value::Num(n) = v { keyswitches.insert(k.clone(), *n as u8); }
                        }
                    }

                    let mut perc_map = HashMap::new();
                    if let Some(Value::Map(pm)) = attributes.get("map") {
                        for (k, v) in pm {
                            if let Value::Array(arr) = v {
                                if arr.len() > 1 {
                                    if let Value::Num(midi) = arr[1] { perc_map.insert(k.clone(), midi as u8); }
                                }
                            }
                        }
                    }

                    timeline.tracks.insert(id.clone(), Track {
                        label: label.clone().unwrap_or_else(|| id.clone()),
                        patch,
                        tuning: vec![40, 45, 50, 55, 59, 64], 
                        keyswitches,
                        perc_map,
                        events: Vec::new(),
                        current_key: current_key.clone(),
                        spelling_state: MeasureSpellingState::new(current_key.clone()),
                    });
                },
                TopLevel::Group { items: inner_items, .. } => build_context(inner_items, timeline, current_key),
                _ => {}
            }
        }
    }

    let mut initial_key = KeySignature::default();
    build_context(&score.items, &mut timeline, &mut initial_key);

    let mut current_measure_start: u64 = 0;
    let mut active_cursors: HashMap<String, Cursor> = HashMap::new();

    fn process_logic_stream(
        items: Vec<TopLevel>, 
        timeline: &mut Timeline, 
        cursors: &mut HashMap<String, Cursor>, 
        measure_start: &mut u64, 
        ppq: u32, 
        strict: bool
    ) -> Result<(), String> {
        for item in items {
            match item {
                TopLevel::Measure { content, .. } => {
                    *measure_start += measure_pass(content, timeline, cursors, *measure_start, ppq, strict)?;
                    
                    for track in timeline.tracks.values_mut() {
                        track.spelling_state.reset_at_barline();
                    }
                },
                TopLevel::Group { items: inner_items, .. } => {
                    process_logic_stream(inner_items, timeline, cursors, measure_start, ppq, strict)?;
                },
                _ => {}
            }
        }
        Ok(())
    }

    process_logic_stream(score.items, &mut timeline, &mut active_cursors, &mut current_measure_start, ppq, strict_mode)?;

    for track in timeline.tracks.values_mut() { track.events.sort_by_key(|e| e.tick); }
    Ok(timeline)
}

fn measure_pass(content: Vec<Logic>, timeline: &mut Timeline, active_cursors: &mut HashMap<String, Cursor>, start_tick: u64, ppq: u32, strict_mode: bool) -> Result<u64, String> {
    let mut max_measure_duration: u64 = 0;

    for logic in content {
        if let Logic::Assignment { staff_id, voices } = logic {
            let track = timeline.tracks.get_mut(&staff_id).ok_or_else(|| format!("E2001: Undefined staff '{}'", staff_id))?;
            let mut voice_end_ticks = Vec::new();

            for (idx, voice) in voices.iter().enumerate() {
                let mut cursor = if idx == 0 {
                    let mut c = active_cursors.remove(&staff_id).unwrap_or_else(|| Cursor::new(start_tick, ppq));
                    if strict_mode {
                        c.last_duration = Rational::new(1, 4);
                        c.last_octave = 4;
                    }
                    c.current_tick = start_tick;
                    c
                } else {
                    Cursor::new(start_tick, ppq)
                };

                process_voice_events(voice, &mut cursor, track)?;
                let duration_consumed = cursor.current_tick - start_tick;
                voice_end_ticks.push(duration_consumed);
                max_measure_duration = max_measure_duration.max(duration_consumed);

                if idx == 0 {
                    active_cursors.insert(staff_id.clone(), cursor);
                }
            }

            if strict_mode && voice_end_ticks.len() > 1 && !voice_end_ticks.iter().all(|&d| d == voice_end_ticks[0]) {
                return Err(format!("E3002: Voice Sync Failure in polyphonic block for staff '{}'", staff_id));
            }
        }
    }
    Ok(max_measure_duration)
}

fn emit_keyswitches(attributes: &[Attribute], track: &mut Track, tick: u64) {
    for attr in attributes {
        if let Some(&ks_midi) = track.keyswitches.get(&attr.name) {
            let spelling = SpelledPitch::from_midi(ks_midi, 0, &track.current_key);
            track.events.push(AtomicEvent {
                tick, duration_ticks: 1, gate_ticks: 1,
                kind: EventKind::Note { pitch_midi: ks_midi, cents: 0, velocity: 1, spelling },
                tuplet_state: None,
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
                let (log, mut gate) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier);

                let midi = if let Some(&mapped) = track.perc_map.get(pitch) {
                    mapped
                } else {
                    cursor.parse_pitch(pitch)
                };

                let mut spelling = SpelledPitch::from_string(pitch, cursor.last_octave as i8)
                    .unwrap_or_else(|_| SpelledPitch::from_midi(midi, cents.unwrap_or(0), &track.current_key));
                
                spelling = track.spelling_state.process_pitch(spelling);

                for attr in attributes { if attr.name == "stacc" { gate /= 2; } }
                emit_keyswitches(attributes, track, cursor.current_tick);

                if cursor.tied_pitches.contains(&midi) {
                    handle_tie(track, midi, log)?;
                    cursor.tied_pitches.retain(|&p| p != midi); 
                    if *is_tied { cursor.tied_pitches.push(midi); }
                } else {
                    track.events.push(AtomicEvent {
                        tick: cursor.current_tick, duration_ticks: log, gate_ticks: gate,
                        kind: EventKind::Note { pitch_midi: midi, cents: cents.unwrap_or(0), velocity: cursor.last_velocity, spelling },
                        tuplet_state: t_state,
                    });
                    if *is_tied { cursor.tied_pitches.push(midi); }
                }
                cursor.current_tick += log;
            },
            Event::Chord { notes, duration, dots, multiplier, is_tied: _is_tied, attributes: _attributes } => {
                let (log, gate) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier);
                for note_event in notes {
                    if let Event::Note { pitch, cents, attributes: note_attrs, .. } = note_event {
                        let midi = cursor.parse_pitch(pitch);
                        let mut local_gate = gate;

                        let mut spelling = SpelledPitch::from_string(pitch, cursor.last_octave as i8)
                            .unwrap_or_else(|_| SpelledPitch::from_midi(midi, cents.unwrap_or(0), &track.current_key));
                        spelling = track.spelling_state.process_pitch(spelling);

                        for attr in note_attrs { if attr.name == "stacc" { local_gate /= 2; } }
                        emit_keyswitches(note_attrs, track, cursor.current_tick);

                        track.events.push(AtomicEvent {
                            tick: cursor.current_tick, duration_ticks: log, gate_ticks: local_gate,
                            kind: EventKind::Note { pitch_midi: midi, cents: cents.unwrap_or(0), velocity: cursor.last_velocity, spelling },
                            tuplet_state: t_state.clone(),
                        });
                    }
                }
                cursor.current_tick += log;
            },
            Event::Percussion { key, duration, dots, multiplier, attributes } => {
                let (log, mut gate) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier);

                let midi = track.perc_map.get(key).copied().unwrap_or(60);

                let mut spelling = SpelledPitch::from_midi(midi, 0, &track.current_key);
                spelling = track.spelling_state.process_pitch(spelling);

                for attr in attributes { if attr.name == "stacc" { gate /= 2; } }
                emit_keyswitches(attributes, track, cursor.current_tick);

                track.events.push(AtomicEvent {
                    tick: cursor.current_tick, duration_ticks: log, gate_ticks: gate,
                    kind: EventKind::Note { pitch_midi: midi, cents: 0, velocity: cursor.last_velocity, spelling },
                    tuplet_state: t_state,
                });
                cursor.current_tick += log;
            },
            Event::Tab { fret, string, duration, dots, multiplier, .. } => {
                let (log, gate) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier);

                let str_idx = track.tuning.len().saturating_sub(*string as usize);
                let base_pitch = track.tuning.get(str_idx).cloned().unwrap_or(60);
                let midi = base_pitch + fret.parse::<u8>().unwrap_or(0);

                let mut spelling = SpelledPitch::from_midi(midi, 0, &track.current_key);
                spelling = track.spelling_state.process_pitch(spelling);

                track.events.push(AtomicEvent {
                    tick: cursor.current_tick, duration_ticks: log, gate_ticks: gate,
                    kind: EventKind::Note { pitch_midi: midi, cents: 0, velocity: cursor.last_velocity, spelling },
                    tuplet_state: t_state,
                });
                cursor.current_tick += log;
            },
            Event::Frequency { hz, duration, dots, multiplier, .. } => {
                let (log, gate) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier);
                let freq: f64 = hz.parse().unwrap_or(440.0);
                track.events.push(AtomicEvent {
                    tick: cursor.current_tick, duration_ticks: log, gate_ticks: gate,
                    kind: EventKind::Frequency { hz: freq, velocity: cursor.last_velocity },
                    tuplet_state: t_state,
                });
                cursor.current_tick += log;
            },
            Event::Rest { duration, dots, multiplier } => {
                let (log, _) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier);
                track.events.push(AtomicEvent {
                    tick: cursor.current_tick, duration_ticks: log, gate_ticks: log,
                    kind: EventKind::Rest,
                    tuplet_state: t_state,
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
                    if let Some(ts) = &mut track.events[start_idx].tuplet_state {
                        ts.is_start = true;
                    }
                    let last_idx = track.events.len() - 1;
                    if let Some(ts) = &mut track.events[last_idx].tuplet_state {
                        ts.is_stop = true;
                    }
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