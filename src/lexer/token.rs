// =============================================================================
// token.rs — Logos-generated lexer
//
// Logos turns this enum into a fast state-machine lexer at compile time.
// Each #[token] / #[regex] attribute declares how to recognize that variant.
// The #[logos(skip ...)] line discards whitespace between tokens.
// =============================================================================

use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]            // skip whitespace
#[logos(skip r"//[^\n]*")]                // skip line comments
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
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    IntLit(i64),

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