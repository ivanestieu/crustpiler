use crate::ast::ast::{Expr, GenericAssoc};
use crate::ast::operators::{AsAssignOp, AsBinaryOp, AsUnaryOp, PostfixOp, TraitBinaryOp, UnaryOp};
use crate::ast::span::{Span, Spanned};
use crate::lexer::token::Token;
use crate::parse_error;
use crate::parser::errors::{Contextualize, ParseError};
use crate::parser::parser::Parser;

impl Parser {
    fn as_basic_expr(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let tok = self.advance();
        let expr = match tok.unwrap().token {
            Token::Ident(s) => Expr::Ident(s),
            // Constant
            Token::Int(n) => Expr::IntLit(n),
            Token::Float(f) => Expr::FloatLit(f),
            Token::CharLit(c) => Expr::CharLit(c),
            // (Enum constants)
            // String
            Token::StringLit(s) => Expr::StringLit(s),
            Token::KwFuncName => Expr::FuncName("".to_string()),
            _ => unreachable!(),
        };
        let res = Ok(Spanned {
            node: expr,
            span: self.prev_span(),
        });
        res
    }

    fn parse_primary_expr(&mut self) -> Result<Spanned<Expr>, ParseError> {
        match self.peek() {
            // Identifier
            Some(Token::Ident(_)
                 // Constant
                 | Token::Int(_)
                 | Token::Float(_)
                 | Token::CharLit(_)
                 // (Enum constants)
                 // String
                 | Token::StringLit(_)
                 | Token::KwFuncName) => self.as_basic_expr(),
            // Nested in parentheses Expr
            Some(Token::LeftParenthesis) => {
                let start = self.peek_span();
                self.consumes_token();
                let inner_expr = self.parse_expr()
                    .on_err_context("parse_primary_expr", "in a primary expression starting with a left parenthesis, an inner expression is expected")?;
                self.expect(&Token::RightParenthesis)
                    .on_err_context("parse_primary_expr", "a nested expression inside parenthesis must end with a right parenthesis")?;
                Ok(Spanned { node: inner_expr.node, span: start.merge(&self.prev_span()) })
            }
            // Generic Selection
            Some(Token::KwGeneric) => {
                let start = self.peek_span();
                self.consumes_token();
                self.expect(&Token::LeftParenthesis)
                    .on_err_context("parse_primary_expr", "a left parenthesis is expected after the keyword Generic")?;
                let controlling = self.parse_assignment_expr()
                    .on_err_context("parse_primary_expr", "an assignment expression is expected between the parenthesis directly following the Generic keyword")?;
                self.expect(&Token::Comma)
                    .on_err_context("parse_primary_expr", "a right parenthesis should close a generic selection")?;
                let mut associated = Vec::new();
                loop {
                    match self.peek() {
                        Some(Token::RightParenthesis) => {self.consumes_token(); break},
                        Some(Token::Comma) => {self.consumes_token(); continue},
                        Some(Token::KwDefault) => {
                            self.consumes_token();
                            self.expect(&Token::Colon).on_err_context("parse_primary_expr", "a colon should follow the keyword Default")?;
                            associated.push(
                                GenericAssoc {
                                    type_name: None, value: self.parse_assignment_expr().on_err_context("parse_primary_expr", "an assignment expression is expected after the colon in the default cas of a Generic expression")?
                                }
                            );
                        }
                        _ => {
                            if let Ok(type_name) = self.parse_type_name() {
                                self.expect(&Token::Colon).on_err_context("parse_primary_expr", "a colon should follow the type name in a generic association")?;
                                associated.push(GenericAssoc { type_name: Some(type_name), value: self.parse_assignment_expr().on_err_context("parse_primary_expr", "an assignment expression is expected after the colon in a generic association")?});
                            }
                            else {
                                return Err(parse_error!("Generic selection: Expected identifier, found {:?}", self.peek()));
                            }
                        }
                    }
                };
                Ok(Spanned { node: Expr::Generic {
                    controlling: Box::new(controlling),
                    associated
                }, span: start.merge(&self.prev_span()) })
            }
            other => Err(parse_error!("Primary expression: Expected identifier, literal, '(' or _Generic, found {:?} @ {}", other, self.peek_span().to_string())),
        }
    }

    fn parse_compound_literal(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let start = self.peek_span();
        self.expect(&Token::LeftParenthesis).on_err_context(
            "parse_compound_literal",
            "expected left parenthesis to open compound literal",
        )?;
        let type_name = self.parse_type_name().on_err_context(
            "parse_type_name",
            "failed to parse type name in compound literal",
        )?;
        self.expect(&Token::RightParenthesis).on_err_context(
            "parse_compound_literal",
            "expected right parenthesis to close compound literal",
        )?;
        self.expect(&Token::LeftBrace).on_err_context(
            "parse_compound_literal",
            "expected left brace to open compound literal initializer list",
        )?;
        let initializers = self.parse_initializer_list().on_err_context(
            "parse_compound_literal",
            "failed to parse compound literal initializer list",
        )?;
        self.expect(&Token::Comma).ok(); // Optional trailing Comma
        self.expect(&Token::RightBrace).on_err_context(
            "parse_compound_literal",
            "expected right brace to close compound literal initializer list",
        )?;
        Ok(Spanned {
            node: Expr::CompoundLit {
                type_name,
                init: initializers,
            },
            span: start.merge(&self.prev_span()),
        })
    }

    fn parse_postfix_expr(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let start = self.peek_span();
        let mut base = self
            .attempt(|p| p.parse_compound_literal())
            .or_else(|_| self.parse_primary_expr())
            .on_err_context("parse_postfix_expr", "failed to parse postfix expression")?;
        loop {
            match self.peek() {
                Some(Token::LeftBracket) => {
                    self.consumes_token();
                    base = Spanned {
                        node: Expr::Index {
                            array: Box::new(base),
                            index: Box::new(self.parse_expr().on_err_context(
                                "parse_postfix_expr",
                                "failed to parse index expression",
                            )?),
                        },
                        span: start.merge(&self.prev_span()),
                    };
                    self.expect(&Token::RightBracket).on_err_context(
                        "parse_postfix_expr",
                        "expected right bracket to close index expression",
                    )?;
                }
                Some(Token::LeftParenthesis) => {
                    self.consumes_token();
                    let mut args = Vec::new();
                    while self.expect(&Token::RightParenthesis).is_err() {
                        args.push(self.parse_assignment_expr().on_err_context(
                            "parse_postfix_expr",
                            "failed to parse assignment expression",
                        )?);
                        self.expect(&Token::Comma).or(
                            if self.peek() == Some(&Token::RightParenthesis) {
                                continue;
                            } else {
                                Err(parse_error!(
                                    "Elements should be separated by commas in an argument list"
                                )
                                .span(self.peek_span().start, self.peek_span().end))
                            },
                        )?;
                    }
                    base = Spanned {
                        node: Expr::Call {
                            callee: Box::new(base),
                            args,
                        },
                        span: start.merge(&self.prev_span()),
                    };
                }
                Some(tok @ (Token::Dot | Token::PtrOp)) => {
                    let arrow = tok == &Token::PtrOp;
                    self.consumes_token();
                    let identifier = self.expect_identifier().on_err_context(
                        "parse_postfix_expr",
                        "expected identifier after member operator",
                    )?;
                    base = Spanned {
                        node: Expr::Member {
                            expr: Box::new(base),
                            field: identifier,
                            arrow,
                        },
                        span: start.merge(&self.prev_span()),
                    };
                }
                Some(tok @ (Token::IncOp | Token::DecOp)) => {
                    let inc_op = if tok == &Token::IncOp {
                        PostfixOp::PostInc
                    } else {
                        PostfixOp::PostDec
                    };
                    self.advance();
                    base = Spanned {
                        node: Expr::PostfixOp {
                            operand: Box::new(base),
                            op: inc_op,
                        },
                        span: start.merge(&self.prev_span()),
                    }
                }
                _ => break,
            }
        }
        Ok(base)
    }

    fn parse_unary_expr(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let start = self.peek_span();
        let to_spanned = |expr: Expr, end: Span| {
            Ok(Spanned {
                node: expr,
                span: start.merge(&end),
            })
        };
        if let postfix_expr = self.parse_postfix_expr()
            && postfix_expr.is_ok()
        {
            return postfix_expr;
        };
        match self.peek() {
            Some(tok @ (Token::IncOp | Token::DecOp)) => {
                let inc_op = if tok == &Token::IncOp {
                    UnaryOp::PreInc
                } else {
                    UnaryOp::PreDec
                };
                self.advance();
                to_spanned(
                    Expr::UnaryOp {
                        op: inc_op,
                        operand: Box::new(self.parse_unary_expr()?),
                    },
                    self.prev_span(),
                )
            }
            Some(
                tok @ (Token::Minus
                | Token::Plus
                | Token::ExclamationMark
                | Token::Tilde
                | Token::Star
                | Token::Ampersand),
            ) => {
                let unary_op = tok
                    .as_unary_op()
                    .on_err_context("parse_unary_expr", "failed to parse unary operator")?;
                self.advance();
                to_spanned(
                    Expr::UnaryOp {
                        op: unary_op,
                        operand: Box::new(self.parse_cast_expr().on_err_context(
                            "parse_unary_expr",
                            "failed to parse cast expression following a unary operator",
                        )?),
                    },
                    self.prev_span(),
                )
            }
            Some(Token::KwSizeof) => {
                self.consumes_token();
                self.attempt(|p| {
                    p.expect(&Token::LeftParenthesis)?;
                    let span = to_spanned(
                        Expr::SizeofType(p.parse_type_name().on_err_context(
                            "parse_unary_expr",
                            "failed to parse type name in sizeof expression",
                        )?),
                        p.prev_span(),
                    );
                    p.expect(&Token::RightParenthesis).on_err_context(
                        "parse_unary_expr",
                        "failed to parse right parenthesis in sizeof expression",
                    )?;
                    span
                })
                .or_else(|_| {
                    to_spanned(
                        Expr::SizeofExpr(Box::new(self.parse_unary_expr().on_err_context(
                            "parse_unary_expr",
                            "failed to parse unary expression in sizeof expression",
                        )?)),
                        self.prev_span(),
                    )
                })
            }
            Some(Token::KwAlignof) => {
                self.advance();
                self.expect(&Token::LeftParenthesis)
                    .on_err_context("parse_unary_expr", "failed to parse alignof expression")?;
                let span = to_spanned(
                    Expr::AlignofType(self.parse_type_name().on_err_context(
                        "parse_unary_expr",
                        "failed to parse type name in alignof expression",
                    )?),
                    self.prev_span(),
                );
                self.expect(&Token::RightParenthesis)
                    .on_err_context("parse_unary_expr", "')' expected to end Alignof expression")?;
                span
            }
            other => Err(parse_error!(
                "Unary expression: Expected unary expression, found {:?} @ {}..{}",
                other,
                start.start,
                self.peek_span().end
            )),
        }
    }

    fn parse_cast_expr(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let start = self.peek_span();
        self.attempt(|p| {
            p.expect(&Token::LeftParenthesis)?;
            if let Ok(type_name) = p.parse_type_name() {
                p.expect(&Token::RightParenthesis).on_err_context(
                    "parse_cast_expr",
                    "failed to parse right parenthesis in cast expression",
                )?;
                return Ok(Spanned {
                    node: Expr::Cast {
                        type_name,
                        expr: Box::new(p.parse_cast_expr().on_err_context(
                            "parse_cast_expr",
                            "failed to parse cast expression",
                        )?),
                    },
                    span: start.merge(&p.prev_span()),
                });
            } else {
                Err(parse_error!("Error will be discard by .or() call."))
            }
        })
        .or_else(|_| self.parse_unary_expr())
    }

    fn parse_binary_expr(&mut self, min_bp: Option<usize>) -> Result<Spanned<Expr>, ParseError> {
        let mut lhs = self
            .parse_cast_expr()
            .on_err_context("parse_binary_expr", "failed to parse cast expression")?;
        while let Some(token) = self.peek()
            && let Ok(op) = token.as_binary_op()
        {
            let (left_bp, right_bp) = op.binding_power();
            if left_bp < min_bp.unwrap_or(0) {
                break;
            }

            self.consumes_token();
            let rhs = self
                .parse_binary_expr(Some(right_bp))
                .on_err_context("parse_binary_expr", "failed to parse binary expression")?;
            let span = lhs.span.merge(&rhs.span);
            lhs = Spanned {
                node: Expr::BinaryOp {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            };
        }
        Ok(lhs)
    }

    pub(super) fn parse_conditional_expr(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let lhs = self.parse_binary_expr(None).on_err_context(
            "parse_conditional_expr",
            "failed to parse binary expression",
        )?;
        let middle: Spanned<Expr> = match self.peek() {
            Some(Token::InterrogationMark) => {
                self.advance();
                self.parse_expr().on_err_context(
                    "parse_conditional_expr",
                    "failed to parse conditional expression",
                )?
            }
            _ => return Ok(lhs),
        };
        self.expect(&Token::Colon).on_err_context(
            "parse_conditional_expr",
            "failed to parse colon in conditional expression",
        )?;
        let rhs = self.parse_conditional_expr().on_err_context(
            "parse_conditional_expr",
            "failed to parse conditional expression",
        )?;
        let span = lhs.span.merge(&rhs.span);
        Ok(Spanned {
            node: Expr::Ternary {
                cond: Box::new(lhs),
                then: Box::new(middle),
                els: Box::new(rhs),
            },
            span,
        })
    }

    pub(super) fn parse_assignment_expr(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let mut lhs = self.parse_conditional_expr().on_err_context(
            "parse_assignment_expr",
            "failed to parse conditional expression",
        )?;
        if !matches!(
            lhs,
            Spanned {
                node: Expr::BinaryOp { .. },
                ..
            }
        ) {
            return Ok(lhs);
        }
        while let Some(token) = self.peek()
            && let Ok(op) = token.as_assign_op()
        {
            self.consumes_token();
            let rhs = self.parse_assignment_expr().on_err_context(
                "parse_assignment_expr",
                "failed to parse assignment expression",
            )?;
            let span = lhs.span.merge(&rhs.span);
            lhs = Spanned {
                node: Expr::Assign {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            };
        }
        Ok(lhs)
    }

    pub(super) fn parse_expr(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let start = self.peek_span();
        let mut lhs = self
            .parse_assignment_expr()
            .on_err_context("parse_expr", "failed to parse assignment expression")?;
        while self.expect(&Token::Comma).is_ok() {
            let rhs = self
                .parse_assignment_expr()
                .on_err_context("parse_expr", "failed to parse assignment expression")?;
            lhs = Spanned {
                node: Expr::Comma(Box::new(lhs), Box::new(rhs)),
                span: start.merge(&self.prev_span()),
            }
        }
        Ok(lhs)
    }
}
