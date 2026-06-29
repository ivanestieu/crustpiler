use itertools::Itertools;
use crate::lexer::token::Token;

pub fn lex_char_lit(lex: &mut logos::Lexer<Token>) -> Option<char> {
    let slice : &str = lex.slice();
    let inner : &str = &slice[1..slice.len() - 1]; // strip quotes
    lex_char(&mut inner.chars().peekable())
}

pub fn lex_char(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<char> {
    if chars.peek().is_none() {
        return None;
    }
    if chars.peek() != Some(&'\\') {
        return chars.next();
    }
    match chars.next()? {
        '\'' => Some('\''),
        '"'  => Some('"'),
        '?'  => Some('?'),
        '\\' => Some('\\'),
        'n'  => Some('\n'),
        't'  => Some('\t'),
        'r'  => Some('\r'),
        'a'  => Some('\x07'),
        'b'  => Some('\x08'),
        'f'  => Some('\x0C'),
        'v'  => Some('\x0B'),
        'x' => lex_hex_char(chars),
        'u' => lex_unicode_char(chars, 4),
        'U' => lex_unicode_char(chars, 8),
        '0'..='7' => lex_char_octal(chars),
        _ => None,
    }
}

fn lex_hex_char(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<char> {
    std::char::from_u32(
        u32::from_str_radix(
            chars.peeking_take_while(|c| c.is_ascii_hexdigit()).collect::<String>().as_str(),
            16
        ).ok()?
    )
}

fn lex_char_octal(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<char> {
    let mut count = 0;
    std::char::from_u32(
        u32::from_str_radix(
            chars.peeking_take_while(
                |c| c.is_digit(8) && { count += 1; count <= 3 }
            ).collect::<String>().as_str(),
            8
        ).ok()?
    )
}

fn lex_unicode_char(chars: &mut std::iter::Peekable<std::str::Chars>, size: usize) -> Option<char> {
    let mut count = 0;
    std::char::from_u32(
        u32::from_str_radix(
            chars.peeking_take_while(
                |c| c.is_ascii_hexdigit() && { count += 1; count <=  size}
            ).collect::<String>().as_str(),
        16
        ).ok()?
    )
}
