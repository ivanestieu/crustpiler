// =============================================================================
// token.rs — Logos-generated lexer
//
// Logos turns this enum into a fast state-machine lexer at compile time.
// Each #[token] / #[regex] attribute declares how to recognize that variant.
// The #[logos(skip ...)] line discards whitespace between tokens.
// =============================================================================

use crate::lexer::errors::LexError;
use crate::lexer::string::lex_string_lit;
use crate::lexer::float::{ lex_hex_float, lex_std_float};
use crate::lexer::int::lex_int;
use crate::lexer::char::lex_char_lit;
use crate::literals::{FloatLit, IntLit, StringLit};
use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(error = LexError)]
#[logos(subpattern O = r"[0-7]")]
#[logos(subpattern D = r"[0-9]")]
#[logos(subpattern NZ = r"[1-9]")]
#[logos(subpattern L = r"[a-zA-Z_]")]
#[logos(subpattern A = r"[a-zA-Z_0-9]")]
#[logos(subpattern H = r"[a-fA-F0-9]")]
#[logos(subpattern HP = r"(0[xX])")]
#[logos(subpattern E = r"([Ee][+-]?(?&D)+)")]
#[logos(subpattern P = r"([Pp][+-]?(?&D)+)")]
#[logos(subpattern FS = r"(f|F|l|L)")]
#[logos(subpattern IS = r"(((u|U)(l|L|ll|LL)?)|((l|L|ll|LL)(u|U)?))")]
#[logos(subpattern CP = r"(u|U|L)")]
#[logos(subpattern SP = r"(u8|u|U|L)")]
#[logos(subpattern ES = r#"(\\(['"?\\abfnrtv]|(?&O){1,3}|x(?&H)+|u(?&H){4}|U(?&H){8}))"#)]
#[logos(subpattern WS = r"[ \t\v\n\f]")]
#[logos(skip(r"//[^\n]*\n", allow_greedy = true))] // skip line comments
#[logos(skip r"[ \t\r\n\f]+")] // skip whitespace
#[logos(skip(r"/\*([^*]|\*+[^*/])*\*+/"))]
pub enum Token {
    // ── Keywords ──────────────────────────────────────────
    #[token("auto")]
    KwAuto,
    #[token("break")]
    KwBreak,
    #[token("case")]
    KwCase,
    #[token("char")]
    KwChar,
    #[token("const")]
    KwConst,
    #[token("continue")]
    KwContinue,
    #[token("default")]
    KwDefault,
    #[token("do")]
    KwDo,
    #[token("double")]
    KwDouble,
    #[token("else")]
    KwElse,
    #[token("enum")]
    KwEnum,
    #[token("extern")]
    KwExtern,
    #[token("float")]
    KwFloat,
    #[token("for")]
    KwFor,
    #[token("goto")]
    KwGoto,
    #[token("if")]
    KwIf,
    #[token("inline")]
    KwInline,
    #[token("int")]
    KwInt,
    #[token("long")]
    KwLong,
    #[token("register")]
    KwRegister,
    #[token("restrict")]
    KwRestrict,
    #[token("return")]
    KwReturn,
    #[token("short")]
    KwShort,
    #[token("signed")]
    KwSigned,
    #[token("sizeof")]
    KwSizeof,
    #[token("static")]
    KwStatic,
    #[token("struct")]
    KwStruct,
    #[token("switch")]
    KwSwitch,
    #[token("typedef")]
    KwTypedef,
    #[token("union")]
    KwUnion,
    #[token("unsigned")]
    KwUnsigned,
    #[token("void")]
    KwVoid,
    #[token("volatile")]
    KwVolatile,
    #[token("while")]
    KwWhile,

    // ── Specific Keywords ──────────────────────────────────────────
    #[token("_Alignas")]
    KwAlignas,
    #[token("_Alignof")]
    KwAlignof,
    #[token("_Atomic")]
    KwAtomic,
    #[token("_Bool")]
    KwBool,
    #[token("_Complex")]
    KwComplex,
    #[token("_Generic")]
    KwGeneric,
    #[token("_Imaginary")]
    KwImaginary,
    #[token("_Noreturn")]
    KwNoreturn,
    #[token("_Static_assert")]
    KwStaticAssert,
    #[token("_Thread_local")]
    KwThreadLocal,
    #[token("__func__")]
    KwFuncName,

    // ── Identifier ───────────────────────────────────────────────────────
    // Must come after keywords: Logos prefers the longer/explicit match, but
    // ordering keeps intent clear. Owned String so the token outlives the src.
    #[regex(r"(?&L)(?&A)*", |lex| lex.slice().to_string())]
    Ident(String),

    // ── Integer literal ──────────────────────────────────────────────────
    // The callback parses the matched slice into an i64.
    // Returning Result lets a bad number become a lex error.
    #[regex(r"(?&HP)(?&H)+(?&IS)?", lex_int)]
    #[regex(r"(?&NZ)(?&D)*(?&IS)?", lex_int)]
    #[regex(r"0(?&O)*(?&IS)?", lex_int)]
    Int(IntLit),

    // ── Float literal ──────────────────────────────────────────────────
    // The callback parses the matched slice into a f64.
    // Returning Result lets a bad number become a lex error.
    #[regex(r"(?&D)+(?&E)(?&FS)?", lex_std_float)]
    #[regex(r"(?&D)*\.(?&D)+(?&E)?(?&FS)?", lex_std_float)]
    #[regex(r"(?&D)+\.(?&E)?(?&FS)?", lex_std_float)]
    #[regex(r"(?&HP)(?&H)+(?&P)(?&FS)?", lex_hex_float)]
    #[regex(r"(?&HP)(?&H)*\.(?&H)+(?&P)(?&FS)?", lex_hex_float)]
    #[regex(r"(?&HP)(?&H)+\.(?&P)(?&FS)?", lex_hex_float)]
    Float(FloatLit),

    // ── Char literal ──────────────────────────────────────────────────
    // The callback parses the matched slice into a char.
    // Returning Result lets a bad char become a lex error.
    #[regex(r"(?&CP)?'([^'\\\n]|(?&ES))+'", lex_char_lit)]
    CharLit(char),

    // ── String literal ──────────────────────────────────────────────────
    // The callback parses the matched slice into a String.
    // Returning Result lets a bad string become a lex error.
    #[regex(r#"((?&SP)?"([^"\\\n]|(?&ES))*"(?&WS)*)+"#, lex_string_lit)]
    StringLit(StringLit),

    // ── Operators ──────────────────────────────────────────────────────
    #[token("...")]
    Ellipsis,
    #[token(">>=")]
    RightAssign,
    #[token("<<=")]
    LeftAssign,
    #[token("+=")]
    AddAssign,
    #[token("-=")]
    SubAssign,
    #[token("*=")]
    MulAssign,
    #[token("/=")]
    DivAssign,
    #[token("%=")]
    ModAssign,
    #[token("&=")]
    AndAssign,
    #[token("^=")]
    XorAssign,
    #[token("|=")]
    OrAssign,
    #[token(">>")]
    RightOp,
    #[token("<<")]
    LeftOp,
    #[token("++")]
    IncOp,
    #[token("--")]
    DecOp,
    #[token("->")]
    PtrOp,
    #[token("&&")]
    AndOp,
    #[token("||")]
    OrOp,
    #[token("<=")]
    LeOp,
    #[token(">=")]
    GeOp,
    #[token("==")]
    EqOp,
    #[token("!=")]
    NeOp,

    // ── Punctuation ──────────────────────────────────────────────────────
    #[token(";")]
    SemiColon,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("=")]
    Equals,
    #[token("(")]
    LeftParenthesis,
    #[token(")")]
    RightParenthesis,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token(".")]
    Dot,
    #[token("&")]
    Ampersand,
    #[token("!")]
    ExclamationMark,
    #[token("~")]
    Tilde,
    #[token("-")]
    Minus,
    #[token("+")]
    Plus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percentage,
    #[token("<")]
    LessThan,
    #[token(">")]
    GreaterThan,
    #[token("^")]
    Caret,
    #[token("|")]
    Pipe,
    #[token("?")]
    InterrogationMark,

    EOF,
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
pub fn lex(src: &str) -> Result<Vec<SpannedToken>, LexError> {
    let mut out : Vec<SpannedToken> = Vec::new();
    for (result, range) in Token::lexer(src).spanned() {
        match result {
            Ok(token) => out.push(SpannedToken { token, span: range.into() }),
            Err(mut e) => {
                if e.span.is_none() {
                    e.span = Some(range);
                }
                return Err(e);
            }
        }
    }
    Ok(out)
}