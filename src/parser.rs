use chumsky::prelude::*;
use crate::lexer::Token;

// --- AST Structures ---

#[derive(Debug, Clone)]
pub struct Score {
    pub header: Option<String>, 
    pub items: Vec<TopLevel>,
}

#[derive(Debug, Clone)]
pub enum TopLevel {
    Meta(Vec<(String, Value)>),
    Def { id: String, label: String, attributes: Vec<(String, Value)> },
    Measure { id: Option<i64>, content: Vec<Statement> },
    Import(String),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assignment { staff_id: String, voices: Vec<Voice> },
    LocalMeta(Vec<(String, Value)>),
}

#[derive(Debug, Clone)]
pub struct Voice {
    pub events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub enum Event {
    Note { pitch: String, duration: Option<String>, attributes: Vec<Attribute> },
    Chord { notes: Vec<String>, duration: Option<String>, attributes: Vec<Attribute> },
    Rest { duration: Option<String> },
    Tab { fret: u8, string: u8, duration: Option<String>, attributes: Vec<Attribute> },
    Percussion { key: String, duration: Option<String>, attributes: Vec<Attribute> },
    // Recursive Voice for Tuplets
    Tuplet { content: Voice, p: u64, q: u64 }, 
}

#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Num(i64),
    Float(f64),
    Id(String),
    Array(Vec<Value>),
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<Value>,
}

// --- Parser Logic ---

pub fn parser() -> impl Parser<Token, Score, Error = Simple<Token>> {
    let identifier = select! { Token::Identifier(s) => s };
    let string_lit = select! { Token::StringLit(s) => s };
    let integer = select! { Token::Integer(i) => i };
    let float = select! { Token::Float(s) => s }.map(|s| s.parse::<f64>().unwrap_or(0.0));
    let pitch = select! { Token::PitchLit(p) => p };
    let duration = select! { Token::DurationLit(d) => d };
    let tab_lit = select! { Token::TabLit(t) => t };

    let val_str = string_lit.clone().map(Value::Str);
    let val_int = integer.clone().map(Value::Num);
    let val_flt = float.clone().map(Value::Float);
    let val_id  = identifier.clone().map(Value::Id);
    let value = val_str.or(val_flt).or(val_int).or(val_id).boxed();

    let attribute = just(Token::Dot)
        .ignore_then(identifier.clone())
        .then(just(Token::LParen).ignore_then(value.clone().separated_by(just(Token::Comma))).then_ignore(just(Token::RParen)).or_not())
        .map(|(name, args)| Attribute { name, args: args.unwrap_or_default() })
        .boxed();

    // Recursive Event Parser for Tuplets
    let event = recursive(|event| {
        let note_event = pitch.then(duration.clone().or_not()).then(attribute.clone().repeated())
            .map(|((p, d), attrs)| Event::Note { pitch: p, duration: d, attributes: attrs });

        // Chord: [ c4 e4 g4 ]
        let chord_event = just(Token::LBracket)
            .ignore_then(pitch.repeated())
            .then_ignore(just(Token::RBracket))
            .then(duration.clone().or_not())
            .then(attribute.clone().repeated())
            .map(|((notes, d), attrs)| Event::Chord { notes, duration: d, attributes: attrs });

        let rest_event = select! { Token::Identifier(s) if s == "r" => s }
            .ignore_then(duration.clone().or_not())
            .map(|d| Event::Rest { duration: d });
            
        let tab_event = tab_lit.then(duration.clone().or_not()).then(attribute.clone().repeated())
            .map(|((t, d), attrs)| {
                let parts: Vec<&str> = t.split('-').collect();
                Event::Tab { fret: parts[0].parse().unwrap_or(0), string: parts[1].parse().unwrap_or(1), duration: d, attributes: attrs }
            });

        let perc_event = select! { Token::Identifier(s) if s != "r" => s }
            .then(duration.clone().or_not()).then(attribute.clone().repeated())
            .map(|((k, d), attrs)| Event::Percussion { key: k, duration: d, attributes: attrs });

        // Tuplet: ( c d e ):3/2
        let tuplet_event = just(Token::LParen)
            .ignore_then(event.repeated().map(|events| Voice { events }))
            .then_ignore(just(Token::RParen))
            .then_ignore(just(Token::Colon))
            .then(integer.clone())
            .then_ignore(just(Token::Slash))
            .then(integer.clone())
            .map(|((content, p), q)| Event::Tuplet { content, p: p as u64, q: q as u64 });

        choice((
            tuplet_event, // Try recursive structure first
            rest_event,
            chord_event,  // Then chords
            note_event, 
            tab_event,
            perc_event
        ))
    });

    let voice = event.repeated().map(|events| Voice { events });
    
    let voice_group = voice.separated_by(just(Token::Pipe)).allow_trailing().map(|voices| voices);

    let assignment = identifier.clone()
        .then_ignore(just(Token::Colon))
        .then(voice_group)
        .then_ignore(just(Token::Pipe).or_not())
        .map(|(id, voices)| Statement::Assignment { staff_id: id, voices });

    let key_value = identifier.clone().then_ignore(just(Token::Colon)).then(value.clone());
    let meta_block = just(Token::KwMeta).ignore_then(just(Token::LBrace))
        .ignore_then(key_value.separated_by(just(Token::Comma)))
        .then_ignore(just(Token::RBrace));

    let statement = choice((
        assignment,
        meta_block.clone().map(Statement::LocalMeta)
    ));

    let def_attr = identifier.clone().then_ignore(just(Token::Equals)).then(value.clone());
    
    // CRITICAL FIX: Added explicit type annotations to map closure
    let def_block = just(Token::KwDef).ignore_then(identifier.clone())
        .then(string_lit.clone().or_not())
        .then(def_attr.repeated())
        .map(|((id, label), attrs): ((String, Option<String>), Vec<(String, Value)>)| TopLevel::Def { 
            id, 
            label: label.unwrap_or_default(), 
            attributes: attrs 
        });

    let measure_block = just(Token::KwMeasure).ignore_then(integer.clone().or_not())
        .then_ignore(just(Token::LBrace)).then(statement.repeated()).then_ignore(just(Token::RBrace))
        .map(|(num, content)| TopLevel::Measure { id: num, content });

    let root_content = choice((
        meta_block.map(TopLevel::Meta),
        def_block,
        measure_block
    )).repeated();

    just(Token::KwTenuto)
        .ignore_then(just(Token::LBrace)).ignore_then(root_content).then_ignore(just(Token::RBrace))
        .map(|items| Score { header: Some("2.0".into()), items })
}