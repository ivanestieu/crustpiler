// =============================================================================
// error.rs — a custom error type for the Logos lexer.
//
// Logos requires the error type to implement Default (used for the implicit
// "no rule matched" case). We carry a structured `kind` plus an optional span
// so callers can render located diagnostics.
// =============================================================================

use std::fmt;
use std::num::{ParseIntError, ParseFloatError};

/// What specifically went wrong while lexing a token.
#[derive(Debug, Clone, PartialEq)]
pub enum LexErrorKind {
    /// Catch-all for "no regex matched here" — Logos produces this via Default.
    Unrecognized,

    // ── Numeric literals ────────────────────────────────────────────────
    /// Integer text didn't parse (overflow, empty, bad digit for the base).
    InvalidInteger { text: String, reason: String },
    /// Float text didn't parse.
    InvalidFloat { text: String, reason: String },
    /// Hex float couldn't be converted (out of range, malformed).
    InvalidHexFloat { text: String },

    // ── Char / string escapes ───────────────────────────────────────────
    /// `\q` and friends — backslash followed by an unknown letter.
    UnknownEscape { escape: char },
    /// `\x` with no hex digits, or `\u`/`\U` with too few.
    MalformedEscape { text: String },
    /// `\uD800` (surrogate) or `\U00110000` (> U+10FFFF): not a Unicode scalar.
    InvalidCodePoint { value: u32 },
    /// A char literal that contained zero or more-than-allowed characters.
    BadCharLiteral { text: String },
    BadStringLiteral { text: String },
}

impl fmt::Display for LexErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexErrorKind::Unrecognized => {
                write!(f, "unrecognized token")
            }
            LexErrorKind::InvalidInteger { text, reason } => {
                write!(f, "invalid integer literal `{}`: {}", text, reason)
            }
            LexErrorKind::InvalidFloat { text, reason } => {
                write!(f, "invalid float literal `{}`: {}", text, reason)
            }
            LexErrorKind::InvalidHexFloat { text } => {
                write!(f, "invalid hexadecimal float literal `{}`", text)
            }
            LexErrorKind::UnknownEscape { escape } => {
                write!(f, "unknown escape sequence `\\{}`", escape)
            }
            LexErrorKind::MalformedEscape { text } => {
                write!(f, "malformed escape sequence `{}`", text)
            }
            LexErrorKind::InvalidCodePoint { value } => {
                write!(f, "`\\u{{{:04X}}}` is not a valid Unicode scalar value", value)
            }
            LexErrorKind::BadCharLiteral { text } => {
                write!(f, "invalid character literal `{}`", text)
            }
            LexErrorKind::BadStringLiteral { text } => {
                write!(f, "invalid string literal `{}`", text)
            }
        }
    }
}

/// The error Logos carries. `kind` says what; `span` says where (byte range).
/// `span` is None for the Default/Unrecognized case because Logos fills the
/// span in itself when it reports the error position — but you can attach one
/// in your own callbacks for precise locations.
#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub kind: LexErrorKind,
    pub span: Option<std::ops::Range<usize>>,
}

impl LexError {
    pub fn new(kind: LexErrorKind) -> Self {
        Self { kind, span: None }
    }

    pub fn with_span(kind: LexErrorKind, span: std::ops::Range<usize>) -> Self {
        Self { kind, span: Some(span) }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.span {
            Some(s) => write!(f, "{} (at bytes {}..{})", self.kind, s.start, s.end),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl std::error::Error for LexError {}

// ── REQUIRED by Logos ────────────────────────────────────────────────────
// When no regex matches, Logos constructs the error via Default. We map that
// to the Unrecognized kind. (Logos associates the offending span separately,
// retrievable via the lexer's .span() when you drive it manually.)
impl Default for LexError {
    fn default() -> Self {
        LexError::new(LexErrorKind::Unrecognized)
    }
}

// ── Ergonomic conversions so callbacks can use `?` ───────────────────────
// These let a callback write e.g. `text.parse::<u64>()?` and have the
// std error auto-convert into our LexError.
impl From<ParseIntError> for LexError {
    fn from(e: ParseIntError) -> Self {
        LexError::new(LexErrorKind::InvalidInteger {
            text: String::new(),
            reason: e.to_string(),
        })
    }
}

impl From<ParseFloatError> for LexError {
    fn from(e: ParseFloatError) -> Self {
        LexError::new(LexErrorKind::InvalidFloat {
            text: String::new(),
            reason: e.to_string(),
        })
    }
}