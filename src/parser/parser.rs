use crate::ast::ast::Item;
use crate::ast::span::Span;
use crate::lexer::token::{SpannedToken, Token};
use crate::parse_error;
use crate::parser::env::Env;
use crate::parser::errors::{Contextualize, ParseError};

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
    pub(super) env: Env,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self {
            tokens,
            pos: 0,
            env: Default::default(),
        }
    }

    pub(super) fn attempt<T, F>(&mut self, f: F) -> Result<T, ParseError>
    where
        F: FnOnce(&mut Self) -> Result<T, ParseError>,
    {
        let save = self.pos;
        match f(self) {
            Ok(v) => Ok(v),
            Err(e) => {
                self.pos = save;
                Err(e)
            }
        }
    }

    pub(super) fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|st: &SpannedToken| &st.token)
    }

    pub(super) fn peek_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|st| st.span.clone().into_ast())
            .unwrap_or(Span { start: 0, end: 0 })
    }

    // span of the most recently consumed token
    pub(super) fn prev_span(&self) -> Span {
        self.tokens
            .get(self.pos.saturating_sub(1))
            .map(|st| st.span.clone().into_ast())
            .unwrap_or(Span { start: 0, end: 0 })
    }

    pub(super) fn advance(&mut self) -> Option<SpannedToken> {
        let tok: Option<SpannedToken> = self.tokens.get(self.pos).cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    /// Rename of advance for understanding purposes
    #[inline]
    pub(super) fn consumes_token(&mut self) -> () {
        self.advance();
    }

    pub(super) fn expect(&mut self, expected: &Token) -> Result<SpannedToken, ParseError> {
        match self.peek() {
            Some(t) if t == expected => Ok(self.advance().unwrap()),
            other => Err(parse_error!(
                "expected {:?}, found {:?} @ {}..{}",
                expected,
                other,
                self.peek_span().start,
                self.peek_span().end
            )
            .span(self.peek_span().start, self.peek_span().end)),
        }
    }

    pub(super) fn at_eof(&self) -> bool {
        if self.pos >= self.tokens.len() {
            true
        } else {
            false
        }
    }

    pub fn parse_translation_unit(&mut self) -> Result<Vec<Item>, ParseError> {
        self.env.push();
        let mut items: Vec<Item> = Vec::new();
        while !self.at_eof() {
            items.push(
                self.parse_item()
                    .on_err_context("parse_translation_unit", "failed to parse translation unit")?,
            );
        }
        Ok(items)
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        self.attempt(|p| {
            Ok(Item::FunctionDef(p.parse_function_def().on_err_context(
                "parse_item",
                "failed to parse function definition",
            )?))
        })
        .or_else(|_| {
            Ok(Item::Declaration(self.parse_declaration().on_err_context(
                "parse_item",
                "failed to parse declaration",
            )?))
        })
    }
}

// Bridge token::Span -> ast::Span (kept as separate types for module decoupling)
trait IntoAstSpan {
    fn into_ast(self) -> Span;
}

impl IntoAstSpan for crate::lexer::token::Span {
    fn into_ast(self) -> Span {
        Span {
            start: self.start,
            end: self.end,
        }
    }
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
