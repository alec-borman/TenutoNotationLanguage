use crate::ir::{Timeline, EventKind};
use midly::{Smf, Header, Format, Timing, Track, TrackEvent, TrackEventKind, MidiMessage, MetaMessage};
use midly::num::u28;

pub fn export(timeline: &Timeline) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // 1. Create MIDI Header
    // Tenuto uses 1920 PPQ internally. We map this directly to MIDI PPQ.
    let header = Header::new(
        Format::Parallel, // Type 1: Multiple tracks played simultaneously
        Timing::Metrical(1920.into()), 
    );

    let mut smf = Smf::new(header);

    // 2. Create Conductor Track (Track 0)
    // Contains Tempo, Time Signature, and Title
    let mut conductor_track = Vec::new();
    
    // Title
    conductor_track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::TrackName(timeline.title.as_bytes())),
    });

    // Tempo: Convert BPM to Microseconds per Quarter Note
    // Formula: 60,000,000 / BPM
    let mpq = 60_000_000 / timeline.tempo;
    conductor_track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(mpq.into())),
    });

    // End of Track
    conductor_track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    smf.tracks.push(conductor_track);

    // 3. Process Instrument Tracks
    // Sort keys to ensure deterministic output
    let mut sorted_keys: Vec<_> = timeline.tracks.keys().collect();
    sorted_keys.sort();

    for (idx, key) in sorted_keys.iter().enumerate() {
        let tenuto_track = &timeline.tracks[*key];
        let mut midi_events = Vec::new();
        
        // Channel logic: 0-15. Percussion usually 9 (10 in 1-based).
        // Simple auto-assignment loop, skipping 9 unless explicitly percussion.
        let channel = (idx % 16) as u8; 

        // A. Set Instrument Patch (Program Change)
        // Simple mapping: default to Grand Piano (0) if parsing fails
        let program = parse_patch_name(&tenuto_track.patch);
        midi_events.push(TempEvent {
            tick: 0,
            kind: TrackEventKind::Midi {
                channel: channel.into(),
                message: MidiMessage::ProgramChange { program: program.into() },
            }
        });

        // B. Explode Note Durations into On/Off pairs
        for event in &tenuto_track.events {
            match event.kind {
                EventKind::Note { pitch, velocity } => {
                    // Note On
                    midi_events.push(TempEvent {
                        tick: event.tick,
                        kind: TrackEventKind::Midi {
                            channel: channel.into(),
                            message: MidiMessage::NoteOn { 
                                key: pitch.into(), 
                                vel: velocity.into() 
                            },
                        }
                    });

                    // Note Off (at start + duration)
                    midi_events.push(TempEvent {
                        tick: event.tick + event.duration_ticks,
                        kind: TrackEventKind::Midi {
                            channel: channel.into(),
                            message: MidiMessage::NoteOff { 
                                key: pitch.into(), 
                                vel: 0.into() 
                            },
                        }
                    });
                },
                _ => {} // Rests are implicit in MIDI (gap between events)
            }
        }

        // C. Sort by absolute tick to prepare for Delta calculation
        midi_events.sort_by(|a, b| a.tick.cmp(&b.tick));

        // D. Convert to Delta Time
        let mut final_track = Vec::new();
        let mut current_tick = 0;

        for e in midi_events {
            let delta = e.tick - current_tick;
            
            // midly uses u28 for deltas. Ensure we don't overflow (unlikely in music).
            let delta_u28 = u28::from_int_lossy(delta as u32);

            final_track.push(TrackEvent {
                delta: delta_u28,
                kind: e.kind,
            });

            current_tick = e.tick;
        }

        // End of Track
        final_track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });

        smf.tracks.push(final_track);
    }

    // 4. Serialize to Bytes
    let mut buffer = Vec::new();
    smf.write(&mut buffer)?;
    Ok(buffer)
}

// Temporary struct for sorting before calculating Deltas
struct TempEvent<'a> {
    tick: u64,
    kind: TrackEventKind<'a>,
}

// Helper to map string names to MIDI Program Numbers (0-127)
fn parse_patch_name(name: &str) -> u8 {
    let n = name.to_lowercase();
    if n.contains("piano") { return 0; }
    if n.contains("violin") { return 40; }
    if n.contains("viola") { return 41; }
    if n.contains("cello") { return 42; }
    if n.contains("guitar") { return 24; }
    if n.contains("bass") { return 32; }
    if n.contains("flute") { return 73; }
    if n.contains("drum") || n.contains("kit") { return 0; } // Drums use Channel 10, prog doesn't matter much
    0 // Default
}