use crate::lexer::errors::{LexError, LexErrorKind};
use crate::lexer::token::Token;
use crate::literals::{IntBase, IntLit, IntSuffix, LongKind};
use std::num::ParseIntError;

pub fn lex_int(lex: &mut logos::Lexer<Token>) -> Result<IntLit, LexError> {
    let s: &str = lex.slice();
    let suffix_start: usize = find_suffix_start(s, "uUlL");
    let (number, suffix): (&str, &str) = s.split_at(suffix_start);

    let (base, digits, radix): (IntBase, &str, u32) = if let Some(rest) = number
        .strip_prefix("0x")
        .or_else(|| number.strip_prefix("0X"))
    {
        (IntBase::Hexadecimal, rest, 16)
    } else if number.len() > 1 && number.starts_with('0') {
        (IntBase::Octal, &number[1..], 8)
    } else {
        (IntBase::Decimal, number, 10)
    };

    let value: u64 = u64::from_str_radix(digits, radix).map_err(|e: ParseIntError| {
        LexError::new(LexErrorKind::InvalidInteger {
            text: s.to_string(),
            reason: e.to_string(),
        })
    })?;

    let suffix: IntSuffix = parse_int_suffix(suffix);

    Ok(IntLit {
        value,
        base,
        suffix,
    })
}

pub fn find_suffix_start(s: &str, valid: &str) -> usize {
    s.find(|c| valid.contains(c)).unwrap_or(s.len())
}

fn parse_int_suffix(s: &str) -> IntSuffix {
    let lower_s = s.to_lowercase();
    IntSuffix {
        unsigned: lower_s.contains("u"),
        long: match lower_s.matches("l").count() {
            0 => LongKind::None,
            1 => LongKind::Long,
            _ => LongKind::LongLong,
        },
    }
}
