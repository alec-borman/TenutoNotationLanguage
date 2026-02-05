pub mod lexer;
pub mod parser;
pub mod ir;

use thiserror::Error;
use num_integer::Integer;

/// The Fundamental Unit of Time (Rational Arithmetic)
/// Used to prevent floating point drift in rhythmic calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rational {
    pub num: u64,
    pub den: u64,
}

impl Rational {
    pub fn new(num: u64, den: u64) -> Self {
        if den == 0 {
            panic!("F9002: Internal Error - Division by Zero in Time Engine");
        }
        let gcd = num.gcd(&den);
        Self { num: num / gcd, den: den / gcd }
    }

    pub fn to_ticks(&self, ppq: u32) -> u64 {
        // formula: (num/den) * 4 * ppq (Whole note = 4 beats)
        (self.num * 4 * ppq as u64) / self.den
    }
}

/// Standardized Error Codes conforming to Spec Section 24
#[derive(Error, Debug)]
pub enum TenutoError {
    #[error("E1001: Malformed Token at position {0}")]
    LexicalError(usize),

    #[error("F9001: IO Error: {0}")]
    IoError(#[from] std::io::Error),
}

/// The Compiler Pipeline State
pub struct Pipeline {
    pub source: String,
    pub strict_mode: bool,
}

impl Pipeline {
    pub fn new(source: String) -> Self {
        Self { source, strict_mode: false }
    }
}