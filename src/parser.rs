use chumsky::prelude::*;
use crate::lexer::Token;

// ========================================================================
// 1. ABSTRACT SYNTAX TREE (AST)
// ========================================================================

#[derive(Debug, Clone)]
pub struct Score {
    pub header: Option<String>, 
    pub items: Vec<TopLevel>,
}

#[derive(Debug, Clone)]
pub enum TopLevel {
    Meta(Vec<(String, Value)>),
    Def {
        id: String,
        label: String,
        attributes: Vec<(String, Value)>,
    },
    Measure {
        id: Option<i64>, 
        content: Vec<Statement>,
    },
    Import(String),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assignment {
        staff_id: String,
        voices: Vec<Voice>,
    },
    LocalMeta(Vec<(String, Value)>),
}

#[derive(Debug, Clone)]
pub struct Voice {
    pub events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub enum Event {
    Note {
        pitch: String,
        duration: Option<String>,
        attributes: Vec<Attribute>,
    },
    Chord {
        notes: Vec<String>,
        duration: Option<String>,
        attributes: Vec<Attribute>,
    },
    Rest {
        duration: Option<String>,
    },
    Tab {
        fret: u8,
        string: u8,
        duration: Option<String>,
        attributes: Vec<Attribute>,
    },
    Percussion {
        key: String,
        duration: Option<String>,
        attributes: Vec<Attribute>,
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Num(i64),
    Float(f64),
    Id(String), // ADDED: Supports 'style=standard'
    Array(Vec<Value>),
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<Value>,
}

// ========================================================================
// 2. PARSER COMBINATORS
// ========================================================================

pub fn parser() -> impl Parser<Token, Score, Error = Simple<Token>> {
    
    // --- Primitives ---
    let identifier = select! { Token::Identifier(s) => s };
    let string_lit = select! { Token::StringLit(s) => s };
    let integer = select! { Token::Integer(i) => i };
    
    // Parse the string token into a float here
    let float = select! { Token::Float(s) => s }
        .map(|s| s.parse::<f64>().unwrap_or(0.0));

    let pitch = select! { Token::PitchLit(p) => p };
    let duration = select! { Token::DurationLit(d) => d };
    let tab_lit = select! { Token::TabLit(t) => t };

    // --- Values ---
    let val_str = string_lit.clone().map(Value::Str);
    let val_int = integer.clone().map(Value::Num);
    let val_flt = float.clone().map(Value::Float);
    let val_id  = identifier.clone().map(Value::Id); // ADDED
    
    // Box complex types to help compiler
    // We now accept Strings OR Floats OR Integers OR Identifiers
    let value = val_str
        .or(val_flt)
        .or(val_int)
        .or(val_id)
        .boxed();

    // --- Attributes ---
    // Example: .stacc or .vol(80)
    let attribute = just(Token::Dot)
        .ignore_then(identifier.clone())
        .then(
            just(Token::LParen)
                .ignore_then(value.clone().separated_by(just(Token::Comma)))
                .then_ignore(just(Token::RParen))
                .or_not()
        )
        .map(|(name, args)| Attribute { 
            name, 
            args: args.unwrap_or_default() 
        })
        .boxed();

    // --- Events ---
    
    // 1. Note: c4:4.stacc
    let note_event = pitch
        .then(duration.clone().or_not())
        .then(attribute.clone().repeated())
        .map(|((p, d), attrs)| Event::Note { 
            pitch: p, 
            duration: d, 
            attributes: attrs 
        });

    // 2. Rest: r:4
    // Using select! guard logic to bypass .filter() trait issues
    let rest_event = select! { Token::Identifier(s) if s == "r" => s }
        .ignore_then(duration.clone().or_not())
        .map(|d| Event::Rest { duration: d });
        
    // 3. Tab: 0-6:4
    let tab_event = tab_lit
        .then(duration.clone().or_not())
        .then(attribute.clone().repeated())
        .map(|((t, d), attrs)| {
            let parts: Vec<&str> = t.split('-').collect();
            let fret = parts[0].parse().unwrap_or(0);
            let string = parts[1].parse().unwrap_or(1);
            Event::Tab { fret, string, duration: d, attributes: attrs }
        });

    // 4. Percussion/Generic: k:4
    // Using select! guard logic
    let perc_event = select! { Token::Identifier(s) if s != "r" => s }
        .then(duration.clone().or_not())
        .then(attribute.clone().repeated())
        .map(|((k, d), attrs)| Event::Percussion { 
            key: k, 
            duration: d, 
            attributes: attrs 
        });

    // Priority: Rest -> Note -> Tab -> Percussion
    let event = choice((
        rest_event,
        note_event, 
        tab_event,
        perc_event
    ));

    // --- Voices ---
    let voice = event.repeated().map(|events| Voice { events });
    
    // VoiceGroup: v1 | v2 |
    let voice_group = voice
        .separated_by(just(Token::Pipe))
        .allow_trailing() 
        .map(|voices| voices);

    // --- Statements ---
    // vln: c4 d e |
    let assignment = identifier.clone()
        .then_ignore(just(Token::Colon))
        .then(voice_group)
        .then_ignore(just(Token::Pipe).or_not()) 
        .map(|(id, voices)| Statement::Assignment { 
            staff_id: id, 
            voices 
        });

    // meta { ... }
    let key_value = identifier.clone()
        .then_ignore(just(Token::Colon))
        .then(value.clone());
        
    let meta_block = just(Token::KwMeta)
        .ignore_then(just(Token::LBrace))
        .ignore_then(key_value.separated_by(just(Token::Comma)))
        .then_ignore(just(Token::RBrace));

    let statement = choice((
        assignment,
        meta_block.clone().map(Statement::LocalMeta)
    ));

    // --- Blocks ---

    // def vln "Violin" key=val
    let def_attr = identifier.clone()
        .then_ignore(just(Token::Equals))
        .then(value.clone());

    let def_block = just(Token::KwDef)
        .ignore_then(identifier.clone())
        .then(string_lit.clone().or_not()) // Label optional
        .then(def_attr.repeated())
        .map(|((id, label), attrs): ((String, Option<String>), Vec<(String, Value)>)| TopLevel::Def { 
            id, 
            label: label.unwrap_or_default(), 
            attributes: attrs 
        });

    // measure 1 { ... }
    let measure_block = just(Token::KwMeasure)
        .ignore_then(integer.clone().or_not()) 
        .then_ignore(just(Token::LBrace))
        .then(statement.repeated())
        .then_ignore(just(Token::RBrace))
        .map(|(num, content)| TopLevel::Measure { 
            id: num, 
            content 
        });

    // omniscore { ... }
    let root_content = choice((
        meta_block.map(TopLevel::Meta),
        def_block,
        measure_block
    )).repeated();

    let score = just(Token::KwOmniscore)
        .ignore_then(just(Token::LBrace))
        .ignore_then(root_content)
        .then_ignore(just(Token::RBrace))
        .map(|items| Score { header: Some("2.0".into()), items });

    score
}