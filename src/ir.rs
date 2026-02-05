use crate::parser::{Score, TopLevel, Statement, Event as AstEvent, Value, Voice};
use crate::Rational;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Timeline {
    pub title: String,
    pub tempo: u32,
    pub tracks: HashMap<String, Track>,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub label: String,
    pub patch: String,
    pub events: Vec<AtomicEvent>,
}

#[derive(Debug, Clone)]
pub struct AtomicEvent {
    pub tick: u64,          
    pub duration_ticks: u64,
    pub kind: EventKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventKind {
    Note { pitch: u8, velocity: u8 }, 
    Rest,
}

struct Cursor {
    current_tick: u64,
    last_duration: Rational, 
    last_octave: u8,
    // Time Scalar for Tuplets. Standard = 1/1. Triplet = 2/3.
    time_scalar: Rational,
    ppq: u32, 
}

impl Cursor {
    fn new(ppq: u32) -> Self {
        Self {
            current_tick: 0,
            last_duration: Rational::new(1, 4), 
            last_octave: 4,  
            time_scalar: Rational::new(1, 1),
            ppq,
        }
    }

    fn parse_duration(&mut self, d_str: Option<&String>) -> u64 {
        let base_rat = if let Some(s) = d_str {
            let raw = &s[1..];
            let dots = raw.chars().filter(|&c| c == '.').count();
            let base_str: String = raw.chars().take_while(|&c| c != '.').collect();
            let denominator: u64 = base_str.parse().unwrap_or(4);
            
            let mut rat = Rational::new(1, denominator);
            if dots == 1 { rat = Rational::new(3, denominator * 2); }
            else if dots == 2 { rat = Rational::new(7, denominator * 4); }
            
            self.last_duration = rat;
            rat
        } else {
            self.last_duration
        };

        // Apply Time Scalar (for Tuplets)
        // Duration = Base * Scalar
        // e.g. 1/8 * (2/3) = 1/12 (Triplet eighth)
        let final_rat = Rational::new(
            base_rat.num * self.time_scalar.num,
            base_rat.den * self.time_scalar.den
        );

        final_rat.to_ticks(self.ppq)
    }

    fn parse_pitch(&mut self, p_str: &str) -> u8 {
        let chars: Vec<char> = p_str.chars().collect();
        if chars.is_empty() { return 60; }
        let step = chars[0].to_ascii_lowercase();
        let mut base = match step {
            'c' => 0, 'd' => 2, 'e' => 4, 'f' => 5, 'g' => 7, 'a' => 9, 'b' => 11, _ => 0
        };
        let mut octave = self.last_octave; 
        let mut has_explicit_octave = false;
        let mut i = 1;
        while i < chars.len() {
            match chars[i] {
                '#' => base += 1, 'b' => base -= 1, 
                c if c.is_digit(10) => {
                    if !has_explicit_octave {
                        octave = c.to_digit(10).unwrap() as u8;
                        has_explicit_octave = true;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        if has_explicit_octave { self.last_octave = octave; }
        (octave + 1) * 12 + base
    }
}

pub fn compile(score: Score) -> Result<Timeline, String> {
    let mut timeline = Timeline {
        title: "Untitled".into(),
        tempo: 120,
        tracks: HashMap::new(),
    };

    // 1. Context Building
    for item in &score.items {
        match item {
            TopLevel::Meta(kvs) => {
                for (k, v) in kvs {
                    if k == "title" { if let Value::Str(s) = v { timeline.title = s.clone(); } }
                    else if k == "tempo" { if let Value::Num(n) = v { timeline.tempo = *n as u32; } }
                }
            },
            TopLevel::Def { id, label, attributes } => {
                let mut patch = "Grand Piano".to_string();
                for (attr, val) in attributes {
                    if attr == "patch" { if let Value::Str(s) = val { patch = s.clone(); } }
                }
                timeline.tracks.insert(id.clone(), Track {
                    label: label.clone(),
                    patch,
                    events: Vec::new(),
                });
            },
            _ => {}
        }
    }

    // 2. Linearization
    let ppq = 1920;
    // Map of StaffID -> [Cursor for Voice 1, Cursor for Voice 2...]
    let mut cursors: HashMap<String, Vec<Cursor>> = HashMap::new();

    for id in timeline.tracks.keys() {
        // Start with 4 voices per track as default, can expand dynamically
        cursors.insert(id.clone(), vec![
            Cursor::new(ppq), Cursor::new(ppq), Cursor::new(ppq), Cursor::new(ppq)
        ]);
    }

    for item in &score.items {
        if let TopLevel::Measure { content, .. } = item {
            for stmt in content {
                match stmt {
                    Statement::Assignment { staff_id, voices } => {
                        if let Some(track) = timeline.tracks.get_mut(staff_id) {
                            let track_cursors = cursors.get_mut(staff_id).unwrap();
                            
                            // Process each voice in parallel
                            for (v_idx, voice) in voices.iter().enumerate() {
                                if v_idx >= track_cursors.len() {
                                    track_cursors.push(Cursor::new(ppq));
                                }
                                let cursor = &mut track_cursors[v_idx];
                                process_voice(voice, cursor, track);
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    // Sort events by tick (since multi-voice processing implies out-of-order insertion)
    for track in timeline.tracks.values_mut() {
        track.events.sort_by_key(|e| e.tick);
    }

    Ok(timeline)
}

/// Recursively processes events (supports Tuplets)
fn process_voice(voice: &Voice, cursor: &mut Cursor, track: &mut Track) {
    for event in &voice.events {
        match event {
            AstEvent::Note { pitch, duration, .. } => {
                let ticks = cursor.parse_duration(duration.as_ref());
                let midi = cursor.parse_pitch(pitch);
                track.events.push(AtomicEvent {
                    tick: cursor.current_tick,
                    duration_ticks: ticks,
                    kind: EventKind::Note { pitch: midi, velocity: 100 },
                });
                cursor.current_tick += ticks;
            },
            AstEvent::Chord { notes, duration, .. } => {
                let ticks = cursor.parse_duration(duration.as_ref());
                // Chords: Multiple notes at SAME cursor tick
                for note in notes {
                    let midi = cursor.parse_pitch(note);
                    track.events.push(AtomicEvent {
                        tick: cursor.current_tick,
                        duration_ticks: ticks,
                        kind: EventKind::Note { pitch: midi, velocity: 100 },
                    });
                }
                // Only advance cursor once per chord
                cursor.current_tick += ticks;
            },
            AstEvent::Rest { duration } => {
                let ticks = cursor.parse_duration(duration.as_ref());
                cursor.current_tick += ticks;
            },
            AstEvent::Tuplet { content, p, q } => {
                // Spec 5.3: "Play P notes in the time of Q"
                // Scalar = Q / P
                let old_scalar = cursor.time_scalar;
                let scale_factor = Rational::new(*q, *p);
                
                // Update scalar: New = Old * (Q/P)
                cursor.time_scalar = Rational::new(
                    old_scalar.num * scale_factor.num,
                    old_scalar.den * scale_factor.den
                );

                process_voice(content, cursor, track);

                // Restore scalar
                cursor.time_scalar = old_scalar;
            },
            _ => {} // Tab/Percussion placeholders for now
        }
    }
}