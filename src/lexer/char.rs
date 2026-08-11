use crate::lexer::errors::{LexError, LexErrorKind};
use crate::lexer::token::Token;
use itertools::Itertools;

pub fn lex_char_lit(lex: &mut logos::Lexer<Token>) -> Result<char, LexError> {
    let slice: &str = lex.slice();
    let inner = &slice[1..slice.len() - 1];
    let inner_iter = &mut inner.chars().peekable(); // strip quotes
    let result = lex_char(inner_iter).map_err(|e| LexError::with_span(e.kind, lex.span()));
    if inner_iter.peek() != None {
        Err(LexError::with_span(
            LexErrorKind::BadCharLiteral {
                text: inner.to_string(),
            },
            lex.span(),
        ))
    } else {
        result
    }
}

pub fn lex_char(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<char, LexError> {
    let c = chars
        .next()
        .ok_or(LexError::new(LexErrorKind::BadCharLiteral {
            text: String::from("char expected"),
        }))?;
    if c != '\\' {
        return Ok(c);
    }
    match chars.next() {
        Some('\'') => Ok('\''),
        Some('"') => Ok('"'),
        Some('?') => Ok('?'),
        Some('\\') => Ok('\\'),
        Some('n') => Ok('\n'),
        Some('t') => Ok('\t'),
        Some('r') => Ok('\r'),
        Some('a') => Ok('\x07'),
        Some('b') => Ok('\x08'),
        Some('f') => Ok('\x0C'),
        Some('v') => Ok('\x0B'),
        Some('x') => lex_hex_char(chars),
        Some('u') => lex_unicode_char(chars, 4),
        Some('U') => lex_unicode_char(chars, 8),
        Some(c @ '0'..='7') => lex_char_octal(c, chars),
        Some(unknown) => Err(LexError::new(LexErrorKind::UnknownEscape {
            escape: unknown,
        })),
        _ => Err(LexError::new(LexErrorKind::MalformedEscape {
            text: "\\".to_string(),
        })),
    }
}

fn lex_hex_char(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<char, LexError> {
    let digits = chars
        .peeking_take_while(|c| c.is_ascii_hexdigit())
        .collect::<String>();
    let n: u32 = u32::from_str_radix(&digits, 16).map_err(|_| {
        LexError::new(LexErrorKind::MalformedEscape {
            text: format!("\\{}", digits),
        })
    })?;
    std::char::from_u32(n).ok_or(LexError::new(LexErrorKind::InvalidCodePoint { value: n }))
}

fn lex_char_octal(
    popped_char: char,
    chars: &mut std::iter::Peekable<std::str::Chars>,
) -> Result<char, LexError> {
    let mut count = 0;
    let digits = chars
        .peeking_take_while(|c| {
            c.is_digit(8) && {
                count += 1;
                count <= 3
            }
        })
        .collect::<String>();
    let octal_str = format!("{}{}", popped_char, digits);
    let n = u32::from_str_radix(octal_str.as_str(), 8).map_err(|_| {
        LexError::new(LexErrorKind::MalformedEscape {
            text: format!("\\{}", digits),
        })
    })?;
    std::char::from_u32(n).ok_or(LexError::new(LexErrorKind::InvalidCodePoint { value: n }))
}

fn lex_unicode_char(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    size: usize,
) -> Result<char, LexError> {
    let mut count = 0;
    let digits = chars
        .peeking_take_while(|c| {
            c.is_ascii_hexdigit() && {
                count += 1;
                count <= size
            }
        })
        .collect::<String>();
    let n = u32::from_str_radix(&digits, 16).map_err(|_| {
        LexError::new(LexErrorKind::MalformedEscape {
            text: format!("\\{}", digits),
        })
    })?;
    std::char::from_u32(n).ok_or(LexError::new(LexErrorKind::InvalidCodePoint { value: n }))
}
