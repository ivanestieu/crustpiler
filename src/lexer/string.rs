use crate::lexer::char::lex_char;
use crate::lexer::errors::{LexError, LexErrorKind};
use crate::lexer::token::Token;
use crate::literals::{ StringLit, StringPrefix};

pub fn lex_string_lit(lex : &mut logos::Lexer<Token>) -> Result<StringLit, LexError> {
    let s = lex.slice();
    let mut chars = s.chars();
    Ok(match chars.next() {
        Some('L') => StringLit { prefix: StringPrefix::Wide, value: interpret_string(&s[2..])? },
        Some('u') => match chars.next().ok_or(LexError::new( LexErrorKind:: BadStringLiteral { text : String::from("'u' prefix should be followed by either '8' or a '\"'") }))? {
            '8' => StringLit { prefix: StringPrefix::Utf8, value: interpret_string(&s[3..])? },
            _ => StringLit { prefix: StringPrefix::Utf16, value: interpret_string(&s[2..])? },
        },
        Some('U') => StringLit { prefix: StringPrefix::Utf32, value: interpret_string(&s[2..])? },
        _ => StringLit { prefix: StringPrefix::None, value: interpret_string(&s[1..])? },
    })
}

fn interpret_string(string: &str) -> Result<String, LexError> {
    let mut chars = string.chars().peekable();
    let mut result = String::new();
    while let Some(&c) = chars.peek() {
        match c {
            '"' => break,
            _ => {
                result.push(lex_char(&mut chars)?);
            }
        }
    }
    Ok(result)
}