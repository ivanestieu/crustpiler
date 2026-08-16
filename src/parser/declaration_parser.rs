use crate::ast::decl_specifiers::TypeExprContext;
use crate::ast::declarations::{Decl, Declaration, InitDeclarator, StaticAssert};
use crate::ast::span::Spanned;
use crate::ast::struct_union::{FieldDecl, StructMember, StructOrUnion};
use crate::lexer::token::Token;
use crate::parse_error;
use crate::parser::errors::{Contextualize, ParseError};
use crate::parser::parser::Parser;

impl Parser {
    fn parse_static_assert_declaration(&mut self) -> Result<StaticAssert, ParseError> {
        self.expect(&Token::KwStaticAssert).on_err_context(
            "parse_static_assert_declaration",
            "failed to parse static assert keyword",
        )?;
        self.expect(&Token::LeftParenthesis).on_err_context(
            "parse_static_assert_declaration",
            "failed to parse left parenthesis in static assert declaration",
        )?;
        let cond = self.parse_conditional_expr().on_err_context(
            "parse_static_assert_declaration",
            "failed to parse conditional expression in static assert declaration",
        )?; // evaluable at compile-time
        self.expect(&Token::Comma).on_err_context(
            "parse_static_assert_declaration",
            "failed to parse comma in static assert declaration",
        )?;
        let message = match self.peek() {
            Some(Token::StringLit(s)) => {
                let slit = s.clone();
                self.advance();
                self.expect(&Token::RightParenthesis).on_err_context(
                    "parse_static_assert_declaration",
                    "failed to parse right parenthesis in static assert declaration",
                )?;
                self.expect(&Token::SemiColon).on_err_context(
                    "parse_static_assert_declaration",
                    "failed to parse semicolon in static assert declaration",
                )?;
                slit
            }
            other => {
                return Err(parse_error!(
                    "Static assert: Expected string literal, found {:?}",
                    other
                ));
            }
        };
        Ok(StaticAssert {
            cond: Box::new(cond.node),
            message,
        })
    }

    pub(super) fn parse_declaration(&mut self) -> Result<Declaration, ParseError> {
        let start = self.peek_span();

        if let Ok(static_assert) = self.attempt(|p| p.parse_static_assert_declaration()) {
            return Ok(Declaration::StaticAssert(Spanned {
                node: static_assert,
                span: start.merge(&self.prev_span()),
            }));
        }

        let type_expr = self
            .parse_type_expr(TypeExprContext::Declaration)
            .on_err_context("parse_declaration", "Expected a mandatory type expression")?;

        let mut declarators: Vec<InitDeclarator> = if let Ok(ok) = self.parse_init_declarator() {
            vec![ok]
        } else {
            vec![]
        };
        while self.expect(&Token::Comma).is_ok() {
            declarators.push(self.parse_init_declarator().on_err_context(
                "parse_declaration",
                "Expected another initializer after one followed by a comma",
            )?);
        }

        if type_expr.is_typedef() {
            declarators.iter().for_each(|init_decl| {
                if let Some(ident) = init_decl.declarator.ident() {
                    self.env.define_typedef(String::from(ident));
                }
            });
        }

        self.expect(&Token::SemiColon)
            .on_err_context("parse_declaration", "a semicolon should end a declaration")?;
        Ok(Declaration::Normal(Spanned {
            node: Decl {
                specifiers: type_expr,
                declarators,
            },
            span: start.merge(&self.prev_span()),
        }))
    }

    fn parse_struct_declaration(&mut self) -> Result<StructMember, ParseError> {
        if let Ok(static_assert) = self.parse_static_assert_declaration() {
            return Ok(StructMember::StaticAssert(static_assert));
        }
        let specifiers = self
            .parse_type_expr(TypeExprContext::StructUnionField)
            .on_err_context(
                "parse_struct_declaration",
                "failed to parse type expression in struct declaration",
            )?;
        let declarators = self.parse_struct_declarator_list().on_err_context(
            "parse_struct_declaration",
            "failed to parse struct declarator list in struct declaration",
        )?;
        self.expect(&Token::SemiColon).on_err_context(
            "parse_struct_declaration",
            "a struct declaration must be followed by a semicolon",
        )?;
        Ok(StructMember::Field(FieldDecl {
            type_expr: specifiers,
            declarators,
        }))
    }

    pub(super) fn parse_struct_or_union(&mut self) -> Result<StructOrUnion, ParseError> {
        let name = self.expect_identifier().ok();
        if self.expect(&Token::LeftBrace).is_err() {
            if name.is_some() {
                return Ok(StructOrUnion { name, fields: None });
            }
            return Err(parse_error!(
                "StructOrUnion: struct and unions cannot be anonymous without a body, found {:?}",
                self.peek()
            ));
        }
        let mut fields = Vec::new();
        while let Ok(field) = self.parse_struct_declaration() {
            fields.push(field);
        }
        self.expect(&Token::RightBrace).on_err_context(
            "parse_struct_or_union",
            "failed to parse right brace in struct or union declaration",
        )?;
        Ok(StructOrUnion {
            name,
            fields: Some(fields),
        })
    }
}
