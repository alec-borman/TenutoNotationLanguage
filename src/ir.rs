use crate::parser::{Score, TopLevel, Statement, Event as AstEvent, Value};
use crate::Rational;
use std::collections::HashMap;

/// The flattened, fully resolved event stream.
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
    pub channel: u8,
    pub events: Vec<AtomicEvent>,
}

#[derive(Debug, Clone)]
pub struct AtomicEvent {
    pub tick: u64,          
    pub duration_ticks: u64,
    pub kind: EventKind,
}

#[derive(Debug, Clone)]
pub enum EventKind {
    Note { pitch: u8, velocity: u8 }, 
    Rest,
}

// ========================================================================
// THE INFERENCE ENGINE
// ========================================================================

struct Cursor {
    current_tick: u64,
    last_duration: Rational, 
    last_octave: u8,         
    _ppq: u32, // CHANGED: Added underscore to suppress unused warning
}

impl Cursor {
    fn new(ppq: u32) -> Self {
        Self {
            current_tick: 0,
            last_duration: Rational::new(1, 4), 
            last_octave: 4,                     
            _ppq: ppq, // CHANGED
        }
    }

    /// Converts AST Duration string (":4.") to Rational
    fn parse_duration(&mut self, d_str: Option<&String>) -> Rational {
        if let Some(s) = d_str {
            let raw = &s[1..];
            let dots = raw.chars().filter(|&c| c == '.').count();
            let base_str: String = raw.chars().take_while(|&c| c != '.').collect();
            
            let denominator: u64 = base_str.parse().unwrap_or(4);
            let mut rat = Rational::new(1, denominator);

            if dots > 0 {
                rat = Rational::new(3, denominator * 2);
            }

            self.last_duration = rat; 
            rat
        } else {
            self.last_duration 
        }
    }

    /// Converts AST Pitch string ("c4") to MIDI Note Number
    fn parse_pitch(&mut self, p_str: &str) -> u8 {
        let chars: Vec<char> = p_str.chars().collect();
        if chars.is_empty() { return 60; }

        let step = chars[0].to_ascii_lowercase();
        
        let mut base = match step {
            'c' => 0, 'd' => 2, 'e' => 4, 'f' => 5, 'g' => 7, 'a' => 9, 'b' => 11,
            _ => 0,
        };

        let mut octave = self.last_octave; 
        let mut has_explicit_octave = false;

        let mut i = 1;
        while i < chars.len() {
            match chars[i] {
                '#' => base += 1,
                'b' => base -= 1, 
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

        if has_explicit_octave {
            self.last_octave = octave; 
        }

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
                    if k == "title" {
                        if let Value::Str(s) = v { timeline.title = s.clone(); }
                    } else if k == "tempo" {
                        if let Value::Num(n) = v { timeline.tempo = *n as u32; }
                    }
                }
            },
            TopLevel::Def { id, label, attributes } => {
                // CHANGED: Removed 'mut' (Fixes unused_mut warning)
                let channel = 0; 
                let mut patch = "Grand Piano".to_string();
                
                for (attr, val) in attributes {
                    if attr == "patch" {
                         if let Value::Str(s) = val { patch = s.clone(); }
                    }
                }

                timeline.tracks.insert(id.clone(), Track {
                    label: label.clone(),
                    patch,
                    channel,
                    events: Vec::new(),
                });
            },
            _ => {}
        }
    }

    // 2. Linearization
    let ppq = 1920;
    let mut cursors: HashMap<String, Cursor> = HashMap::new();

    for id in timeline.tracks.keys() {
        cursors.insert(id.clone(), Cursor::new(ppq));
    }

    for item in &score.items {
        if let TopLevel::Measure { id: _, content } = item {
            for stmt in content {
                match stmt {
                    Statement::Assignment { staff_id, voices } => {
                        if let Some(track) = timeline.tracks.get_mut(staff_id) {
                            let cursor = cursors.get_mut(staff_id).unwrap();

                            if let Some(voice) = voices.first() {
                                for event in &voice.events {
                                    match event {
                                        AstEvent::Note { pitch, duration, attributes: _ } => {
                                            let dur_rat = cursor.parse_duration(duration.as_ref());
                                            let dur_ticks = dur_rat.to_ticks(ppq);
                                            let midi_pitch = cursor.parse_pitch(pitch);

                                            track.events.push(AtomicEvent {
                                                tick: cursor.current_tick,
                                                duration_ticks: dur_ticks,
                                                kind: EventKind::Note { 
                                                    pitch: midi_pitch, 
                                                    velocity: 100 
                                                },
                                            });

                                            cursor.current_tick += dur_ticks;
                                        },
                                        AstEvent::Rest { duration } => {
                                            let dur_rat = cursor.parse_duration(duration.as_ref());
                                            let dur_ticks = dur_rat.to_ticks(ppq);
                                            cursor.current_tick += dur_ticks;
                                        }
                                        _ => {} 
                                    }
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    Ok(timeline)
}