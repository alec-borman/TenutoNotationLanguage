use crate::ir::{Timeline, EventKind, TabArticulation};
use midly::{Smf, Header, Format, Timing, TrackEvent, TrackEventKind, MidiMessage, MetaMessage};
use midly::num::{u28, u14, u15};

// ============================================================================
// 1. TEMPORAL SORTING STRUCTURE
// ============================================================================
#[derive(Debug, Clone)]
struct AbsEvent<'a> {
    pub tick: u64,
    pub priority: u8, // 0=Meta, 1=CC/Bend, 2=NoteOff, 3=NoteOn, 4=BendReset
    pub kind: TrackEventKind<'a>,
}

// ============================================================================
// 2. THE MIDI ENCODER ENGINE (Tenuto 2.1.0 Compliant)
// ============================================================================

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
            match &event.kind {
                EventKind::Note { pitch_midi, cents, velocity, spelling: _ } => {
                    let key = (*pitch_midi).min(127).into();
                    let vel = (*velocity).min(127).into();
                    let cents_val = *cents;

                    // --- CONTINUOUS CONTROL (CC Automations & Tab Bends) ---
                    // Generates a high-res sweep of MIDI events every 48 ticks (approx 10ms)
                    if !event.cc_automations.is_empty() || event.tab_articulation != TabArticulation::None {
                        let resolution = 48; 
                        let steps = (event.duration_ticks / resolution).max(1);
                        
                        for i in 0..=steps {
                            let current_tick = event.tick + (i * event.duration_ticks / steps);
                            let progress = i as f32 / steps as f32; // 0.0 to 1.0

                            // 1. CC Automations
                            for cc in &event.cc_automations {
                                // Linear interpolation
                                let val = cc.start_val as f32 + (cc.end_val as f32 - cc.start_val as f32) * progress;
                                abs_events.push(AbsEvent {
                                    tick: current_tick, priority: 1,
                                    kind: TrackEventKind::Midi {
                                        channel: channel.into(),
                                        message: MidiMessage::Controller {
                                            controller: cc.controller.into(),
                                            value: (val as u8).min(127).into(),
                                        }
                                    }
                                });
                            }

                            // 2. Tablature Bends (Assumes standard GM pitch bend range is +/- 2 semitones = 1 whole step)
                            // Center = 8192. Max Up (1 whole step) = 16383. Max Down = 0.
                            match event.tab_articulation {
                                TabArticulation::BendUp(target) => {
                                    let bend_val = 8192.0 + (target * 8191.0 * progress);
                                    abs_events.push(AbsEvent {
                                        tick: current_tick, priority: 1,
                                        kind: TrackEventKind::Midi {
                                            channel: channel.into(),
                                            message: MidiMessage::PitchBend { bend: midly::PitchBend(u14::from_int_lossy(bend_val.clamp(0.0, 16383.0) as u16)) }
                                        }
                                    });
                                },
                                TabArticulation::BendDown(target) => {
                                    // Bend down releases back to center
                                    let bend_val = 8192.0 + (target * 8191.0 * (1.0 - progress));
                                    abs_events.push(AbsEvent {
                                        tick: current_tick, priority: 1,
                                        kind: TrackEventKind::Midi {
                                            channel: channel.into(),
                                            message: MidiMessage::PitchBend { bend: midly::PitchBend(u14::from_int_lossy(bend_val.clamp(0.0, 16383.0) as u16)) }
                                        }
                                    });
                                },
                                _ => {}
                            }
                        }
                    }

                    // --- STATIC MICROTONALITY ---
                    if cents_val != 0 && event.tab_articulation == TabArticulation::None {
                        let bend_val = 8192 + (cents_val as f32 * 8192.0 / 200.0) as i32;
                        abs_events.push(AbsEvent {
                            tick: event.tick, priority: 1,
                            kind: TrackEventKind::Midi {
                                channel: channel.into(),
                                message: MidiMessage::PitchBend { bend: midly::PitchBend(u14::from_int_lossy(bend_val.clamp(0, 16383) as u16)) },
                            },
                        });
                    }

                    // --- NOTE GENERATION & TREMOLO ROLLS ---
                    if let Some(slashes) = event.tremolo_slashes {
                        // Unroll the tremolo into rapid-fire notes!
                        // 1 slash = divide by 2. 2 slashes = divide by 4. 3 slashes = divide by 8.
                        let repeats = 1 << slashes; 
                        let sub_duration = event.gate_ticks / repeats;
                        
                        for i in 0..repeats {
                            let start = event.tick + (i * sub_duration);
                            let end = start + sub_duration - 10; // Slight 10-tick gap so NoteOff processes before next NoteOn
                            
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
                        // Standard Note Generation
                        abs_events.push(AbsEvent {
                            tick: event.tick, priority: 3,
                            kind: TrackEventKind::Midi { channel: channel.into(), message: MidiMessage::NoteOn { key, vel } },
                        });

                        let off_tick = event.tick + event.gate_ticks;
                        abs_events.push(AbsEvent {
                            tick: off_tick, priority: 2,
                            kind: TrackEventKind::Midi { channel: channel.into(), message: MidiMessage::NoteOff { key, vel: 0.into() } },
                        });
                    }

                    // Reset Pitch Bend after note finishes
                    if cents_val != 0 || event.tab_articulation != TabArticulation::None {
                        let off_tick = event.tick + event.gate_ticks;
                        abs_events.push(AbsEvent {
                            tick: off_tick, priority: 4,
                            kind: TrackEventKind::Midi {
                                channel: channel.into(),
                                message: MidiMessage::PitchBend { bend: midly::PitchBend(u14::from_int_lossy(8192)) },
                            },
                        });
                    }
                },
                EventKind::Frequency { .. } => {},
                EventKind::Rest => {},
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