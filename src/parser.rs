//! # Tenuto Deterministic Parser
//! 
//! Implements a strict, predictive LL(1) parser using `chumsky`. 
//! Translates the flat `Token` stream into the robust, memory-safe AST.
//! 
//! **V3.0.0 Updates:** 
//! - Slice 1: Bifurcated Tuplet parsing to natively support Euclidean rhythm topologies `(k):3/8`.
//! - Slice 2: Isolated Space token (`s`) for decoupled Action Notation and Control Lanes.
//! - Slice 5/6: Map keys dynamically coerce Integers and PitchLits into Strings.
//! - Slice 7: Unary signs (+/-) supported natively inside value arguments.
//! - MASTER SLICE: Added `repeat_block` and `lyric_assignment` parsers.

use chumsky::prelude::*;
use chumsky::recovery::skip_then_retry_until;
use std::collections::HashMap;

use crate::lexer::Token;
use crate::ast::*;

// ============================================================================
// 1. PRIMITIVES & VALUES
// ============================================================================

fn value_parser() -> impl Parser<Token, Value, Error = Simple<Token>> + Clone {
    recursive(|value| {
        let val_str = select! { Token::StringLit(s) => Value::Str(s) };
        let val_bool = select! { Token::Boolean(b) => Value::Bool(b) };
        let val_id = select! { Token::Identifier(s) => Value::Id(s) };
        let val_time = select! { Token::TimeVal(t) => Value::TimeVal(t) };

        let sign = choice((just(Token::Plus), just(Token::Minus))).or_not();
        let val_num = sign.clone().then(select! { Token::Integer(i) => i })
            .map(|(s, i)| if s == Some(Token::Minus) { Value::Num(-i) } else { Value::Num(i) });
        let val_flt = sign.clone().then(select! { Token::Float(f) => f.parse::<f64>().unwrap_or(0.0) })
            .map(|(s, f)| if s == Some(Token::Minus) { Value::Float(-f) } else { Value::Float(f) });

        let val_var = just(Token::Dollar).ignore_then(select! { Token::Identifier(s) => s }).map(|s| Value::Id(format!("${}", s)));

        let domain_lits = select! {
            Token::PitchLit(p) => Value::Str(p),
            Token::FreqLit(f) => Value::Str(f),
            Token::DurationLit(d) => Value::Str(d),
            Token::TabLit(t) => Value::Str(t),
        };

        let array = value.clone().separated_by(just(Token::Comma)).allow_trailing()
            .delimited_by(just(Token::LBracket), just(Token::RBracket)).map(Value::Array);

        let kv_pair = choice((
            select! { Token::Identifier(s) => s },
            select! { Token::StringLit(s) => s },
            select! { Token::Integer(i) => i.to_string() },
            select! { Token::PitchLit(p) => p } 
        )).then_ignore(just(Token::Colon)).then(value.clone());
            
        let map = kv_pair.separated_by(just(Token::Comma)).allow_trailing()
            .delimited_by(just(Token::MapStart), just(Token::RBrace))
            .map(|pairs| {
                let mut hm = HashMap::new();
                for (k, v) in pairs { hm.insert(k, v); }
                Value::Map(hm)
            });

        choice((val_str, val_num, val_flt, val_bool, val_time, val_var, val_id, domain_lits, array, map))
    })
}

// ============================================================================
// 2. THE EVENT ENGINE
// ============================================================================

fn event_parser() -> impl Parser<Token, Event, Error = Simple<Token>> + Clone {
    recursive(|event| {
        let duration = select! { Token::DurationLit(d) => d }.or_not();
        let dots = just(Token::Dot).repeated().map(|d| d.len() as u8);
        let multiplier = just(Token::Star).ignore_then(select! { Token::Integer(i) => i as u32 }).or_not();
        let is_tied = just(Token::Tilde).or_not().map(|t| t.is_some());

        let attribute = select! { Token::AttributeLit(a) => a.trim_start_matches('.').to_string() }
            .then(value_parser().separated_by(just(Token::Comma)).allow_trailing()
                  .delimited_by(just(Token::LParen), just(Token::RParen)).or_not())
            .map(|(name, args)| Attribute { name, args: args.unwrap_or_default() });

        let note = select! { Token::PitchLit(p) => p }
            .then(duration.clone()).then(dots.clone()).then(multiplier.clone()).then(is_tied.clone())
            .then(attribute.clone().repeated())
            .map(|(((((pitch, duration), dots), multiplier), is_tied), attributes)| { Event::Note { pitch, cents: None, duration, dots, multiplier, is_tied, attributes } });

        let tab = select! { Token::TabLit(t) => t }.then(duration.clone()).then(dots.clone()).then(multiplier.clone()).then(attribute.clone().repeated())
            .map(|((((t_str, duration), dots), multiplier), attributes)| {
                let parts: Vec<&str> = t_str.split('-').collect();
                Event::Tab { fret: parts[0].to_string(), string: parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1), duration, dots, multiplier, attributes }
            });

        let perc = select! { Token::Identifier(id) if id != "r" && id != "s" => id }
            // --- V3.0 MASTER SLICE FIX: Prevent Percussion from swallowing staff.lyric: ---
            .then_ignore(
                just(Token::AttributeLit(".lyric".to_string()))
                    .then(just(Token::Colon))
                    .not()
                    .rewind()
            )
            .then_ignore(just(Token::Colon).not().rewind()) 
            .then(duration.clone()).then(dots.clone()).then(multiplier.clone())
            .then(attribute.clone().repeated())
            .map(|((((key, duration), dots), multiplier), attributes)| {
                Event::Percussion { key, duration, dots, multiplier, attributes }
            });

        let freq = select! { Token::FreqLit(f) => f }.then(duration.clone()).then(dots.clone()).then(multiplier.clone()).then(attribute.clone().repeated())
            .map(|((((hz, duration), dots), multiplier), attributes)| {
                Event::Frequency { hz: hz.trim_start_matches("hz(").trim_end_matches(')').to_string(), duration, dots, multiplier, attributes }
            });

        let rest = select! { Token::Identifier(id) if id == "r" => id }.then(duration.clone()).then(dots.clone()).then(multiplier.clone())
            .map(|(((_kind, duration), dots), multiplier)| { Event::Rest { duration, dots, multiplier } });

        let space = select! { Token::Identifier(id) if id == "s" => id }.then(duration.clone()).then(dots.clone()).then(multiplier.clone()).then(attribute.clone().repeated())
            .map(|((((_kind, duration), dots), multiplier), attributes)| { Event::Space { duration, dots, multiplier, attributes } });

        let barline = choice((
            just(Token::RepeatStart).to(BarlineType::RepeatStart), just(Token::RepeatEnd).to(BarlineType::RepeatEnd),
            just(Token::Pipe).to(BarlineType::Single), just(Token::DoubleBar).to(BarlineType::Double),
            just(Token::FinalBar).to(BarlineType::Final),
        )).map(Event::Barline);

        let transposition = choice((just(Token::Plus), just(Token::Minus))).then(select! { Token::Integer(i) => i as i32 })
            .map(|(sign, val)| if sign == Token::Plus { val } else { -val }).or_not();

        let macro_call = just(Token::Dollar).ignore_then(select! { Token::Identifier(s) => s })
            .then(value_parser().separated_by(just(Token::Comma)).allow_trailing().delimited_by(just(Token::LParen), just(Token::RParen)).or_not())
            .then(transposition).then(duration.clone()).then(dots.clone()).then(multiplier.clone()).then(attribute.clone().repeated())
            .map(|((((((name, args), transpose), duration), dots), multiplier), attributes)| {
                Event::MacroCall { name, args: args.unwrap_or_default(), transpose, duration, dots, multiplier, attributes }
            });

        let tuplet_ratio = select! { Token::DurationLit(d) => d.trim_start_matches(':').parse::<u64>().unwrap_or(1) }.then_ignore(just(Token::Slash)).then(select! { Token::Integer(i) => i as u64 });

        let tuplet = event.clone().repeated().delimited_by(just(Token::LParen), just(Token::RParen)).then(tuplet_ratio)
            .map(|(events, (p, q))| {
                if events.len() == 1 { Event::Euclidean { content: Box::new(events.into_iter().next().unwrap()), k: p, n: q } } 
                else { Event::Tuplet { content: Voice { voice_id: None, events }, p, q } }
            });

        let chord = event.clone().repeated().delimited_by(just(Token::LBracket), just(Token::RBracket))
            .then(duration.clone()).then(dots.clone()).then(multiplier.clone()).then(is_tied.clone()).then(attribute.clone().repeated())
            .map(|(((((notes, duration), dots), multiplier), is_tied), attributes)| { Event::Chord { notes, duration, dots, multiplier, is_tied, attributes } });

        choice((barline, rest, space, macro_call, tab, freq, perc, note, tuplet, chord))
    })
}

// ============================================================================
// 3. LOGIC & STATEMENTS
// ============================================================================

fn logic_parser() -> impl Parser<Token, Logic, Error = Simple<Token>> + Clone {
    let event = event_parser();

    let voice = select! { Token::Identifier(s) => s }.then_ignore(just(Token::Colon)).or_not().then(event.clone().repeated().at_least(1))
        .map(|(id, evs)| Voice { voice_id: id, events: evs });

    let voice_block = voice.clone().repeated().delimited_by(just(Token::VoiceBracketStart), just(Token::VoiceBracketEnd));

    let assignment = select! { Token::Identifier(s) => s }.then_ignore(just(Token::Colon)).then(voice_block.or(voice.map(|v| vec![v])))
        .map(|(staff_id, voices)| Logic::Assignment { staff_id, voices });

    // --- V3.0 MASTER SLICE: Lyric Engine Parser ---
    let lyric_assignment = select! { Token::Identifier(s) => s }
        .then_ignore(select! { Token::AttributeLit(a) if a == ".lyric" => () })
        .then_ignore(just(Token::Colon))
        .then(select! { Token::StringLit(s) => s })
        .map(|(staff_id, text)| Logic::LyricAssignment { staff_id, text });

    let local_meta = just(Token::KwMeta).ignore_then(value_parser())
        .try_map(|v, span| { if let Value::Map(m) = v { Ok(Logic::LocalMeta(m)) } else { Err(Simple::custom(span, "Expected map")) } });

    just(Token::RBrace).not().rewind()
        .ignore_then(just(Token::VoiceBracketEnd).not().rewind())
        .ignore_then(choice((assignment, lyric_assignment, local_meta)).recover_with(skip_then_retry_until([Token::RBrace, Token::VoiceBracketEnd, Token::KwMeasure])))
}

// ============================================================================
// 4. TOP-LEVEL STRUCTURE
// ============================================================================

fn top_level_parser() -> impl Parser<Token, TopLevel, Error = Simple<Token>> + Clone {
    recursive(|top_level| {
        let value = value_parser();
        let def_attr = select! { Token::Identifier(s) => s }.then_ignore(just(Token::Equals).or(just(Token::Colon))).then(value.clone());

let measure_range = choice((
            // 1. The Lexer swallows `1-8` as a TabLit. We intercept and split it!
            select! { Token::TabLit(t) => t }.map(|t| {
                let parts: Vec<&str> = t.split('-').collect();
                let start = parts[0].parse::<i64>().unwrap_or(1);
                let end = parts[1].parse::<i64>().unwrap_or(1);
                MeasureRange::Range(start, end)
            }),
            // 2. Parses ranges separated by spaces like `1 - 8`
            select! { Token::Integer(i) => i }
                .then_ignore(just(Token::Minus))
                .then(select! { Token::Integer(j) => j })
                .map(|(start, end)| MeasureRange::Range(start, end)),
            // 3. Parses single numbers like `1`
            select! { Token::Integer(i) => MeasureRange::Single(i) },
            // 4. Parses named sections like `Chorus`
            select! { Token::Identifier(s) => MeasureRange::Identifier(s) },
        )).or_not().map(|r| r.unwrap_or(MeasureRange::Implicit));

        let measure_block = just(Token::KwMeasure)
            .ignore_then(measure_range)
            .then(logic_parser().repeated().delimited_by(just(Token::LBrace), just(Token::RBrace)))
            .map(|(range, content)| TopLevel::Measure { range, attributes: HashMap::new(), content });

        // --- V3.0 MASTER SLICE: Graph Unroller Repeat Block Parser ---
        let repeat_block = just(Token::KwRepeat).ignore_then(select! { Token::Integer(i) => i as u32 }.or_not())
            .then(logic_parser().repeated().delimited_by(just(Token::LBrace), just(Token::RBrace)))
            .map(|(count, content)| TopLevel::Repeat { count, content });

        let def_block = just(Token::KwDef).ignore_then(select! { Token::Identifier(s) => s })
            .then(select! { Token::StringLit(s) => s }.or(select! { Token::Identifier(s) => s }).or_not()).then(def_attr.clone().repeated())
            .map(|((id, label), attrs)| {
                let mut hm = HashMap::new(); for (k, v) in attrs { hm.insert(k, v); }
                TopLevel::Def { id, label, attributes: hm }
            });

        let macro_def = just(Token::KwMacro).ignore_then(select! { Token::Identifier(s) => s })
            .then(select! { Token::Identifier(s) => (s, None) }.separated_by(just(Token::Comma)).delimited_by(just(Token::LParen), just(Token::RParen)).or_not())
            .then_ignore(just(Token::Equals)).then(event_parser().repeated().delimited_by(just(Token::LBrace), just(Token::RBrace)))
            .map(|((name, args), events)| TopLevel::MacroDef { name, args: args.unwrap_or_default(), body: Voice { voice_id: None, events } });

        let group_block = just(Token::KwGroup).ignore_then(select! { Token::StringLit(s) => s }.or_not())
            .then(top_level.clone().repeated().delimited_by(just(Token::LBrace), just(Token::RBrace)))
            .map(|(label, items)| TopLevel::Group { label, attributes: HashMap::new(), items });

        let meta_block = just(Token::KwMeta).ignore_then(value)
            .map(|v| if let Value::Map(m) = v { TopLevel::Meta(m) } else { TopLevel::Meta(HashMap::new()) });

        let var_decl = just(Token::KwVar).ignore_then(select! { Token::Identifier(s) => s }).then_ignore(just(Token::Equals)).then(value_parser())
            .map(|(name, value)| TopLevel::VariableDecl { name, value });

        just(Token::RBrace).not().rewind().ignore_then(choice((measure_block, repeat_block, macro_def, def_block, group_block, meta_block, var_decl)).recover_with(skip_then_retry_until([Token::KwMeasure, Token::KwDef, Token::KwGroup, Token::KwMacro, Token::KwMeta, Token::KwVar, Token::RBrace])))
    })
}

// ============================================================================
// 5. PUBLIC PARSER EXPORT
// ============================================================================

pub fn parser() -> impl Parser<Token, Score, Error = Simple<Token>> {
    let header = just(Token::KwTenuto).ignore_then(select! { Token::StringLit(s) => s }.or_not());
    header.then(top_level_parser().repeated().delimited_by(just(Token::LBrace), just(Token::RBrace)))
        .map(|(version, items)| Score { header_version: version, items }).then_ignore(end())
}