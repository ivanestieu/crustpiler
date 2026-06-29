use itertools::Itertools;
use crate::lexer::char::lex_char;
use crate::lexer::token::Token;
use crate::literals::{ StringLit, StringPrefix};

pub fn lex_string_lit(lex : &mut logos::Lexer<Token>) -> Option<StringLit> {
    let s = lex.slice();
    match s.chars().next() {
        Some('L') => Some(StringLit { prefix: StringPrefix::Wide, value: s[2..s.len()-1].to_string() }),
        Some('u') => Some(StringLit { prefix: StringPrefix::Utf16, value: s[2..s.len()-1].to_string() }),
        Some('U') => Some(StringLit { prefix: StringPrefix::Utf32, value: s[2..s.len()-1].to_string() }),
        _ => Some(StringLit { prefix: StringPrefix::None, value: s[1..s.len()-1].to_string() }),
    }
}

fn interpret_chars(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<String> {
    let mut result = String::new();
    while let Some(&c) = chars.peek() {
        match c {
            '"' => break,
            '\\' => {
                chars.next(); // consume the backslash
                if let Some(escaped) = lex_char(chars) {
                    result.push(escaped);
                } else {
                    return None; // invalid escape sequence
                }
            }
            _ => {
                result.push(c);
                chars.next(); // consume the character
            }
        }
    }
    Some(result)
}