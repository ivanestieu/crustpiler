use std::num::ParseFloatError;
use crate::lexer::errors::{LexError, LexErrorKind};
use crate::lexer::int::find_suffix_start;
use crate::lexer::token::Token;
use crate::literals::{FloatLit, FloatSuffix};

pub fn lex_hex_float(lex : &mut logos::Lexer<Token>) -> Result<FloatLit, LexError> {
    let s = lex.slice();
    let numeric = s.trim_end_matches(['f', 'F', 'l', 'L']);
    hexf_parse::parse_hexf64(numeric, false).map(|value| {
        let suffix_start : usize = find_suffix_start(s, "fFlL");
        let suffix : FloatSuffix = parse_float_suffix(&s[suffix_start..]);
        FloatLit { value, suffix }
    }).map_err(|_| LexError::new(LexErrorKind::InvalidHexFloat {text: s.to_string()}))
}

pub fn lex_std_float(lex: &mut logos::Lexer<Token>) -> Result<FloatLit, LexError> {
    let s = lex.slice();
    let suffix_start : usize = find_suffix_start(s, "fFlL");
    let (number, suffix) : (&str, &str)  = s.split_at(suffix_start);

    let value: f64 = number.parse().map_err(
        |e : ParseFloatError| LexError::new(LexErrorKind::InvalidFloat {text: s.to_string(), reason: e.to_string()})
    )?;

    let suffix : FloatSuffix = parse_float_suffix(suffix);

    Ok(FloatLit { value, suffix })
}

fn parse_float_suffix(s: &str) -> FloatSuffix {
    match s {
        "f" | "F" => FloatSuffix::Float,
        "l" | "L" => FloatSuffix::LongDouble,
        _ => FloatSuffix::Double,
    }
}
