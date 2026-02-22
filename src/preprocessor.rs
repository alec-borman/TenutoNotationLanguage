use std::collections::HashMap;
use crate::ast::*;

const MAX_RECURSION_DEPTH: usize = 64; // Spec 15.4

pub struct Preprocessor {
    variables: HashMap<String, Value>,
    macros: HashMap<String, TopLevel>, // Stores TopLevel::MacroDef
    depth: usize,
}

impl Preprocessor {
    pub fn new(mut initial_env: HashMap<String, Value>) -> Self {
        // Spec 22.4: Default environment variables for conditional compilation
        initial_env.entry("target".into()).or_insert(Value::Str("audio".into()));
        initial_env.entry("debug".into()).or_insert(Value::Bool(false));

        Self {
            variables: initial_env,
            macros: HashMap::new(),
            depth: 0,
        }
    }

    pub fn expand(&mut self, mut score: Score) -> Result<Score, String> {
        let expanded_items = self.process_top_levels(score.items)?;
        score.items = expanded_items;
        Ok(score)
    }

    fn process_top_levels(&mut self, items: Vec<TopLevel>) -> Result<Vec<TopLevel>, String> {
        let mut result = Vec::new();
        for item in items {
            match item {
                TopLevel::VariableDecl { name, value } => {
                    let resolved_value = self.resolve_value(&value)?;
                    self.variables.insert(name, resolved_value);
                }
                TopLevel::MacroDef { name, args, body } => {
                    if self.macros.contains_key(&name) {
                        return Err(format!("E2002: Duplicate Definition for macro '{}'", name));
                    }
                    self.macros.insert(name.clone(), TopLevel::MacroDef { name, args, body });
                }
                TopLevel::Condition { expression, content } => {
                    if self.evaluate_condition(&expression)? {
                        let mut inner = self.process_top_levels(content)?;
                        result.append(&mut inner);
                    }
                }
                // V2.1: Resolve variables inside Global Meta maps (@{})
                TopLevel::Meta(map) => {
                    let resolved_map = self.resolve_map(map)?;
                    result.push(TopLevel::Meta(resolved_map));
                }
                // V2.1: Resolve variables inside Def attributes
                TopLevel::Def { id, label, attributes } => {
                    let resolved_attrs = self.resolve_map(attributes)?;
                    result.push(TopLevel::Def { id, label, attributes: resolved_attrs });
                }
                // V2.1: Resolve variables inside Measure attributes (local meta)
                TopLevel::Measure { range, attributes, content } => {
                    let resolved_attrs = self.resolve_map(attributes)?;
                    let expanded_content = self.process_logics(content)?;
                    result.push(TopLevel::Measure { range, attributes: resolved_attrs, content: expanded_content });
                }
                TopLevel::Group { label, attributes, items } => {
                    let resolved_attrs = self.resolve_map(attributes)?;
                    let expanded_items = self.process_top_levels(items)?;
                    result.push(TopLevel::Group { label, attributes: resolved_attrs, items: expanded_items });
                }
                other => result.push(other),
            }
        }
        Ok(result)
    }

    fn process_logics(&mut self, logics: Vec<Logic>) -> Result<Vec<Logic>, String> {
        let mut result = Vec::new();
        for logic in logics {
            match logic {
                Logic::Condition { expression, content } => {
                    if self.evaluate_condition(&expression)? {
                        let mut inner = self.process_logics(content)?;
                        result.append(&mut inner);
                    }
                }
                Logic::Assignment { staff_id, voices } => {
                    let mut expanded_voices = Vec::new();
                    // V2.1: Handles `<[ ]>` polyphony implicitly via vec iteration
                    for voice in voices {
                        expanded_voices.push(self.process_voice(voice)?);
                    }
                    result.push(Logic::Assignment { staff_id, voices: expanded_voices });
                }
                other => result.push(other),
            }
        }
        Ok(result)
    }

    fn process_voice(&mut self, mut voice: Voice) -> Result<Voice, String> {
        let mut expanded_events = Vec::new();

        for mut event in voice.events {
            // V2.1: Recursively expand variables provided to dot attributes (e.g., .vol($Var))
            self.resolve_event_attributes(&mut event)?;

            match event {
                Event::MacroCall { name, args, transpose, duration, dots, multiplier, attributes } => {
                    self.depth += 1;
                    if self.depth > MAX_RECURSION_DEPTH {
                        return Err(format!("E5002: Recursion Limit Exceeded (>{}) in macro '{}'", MAX_RECURSION_DEPTH, name));
                    }

                    // ----- FIX: Resolve macro name if it's a variable (e.g., $root:16) -----
                    let actual_name = if name.starts_with('$') {
                        let var_name = &name[1..];
                        match self.variables.get(var_name) {
                            Some(Value::Id(s)) | Some(Value::Str(s)) => s.clone(),
                            Some(Value::Num(n)) => n.to_string(),
                            _ => return Err(format!("E5003: Variable '{}' does not resolve to a macro name", name)),
                        }
                    } else {
                        name.clone()
                    };
                    // ----------------------------------------------------------------------

                    // Resolve arguments before passing to macro
                    let mut resolved_args = Vec::new();
                    for arg in args {
                        resolved_args.push(self.resolve_value(&arg)?);
                    }

                    let mut macro_events = self.expand_macro(&actual_name, resolved_args, transpose)?;

                    // Universally apply Macro call modifiers (duration, attributes) to all resulting events
                    for ev in &mut macro_events {
                        if let Some((d, ds, m, a_opt)) = get_event_fields_mut(ev) {
                            if duration.is_some() { *d = duration.clone(); }
                            if dots > 0 { *ds = dots; }
                            if multiplier.is_some() { *m = multiplier; }
                            if let Some(ev_attrs) = a_opt {
                                ev_attrs.extend(attributes.clone());
                            }
                        }
                    }

                    expanded_events.extend(macro_events);
                    self.depth -= 1;
                }
                Event::Tuplet { content, p, q } => {
                    let expanded_content = self.process_voice(content)?;
                    expanded_events.push(Event::Tuplet { content: expanded_content, p, q });
                }
                Event::Chord { notes, duration, dots, multiplier, is_tied, attributes } => {
                    let mut expanded_notes = Vec::new();
                    for mut n in notes {
                        self.resolve_event_attributes(&mut n)?;
                        if let Event::MacroCall { name, args, transpose, .. } = n {
                            // For chord notes, also resolve variable macro names
                            let actual_name = if name.starts_with('$') {
                                let var_name = &name[1..];
                                match self.variables.get(var_name) {
                                    Some(Value::Id(s)) | Some(Value::Str(s)) => s.clone(),
                                    Some(Value::Num(n)) => n.to_string(),
                                    _ => return Err(format!("E5003: Variable '{}' does not resolve to a macro name", name)),
                                }
                            } else {
                                name.clone()
                            };
                            let mut resolved_args = Vec::new();
                            for arg in args { resolved_args.push(self.resolve_value(&arg)?); }
                            expanded_notes.extend(self.expand_macro(&actual_name, resolved_args, transpose)?);
                        } else {
                            expanded_notes.push(n);
                        }
                    }
                    expanded_events.push(Event::Chord { notes: expanded_notes, duration, dots, multiplier, is_tied, attributes });
                }
                other => expanded_events.push(other),
            }
        }

        voice.events = expanded_events;
        Ok(voice)
    }

    fn evaluate_condition(&self, expr: &Expression) -> Result<bool, String> {
        let left_val = self.resolve_value(&expr.left)?;
        let right_val = self.resolve_value(&expr.right)?;
        match expr.operator.as_str() {
            "==" => Ok(left_val == right_val),
            "!=" => Ok(left_val != right_val),
            _ => Err(format!("E5004: Unsupported operator '{}'", expr.operator)),
        }
    }

    /// Deeply resolves variables, expanding arrays and V2.1 `@{}` Maps.
    fn resolve_value(&self, val: &Value) -> Result<Value, String> {
        match val {
            Value::Id(id) if id.starts_with('$') => {
                let var_name = &id[1..];
                if let Some(resolved) = self.variables.get(var_name) {
                    // Recurse in case the variable itself points to a map containing variables
                    self.resolve_value(resolved)
                } else {
                    Err(format!("E2001: Undefined Variable '{}'", var_name))
                }
            },
            Value::Array(arr) => {
                let mut resolved_arr = Vec::new();
                for item in arr {
                    resolved_arr.push(self.resolve_value(item)?);
                }
                Ok(Value::Array(resolved_arr))
            },
            Value::Map(map) => Ok(Value::Map(self.resolve_map(map.clone())?)),
            _ => Ok(val.clone()),
        }
    }

    /// Helper for resolving the values inside HashMaps (TopLevel Attributes, Meta, etc)
    fn resolve_map(&self, map: HashMap<String, Value>) -> Result<HashMap<String, Value>, String> {
        let mut resolved = HashMap::new();
        for (k, v) in map {
            resolved.insert(k, self.resolve_value(&v)?);
        }
        Ok(resolved)
    }

    /// Expands variables in the arguments attached to a specific Event modifier
    fn resolve_event_attributes(&self, event: &mut Event) -> Result<(), String> {
        if let Some((_, _, _, Some(attributes))) = get_event_fields_mut(event) {
            for attr in attributes.iter_mut() {
                for arg in attr.args.iter_mut() {
                    *arg = self.resolve_value(arg)?;
                }
            }
        }
        Ok(())
    }

    fn expand_macro(&mut self, name: &str, provided_args: Vec<Value>, transpose: Option<i32>) -> Result<Vec<Event>, String> {
        let macro_def = self.macros.get(name).cloned().ok_or_else(|| format!("E2001: Undefined Macro '{}'", name))?;
        if let TopLevel::MacroDef { args: def_args, body, .. } = macro_def {
            let mut local_vars = self.variables.clone();
            
            if provided_args.len() > def_args.len() {
                return Err(format!("E5003: Argument Mismatch in '{}'", name));
            }
            
            for (i, (arg_name, default_val)) in def_args.into_iter().enumerate() {
                let val = if i < provided_args.len() {
                    provided_args[i].clone()
                } else if let Some(def) = default_val {
                    def
                } else {
                    return Err(format!("E5003: Missing required argument '{}'", arg_name));
                };
                local_vars.insert(arg_name, val);
            }
            
            let old_vars = std::mem::replace(&mut self.variables, local_vars);
            let mut expanded_voice = self.process_voice(body)?;
            self.variables = old_vars;
            
            if let Some(delta) = transpose {
                for event in &mut expanded_voice.events { Self::apply_transposition(event, delta); }
            }
            Ok(expanded_voice.events)
        } else { unreachable!() }
    }

    fn apply_transposition(event: &mut Event, delta: i32) {
        match event {
            Event::Note { pitch, .. } => { *pitch = shift_spn(pitch, delta); }
            Event::Chord { notes, .. } => { for n in notes { Self::apply_transposition(n, delta); } }
            Event::Tuplet { content, .. } => { for e in &mut content.events { Self::apply_transposition(e, delta); } }
            _ => {} // Transposition ignores Rests, Percussion (Map Keys), Tablature (Coords)
        }
    }
}

/// Universal helper to extract mutable references to common duration/attribute fields
fn get_event_fields_mut(event: &mut Event) -> Option<(&mut Option<String>, &mut u8, &mut Option<u32>, Option<&mut Vec<Attribute>>)> {
    match event {
        Event::Note { duration, dots, multiplier, attributes, .. } => Some((duration, dots, multiplier, Some(attributes))),
        Event::Chord { duration, dots, multiplier, attributes, .. } => Some((duration, dots, multiplier, Some(attributes))),
        Event::Percussion { duration, dots, multiplier, attributes, .. } => Some((duration, dots, multiplier, Some(attributes))),
        Event::Tab { duration, dots, multiplier, attributes, .. } => Some((duration, dots, multiplier, Some(attributes))),
        Event::MacroCall { duration, dots, multiplier, attributes, .. } => Some((duration, dots, multiplier, Some(attributes))),
        Event::Frequency { duration, dots, multiplier, attributes, .. } => Some((duration, dots, multiplier, Some(attributes))),
        Event::Rest { duration, dots, multiplier } => Some((duration, dots, multiplier, None)),
        Event::Space { duration, dots, multiplier } => Some((duration, dots, multiplier, None)),
        _ => None,
    }
}

fn shift_spn(pitch: &str, delta: i32) -> String {
    if delta == 0 { return pitch.to_string(); }
    let chars: Vec<char> = pitch.chars().collect();
    if chars.is_empty() { return pitch.to_string(); }
    
    let mut base_midi = match chars[0].to_ascii_lowercase() {
        'c' => 0, 'd' => 2, 'e' => 4, 'f' => 5, 'g' => 7, 'a' => 9, 'b' => 11, _ => return pitch.to_string(),
    };
    
    let mut octave = 4;
    let mut i = 1;
    
    // Accumulate standard accidentals
    while i < chars.len() && !chars[i].is_ascii_digit() {
        match chars[i] { 
            '#' => base_midi += 1, 
            'b' => base_midi -= 1, 
            'x' => base_midi += 2, 
            _ => {} 
        }
        i += 1;
    }
    
    if let Ok(o) = chars[i..].iter().collect::<String>().parse::<i32>() { octave = o; }
    
    let absolute_midi = (octave + 1) * 12 + base_midi + delta;
    let new_octave = (absolute_midi / 12) - 1;
    let pc_str = match absolute_midi % 12 {
        0 => "c", 1 => "c#", 2 => "d", 3 => "eb", 4 => "e", 5 => "f", 
        6 => "f#", 7 => "g", 8 => "g#", 9 => "a", 10 => "bb", 11 => "b", _ => "c",
    };
    
    format!("{}{}", pc_str, new_octave)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_transposition_logic() {
        assert_eq!(shift_spn("c4", 12), "c5");
        assert_eq!(shift_spn("b4", 1), "c5");
        assert_eq!(shift_spn("eb3", -2), "c#3");
    }
}