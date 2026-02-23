//! # Tenuto MusicXML 4.0 Exporter
//! 
//! Transforms the `VisualScore` IR into valid MusicXML. 
//! Implements strict Polyphony rewinding (<backup>), Chord detection, 
//! and Visual Type inference (<type> and <dot/>).

use crate::rebar::{VisualScore, VisualEvent};
use crate::ir::EventKind;
use crate::spelling::AccidentalDisplay;

// ============================================================================
// 1. PUBLIC EXPORTER API
// ============================================================================

pub fn export(score: &VisualScore, ppq: u32) -> Result<String, String> {
    let mut xml = String::with_capacity(1024 * 100);

    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<!DOCTYPE score-partwise PUBLIC \"-//Recordare//DTD MusicXML 4.0 Partwise//EN\" \"http://www.musicxml.org/dtds/partwise.dtd\">\n");
    xml.push_str("<score-partwise version=\"4.0\">\n");

    xml.push_str("  <work>\n");
    xml.push_str(&format!("    <work-title>{}</work-title>\n", escape_xml(&score.title)));
    xml.push_str("  </work>\n");

    let mut staves: Vec<_> = score.staves.iter().collect();
    staves.sort_by_key(|(id, _)| *id);

    xml.push_str("  <part-list>\n");
    for (idx, (staff_id, _)) in staves.iter().enumerate() {
        let part_id = format!("P{}", idx + 1);
        xml.push_str(&format!("    <score-part id=\"{}\">\n", part_id));
        xml.push_str(&format!("      <part-name>{}</part-name>\n", escape_xml(staff_id)));
        xml.push_str("    </score-part>\n");
    }
    xml.push_str("  </part-list>\n");

    for (idx, (_, staff)) in staves.iter().enumerate() {
        let part_id = format!("P{}", idx + 1);
        xml.push_str(&format!("  <part id=\"{}\">\n", part_id));

        for (m_idx, measure) in staff.measures.iter().enumerate() {
            xml.push_str(&format!("    <measure number=\"{}\">\n", m_idx + 1));

            if m_idx == 0 {
                xml.push_str("      <attributes>\n");
                xml.push_str(&format!("        <divisions>{}</divisions>\n", ppq));
                xml.push_str("        <key><fifths>0</fifths></key>\n");
                xml.push_str("        <time>\n");
                xml.push_str(&format!("          <beats>{}</beats>\n", measure.time_signature.numerator));
                xml.push_str(&format!("          <beat-type>{}</beat-type>\n", measure.time_signature.denominator));
                xml.push_str("        </time>\n");
                xml.push_str("        <clef><sign>G</sign><line>2</line></clef>\n");
                xml.push_str("      </attributes>\n");
            }

            // --- V2.1 POLYPHONY & CHORD ORCHESTRATOR ---
            let mut current_xml_tick = measure.start_tick;
            let mut last_tick = None;
            let mut current_voice = 1;

            for event in &measure.events {
                let is_chord = last_tick == Some(event.atomic.tick);

                if !is_chord {
                    if event.atomic.tick < current_xml_tick {
                        let backup = current_xml_tick - event.atomic.tick;
                        xml.push_str(&format!("      <backup>\n        <duration>{}</duration>\n      </backup>\n", backup));
                        current_xml_tick = event.atomic.tick;
                        current_voice += 1; // Increment voice for polyphonic rewind
                    } else if event.atomic.tick > current_xml_tick {
                        let forward = event.atomic.tick - current_xml_tick;
                        xml.push_str(&format!("      <forward>\n        <duration>{}</duration>\n      </forward>\n", forward));
                        current_xml_tick = event.atomic.tick;
                    }
                }

                write_event(&mut xml, event, ppq, is_chord, current_voice);
                
                if !is_chord {
                    current_xml_tick += event.atomic.duration_ticks;
                }
                last_tick = Some(event.atomic.tick);
            }

            xml.push_str("    </measure>\n");
        }
        xml.push_str("  </part>\n");
    }

    xml.push_str("</score-partwise>\n");
    Ok(xml)
}

fn escape_xml(input: &str) -> String {
    input.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;")
}

// ============================================================================
// 2. THE EVENT WRITER
// ============================================================================

fn write_event(xml: &mut String, event: &VisualEvent, ppq: u32, is_chord: bool, voice: u32) {
    xml.push_str("      <note>\n");

    if is_chord {
        xml.push_str("        <chord/>\n");
    }

    match &event.atomic.kind {
        EventKind::Rest => { xml.push_str("        <rest/>\n"); },
        EventKind::Note { spelling, .. } => {
            xml.push_str("        <pitch>\n");
            xml.push_str(&format!("          <step>{}</step>\n", spelling.step));
            let total_alter = spelling.alter as f32 + spelling.micro_alter;
            if total_alter != 0.0 { xml.push_str(&format!("          <alter>{}</alter>\n", total_alter)); }
            xml.push_str(&format!("          <octave>{}</octave>\n", spelling.octave));
            xml.push_str("        </pitch>\n");
        },
        EventKind::Frequency { .. } => { xml.push_str("        <rest/>\n"); }
    }

    xml.push_str(&format!("        <duration>{}</duration>\n", event.atomic.duration_ticks));
    
    if event.tie_stop { xml.push_str("        <tie type=\"stop\"/>\n"); }
    if event.tie_start { xml.push_str("        <tie type=\"start\"/>\n"); }

    xml.push_str(&format!("        <voice>{}</voice>\n", voice));

    // Calculate `<type>` and `<dot/>` visually
    write_type_and_dots(xml, event, ppq);

    if let Some(ts) = &event.atomic.tuplet_state {
        xml.push_str("        <time-modification>\n");
        xml.push_str(&format!("          <actual-notes>{}</actual-notes>\n", ts.actual_notes));
        xml.push_str(&format!("          <normal-notes>{}</normal-notes>\n", ts.normal_notes));
        xml.push_str("        </time-modification>\n");
    }

    let has_accidental = if let EventKind::Note { spelling, .. } = &event.atomic.kind { spelling.display != AccidentalDisplay::Implicit } else { false };
    let has_tuplet_bracket = event.atomic.tuplet_state.as_ref().map_or(false, |ts| ts.is_start || ts.is_stop);
    let has_notations = event.tie_start || event.tie_stop || has_accidental || has_tuplet_bracket;

    if has_notations {
        xml.push_str("        <notations>\n");

        if event.tie_stop { xml.push_str("          <tied type=\"stop\"/>\n"); }
        if event.tie_start { xml.push_str("          <tied type=\"start\"/>\n"); }

        if let Some(ts) = &event.atomic.tuplet_state {
            if ts.is_start { xml.push_str("          <tuplet type=\"start\" bracket=\"yes\"/>\n"); }
            if ts.is_stop { xml.push_str("          <tuplet type=\"stop\"/>\n"); }
        }

        if let EventKind::Note { spelling, .. } = &event.atomic.kind {
            if spelling.display != AccidentalDisplay::Implicit {
                let acc_str = match (spelling.alter, spelling.micro_alter) {
                    (0, 0.0) => "natural", (1, 0.0) => "sharp", (-1, 0.0) => "flat",
                    (2, 0.0) => "double-sharp", (-2, 0.0) => "flat-flat",
                    _ => "natural",
                };
                let p = if spelling.display == AccidentalDisplay::Courtesy { " parentheses=\"yes\"" } else { "" };
                xml.push_str(&format!("          <accidental{}>{}</accidental>\n", p, acc_str));
            }
        }

        xml.push_str("        </notations>\n");
    }

    xml.push_str("      </note>\n");
}

/// Reverse-engineers duration ticks back into graphical note types and dots
fn write_type_and_dots(xml: &mut String, event: &VisualEvent, ppq: u32) {
    let mut ticks = event.atomic.duration_ticks;
    
    // Undo tuplet scaling to find the base visual note type
    if let Some(ts) = &event.atomic.tuplet_state {
        ticks = (ticks * ts.actual_notes) / ts.normal_notes;
    }

    // FIXED: Explicitly cast ppq to u64 so it can be compared against ticks
    let ppq64 = ppq as u64;
    let q = ppq64;
    let e = ppq64 / 2;
    let s = ppq64 / 4;
    let t = ppq64 / 8;
    let h = ppq64 * 2;
    let w = ppq64 * 4;

    let (type_str, dots) = if ticks == w { ("whole", 0) }
    else if ticks == w + h { ("whole", 1) }
    else if ticks == h { ("half", 0) }
    else if ticks == h + q { ("half", 1) }
    else if ticks == h + q + e { ("half", 2) }
    else if ticks == q { ("quarter", 0) }
    else if ticks == q + e { ("quarter", 1) }
    else if ticks == q + e + s { ("quarter", 2) }
    else if ticks == e { ("eighth", 0) }
    else if ticks == e + s { ("eighth", 1) }
    else if ticks == s { ("16th", 0) }
    else if ticks == s + t { ("16th", 1) }
    else if ticks == t { ("32nd", 0) }
    else { ("", 0) }; // Fallback if perfectly irregular

    if !type_str.is_empty() {
        xml.push_str(&format!("        <type>{}</type>\n", type_str));
        for _ in 0..dots {
            xml.push_str("        <dot/>\n");
        }
    }
}