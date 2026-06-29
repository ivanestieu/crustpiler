// =============================================================================
// token.rs — Logos-generated lexer
//
// Logos turns this enum into a fast state-machine lexer at compile time.
// Each #[token] / #[regex] attribute declares how to recognize that variant.
// The #[logos(skip ...)] line discards whitespace between tokens.
// =============================================================================

use crate::lexer::string::lex_string_lit;
use crate::lexer::float::{ lex_hex_float, lex_std_float};
use crate::lexer::int::lex_int;
use crate::lexer::char::lex_char_lit;
use crate::literals::{FloatLit, IntLit, StringLit};
use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]            // skip whitespace
#[logos(skip(r"//[^\n]*", allow_greedy = true))]                // skip line comments
pub enum Token {
    // ── Type-specifier keywords ──────────────────────────────────────────
    // (only `int` in this slice; add #[token("char")] etc. as you grow)
    #[token("void")]
    KwVoid,
    #[token("bool")]
    KwBool,
    #[token("char")]
    KwChar,
    #[token("short")]
    KwShort,
    #[token("int")]
    KwInt,
    #[token("float")]
    KwFloat,
    #[token("double")]
    KwDouble,

    #[token("signed")]
    KwSigned,
    #[token("unsigned")]
    KwUnsigned,
    #[token("long")]
    KwLong,

    // ── Punctuation ──────────────────────────────────────────────────────
    #[token("=")]
    Equals,
    #[token(";")]
    SemiColon,
    #[token(",")]
    Comma,

    // ── Integer literal ──────────────────────────────────────────────────
    // The callback parses the matched slice into an i64.
    // Returning Result lets a bad number become a lex error.
    #[regex(r"0[xX][0-9a-fA-F]+(?:[uU][lL]{0,2}|[lL]{0,2}[uU]?)", lex_int)]
    #[regex(r"0[0-7]*(?:[uU][lL]{0,2}|[lL]{0,2}[uU]?)", lex_int)] //  0 is handled here
    #[regex(r"[1-9][0-9]*(?:[uU][lL]{0,2}|[lL]{0,2}[uU]?)?", lex_int)]
    #[regex(r"0[bB][01]+(?:[uU][lL]{0,2}|[lL]{0,2}[uU]?)", lex_int)]
    Int(IntLit),

    // ── Float literal ──────────────────────────────────────────────────
    // The callback parses the matched slice into a f64.
    // Returning Result lets a bad number become a lex error.
    #[regex(r"(?:[0-9]*\.[0-9]+|[0-9]+\.[0-9]*)(?:[eE][+-]?[0-9]+)?[fFlL]?|[0-9]+[eE][+-]?[0-9]+[fFlL]?", lex_std_float)]
    #[regex(r"0[xX][0-9a-fA-F]+[pP][+-]?[0-9]+[fFlL]?", lex_hex_float)]
    #[regex(r"0[xX][0-9a-fA-F]*\.[0-9a-fA-F]+[pP][+-]?[0-9]+[fFlL]?", lex_hex_float)]
    #[regex(r"0[xX][0-9a-fA-F]+\.[pP][+-]?[0-9]+[fFlL]?", lex_hex_float)]
    Float(FloatLit),

    // ── Char literal ──────────────────────────────────────────────────
    // The callback parses the matched slice into a char.
    // Returning Result lets a bad char become a lex error.
    #[regex(r#"'\\['"?\\abfnrtv]'"#, lex_char_lit)]
    #[regex(r"'\\x[0-9a-fA-F]+'", lex_char_lit)]
    #[regex(r"'\\[0-7]{1,3}'", lex_char_lit)]
    #[regex(r"'\\u[0-9a-fA-F]{4}'", lex_char_lit)]
    #[regex(r"'\\U[0-9a-fA-F]{8}'", lex_char_lit)]
    #[regex(r"'[^']'", lex_char_lit)]
    CharLit(char),

    // ── String literal ──────────────────────────────────────────────────
    // The callback parses the matched slice into a String.
    // Returning Result lets a bad string become a lex error.
    #[regex(r#"([LuU]|(u8))?"([^"\\]|\\.)*""#, lex_string_lit)]
    StringLit(StringLit),

    // ── Identifier ───────────────────────────────────────────────────────
    // Must come after keywords: Logos prefers the longer/explicit match, but
    // ordering keeps intent clear. Owned String so the token outlives the src.
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl From<std::ops::Range<usize>> for Span {
    fn from(r: std::ops::Range<usize>) -> Self {
        Span { start: r.start, end: r.end }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

/// Run the Logos lexer over `src`, collecting tokens or the first error.
pub fn lex(src: &str) -> Result<Vec<SpannedToken>, String> {
    let mut out : Vec<SpannedToken> = Vec::new();
    for (result, range) in Token::lexer(src).spanned() {
        match result {
            Ok(token) => out.push(SpannedToken { token, span: range.into() }),
            Err(()) => {
                return Err(format!(
                    "lex error: unrecognized input at bytes {}..{} ({:?})",
                    range.start,
                    range.end,
                    &src[range.clone()]
                ));
            }
        }
    }
    Ok(out)
}