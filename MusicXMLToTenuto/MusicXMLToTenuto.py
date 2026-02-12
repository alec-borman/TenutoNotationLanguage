#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Tenuto Transducer (xml2ten) v1.2 [Grand Staff Support]
The Architect Ant: Handles Piano/Grand Staff separation automatically.

Paradigm:
1. Analyze Parts (Detect Staves count).
2. Generate Physics (Groups for Grand Staves).
3. Route Events (Split by Staff ID).
4. Normalize Voices (v1 for RH, v1 for LH).
"""
import zipfile
import xml.etree.ElementTree as ET
import sys
import argparse
import math
from collections import defaultdict

# --- 1. THE PHYSICS (Constants & Mappings) ---

DYNAMICS_MAP = {
    'p': 'p', 'pp': 'pp', 'ppp': 'ppp', 'pppp': 'pppp',
    'f': 'f', 'ff': 'ff', 'fff': 'fff', 'ffff': 'ffff',
    'mp': 'mp', 'mf': 'mf', 'sf': 'sfz', 'sfz': 'sfz', 'fp': 'fp'
}

ARTICULATION_MAP = {
    'staccato': 'stacc',
    'tenuto': 'ten',
    'accent': 'acc',
    'strong-accent': 'marc',
    'staccatissimo': 'stacciss'
}

# Base resolution for float comparison
EPSILON = 0.001

class TenutoTransducer:
    def __init__(self, input_file):
        self.output = []
        self.state = {} 
        
        # Mapping: xml_part_id -> { 'type': 'single'|'grand', 'map': {staff_idx: tenuto_id} }
        self.part_config = {}

        # --- THE SHELL CRACKER (MXL Support) ---
        if input_file.endswith('.mxl'):
            try:
                with zipfile.ZipFile(input_file, 'r') as z:
                    xml_files = [n for n in z.namelist() if n.endswith('.xml') and not n.startswith('META-INF')]
                    if not xml_files:
                        raise ValueError("No XML found in .mxl archive.")
                    with z.open(xml_files[0]) as f:
                        self.tree = ET.parse(f)
            except zipfile.BadZipFile:
                sys.stderr.write("[Error] File is not a valid zip archive.\n")
                sys.exit(1)
        else:
            self.tree = ET.parse(input_file)
            
        self.root = self.tree.getroot()

    def log(self, msg):
        sys.stderr.write(f"[Transducer] {msg}\n")

    # --- 2. THE ARCHITECT (Analysis & Physics) ---

    def analyze_structure(self):
        """Pre-scan to detect Grand Staves (Piano)."""
        # Get list of parts from header
        part_list = self.root.find('part-list')
        defined_parts = {}
        if part_list is not None:
            for sp in part_list.findall('score-part'):
                defined_parts[sp.get('id')] = sp.find('part-name').text if sp.find('part-name') is not None else "Inst"

        # Scan actual parts for <staves>
        for xml_part in self.root.findall('part'):
            pid = xml_part.get('id')
            safe_id = pid.replace('-', '_').lower()
            name = defined_parts.get(pid, "Unknown")
            
            # Check first measure for attributes
            m1 = xml_part.find('measure')
            staves = 1
            if m1 is not None:
                attrs = m1.find('attributes')
                if attrs is not None:
                    s_tag = attrs.find('staves')
                    if s_tag is not None:
                        staves = int(s_tag.text)
            
            # Build Configuration
            if staves > 1:
                self.part_config[pid] = {
                    'type': 'grand',
                    'name': name,
                    'map': {
                        1: f"{safe_id}_rh",
                        2: f"{safe_id}_lh"
                    }
                }
                # Initialize state for both hands
                self.state[f"{safe_id}_rh"] = {'octave': 4, 'duration': ':4'}
                self.state[f"{safe_id}_lh"] = {'octave': 3, 'duration': ':4'}
            else:
                self.part_config[pid] = {
                    'type': 'single',
                    'name': name,
                    'map': { 1: safe_id }
                }
                self.state[safe_id] = {'octave': 4, 'duration': ':4'}

    def extract_physics(self):
        """Emit Definitions based on Analysis."""
        self.output.append("tenuto {")
        
        # Meta
        work = self.root.find('work')
        title = work.find('work-title').text if work is not None and work.find('work-title') is not None else "Untitled"
        self.output.append(f'  meta {{ title: "{title}", tenuto_version: "2.0" }}')
        self.output.append("")

        # Defs
        for pid, config in self.part_config.items():
            if config['type'] == 'grand':
                # Group for Piano
                self.output.append(f'  group "{config["name"]}" symbol=brace {{')
                self.output.append(f'    def {config["map"][1]} "Right Hand" style=standard clef=treble')
                self.output.append(f'    def {config["map"][2]} "Left Hand"  style=standard clef=bass')
                self.output.append('  }')
            else:
                safe_id = config['map'][1]
                self.output.append(f'  def {safe_id} "{config["name"]}" style=standard')
        
        self.output.append("")

    # --- 3. THE WEAVER (Logic Construction) ---

    def transmute(self):
        self.analyze_structure()
        self.extract_physics()

        # Matrix: measure -> tenuto_id -> logic_string
        score_matrix = defaultdict(dict)
        
        xml_parts = self.root.findall('part')
        for xp in xml_parts:
            self.process_part(xp, xp.get('id'), score_matrix)

        # Output
        sorted_measures = sorted(score_matrix.keys())
        for m_num in sorted_measures:
            self.output.append(f"  measure {m_num} {{")
            
            # Global Meta (Time/Key from first RH part found)
            # Find a valid RH ID to pull meta from
            meta_source = None
            for pid, conf in self.part_config.items():
                target = conf['map'][1] # RH or Single
                if target in score_matrix[m_num]:
                    meta_source = target
                    break
            
            if meta_source and 'meta' in score_matrix[m_num][meta_source]:
                meta_events = score_matrix[m_num][meta_source]['meta']
                if meta_events:
                    self.output.append(f"    meta {{ {', '.join(meta_events)} }}")

            # Part Logic
            # Sort IDs to keep RH/LH together
            active_ids = sorted([k for k in score_matrix[m_num].keys()])
            for tid in active_ids:
                logic = score_matrix[m_num][tid]['logic']
                self.output.append(f"    {tid}: {logic} |")
            
            self.output.append("  }")
        
        self.output.append("}")
        return "\n".join(self.output)

    def process_part(self, xml_part, pid, score_matrix):
        config = self.part_config[pid]
        divisions = 1 

        for m in xml_part.findall('measure'):
            try:
                m_num = int(m.get('number'))
            except: continue

            # Meta Collection
            attrs = m.find('attributes')
            measure_meta = []
            if attrs is not None:
                div_tag = attrs.find('divisions')
                if div_tag is not None: divisions = int(div_tag.text)
                
                # Extract Time/Key/Tempo (same as before)
                time = attrs.find('time')
                if time is not None:
                    beats = time.find('beats').text
                    btype = time.find('beat-type').text
                    measure_meta.append(f"time: {beats}/{btype}")
                
                key = attrs.find('key')
                if key is not None:
                    f = int(key.find('fifths').text)
                    kmap = {0:'C',1:'G',2:'D',3:'A',4:'E',5:'B',6:'F#',-1:'F',-2:'Bb',-3:'Eb'}
                    if f in kmap: measure_meta.append(f'key: "{kmap[f]}"')

            # --- VOICE ROUTING ---
            # Bucket: { tenuto_id: { xml_voice_id: [events] } }
            hand_buckets = defaultdict(lambda: defaultdict(list))
            
            current_voice = '1'
            current_staff = 1 # Default to RH

            for child in m:
                tag = child.tag
                
                if tag == 'note':
                    # Check Staff
                    staff_tag = child.find('staff')
                    if staff_tag is not None:
                        current_staff = int(staff_tag.text)
                    else:
                        current_staff = 1 # Default
                    
                    # Check Voice
                    v_tag = child.find('voice')
                    if v_tag is not None:
                        current_voice = v_tag.text
                    
                    # Route to correct Tenuto ID
                    target_id = config['map'].get(current_staff, config['map'][1])
                    
                    token = self.parse_note(child, divisions, target_id)
                    
                    # Chord Logic
                    bucket = hand_buckets[target_id][current_voice]
                    if child.find('chord') is not None and bucket and bucket[-1]['type'] == 'note':
                        bucket[-1]['chord_notes'].append(token)
                    else:
                        bucket.append({'type': 'note', 'chord_notes': [token], 'tuplet_info': token.get('tuplet_info')})

                elif tag == 'backup': pass # Implicit handling
                elif tag == 'forward': pass

            # --- RENDER BY HAND ---
            for tid, voice_dict in hand_buckets.items():
                # Normalize Voices: Sort XML voices and map to v1, v2...
                # This ensures RH gets v1, and LH gets v1 (instead of v5)
                sorted_xml_voices = sorted(voice_dict.keys())
                
                rendered_parts = []
                
                # If single voice, no brace needed
                if len(sorted_xml_voices) == 1:
                    logic = self.render_voice_stream(voice_dict[sorted_xml_voices[0]], tid)
                    final = f"{{ v1: {logic} }}" # Always explicit voice group for consistency? Or simple?
                    # Let's use simple if single, explicit if poly
                    final = logic # Simple
                else:
                    # Polyphony inside the hand
                    blocks = []
                    for idx, xv in enumerate(sorted_xml_voices):
                        tenuto_v_num = idx + 1
                        logic = self.render_voice_stream(voice_dict[xv], tid, force_reset=(tenuto_v_num > 1))
                        blocks.append(f"v{tenuto_v_num}: {logic}")
                    final = f"{{ {' | '.join(blocks)} }}"

                # Store result
                score_matrix[m_num][tid] = {'logic': final}
                
                # Attach Meta to RH only
                if tid == config['map'][1]:
                    score_matrix[m_num][tid]['meta'] = measure_meta

    # --- 4. ATOMIZER & COMPRESSOR (Same as v1.1) ---
    # (Copied from previous artifact, ensuring self.state access uses tid)

    def parse_note(self, note_elem, divisions, tid):
        # ... [Logic identical to v1.1, just pass] ...
        # Copied verbatim for completeness in execution
        data = {'attr': [], 'suffix': ''}
        dur_elem = note_elem.find('duration')
        if dur_elem is not None:
            try:
                raw_dur = int(dur_elem.text)
                quarters = raw_dur / divisions
                data['quarters'] = quarters
                data['dur_token'] = self.quarters_to_tenuto(quarters)
            except: data.update({'quarters':0, 'dur_token':''})
        else:
            if note_elem.find('grace') is not None: data.update({'quarters':0, 'dur_token':':grace'})
            else: data.update({'quarters':0, 'dur_token':''})

        if note_elem.find('rest') is not None:
            data.update({'step':'r', 'octave':None, 'acc':''})
        else:
            pitch = note_elem.find('pitch')
            if pitch is not None:
                step = pitch.find('step').text.lower()
                octave = int(pitch.find('octave').text)
                alter = pitch.find('alter')
                acc = ''
                if alter is not None:
                    try:
                        v = float(alter.text)
                        if v==1: acc='#'
                        elif v==-1: acc='b'
                    except: pass
                data.update({'step':step, 'octave':octave, 'acc':acc})
            else:
                data.update({'step':'x', 'octave':4, 'acc':''})

        notations = note_elem.find('notations')
        if notations is not None:
            if notations.find('tied') is not None: 
                if notations.find('tied').get('type')=='start': data['suffix']='~'
            arts = notations.find('articulations')
            if arts is not None:
                for a in arts:
                    if a.tag in ARTICULATION_MAP: data['attr'].append(f".{ARTICULATION_MAP[a.tag]}")
            
            tmod = note_elem.find('time-modification')
            if tmod is not None:
                try:
                    data['tuplet_info'] = (int(tmod.find('actual-notes').text), int(tmod.find('normal-notes').text))
                except: data['tuplet_info']=None
            else: data['tuplet_info']=None
        else: data['tuplet_info']=None
        
        return data

    def quarters_to_tenuto(self, q):
        if q == 0: return ""
        if q > 0:
            un_dotted = q / 1.5
            if un_dotted > 0:
                dd = 4/un_dotted
                if abs(dd-round(dd)) < EPSILON: return f":{int(round(dd))}."
        denom = 4/q
        return f":{int(round(denom))}"

    def render_voice_stream(self, events, tid, force_reset=False):
        if not events: return ""
        prev_dur = self.state[tid]['duration'] if not force_reset else None
        prev_oct = self.state[tid]['octave'] if not force_reset else None
        
        tokens = []
        tuplet_buffer = []
        current_tuplet_ratio = None
        
        def flush_tuplet():
            nonlocal tuplet_buffer, current_tuplet_ratio
            if not tuplet_buffer: return
            tokens.append(f"({' '.join(tuplet_buffer)}):{current_tuplet_ratio[0]}/{current_tuplet_ratio[1]}")
            tuplet_buffer = []
            current_tuplet_ratio = None

        for evt in events:
            t_info = evt['tuplet_info']
            if t_info:
                if current_tuplet_ratio and t_info != current_tuplet_ratio: flush_tuplet()
                current_tuplet_ratio = t_info
            else:
                if current_tuplet_ratio: flush_tuplet()
            
            head = evt['chord_notes'][0]
            token_str = ""
            if head['step'] == 'r': token_str = "r"
            else:
                if len(evt['chord_notes']) > 1:
                    ins = []
                    for n in evt['chord_notes']:
                        s = f"{n['step']}{n['acc']}"
                        if n['octave'] != prev_oct: s += str(n['octave'])
                        ins.append(s)
                    token_str = f"[{' '.join(ins)}]"
                    prev_oct = evt['chord_notes'][-1]['octave']
                else:
                    token_str = f"{head['step']}{head['acc']}"
                    if head['octave'] != prev_oct:
                        token_str += str(head['octave'])
                        prev_oct = head['octave']

            d_tok = head['dur_token']
            if d_tok != prev_dur:
                token_str += d_tok
                prev_dur = d_tok
            
            for a in head['attr']: token_str += a
            token_str += head['suffix']

            if current_tuplet_ratio: tuplet_buffer.append(token_str)
            else: tokens.append(token_str)
        
        flush_tuplet()
        if not force_reset:
            self.state[tid]['duration'] = prev_dur
            self.state[tid]['octave'] = prev_oct
        return " ".join(tokens)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='MusicXML to Tenuto Transducer v1.2')
    parser.add_argument('input', help='Input MusicXML file')
    args = parser.parse_args()
    print(TenutoTransducer(args.input).transmute())
