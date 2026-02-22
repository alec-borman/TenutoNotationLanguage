use crate::ir::{Timeline, EventKind};
use midly::{Smf, Header, Format, Timing, TrackEvent, TrackEventKind, MidiMessage, MetaMessage};
use midly::num::{u28, u14, u15}; // FIXED: Added u15 to the imports

// ============================================================================
// 1. TEMPORAL SORTING STRUCTURE
// ============================================================================
#[derive(Debug, Clone)]
struct AbsEvent<'a> {
    pub tick: u64,
    pub priority: u8, // 0=Meta, 1=PitchBend, 2=NoteOff, 3=NoteOn, 4=BendReset
    pub kind: TrackEventKind<'a>,
}

// ============================================================================
// 2. THE MIDI ENCODER ENGINE (Tenuto 2.1.0 Compliant)
// ============================================================================

pub fn export(timeline: &Timeline) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Spec 27.2: Files SHOULD use a resolution of 480 PPQ or higher.
    // Tenuto native IR uses 1920 PPQ for perfect rational tuplet alignment.
    
    // FIXED: Safely cast the u32 PPQ to u16 so midly's u15 wrapper accepts it
    let safe_ppq = u15::from_int_lossy(timeline.ppq as u16);
    
    let header = Header::new(
        Format::Parallel,
        Timing::Metrical(safe_ppq), 
    );

    let mut smf = Smf::new(header);

    // ------------------------------------------------------------------------
    // TRACK 0: The Conductor Track (Tempo & Global Meta)
    // ------------------------------------------------------------------------
    let mut conductor_track = Vec::new();
    
    conductor_track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::TrackName(timeline.title.as_bytes())),
    });

    let mpq = 60_000_000 / timeline.tempo.max(1);
    conductor_track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(mpq.into())),
    });

    conductor_track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

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
        
        // Spec 27.2 & 23.3: Percussion maps MUST default to MIDI Channel 10 (Index 9)
        let is_drum = track_data.patch.to_lowercase().contains("drum") 
                   || track_data.patch.to_lowercase().contains("kit")
                   || track_data.patch == "gm_kit";

        let channel = if is_drum {
            9 // Channel 10 in 0-indexed MIDI
        } else {
            // Prevent melodic instruments from overwriting the percussion channel
            if channel_allocator == 9 { 
                channel_allocator = 10; 
            }
            let ch = channel_allocator;
            channel_allocator = (channel_allocator + 1) % 16;
            ch
        };

        // 1. Program Change Resolution (Spec 23.5)
        let program = parse_patch_name(&track_data.patch);
        abs_events.push(AbsEvent {
            tick: 0,
            priority: 0,
            kind: TrackEventKind::Midi {
                channel: channel.into(),
                message: MidiMessage::ProgramChange { program: program.into() },
            }
        });

        // Track Name Meta
        abs_events.push(AbsEvent {
            tick: 0,
            priority: 0,
            kind: TrackEventKind::Meta(MetaMessage::TrackName(track_data.label.as_bytes())),
        });

        // 2. Unroll IR Events into MIDI strikes
        for event in &track_data.events {
            match event.kind {
                EventKind::Note { pitch_midi, cents, velocity } => {
                    let key = pitch_midi.min(127).into();
                    let vel = velocity.min(127).into();

                    // V2.1 Spec 19: Microtonal Pitch Bend
                    // Assumes synth Pitch Bend Range is set to standard +/- 2 Semitones (200 cents)
                    if cents != 0 {
                        let bend_val = 8192 + (cents as f32 * 8192.0 / 200.0) as i32;
                        let clamped_bend = bend_val.clamp(0, 16383) as u16;
                        abs_events.push(AbsEvent {
                            tick: event.tick,
                            priority: 1, // Before NoteOn
                            kind: TrackEventKind::Midi {
                                channel: channel.into(),
                                message: MidiMessage::PitchBend { 
                                    bend: midly::PitchBend(u14::from_int_lossy(clamped_bend)) 
                                },
                            },
                        });
                    }

                    // Note On (Keyswitches, Grace notes, and Standard Notes all hit here)
                    abs_events.push(AbsEvent {
                        tick: event.tick,
                        priority: 3,
                        kind: TrackEventKind::Midi {
                            channel: channel.into(),
                            message: MidiMessage::NoteOn { key, vel },
                        },
                    });

                    // Note Off (Physical duration dictated by articulation gate_ticks)
                    let off_tick = event.tick + event.gate_ticks;
                    abs_events.push(AbsEvent {
                        tick: off_tick,
                        priority: 2,
                        kind: TrackEventKind::Midi {
                            channel: channel.into(),
                            message: MidiMessage::NoteOff { key, vel: 0.into() },
                        },
                    });

                    // Pitch Bend Reset (Immediately following NoteOff to prevent smearing)
                    if cents != 0 {
                        abs_events.push(AbsEvent {
                            tick: off_tick,
                            priority: 4,
                            kind: TrackEventKind::Midi {
                                channel: channel.into(),
                                message: MidiMessage::PitchBend { 
                                    bend: midly::PitchBend(u14::from_int_lossy(8192)) 
                                },
                            },
                        });
                    }
                },
                EventKind::Frequency { .. } => { 
                    /* Spec 19.4: Future Native MPE implementation for exact Hz assignment */ 
                },
                EventKind::Rest => {
                    /* Rests are logically preserved in IR but emit no explicit MIDI payload */
                },
            }
        }

        // 3. Sort chronologically, relying on priority for identical tick resolution
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

        final_track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });

        smf.tracks.push(final_track);
    }

    let mut buffer = Vec::new();
    smf.write(&mut buffer)?;
    Ok(buffer)
}

/// Resolves standard text definitions to 0-127 General MIDI patches
/// Maps directly to the V2.1 Spec Section 23.5 Standard Constants.
fn parse_patch_name(name: &str) -> u8 {
    let n = name.to_lowercase();
    
    // Exact V2.1 Spec Constants
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
    if n == "gm_kit" { return 0; } // Channel 10 default handles the mapping natively

    // Graceful fuzzy fallback for generic string declarations
    if n.contains("piano") { 0 }
    else if n.contains("epiano") { 4 }
    else if n.contains("organ") { 16 }
    else if n.contains("guitar") { 24 }
    else if n.contains("bass") { 32 }
    else if n.contains("violin") { 40 }
    else if n.contains("strings") { 48 }
    else if n.contains("trumpet") { 56 }
    else if n.contains("sax") { 65 }
    else if n.contains("flute") { 73 }
    else { 0 } // Default Acoustic Grand
}