//! # Tenuto MIDI 1.0 / 2.0 Exporter
//! 
//! Converts the absolute Timeline IR into a standard MIDI file (.mid).
//! 
//! **V3.0.0 Updates:**
//! - Slice 2: Hoisted CC automation extraction for Space events.
//! - Slice 4: Injects physical_tick_offset for Micro-Timing.
//! - Slice 6: Safely ignores abstract `Concrete` audio buffer slices.
//! - Slice 7: Calculates continuous 14-bit Pitch Bend arrays for 
//!   Synth Portamento (`.glide`) and Drops (`.accelerate`).

use crate::ir::{Timeline, EventKind, TabArticulation};
use midly::{Smf, Header, Format, Timing, TrackEvent, TrackEventKind, MidiMessage, MetaMessage};
use midly::num::{u28, u14, u15};

#[derive(Debug, Clone)]
struct AbsEvent<'a> {
    pub tick: u64,
    pub priority: u8, // 0=Meta, 1=CC/Bend, 2=NoteOff, 3=NoteOn, 4=BendReset
    pub kind: TrackEventKind<'a>,
}

pub fn export(timeline: &Timeline) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let safe_ppq = u15::from_int_lossy(timeline.ppq as u16);
    let header = Header::new(Format::Parallel, Timing::Metrical(safe_ppq));
    let mut smf = Smf::new(header);

    // ------------------------------------------------------------------------
    // TRACK 0: The Conductor Track
    // ------------------------------------------------------------------------
    let mut conductor_track = Vec::new();
    conductor_track.push(TrackEvent { delta: 0.into(), kind: TrackEventKind::Meta(MetaMessage::TrackName(timeline.title.as_bytes())) });
    
    let mpq = 60_000_000 / timeline.tempo.max(1);
    conductor_track.push(TrackEvent { delta: 0.into(), kind: TrackEventKind::Meta(MetaMessage::Tempo(mpq.into())) });
    conductor_track.push(TrackEvent { delta: 0.into(), kind: TrackEventKind::Meta(MetaMessage::EndOfTrack) });
    smf.tracks.push(conductor_track);

    // ------------------------------------------------------------------------
    // TRACKS 1..N: Instrument Tracks
    // ------------------------------------------------------------------------
    let mut sorted_keys: Vec<_> = timeline.tracks.keys().collect();
    sorted_keys.sort(); 
    let mut channel_allocator = 0;

    for key in sorted_keys {
        let track_data = &timeline.tracks[key];
        let mut abs_events = Vec::new();
        
        // Strict Channel 10 mapping for percussion
        let is_drum = track_data.patch.to_lowercase().contains("drum") || track_data.patch.to_lowercase().contains("kit") || track_data.patch == "gm_kit";
        let channel = if is_drum {
            9 
        } else {
            if channel_allocator == 9 { channel_allocator = 10; }
            let ch = channel_allocator;
            channel_allocator = (channel_allocator + 1) % 16;
            ch
        };

        let program = parse_patch_name(&track_data.patch);
        abs_events.push(AbsEvent {
            tick: 0, priority: 0,
            kind: TrackEventKind::Midi { channel: channel.into(), message: MidiMessage::ProgramChange { program: program.into() } }
        });

        abs_events.push(AbsEvent {
            tick: 0, priority: 0,
            kind: TrackEventKind::Meta(MetaMessage::TrackName(track_data.label.as_bytes()))
        });

        // 2. Unroll IR Events into MIDI strikes
        for event in &track_data.events {
            // --- V3.0 SLICE 4: PHYSICAL TIME OFFSET ---
            let actual_start_tick = (event.tick as i64 + event.physical_tick_offset).max(0) as u64;
            
            // --- CONTINUOUS CONTROL ---
            if !event.cc_automations.is_empty() || event.tab_articulation != TabArticulation::None || event.synth_glide_ticks.is_some() || event.synth_accelerate_semitones.is_some() {
                
                // V3.0 SLICE 7: Determine the continuous tick span to interpolate across
                let sweep_duration = event.synth_glide_ticks.unwrap_or(event.duration_ticks);
                let resolution = 48; 
                let steps = (sweep_duration / resolution).max(1);
                
                for i in 0..=steps {
                    let current_tick = actual_start_tick + (i * sweep_duration / steps);
                    let progress = i as f32 / steps as f32;

                    for cc in &event.cc_automations {
                        let val = cc.start_val as f32 + (cc.end_val as f32 - cc.start_val as f32) * progress;
                        abs_events.push(AbsEvent {
                            tick: current_tick, priority: 1,
                            kind: TrackEventKind::Midi { channel: channel.into(), message: MidiMessage::Controller { controller: cc.controller.into(), value: (val as u8).min(127).into() } }
                        });
                    }

                    match event.tab_articulation {
                        TabArticulation::BendUp(target) => {
                            let bend_val = 8192.0 + (target * 8191.0 * progress);
                            abs_events.push(AbsEvent {
                                tick: current_tick, priority: 1,
                                kind: TrackEventKind::Midi { channel: channel.into(), message: MidiMessage::PitchBend { bend: midly::PitchBend(u14::from_int_lossy(bend_val.clamp(0.0, 16383.0) as u16)) } }
                            });
                        },
                        TabArticulation::BendDown(target) => {
                            let bend_val = 8192.0 + (target * 8191.0 * (1.0 - progress));
                            abs_events.push(AbsEvent {
                                tick: current_tick, priority: 1,
                                kind: TrackEventKind::Midi { channel: channel.into(), message: MidiMessage::PitchBend { bend: midly::PitchBend(u14::from_int_lossy(bend_val.clamp(0.0, 16383.0) as u16)) } }
                            });
                        },
                        _ => {}
                    }

                    // --- V3.0 SLICE 7: SYNTH GLIDE (Portamento) ---
                    // FIX: Prefixed `_glide_ticks` to safely ignore the warning
                    if let (Some(_glide_ticks), Some(start_midi), EventKind::Note { pitch_midi, .. }) = (event.synth_glide_ticks, event.synth_glide_start_midi, &event.kind) {
                        // Most DAWs assume a pitch bend range of 12 semitones (1 octave) for Synths
                        let pitch_bend_range: f32 = 12.0; 
                        let semitone_diff = start_midi as f32 - *pitch_midi as f32;
                        
                        // We start at the previous pitch (start_midi) and glide to the center (the current note)
                        let start_bend_val = 8192.0 + (semitone_diff / pitch_bend_range * 8191.0);
                        let current_bend_val = start_bend_val + ((8192.0 - start_bend_val) * progress);
                        
                        abs_events.push(AbsEvent {
                            tick: current_tick, priority: 1,
                            kind: TrackEventKind::Midi { channel: channel.into(), message: MidiMessage::PitchBend { bend: midly::PitchBend(u14::from_int_lossy(current_bend_val.clamp(0.0, 16383.0) as u16)) } }
                        });
                    }

                    // --- V3.0 SLICE 7: SYNTH ACCELERATE (Pitch Dive) ---
                    if let Some(target_semitones) = event.synth_accelerate_semitones {
                        let pitch_bend_range: f32 = 12.0;
                        let target_bend_val = 8192.0 + (target_semitones / pitch_bend_range * 8191.0);
                        let current_bend_val = 8192.0 + ((target_bend_val - 8192.0) * progress);

                        abs_events.push(AbsEvent {
                            tick: current_tick, priority: 1,
                            kind: TrackEventKind::Midi { channel: channel.into(), message: MidiMessage::PitchBend { bend: midly::PitchBend(u14::from_int_lossy(current_bend_val.clamp(0.0, 16383.0) as u16)) } }
                        });
                    }
                }
            }

            // --- ACOUSTIC TRIGGERS ---
            match &event.kind {
                EventKind::Note { pitch_midi, cents, velocity, spelling: _ } => {
                    let key = (*pitch_midi).min(127).into();
                    let vel = (*velocity).min(127).into();
                    let cents_val = *cents;

                    // Static microtonal detune
                    if cents_val != 0 && event.tab_articulation == TabArticulation::None && event.synth_glide_ticks.is_none() && event.synth_accelerate_semitones.is_none() {
                        let bend_val = 8192 + (cents_val as f32 * 8192.0 / 200.0) as i32;
                        abs_events.push(AbsEvent {
                            tick: actual_start_tick, priority: 1,
                            kind: TrackEventKind::Midi { channel: channel.into(), message: MidiMessage::PitchBend { bend: midly::PitchBend(u14::from_int_lossy(bend_val.clamp(0, 16383) as u16)) } },
                        });
                    }

                    if let Some(slashes) = event.tremolo_slashes {
                        let repeats = 1 << slashes; 
                        let sub_duration = event.gate_ticks / repeats;
                        
                        for i in 0..repeats {
                            let start = actual_start_tick + (i * sub_duration);
                            let end = start + sub_duration - 10; 
                            
                            abs_events.push(AbsEvent {
                                tick: start, priority: 3,
                                kind: TrackEventKind::Midi { channel: channel.into(), message: MidiMessage::NoteOn { key, vel } },
                            });
                            abs_events.push(AbsEvent {
                                tick: end, priority: 2,
                                kind: TrackEventKind::Midi { channel: channel.into(), message: MidiMessage::NoteOff { key, vel: 0.into() } },
                            });
                        }
                    } else {
                        // Standard note
                        abs_events.push(AbsEvent {
                            tick: actual_start_tick, priority: 3,
                            kind: TrackEventKind::Midi { channel: channel.into(), message: MidiMessage::NoteOn { key, vel } },
                        });

                        let off_tick = actual_start_tick + event.gate_ticks;
                        abs_events.push(AbsEvent {
                            tick: off_tick, priority: 2,
                            kind: TrackEventKind::Midi { channel: channel.into(), message: MidiMessage::NoteOff { key, vel: 0.into() } },
                        });
                    }

                    // Reset Pitch Bend after note finishes
                    if cents_val != 0 || event.tab_articulation != TabArticulation::None || event.synth_glide_ticks.is_some() || event.synth_accelerate_semitones.is_some() {
                        let off_tick = actual_start_tick + event.gate_ticks;
                        abs_events.push(AbsEvent {
                            tick: off_tick, priority: 4,
                            kind: TrackEventKind::Midi { channel: channel.into(), message: MidiMessage::PitchBend { bend: midly::PitchBend(u14::from_int_lossy(8192)) } },
                        });
                    }
                },
                EventKind::Frequency { .. } => {},
                EventKind::Rest => {},
                EventKind::Space => {}, 
                EventKind::Concrete { .. } => {}, // SLICE 6: Safely ignore concrete audio buffers in MIDI
            }
        }

        // 3. Sort chronologically
        abs_events.sort_by(|a, b| a.tick.cmp(&b.tick).then_with(|| a.priority.cmp(&b.priority)));

        // 4. Calculate Sequential Delta-Times
        let mut final_track = Vec::new();
        let mut current_tick = 0;

        for e in abs_events {
            let delta = (e.tick - current_tick) as u32;
            final_track.push(TrackEvent {
                delta: u28::from_int_lossy(delta),
                kind: e.kind,
            });
            current_tick = e.tick;
        }

        final_track.push(TrackEvent { delta: 0.into(), kind: TrackEventKind::Meta(MetaMessage::EndOfTrack) });
        smf.tracks.push(final_track);
    }

    let mut buffer = Vec::new();
    smf.write(&mut buffer)?;
    Ok(buffer)
}

fn parse_patch_name(name: &str) -> u8 {
    let n = name.to_lowercase();
    if n == "gm_piano" { return 0; }
    if n == "gm_epiano" { return 4; }
    if n == "gm_organ" { return 16; }
    if n == "gm_guitar" { return 24; }
    if n == "gm_bass" { return 32; }
    if n == "gm_violin" { return 40; }
    if n == "gm_strings" { return 48; }
    if n == "gm_choir" { return 52; }
    if n == "gm_trumpet" { return 56; }
    if n == "gm_sax" { return 65; }
    if n == "gm_flute" { return 73; }
    if n == "gm_kit" { return 0; } 

    if n.contains("piano") { 0 } else if n.contains("epiano") { 4 }
    else if n.contains("organ") { 16 } else if n.contains("guitar") { 24 }
    else if n.contains("bass") { 32 } else if n.contains("violin") { 40 }
    else if n.contains("strings") { 48 } else if n.contains("trumpet") { 56 }
    else if n.contains("sax") { 65 } else if n.contains("flute") { 73 }
    else { 0 } 
}