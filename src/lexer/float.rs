use crate::lexer::int::find_suffix_start;
use crate::lexer::token::Token;
use crate::literals::{FloatLit, FloatSuffix};

pub fn lex_hex_float(lex : &mut logos::Lexer<Token>) -> Option<FloatLit> {
    let s = lex.slice();
    let numeric = s.trim_end_matches(['f', 'F', 'l', 'L']);
    hexf_parse::parse_hexf64(numeric, false).ok().map(|value| {
        let suffix_start : usize = find_suffix_start(s, "fFlL");
        let suffix : FloatSuffix = parse_float_suffix(&s[suffix_start..]);
        FloatLit { value, suffix }
    })
}

pub fn lex_std_float(lex: &mut logos::Lexer<Token>) -> Option<FloatLit> {
    let s = lex.slice();
    let suffix_start : usize = find_suffix_start(s, "fFlL");
    let (number, suffix) : (&str, &str)  = s.split_at(suffix_start);

    let value: f64 = number.parse().ok()?;

    let suffix : FloatSuffix = parse_float_suffix(suffix);

    Some(FloatLit { value, suffix })
}

fn parse_float_suffix(s: &str) -> FloatSuffix {
    if s.contains(r"[fF]") {
        FloatSuffix::Float
    } else if s.contains(r"[lL]") {
        FloatSuffix::LongDouble
    } else {
        FloatSuffix::Double
    }
}
