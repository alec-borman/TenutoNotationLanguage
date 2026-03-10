//! # Tenuto Intermediate Representation (IR) Compiler
//! 
//! Translates the parsed AST into an Absolute Mathematical Timeline.
//! 
//! **V3.0.0 Updates:**
//! - Slice 1: Euclidean rhythms `(k):3/8`.
//! - Slice 2: Action Notation (`s`) and Control Lanes (`pedal:`).
//! - Slice 3: Generative Ergonomics (Auto-Padding & Relative Pitch).
//! - Slice 4: Physical Time Domain (`.push(15ms)`, `.pull()`, `.flam`, `.drag`).
//! - Slice 5: Groove & Humanization.
//! - Slice 6: Concrete Sampling (`.slice(N)`).
//! - Slice 7: Synth Physics (Choke Groups, Glide, Accelerate, ADSR).
//! - MASTER SLICE: Graph Unrolling (Repeats), Demarcation (`print` flag), Lyric Engine.

use std::collections::HashMap;
use crate::ast::*;
use crate::spelling::{SpelledPitch, KeySignature, MeasureSpellingState};

// ============================================================================
// 1. DATA STRUCTURES & MATH
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TupletState {
    pub actual_notes: u64,
    pub normal_notes: u64,
    pub is_start: bool,
    pub is_stop: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CCAutomation { pub controller: u8, pub start_val: u8, pub end_val: u8, pub curve: String }

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TabArticulation { #[default] None, BendUp(f32), BendDown(f32), Slide }

#[derive(Debug, Clone, PartialEq)]
pub struct ConcreteParams { pub sample_start: i64, pub sample_end: i64, pub stretch: bool, pub reverse: bool }

// --- V3.0 MASTER SLICE: Lyric Engine Components ---
#[derive(Debug, Clone, PartialEq, Default)]
pub enum LyricExtension {
    #[default] None, Hyphen, Melisma,
}

#[derive(Debug, Clone)]
pub struct AtomicEvent {
    pub tick: u64,
    pub duration_ticks: u64,
    pub gate_ticks: u64,
    pub physical_tick_offset: i64, 
    pub kind: EventKind,
    pub tuplet_state: Option<TupletState>,
    pub is_grace: bool,
    pub is_ghost: bool,
    pub tremolo_slashes: Option<u8>,
    pub cc_automations: Vec<CCAutomation>,
    pub tab_articulation: TabArticulation,
    pub synth_glide_start_midi: Option<u8>,
    pub synth_glide_ticks: Option<u64>,
    pub synth_accelerate_semitones: Option<f32>,
    pub lyric: Option<String>,
    pub lyric_extension: LyricExtension,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventKind {
    Note { pitch_midi: u8, cents: i32, velocity: u8, spelling: SpelledPitch },
    Frequency { hz: f64, velocity: u8 },
    Rest, Space, 
    Concrete { key: String, params: ConcreteParams }, 
}

#[derive(Debug, Clone)]
pub struct Track {
    pub label: String,
    pub patch: String,
    pub style: String,
    pub print: bool, // SLICE 9: XML Demarcation Flag
    pub tuning: Vec<u8>,
    pub keyswitches: HashMap<String, u8>,
    pub perc_map: HashMap<String, u8>,
    pub concrete_map: HashMap<String, (i64, i64)>, 
    pub env: HashMap<String, i64>,
    pub cut_group: Option<u32>,
    pub events: Vec<AtomicEvent>,
    pub current_key: KeySignature,
    pub spelling_state: MeasureSpellingState,
}

#[derive(Debug, Clone)]
pub struct Timeline {
    pub title: String,
    pub tempo: u32,
    pub ppq: u32,
    pub auto_pad_voices: bool,
    pub swing: HashMap<u32, f64>, 
    pub humanize: f64,            
    pub tracks: HashMap<String, Track>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational { pub num: u64, pub den: u64 }

impl Rational {
    pub fn new(num: u64, den: u64) -> Result<Self, String> {
        if den == 0 { return Err("E3003: Tuplet Ratio Error - Division by Zero".into()); }
        let common = gcd(num, den); Ok(Self { num: num / common, den: den / common })
    }
    pub fn to_ticks(&self, ppq: u32) -> u64 { (self.num * 4 * ppq as u64) / self.den }
}

fn gcd(mut a: u64, mut b: u64) -> u64 { while b != 0 { a %= b; std::mem::swap(&mut a, &mut b); } a }

// ============================================================================
// 2. THE STICKY STATE CURSOR & PARSERS
// ============================================================================

#[derive(Debug, Clone)]
struct Cursor {
    current_tick: u64, last_duration: Rational, last_octave: u8, last_pc: i8, last_pitch_midi: Option<u8>, 
    last_velocity: u8, time_scalar: Rational, ppq: u32, tied_pitches: Vec<u8>, active_tuplet_ratio: Option<(u64, u64)>,
}

impl Cursor {
    fn new(start_tick: u64, ppq: u32) -> Self {
        Self { 
            current_tick: start_tick, last_duration: Rational::new(1, 4).unwrap(), last_octave: 4, 
            last_pc: 0, last_pitch_midi: None, last_velocity: 80, time_scalar: Rational::new(1, 1).unwrap(), 
            ppq, tied_pitches: Vec::new(), active_tuplet_ratio: None 
        }
    }
    
    fn parse_duration(&mut self, d_str: Option<&String>, dots: u8, multiplier: Option<u32>) -> Result<(u64, u64, bool), String> {
        if let Some(s) = d_str { if s.starts_with(":grace") { let gate = self.ppq / 4; return Ok((0, gate as u64, true)); } }
        let base_rat = if let Some(s) = d_str {
            let denominator: u64 = s.trim_start_matches(':').parse().unwrap_or(4);
            let mut rat = Rational::new(1, denominator)?;
            if dots == 1 { rat = Rational::new(3, denominator * 2)?; } else if dots == 2 { rat = Rational::new(7, denominator * 4)?; }
            self.last_duration = rat; rat
        } else { self.last_duration };
        let mut final_rat = Rational::new(base_rat.num * self.time_scalar.num, base_rat.den * self.time_scalar.den)?;
        if let Some(m) = multiplier { final_rat.num *= m as u64; }
        let ticks = final_rat.to_ticks(self.ppq); Ok((ticks, ticks, false))
    }
    
    fn parse_pitch(&mut self, p_str: &str, style: &str) -> u8 {
        let chars: Vec<char> = p_str.chars().collect(); if chars.is_empty() { return 60; }
        let mut base: i8 = match chars[0].to_ascii_lowercase() { 'c' => 0, 'd' => 2, 'e' => 4, 'f' => 5, 'g' => 7, 'a' => 9, 'b' => 11, _ => 0 };
        let mut octave = self.last_octave as i8; let mut has_octave = false; let mut i = 1;
        while i < chars.len() {
            match chars[i] {
                '#' => base += 1, 'b' => base -= 1, 'x' => base += 2, '+' | '-' => break, 
                c if c.is_ascii_digit() && !has_octave => { 
                    let mut o_str = String::new(); while i < chars.len() && chars[i].is_ascii_digit() { o_str.push(chars[i]); i += 1; }
                    if let Ok(o) = o_str.parse::<i8>() { octave = o; has_octave = true; } continue; 
                }
                _ => {}
            }
            i += 1;
        }
        if style == "relative" && !has_octave {
            let pc = base.rem_euclid(12); let prev_pc = self.last_pc.rem_euclid(12); let delta = pc - prev_pc;
            if delta <= -6 { octave += 1; } else if delta > 6 { octave -= 1; }
        }
        self.last_octave = octave as u8; self.last_pc = base;
        let midi = ((octave as i32 + 1) * 12 + base as i32).clamp(0, 127) as u8; self.last_pitch_midi = Some(midi); midi
    }
    
    fn parse_dynamics(&mut self, attributes: &[Attribute]) {
        for attr in attributes {
            match attr.name.as_str() {
                "pppp" => self.last_velocity = 16, "ppp"  => self.last_velocity = 32, "pp"   => self.last_velocity = 48, "p"    => self.last_velocity = 64,
                "mp"   => self.last_velocity = 72, "mf"   => self.last_velocity = 80, "f"    => self.last_velocity = 96, "ff"   => self.last_velocity = 112,
                "fff"  => self.last_velocity = 120, "ffff" => self.last_velocity = 127, _ => {} 
            }
        }
    }
}

fn parse_time_val(arg: Option<&Value>, tempo: u32, ppq: u32) -> i64 {
    if let Some(Value::TimeVal(t)) = arg {
        let s = t.to_lowercase();
        if s.ends_with("ms") { return (s.trim_end_matches("ms").parse::<f64>().unwrap_or(0.0) * tempo as f64 * ppq as f64 / 60000.0).round() as i64; }
        else if s.ends_with("ticks") { return s.trim_end_matches("ticks").parse::<f64>().unwrap_or(0.0).round() as i64; }
        else if s.ends_with("s") { return (s.trim_end_matches("s").parse::<f64>().unwrap_or(0.0) * 1000.0 * tempo as f64 * ppq as f64 / 60000.0).round() as i64; }
    } 0
}

fn parse_time_val_to_ms(val: &Value) -> i64 {
    if let Value::TimeVal(t) = val {
        let s = t.to_lowercase();
        if s.ends_with("ms") { return s.trim_end_matches("ms").parse::<f64>().unwrap_or(0.0).round() as i64; }
        else if s.ends_with("s") { return (s.trim_end_matches("s").parse::<f64>().unwrap_or(0.0) * 1000.0).round() as i64; }
    } 0
}

struct ParsedAttributes {
    gate: u64, velocity: u8, is_ghost: bool, tremolo_slashes: Option<u8>, cc_automations: Vec<CCAutomation>, tab_articulation: TabArticulation,
    physical_tick_offset: i64, flam: bool, drag: bool, slice: Option<u32>, stretch: bool, reverse: bool, glide_ticks: Option<u64>, accelerate: Option<f32>,
}

fn apply_attributes(attributes: &[Attribute], mut gate: u64, mut velocity: u8, tempo: u32, ppq: u32) -> ParsedAttributes {
    let mut is_ghost = false; let mut tremolo_slashes = None; let mut cc_automations = Vec::new(); let mut tab_articulation = TabArticulation::None;
    let mut physical_tick_offset = 0; let mut flam = false; let mut drag = false; let mut slice = None; let mut stretch = false; let mut reverse = false;
    let mut glide_ticks = None; let mut accelerate = None;
    let parse_bend = |arg: Option<&Value>| -> f32 { match arg { Some(Value::Id(s)) | Some(Value::Str(s)) => match s.as_str() { "quarter" => 0.25, "half" => 0.5, "full" => 1.0, _ => 1.0, }, Some(Value::Num(n)) => *n as f32, Some(Value::Float(f)) => *f as f32, _ => 1.0, } };
    for attr in attributes {
        match attr.name.as_str() {
            "stacc" => gate /= 2, "ghost" => { is_ghost = true; velocity = (velocity as f32 * 0.4) as u8; }, "roll" => tremolo_slashes = Some(attr.args.first().and_then(|v| if let Value::Num(num) = v { Some(*num as u8) } else { None }).unwrap_or(3)),
            "bu" => tab_articulation = TabArticulation::BendUp(parse_bend(attr.args.first())), "bd" => tab_articulation = TabArticulation::BendDown(parse_bend(attr.args.first())), "sl" => tab_articulation = TabArticulation::Slide,
            "push" => physical_tick_offset -= parse_time_val(attr.args.first(), tempo, ppq), "pull" => physical_tick_offset += parse_time_val(attr.args.first(), tempo, ppq),
            "flam" => flam = true, "drag" => drag = true, "slice" => { if let Some(Value::Num(n)) = attr.args.first() { slice = Some(*n as u32); } }, "stretch" => stretch = true, "reverse" => reverse = true,
            "glide" => { let ticks = parse_time_val(attr.args.first(), tempo, ppq); if ticks > 0 { glide_ticks = Some(ticks as u64); } },
            "accelerate" => { if let Some(val) = attr.args.first() { accelerate = match val { Value::Num(n) => Some(*n as f32), Value::Float(f) => Some(*f as f32), _ => None, }; } },
            "cc" => {
                if attr.args.len() >= 2 {
                    if let Value::Num(ctrl) = attr.args[0] {
                        let mut start_val = 0; let mut end_val = 0; let mut curve = "linear".to_string();
                        match &attr.args[1] { Value::Num(v) => { start_val = *v as u8; end_val = *v as u8; }, Value::Array(arr) => { if arr.len() >= 2 { if let Value::Num(s) = arr[0] { start_val = s as u8; } if let Value::Num(e) = arr[1] { end_val = e as u8; } } }, _ => {} }
                        if attr.args.len() == 3 { if let Value::Str(c) | Value::Id(c) = &attr.args[2] { curve = c.clone(); } }
                        cc_automations.push(CCAutomation { controller: ctrl as u8, start_val, end_val, curve });
                    }
                }
            },
            _ => {}
        }
    }
    ParsedAttributes { gate, velocity, is_ghost, tremolo_slashes, cc_automations, tab_articulation, physical_tick_offset, flam, drag, slice, stretch, reverse, glide_ticks, accelerate }
}

// ============================================================================
// 3. THE COMPILER PIPELINE
// ============================================================================

pub fn compile(score: Score, strict_mode: bool) -> Result<Timeline, String> {
    let ppq = 1920;
    let mut timeline = Timeline { title: "Untitled".into(), tempo: 120, ppq, auto_pad_voices: false, swing: HashMap::new(), humanize: 0.0, tracks: HashMap::new() };
    let mut initial_ts_ticks = (ppq * 4) as u64; 

    fn build_context(items: &[TopLevel], timeline: &mut Timeline, current_key: &mut KeySignature, initial_ts: &mut u64, ppq: u32) {
        for item in items {
            match item {
                TopLevel::Meta(kvs) => {
                    if let Some(Value::Str(t)) = kvs.get("title") { timeline.title = t.clone(); }
                    if let Some(Value::Num(t)) = kvs.get("tempo") { timeline.tempo = *t as u32; }
                    if let Some(Value::Str(k)) = kvs.get("key") { if let Ok(parsed) = KeySignature::parse(k) { *current_key = parsed; } }
                    if let Some(Value::Bool(b)) = kvs.get("auto_pad_voices") { timeline.auto_pad_voices = *b; }
                    if let Some(Value::Num(n)) = kvs.get("swing") { timeline.swing.insert(16, *n as f64); } 
                    else if let Some(Value::Map(m)) = kvs.get("swing") { for (k, v) in m { if let Ok(div) = k.parse::<u32>() { if let Value::Num(n) = v { timeline.swing.insert(div, *n as f64); } } } }
                    if let Some(Value::Float(f)) = kvs.get("humanize") { timeline.humanize = *f; } else if let Some(Value::Num(n)) = kvs.get("humanize") { timeline.humanize = *n as f64; }
                    if let Some(Value::Str(s)) = kvs.get("time") { let parts: Vec<&str> = s.split('/').collect(); if parts.len() == 2 { if let (Ok(num), Ok(den)) = (parts[0].trim().parse::<u64>(), parts[1].trim().parse::<u64>()) { *initial_ts = (num * 4 * ppq as u64) / den; } } }
                },
                TopLevel::Def { id, label, attributes } => {
                    let patch = attributes.get("patch").and_then(|v| match v { Value::Str(s) | Value::Id(s) => Some(s.clone()), _ => None }).unwrap_or_else(|| "piano".into());
                    let style = attributes.get("style").and_then(|v| match v { Value::Str(s) | Value::Id(s) => Some(s.clone()), _ => None }).unwrap_or_else(|| "standard".into());
                    
                    // --- V3.0 MASTER SLICE: Print Demarcation Evaluator ---
                    let is_printable = style == "standard" || style == "tab" || style == "grid";
                    let print = attributes.get("print").and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None }).unwrap_or(is_printable);
                    
                    let mut keyswitches = HashMap::new(); if let Some(Value::Map(ks_map)) = attributes.get("keyswitch") { for (k, v) in ks_map { if let Value::Num(n) = v { keyswitches.insert(k.clone(), *n as u8); } } }
                    let mut env = HashMap::new(); if let Some(Value::Map(em)) = attributes.get("env") { for (k, v) in em { env.insert(k.to_lowercase(), parse_time_val_to_ms(v)); } }
                    let cut_group = attributes.get("cut_group").and_then(|v| { if let Value::Num(n) = v { Some(*n as u32) } else { None } });
                    
                    let mut perc_map = HashMap::new(); let mut concrete_map = HashMap::new();
                    if let Some(Value::Map(pm)) = attributes.get("map") {
                        for (k, v) in pm {
                            if style == "concrete" { if let Value::Array(arr) = v { if arr.len() == 2 { let start = parse_time_val_to_ms(&arr[0]); let end = parse_time_val_to_ms(&arr[1]); concrete_map.insert(k.to_lowercase(), (start, end)); } } } 
                            else { if let Value::Array(arr) = v { if arr.len() > 1 { if let Value::Num(midi) = arr[1] { perc_map.insert(k.clone(), midi as u8); } } } else if let Value::Num(midi) = v { perc_map.insert(k.clone(), *midi as u8); } }
                        }
                    }
                    timeline.tracks.insert(id.clone(), Track { label: label.clone().unwrap_or_else(|| id.clone()), patch, style, print, tuning: vec![40, 45, 50, 55, 59, 64], keyswitches, perc_map, concrete_map, env, cut_group, events: Vec::new(), current_key: current_key.clone(), spelling_state: MeasureSpellingState::new(current_key.clone()), });
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
                    if let Some(Value::Bool(b)) = kvs.get("auto_pad_voices") { timeline.auto_pad_voices = *b; }
                    if let Some(Value::Num(n)) = kvs.get("swing") { timeline.swing.insert(16, *n as f64); } else if let Some(Value::Map(m)) = kvs.get("swing") { for (k, v) in m { if let Ok(div) = k.parse::<u32>() { if let Value::Num(n) = v { timeline.swing.insert(div, *n as f64); } } } }
                    if let Some(Value::Float(f)) = kvs.get("humanize") { timeline.humanize = *f; } else if let Some(Value::Num(n)) = kvs.get("humanize") { timeline.humanize = *n as f64; }
                    if let Some(Value::Str(s)) = kvs.get("time") { let parts: Vec<&str> = s.split('/').collect(); if parts.len() == 2 { if let (Ok(num), Ok(den)) = (parts[0].trim().parse::<u64>(), parts[1].trim().parse::<u64>()) { current_ts_ticks = (num * 4 * ppq as u64) / den; } } }
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

                    let (m_idx, expected_end_idx) = match range {
                        MeasureRange::Single(idx) => { let s = std::cmp::max(idx, 1) as u64; (s, s) },
                        MeasureRange::Range(start, end) => { let s = std::cmp::max(start, 1) as u64; let e = std::cmp::max(end, 1) as u64; (s, e) },
                        MeasureRange::Implicit | _ => (*next_unassigned_index, *next_unassigned_index),
                    };

                    let absolute_start_tick = if let Some(&t) = measure_starts.get(&m_idx) {
                        t 
                    } else {
                        let gap_measures = m_idx.saturating_sub(*next_unassigned_index);
                        let t = *next_unassigned_tick + (gap_measures * local_ts_ticks);
                        measure_starts.insert(m_idx, t);
                        t
                    };

                    measure_pass(&content, timeline, cursors, absolute_start_tick, ppq, strict)?;
                    
                    let mut max_cursor_tick = absolute_start_tick; 
                    for c in cursors.values() {
                        if c.current_tick > max_cursor_tick { max_cursor_tick = c.current_tick; }
                    }

                    // --- V3.0.1 FIX: GRID SNAPPING ---
                    // 1. Calculate the raw ticks used.
                    let consumed_ticks = max_cursor_tick.saturating_sub(absolute_start_tick);
                    
                    // 2. Ceiling division: Find how many perfect measures we touched.
                    let consumed_measures = if consumed_ticks == 0 { 1 } else { (consumed_ticks as f64 / local_ts_ticks as f64).ceil() as u64 };
                    
                    // 3. Snap the absolute master clock to the end of that perfect measure boundary!
                    let snapped_next_tick = absolute_start_tick + (consumed_measures * local_ts_ticks);

                    for offset in 0..=consumed_measures {
                        measure_starts.insert(m_idx + offset, absolute_start_tick + (offset * local_ts_ticks));
                    }

                    let final_end_idx = std::cmp::max(expected_end_idx, m_idx + consumed_measures.saturating_sub(1));
                    
                    if final_end_idx >= *next_unassigned_index {
                        *next_unassigned_index = final_end_idx + 1;
                        *next_unassigned_tick = snapped_next_tick; // Locked perfectly to the downbeat!
                    }

                    for track in timeline.tracks.values_mut() { track.spelling_state.reset_at_barline(); }
                },
                
                TopLevel::Repeat { count, content } => {
                    let repeats = count.unwrap_or(2);
                    for _ in 0..repeats {
                        let absolute_start_tick = *next_unassigned_tick;
                        measure_pass(&content, timeline, cursors, absolute_start_tick, ppq, strict)?;
                        
                        let mut max_cursor_tick = absolute_start_tick;
                        for c in cursors.values() {
                            if c.current_tick > max_cursor_tick { max_cursor_tick = c.current_tick; }
                        }
                        
                        // --- V3.0.1 FIX: GRID SNAPPING FOR REPEATS ---
                        let consumed_ticks = max_cursor_tick.saturating_sub(absolute_start_tick);
                        let consumed_measures = if consumed_ticks == 0 { 1 } else { (consumed_ticks as f64 / current_ts_ticks as f64).ceil() as u64 };
                        let snapped_next_tick = absolute_start_tick + (consumed_measures * current_ts_ticks);

                        *next_unassigned_tick = snapped_next_tick;
                        *next_unassigned_index += consumed_measures;
                        
                        for track in timeline.tracks.values_mut() { track.spelling_state.reset_at_barline(); }
                    }
                },
                
                // Ensure Repeats also advance based on true cursor consumption!
                TopLevel::Repeat { count, content } => {
                    let repeats = count.unwrap_or(2);
                    for _ in 0..repeats {
                        let absolute_start_tick = *next_unassigned_tick;
                        measure_pass(&content, timeline, cursors, absolute_start_tick, ppq, strict)?;
                        
                        let mut max_cursor_tick = absolute_start_tick + current_ts_ticks;
                        for c in cursors.values() {
                            if c.current_tick > max_cursor_tick { max_cursor_tick = c.current_tick; }
                        }
                        
                        let consumed_measures = ((max_cursor_tick - absolute_start_tick) as f64 / current_ts_ticks as f64).ceil() as u64;
                        *next_unassigned_tick = max_cursor_tick;
                        *next_unassigned_index += consumed_measures;
                        
                        for track in timeline.tracks.values_mut() { track.spelling_state.reset_at_barline(); }
                    }
                },
                
                TopLevel::Group { items: inner_items, .. } => { process_logic_stream(inner_items, timeline, cursors, ppq, strict, current_ts_ticks, measure_starts, next_unassigned_index, next_unassigned_tick)?; },
                _ => {}
            }
        }
        Ok(())
    }

    let mut next_unassigned_index = 1;
    let mut next_unassigned_tick = 0;
    process_logic_stream(score.items, &mut timeline, &mut active_cursors, ppq, strict_mode, initial_ts_ticks, &mut measure_starts, &mut next_unassigned_index, &mut next_unassigned_tick)?;
    
    apply_groove_and_humanize(&mut timeline);
    apply_choke_groups(&mut timeline); 

    for track in timeline.tracks.values_mut() { track.events.sort_by_key(|e| e.tick); }
    Ok(timeline)
}

fn apply_choke_groups(timeline: &mut Timeline) {
    let mut group_events: HashMap<u32, Vec<(String, usize, u64)>> = HashMap::new();
    for (track_id, track) in &timeline.tracks { if let Some(cg) = track.cut_group { for (i, ev) in track.events.iter().enumerate() { if matches!(ev.kind, EventKind::Note{..} | EventKind::Concrete{..}) { let phys_start = (ev.tick as i64 + ev.physical_tick_offset).max(0) as u64; group_events.entry(cg).or_default().push((track_id.clone(), i, phys_start)); } } } }
    for (_, mut events) in group_events { events.sort_by_key(|e| e.2); for i in 0..events.len() { let current = &events[i]; if i + 1 < events.len() { let next = &events[i + 1]; let start_tick = current.2; let next_tick = next.2; if next_tick > start_tick { let track = timeline.tracks.get_mut(&current.0).unwrap(); let ev = &mut track.events[current.1]; let max_gate = next_tick - start_tick; if ev.gate_ticks > max_gate { ev.gate_ticks = max_gate; } } } } }
}

fn apply_groove_and_humanize(timeline: &mut Timeline) {
    let ppq = timeline.ppq;
    for track in timeline.tracks.values_mut() {
        for ev in &mut track.events {
            let mut swing_offset = 0; for (&div, &swing_pct) in &timeline.swing { let grid_size = (ppq * 4) / div; if ev.tick % grid_size as u64 == 0 { let index = ev.tick / grid_size as u64; if index % 2 == 1 { swing_offset += ((swing_pct - 50.0) / 100.0 * (grid_size * 2) as f64).round() as i64; } } } ev.physical_tick_offset += swing_offset;
            if timeline.humanize > 0.0 && !matches!(ev.kind, EventKind::Space | EventKind::Rest) {
                let seed = ev.tick.wrapping_add(ev.duration_ticks).wrapping_add(match &ev.kind { EventKind::Note { pitch_midi, .. } => *pitch_midi as u64, _ => 0, });
                let mut x = seed ^ 0x5bf0363546790d15; x ^= x << 13; x ^= x >> 7; x ^= x << 17; let u1 = ((x as u32) as f64 + 1.0) / 4294967296.0; x ^= x << 13; x ^= x >> 7; x ^= x << 17; let u2 = ((x as u32) as f64 + 1.0) / 4294967296.0;
                let r = (-2.0 * u1.ln()).sqrt().max(0.0); let gauss = r * (2.0 * std::f64::consts::PI * u2).cos(); let clamped_gauss = gauss.clamp(-3.0, 3.0); 
                let tick_shift = (clamped_gauss * timeline.humanize * ppq as f64).round() as i64; ev.physical_tick_offset += tick_shift;
                if let EventKind::Note { velocity, .. } = &mut ev.kind { let vel_shift = (clamped_gauss * timeline.humanize * 127.0).round() as i32; *velocity = (*velocity as i32 + vel_shift).clamp(1, 127) as u8; }
            }
        }
    }
}

// --- V3.0 MASTER SLICE: Pass &content so we can iterate twice (Notes, then Lyrics) ---
fn measure_pass(content: &[Logic], timeline: &mut Timeline, active_cursors: &mut HashMap<String, Cursor>, start_tick: u64, ppq: u32, strict_mode: bool) -> Result<(), String> {
    let auto_pad = timeline.auto_pad_voices;
    let tempo = timeline.tempo; 

    // V3.0 SLICE 8: Snapshot track lengths before processing notes to isolate THIS measure's events
    let mut start_lens: HashMap<String, usize> = HashMap::new();
    for (id, trk) in &timeline.tracks { start_lens.insert(id.clone(), trk.events.len()); }

    for logic in content {
        if let Logic::Assignment { staff_id, voices } = logic {
            let track = timeline.tracks.get_mut(staff_id).ok_or_else(|| format!("E2001: Undefined staff '{}'", staff_id))?;
            let mut voice_end_ticks = Vec::new();

            for (idx, voice) in voices.iter().enumerate() {
                let mut cursor = if idx == 0 {
                    let mut c = active_cursors.remove(staff_id).unwrap_or_else(|| Cursor::new(start_tick, ppq));
                    c.current_tick = start_tick; if strict_mode { c.last_duration = Rational::new(1, 4).unwrap(); c.last_octave = 4; }
                    c
                } else { Cursor::new(start_tick, ppq) };

                process_voice_events(voice, &mut cursor, track, tempo)?;
                voice_end_ticks.push(cursor.current_tick - start_tick);
                if idx == 0 { active_cursors.insert(staff_id.clone(), cursor); }
            }

            if voice_end_ticks.len() > 1 {
                let max_tick = *voice_end_ticks.iter().max().unwrap_or(&0);
                let is_synced = voice_end_ticks.iter().all(|&d| d == max_tick);

                if !is_synced {
                    if auto_pad {
                        for (idx, &end_tick) in voice_end_ticks.iter().enumerate() {
                            if end_tick < max_tick {
                                let gap = max_tick - end_tick;
                                track.events.push(AtomicEvent {
                                    tick: start_tick + end_tick, duration_ticks: gap, gate_ticks: gap, physical_tick_offset: 0,
                                    kind: EventKind::Rest, tuplet_state: None, is_grace: false, is_ghost: false, tremolo_slashes: None, cc_automations: vec![], tab_articulation: TabArticulation::None, synth_glide_start_midi: None, synth_glide_ticks: None, synth_accelerate_semitones: None,
                                    lyric: None, lyric_extension: LyricExtension::None,
                                });
                                if idx == 0 { if let Some(c) = active_cursors.get_mut(staff_id) { c.current_tick += gap; } }
                            }
                        }
                    } else if strict_mode { return Err(format!("E3002: Voice Sync Failure in polyphonic block for staff '{}'", staff_id)); }
                }
            }
        }
    }

    // --- V3.0 MASTER SLICE: Lyric Engine Execution ---
    for logic in content {
        if let Logic::LyricAssignment { staff_id, text } = logic {
            if let Some(track) = timeline.tracks.get_mut(staff_id) {
                let mut syllables = Vec::new();
                let mut current_word = String::new();
                for c in text.chars() {
                    match c {
                        ' ' => { if !current_word.is_empty() { syllables.push(current_word.clone()); current_word.clear(); } },
                        '-' | '_' | '~' | '*' => { 
                            if !current_word.is_empty() { syllables.push(current_word.clone()); current_word.clear(); }
                            syllables.push(c.to_string());
                        },
                        _ => current_word.push(c),
                    }
                }
                if !current_word.is_empty() { syllables.push(current_word); }

                let mut note_idx = start_lens.get(staff_id).copied().unwrap_or(0);
                let mut syl_idx = 0;
                while syl_idx < syllables.len() && note_idx < track.events.len() {
                    let ev = &mut track.events[note_idx];
                    
                    // Skip over rests, spaces, concrete audio, and grace notes!
                    if matches!(ev.kind, EventKind::Rest | EventKind::Space | EventKind::Concrete{..}) || ev.is_grace {
                        note_idx += 1; continue;
                    }
                    
                    let token = &syllables[syl_idx];
                    if token == "*" {
                        note_idx += 1;
                    } else if token == "-" {
                        if note_idx > 0 { track.events[note_idx - 1].lyric_extension = LyricExtension::Hyphen; }
                    } else if token == "_" {
                        if note_idx > 0 { track.events[note_idx - 1].lyric_extension = LyricExtension::Melisma; }
                    } else if token == "~" {
                        syl_idx += 1;
                        if syl_idx < syllables.len() {
                            let next_syl = &syllables[syl_idx];
                            if let Some(ref mut l) = ev.lyric { l.push(' '); l.push_str(next_syl); }
                        }
                        note_idx += 1;
                    } else {
                        ev.lyric = Some(token.clone());
                        note_idx += 1;
                    }
                    syl_idx += 1;
                }
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
                tick, duration_ticks: 1, gate_ticks: 1, physical_tick_offset: 0,
                kind: EventKind::Note { pitch_midi: ks_midi, cents: 0, velocity: 1, spelling },
                tuplet_state: None, is_grace: false, is_ghost: false, tremolo_slashes: None, cc_automations: vec![], tab_articulation: TabArticulation::None, synth_glide_start_midi: None, synth_glide_ticks: None, synth_accelerate_semitones: None,
                lyric: None, lyric_extension: LyricExtension::None,
            });
        }
    }
}

fn emit_rudiments(track: &mut Track, tick: u64, offset: i64, ppq: u32, midi: u8, velocity: u8, spelling: &SpelledPitch, flam: bool, drag: bool) {
    if flam {
        track.events.push(AtomicEvent {
            tick, duration_ticks: 0, gate_ticks: ppq as u64 / 8, physical_tick_offset: offset - (ppq as i64 / 16),
            kind: EventKind::Note { pitch_midi: midi, cents: 0, velocity: (velocity as f32 * 0.6) as u8, spelling: spelling.clone() },
            tuplet_state: None, is_grace: true, is_ghost: true, tremolo_slashes: None, cc_automations: vec![], tab_articulation: TabArticulation::None, synth_glide_start_midi: None, synth_glide_ticks: None, synth_accelerate_semitones: None, lyric: None, lyric_extension: LyricExtension::None,
        });
    }
    if drag {
        track.events.push(AtomicEvent {
            tick, duration_ticks: 0, gate_ticks: ppq as u64 / 8, physical_tick_offset: offset - (ppq as i64 / 8),
            kind: EventKind::Note { pitch_midi: midi, cents: 0, velocity: (velocity as f32 * 0.4) as u8, spelling: spelling.clone() },
            tuplet_state: None, is_grace: true, is_ghost: true, tremolo_slashes: None, cc_automations: vec![], tab_articulation: TabArticulation::None, synth_glide_start_midi: None, synth_glide_ticks: None, synth_accelerate_semitones: None, lyric: None, lyric_extension: LyricExtension::None,
        });
        track.events.push(AtomicEvent {
            tick, duration_ticks: 0, gate_ticks: ppq as u64 / 8, physical_tick_offset: offset - (ppq as i64 / 16),
            kind: EventKind::Note { pitch_midi: midi, cents: 0, velocity: (velocity as f32 * 0.5) as u8, spelling: spelling.clone() },
            tuplet_state: None, is_grace: true, is_ghost: true, tremolo_slashes: None, cc_automations: vec![], tab_articulation: TabArticulation::None, synth_glide_start_midi: None, synth_glide_ticks: None, synth_accelerate_semitones: None, lyric: None, lyric_extension: LyricExtension::None,
        });
    }
}

fn extract_timing(event: &Event) -> (Option<String>, u8, Option<u32>) {
    match event {
        Event::Note { duration, dots, multiplier, .. } | Event::Chord { duration, dots, multiplier, .. } | Event::Percussion { duration, dots, multiplier, .. } | Event::Tab { duration, dots, multiplier, .. } | Event::TabChord { duration, dots, multiplier, .. } | Event::PercussionChord { duration, dots, multiplier, .. } | Event::Frequency { duration, dots, multiplier, .. } | Event::MacroCall { duration, dots, multiplier, .. } | Event::Space { duration, dots, multiplier, .. } | Event::Rest { duration, dots, multiplier } => (duration.clone(), *dots, *multiplier),
        Event::Euclidean { content, .. } => extract_timing(content),
        _ => (None, 0, None),
    }
}

// ============================================================================
// THE EVENT DISPATCHER
// ============================================================================

fn process_voice_events(voice: &Voice, cursor: &mut Cursor, track: &mut Track, tempo: u32) -> Result<(), String> {
    let is_pedal_lane = voice.voice_id.as_deref() == Some("pedal");
    let style = track.style.clone();

    for event in &voice.events {
        let t_state = cursor.active_tuplet_ratio.map(|(act, norm)| TupletState { actual_notes: act, normal_notes: norm, is_start: false, is_stop: false });

        match event {
            Event::Note { pitch, cents, duration, dots, multiplier, is_tied, attributes } => {
                let (log, gate_ticks, is_grace) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier)?;
                cursor.parse_dynamics(attributes); let p_attrs = apply_attributes(attributes, gate_ticks, cursor.last_velocity, tempo, cursor.ppq);

                if style == "concrete" {
                    let key_lc = pitch.to_lowercase(); let (start_ms, end_ms) = track.concrete_map.get(&key_lc).copied().unwrap_or((0, 0));
                    let slices = p_attrs.slice.unwrap_or(1).max(1); let sub_log = log / slices as u64; let sub_gate = p_attrs.gate / slices as u64;
                    for i in 0..slices {
                        let frac_start = start_ms + ((end_ms - start_ms) * i as i64 / slices as i64); let frac_end = start_ms + ((end_ms - start_ms) * (i + 1) as i64 / slices as i64);
                        track.events.push(AtomicEvent {
                            tick: cursor.current_tick + (sub_log * i as u64), duration_ticks: sub_log, gate_ticks: sub_gate, physical_tick_offset: p_attrs.physical_tick_offset,
                            kind: EventKind::Concrete { key: key_lc.clone(), params: ConcreteParams { sample_start: frac_start, sample_end: frac_end, stretch: p_attrs.stretch, reverse: p_attrs.reverse } },
                            tuplet_state: t_state.clone(), is_grace, is_ghost: p_attrs.is_ghost, tremolo_slashes: p_attrs.tremolo_slashes, cc_automations: p_attrs.cc_automations.clone(), tab_articulation: TabArticulation::None, synth_glide_start_midi: None, synth_glide_ticks: None, synth_accelerate_semitones: None, lyric: None, lyric_extension: LyricExtension::None,
                        });
                    } cursor.current_tick += log;
                } else {
                    let midi = if let Some(&mapped) = track.perc_map.get(pitch) { mapped } else { cursor.parse_pitch(pitch, &style) };
                    let mut spelling = SpelledPitch::from_string(pitch, cursor.last_octave as i8).unwrap_or_else(|_| SpelledPitch::from_midi(midi, cents.unwrap_or(0), &track.current_key));
                    spelling = track.spelling_state.process_pitch(spelling);

                    emit_keyswitches(attributes, track, cursor.current_tick); emit_rudiments(track, cursor.current_tick, p_attrs.physical_tick_offset, cursor.ppq, midi, p_attrs.velocity, &spelling, p_attrs.flam, p_attrs.drag);
                    if cursor.tied_pitches.contains(&midi) && !is_grace { handle_tie(track, midi, log)?; cursor.tied_pitches.retain(|&p| p != midi); if *is_tied { cursor.tied_pitches.push(midi); }
                    } else {
                        track.events.push(AtomicEvent {
                            tick: cursor.current_tick, duration_ticks: log, gate_ticks: p_attrs.gate, physical_tick_offset: p_attrs.physical_tick_offset,
                            kind: EventKind::Note { pitch_midi: midi, cents: cents.unwrap_or(0), velocity: p_attrs.velocity, spelling },
                            tuplet_state: t_state, is_grace, is_ghost: p_attrs.is_ghost, tremolo_slashes: p_attrs.tremolo_slashes, cc_automations: p_attrs.cc_automations, tab_articulation: p_attrs.tab_articulation, synth_glide_start_midi: cursor.last_pitch_midi, synth_glide_ticks: p_attrs.glide_ticks, synth_accelerate_semitones: p_attrs.accelerate, lyric: None, lyric_extension: LyricExtension::None,
                        }); if *is_tied && !is_grace { cursor.tied_pitches.push(midi); }
                    } cursor.current_tick += log;
                }
            },
            Event::Chord { notes, duration, dots, multiplier, is_tied: _is_tied, attributes: _attributes } => {
                let (log, gate_ticks, is_grace) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier)?;
                cursor.parse_dynamics(_attributes); 
                for note_event in notes {
                    if let Event::Note { pitch, cents, attributes: note_attrs, .. } = note_event {
                        let p_attrs = apply_attributes(note_attrs, gate_ticks, cursor.last_velocity, tempo, cursor.ppq); let midi = cursor.parse_pitch(pitch, &style);
                        let mut spelling = SpelledPitch::from_string(pitch, cursor.last_octave as i8).unwrap_or_else(|_| SpelledPitch::from_midi(midi, cents.unwrap_or(0), &track.current_key)); spelling = track.spelling_state.process_pitch(spelling);
                        emit_keyswitches(note_attrs, track, cursor.current_tick);
                        track.events.push(AtomicEvent {
                            tick: cursor.current_tick, duration_ticks: log, gate_ticks: p_attrs.gate, physical_tick_offset: p_attrs.physical_tick_offset,
                            kind: EventKind::Note { pitch_midi: midi, cents: cents.unwrap_or(0), velocity: p_attrs.velocity, spelling },
                            tuplet_state: t_state.clone(), is_grace, is_ghost: p_attrs.is_ghost, tremolo_slashes: p_attrs.tremolo_slashes, cc_automations: p_attrs.cc_automations, tab_articulation: p_attrs.tab_articulation, synth_glide_start_midi: cursor.last_pitch_midi, synth_glide_ticks: p_attrs.glide_ticks, synth_accelerate_semitones: p_attrs.accelerate, lyric: None, lyric_extension: LyricExtension::None,
                        });
                    }
                } cursor.current_tick += log;
            },
            Event::Percussion { key, duration, dots, multiplier, attributes } => {
                let (log, gate_ticks, is_grace) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier)?;
                cursor.parse_dynamics(attributes); let mut p_attrs = apply_attributes(attributes, gate_ticks, cursor.last_velocity, tempo, cursor.ppq);
                if is_pedal_lane {
                    let val = match key.as_str() { "down" => 127, "half" => 64, "up" => 0, _ => 0 }; p_attrs.cc_automations.push(CCAutomation { controller: 64, start_val: val, end_val: val, curve: "linear".into() });
                    track.events.push(AtomicEvent {
                        tick: cursor.current_tick, duration_ticks: log, gate_ticks: p_attrs.gate, physical_tick_offset: p_attrs.physical_tick_offset,
                        kind: EventKind::Space, tuplet_state: t_state, is_grace, is_ghost: false, tremolo_slashes: None, cc_automations: p_attrs.cc_automations, tab_articulation: TabArticulation::None, synth_glide_start_midi: None, synth_glide_ticks: None, synth_accelerate_semitones: None, lyric: None, lyric_extension: LyricExtension::None,
                    });
                } else if style == "concrete" {
                    let key_lc = key.to_lowercase(); let (start_ms, end_ms) = track.concrete_map.get(&key_lc).copied().unwrap_or((0, 0)); let slices = p_attrs.slice.unwrap_or(1).max(1); let sub_log = log / slices as u64; let sub_gate = p_attrs.gate / slices as u64;
                    for i in 0..slices {
                        let frac_start = start_ms + ((end_ms - start_ms) * i as i64 / slices as i64); let frac_end = start_ms + ((end_ms - start_ms) * (i + 1) as i64 / slices as i64);
                        track.events.push(AtomicEvent {
                            tick: cursor.current_tick + (sub_log * i as u64), duration_ticks: sub_log, gate_ticks: sub_gate, physical_tick_offset: p_attrs.physical_tick_offset,
                            kind: EventKind::Concrete { key: key_lc.clone(), params: ConcreteParams { sample_start: frac_start, sample_end: frac_end, stretch: p_attrs.stretch, reverse: p_attrs.reverse } },
                            tuplet_state: t_state.clone(), is_grace, is_ghost: p_attrs.is_ghost, tremolo_slashes: p_attrs.tremolo_slashes, cc_automations: p_attrs.cc_automations.clone(), tab_articulation: TabArticulation::None, synth_glide_start_midi: None, synth_glide_ticks: None, synth_accelerate_semitones: None, lyric: None, lyric_extension: LyricExtension::None,
                        });
                    }
                } else {
                    let midi = track.perc_map.get(key).copied().unwrap_or(60); let mut spelling = SpelledPitch::from_midi(midi, 0, &track.current_key); spelling = track.spelling_state.process_pitch(spelling);
                    emit_keyswitches(attributes, track, cursor.current_tick); emit_rudiments(track, cursor.current_tick, p_attrs.physical_tick_offset, cursor.ppq, midi, p_attrs.velocity, &spelling, p_attrs.flam, p_attrs.drag);
                    track.events.push(AtomicEvent {
                        tick: cursor.current_tick, duration_ticks: log, gate_ticks: p_attrs.gate, physical_tick_offset: p_attrs.physical_tick_offset,
                        kind: EventKind::Note { pitch_midi: midi, cents: 0, velocity: p_attrs.velocity, spelling },
                        tuplet_state: t_state, is_grace, is_ghost: p_attrs.is_ghost, tremolo_slashes: p_attrs.tremolo_slashes, cc_automations: p_attrs.cc_automations, tab_articulation: p_attrs.tab_articulation, synth_glide_start_midi: None, synth_glide_ticks: None, synth_accelerate_semitones: None, lyric: None, lyric_extension: LyricExtension::None,
                    });
                } cursor.current_tick += log;
            },
            Event::Tab { fret, string, duration, dots, multiplier, attributes } => {
                let (log, gate_ticks, is_grace) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier)?; cursor.parse_dynamics(attributes); let p_attrs = apply_attributes(attributes, gate_ticks, cursor.last_velocity, tempo, cursor.ppq);
                let str_idx = track.tuning.len().saturating_sub(*string as usize); let base_pitch = track.tuning.get(str_idx).cloned().unwrap_or(60); let midi = base_pitch + fret.parse::<u8>().unwrap_or(0);
                let mut spelling = SpelledPitch::from_midi(midi, 0, &track.current_key); spelling = track.spelling_state.process_pitch(spelling);
                track.events.push(AtomicEvent {
                    tick: cursor.current_tick, duration_ticks: log, gate_ticks: p_attrs.gate, physical_tick_offset: p_attrs.physical_tick_offset,
                    kind: EventKind::Note { pitch_midi: midi, cents: 0, velocity: p_attrs.velocity, spelling },
                    tuplet_state: t_state, is_grace, is_ghost: p_attrs.is_ghost, tremolo_slashes: p_attrs.tremolo_slashes, cc_automations: p_attrs.cc_automations, tab_articulation: p_attrs.tab_articulation, synth_glide_start_midi: None, synth_glide_ticks: None, synth_accelerate_semitones: None, lyric: None, lyric_extension: LyricExtension::None,
                }); cursor.current_tick += log;
            },
            Event::Frequency { hz, duration, dots, multiplier, attributes } => {
                let (log, gate_ticks, is_grace) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier)?; cursor.parse_dynamics(attributes); let p_attrs = apply_attributes(attributes, gate_ticks, cursor.last_velocity, tempo, cursor.ppq);
                let freq: f64 = hz.parse().unwrap_or(440.0);
                track.events.push(AtomicEvent {
                    tick: cursor.current_tick, duration_ticks: log, gate_ticks: p_attrs.gate, physical_tick_offset: p_attrs.physical_tick_offset,
                    kind: EventKind::Frequency { hz: freq, velocity: p_attrs.velocity },
                    tuplet_state: t_state, is_grace, is_ghost: p_attrs.is_ghost, tremolo_slashes: p_attrs.tremolo_slashes, cc_automations: p_attrs.cc_automations, tab_articulation: p_attrs.tab_articulation, synth_glide_start_midi: None, synth_glide_ticks: None, synth_accelerate_semitones: None, lyric: None, lyric_extension: LyricExtension::None,
                }); cursor.current_tick += log;
            },
            Event::Space { duration, dots, multiplier, attributes } => {
                let (log, gate_ticks, is_grace) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier)?; cursor.parse_dynamics(attributes); let p_attrs = apply_attributes(attributes, gate_ticks, cursor.last_velocity, tempo, cursor.ppq);
                track.events.push(AtomicEvent {
                    tick: cursor.current_tick, duration_ticks: log, gate_ticks: log, physical_tick_offset: p_attrs.physical_tick_offset,
                    kind: EventKind::Space, tuplet_state: t_state, is_grace, is_ghost: false, tremolo_slashes: None, cc_automations: p_attrs.cc_automations, tab_articulation: TabArticulation::None, synth_glide_start_midi: None, synth_glide_ticks: None, synth_accelerate_semitones: None, lyric: None, lyric_extension: LyricExtension::None,
                }); cursor.current_tick += log;
            },
            Event::Rest { duration, dots, multiplier } => {
                let (log, _, is_grace) = cursor.parse_duration(duration.as_ref(), *dots, *multiplier)?;
                track.events.push(AtomicEvent {
                    tick: cursor.current_tick, duration_ticks: log, gate_ticks: log, physical_tick_offset: 0,
                    kind: EventKind::Rest, tuplet_state: t_state, is_grace, is_ghost: false, tremolo_slashes: None, cc_automations: vec![], tab_articulation: TabArticulation::None, synth_glide_start_midi: None, synth_glide_ticks: None, synth_accelerate_semitones: None, lyric: None, lyric_extension: LyricExtension::None,
                }); cursor.current_tick += log;
            },
            Event::Tuplet { content, p, q } => {
                let old_scalar = cursor.time_scalar; cursor.time_scalar = Rational::new(old_scalar.num * *q, old_scalar.den * *p)?; cursor.active_tuplet_ratio = Some((*p, *q));
                let start_idx = track.events.len(); process_voice_events(content, cursor, track, tempo)?;
                if start_idx < track.events.len() { if let Some(ts) = &mut track.events[start_idx].tuplet_state { ts.is_start = true; } let last_idx = track.events.len() - 1; if let Some(ts) = &mut track.events[last_idx].tuplet_state { ts.is_stop = true; } }
                cursor.time_scalar = old_scalar; cursor.active_tuplet_ratio = None;
            },
            Event::Euclidean { content, k, n } => {
                let old_scalar = cursor.time_scalar; cursor.time_scalar = Rational::new(old_scalar.num, old_scalar.den * *n)?; let (dur, dots, mult) = extract_timing(content);
                for i in 0..*n {
                    if (i * k) % n < *k { process_voice_events(&Voice { voice_id: None, events: vec![*content.clone()] }, cursor, track, tempo)?; } 
                    else { let rest = Event::Rest { duration: dur.clone(), dots, multiplier: mult }; process_voice_events(&Voice { voice_id: None, events: vec![rest] }, cursor, track, tempo)?; }
                } cursor.time_scalar = old_scalar;
            },
            _ => {}
        }
    }
    Ok(())
}

fn handle_tie(track: &mut Track, target: u8, extra: u64) -> Result<(), String> {
    if let Some(ev) = track.events.iter_mut().rev().find(|e| matches!(e.kind, EventKind::Note { pitch_midi, .. } if pitch_midi == target)) {
        ev.duration_ticks += extra; ev.gate_ticks += extra; Ok(())
    } else { Err(format!("E4005: Tie target not found for pitch {}", target)) }
}