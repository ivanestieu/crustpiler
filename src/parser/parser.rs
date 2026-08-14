use crate::ast::ast::{Expr, GenericAssoc, Item};
use crate::ast::decl_specifiers::{TypeExprBuilder, TypeExprContext};
use crate::ast::declarations::{
    AlignmentSpecifier, Decl, Declaration, Designator, InitDeclarator, InitItem, Initializer,
    StaticAssert,
};
use crate::ast::declarator::{ArraySize, Declarator};
use crate::ast::enums::{EnumSpec, Enumerator};
use crate::ast::function_def::FunctionDef;
use crate::ast::operators::{AsAssignOp, AsBinaryOp, AsUnaryOp, PostfixOp, TraitBinaryOp, UnaryOp};
use crate::ast::parameters::ParamDecl;
use crate::ast::span::{Span, Spanned};
use crate::ast::struct_union::{FieldDecl, FieldDeclarator, StructMember, StructOrUnion};
use crate::ast::types::{
    AsStorageClass, AsTypeQualifier, BaseType, Complex, FunctionSpecifier, Sign, TypeExpr,
    TypeName, TypeQualifier, TypeSpec,
};
use crate::lexer::token::{SpannedToken, Token};
use crate::parse_error;
use crate::parser::env::Env;
use crate::parser::errors::{Contextualize, ParseError};

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
    env: Env,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self {
            tokens,
            pos: 0,
            env: Default::default(),
        }
    }
    fn attempt<T, F>(&mut self, f: F) -> Result<T, ParseError>
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

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|st: &SpannedToken| &st.token)
    }

    fn peek_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|st| st.span.clone().into_ast())
            .unwrap_or(Span { start: 0, end: 0 })
    }

    // span of the most recently consumed token
    fn prev_span(&self) -> Span {
        self.tokens
            .get(self.pos.saturating_sub(1))
            .map(|st| st.span.clone().into_ast())
            .unwrap_or(Span { start: 0, end: 0 })
    }

    fn advance(&mut self) -> Option<SpannedToken> {
        let tok: Option<SpannedToken> = self.tokens.get(self.pos).cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    /// Rename of advance for understanding purposes
    fn consumes_token(&mut self) -> () {
        self.advance();
    }

    fn expect(&mut self, expected: &Token) -> Result<SpannedToken, ParseError> {
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

    fn at_eof(&self) -> bool {
        if self.pos >= self.tokens.len() {
            true
        } else {
            false
        }
    }

    fn extract_identifier(declarator: &Declarator) -> Result<String, ParseError> {
        match declarator {
            Declarator::Ident(name) => Ok(name.clone()),
            Declarator::Pointer { inner, .. }
            | Declarator::Array { inner, .. }
            | Declarator::Function { inner, .. } => Self::extract_identifier(inner),
            _ => Err(parse_error!("Expected identifier, found {:?}", declarator)),
        }
    }

    fn parse_declaration(&mut self) -> Result<Declaration, ParseError> {
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
                if let Ok(ident) = Self::extract_identifier(&init_decl.declarator) {
                    self.env.define_typedef(ident);
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
    fn parse_declarator(&mut self, is_abstract: bool) -> Result<Declarator, ParseError> {
        let pointers = self.parse_pointer();
        let inner = self
            .parse_direct_declarator(is_abstract)
            .on_err_context("parse_declarator", "a declarator is expected")?;
        Ok(Self::apply_pointer_levels(pointers, inner))
    }

    fn parse_direct_declarator(&mut self, is_abstract: bool) -> Result<Declarator, ParseError> {
        let func_name = if is_abstract {
            "parse_direct_abstract_declarator"
        } else {
            "parse_direct_declarator"
        };
        let mut base = match self.peek() {
            Some(Token::Ident(name)) if !is_abstract => {
                let ident = name.clone();
                self.consumes_token();
                Declarator::Ident(ident)
            }
            Some(Token::LeftBracket) if is_abstract => self
                .parse_array_suffix(is_abstract, Declarator::Abstract)
                .on_err_context(func_name, "expected array suffix")?,
            Some(Token::LeftParenthesis) if is_abstract => {
                let grouped = self.attempt(|p| {
                    p.expect(&Token::LeftParenthesis)?;
                    let inner = p.parse_declarator(true)?;
                    if matches!(inner, Declarator::Abstract) {
                        return Err(parse_error!(
                            "{}: empty group is a function suffix, not a grouping",
                            func_name
                        ));
                    }
                    p.expect(&Token::RightParenthesis)?;
                    Ok(inner)
                });
                match grouped {
                    Ok(inner) => inner, // then the loop below applies trailing suffixes
                    Err(_) => self
                        .parse_function_suffix(Declarator::Abstract)
                        .on_err_context(func_name, "expected function suffix")?,
                }
            }
            Some(Token::LeftParenthesis) => {
                self.consumes_token();
                let res = self.attempt(|p| {
                    p.parse_declarator(is_abstract)
                        .on_err_context(func_name, "expected declarator")
                });
                let inner_declarator = res?;
                self.expect(&Token::RightParenthesis).on_err_context(
                    func_name,
                    "a right parenthesis is expected to end a declarator",
                )?;
                inner_declarator
            }
            _ if is_abstract => Declarator::Abstract,
            other => {
                return Err(parse_error!(
                    "{}: expected identifier, found {:?} @ {}..{}",
                    func_name,
                    other,
                    self.peek_span().start,
                    self.peek_span().end
                ).span(self.peek_span().start, self.peek_span().end));
            }
        };
        loop {
            match self.peek() {
                Some(Token::LeftBracket) => {
                    base = self
                        .parse_array_suffix(is_abstract, base)
                        .on_err_context(func_name, "expected trailing array suffix")?;
                }
                Some(Token::LeftParenthesis) => {
                    base = self
                        .parse_function_suffix(base)
                        .on_err_context(func_name, "expected trailing function suffix")?;
                }
                _ => break,
            }
        }
        Ok(base)
    }

    /// init_decl := ident ("=" initializer)?
    fn parse_init_declarator(&mut self) -> Result<InitDeclarator, ParseError> {
        let declarator = self.parse_declarator(false).on_err_context(
            "parse_init_declarator",
            "an init declarator must start with a declarator",
        )?;

        let init = if self.expect(&Token::Equals).is_ok() {
            self.parse_initializer().ok()
        } else {
            None
        };

        Ok(InitDeclarator { declarator, init })
    }

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

    fn parse_designation(&mut self) -> Result<Vec<Designator>, ParseError> {
        let mut designators = Vec::new();
        while self.expect(&Token::Equals).is_err() {
            designators.push(match self.peek() {
                Some(Token::LeftBracket) => {
                    self.consumes_token();
                    let expr = self.parse_conditional_expr().on_err_context(
                        "parse_designation",
                        "in a designation, brackets must go around conditional expression",
                    )?;
                    self.expect(&Token::RightBracket).on_err_context(
                        "parse_designation",
                        "a designation starting by '[' must end with a right bracket",
                    )?;
                    Designator::Index(Box::new(expr.node)) // evaluable at compile-time
                }
                Some(Token::Dot) => {
                    self.consumes_token();
                    Designator::Field(match self.advance() {
                        Some(SpannedToken {
                            token: Token::Ident(name),
                            ..
                        }) => name,
                        other => {
                            return Err(parse_error!(
                                "Designator: Expected identifier after '.', found {:?} @ {}..{}",
                                other,
                                self.peek_span().start,
                                self.peek_span().end
                            ));
                        }
                    })
                }
                _ => {
                    return Err(parse_error!(
                        "Designator: Expected '[' or '.', found {:?} @ {}..{}",
                        self.peek(),
                        self.peek_span().start,
                        self.peek_span().end
                    ));
                }
            });
        }
        Ok(designators)
    }

    fn parse_initializer(&mut self) -> Result<Initializer, ParseError> {
        match self.peek() {
            Some(Token::LeftBrace) => {
                self.consumes_token();
                let initializers = self
                    .parse_initializer_list()
                    .on_err_context("parse_initializer", "failed to parse initializer list")?;
                self.expect(&Token::Comma).ok();
                self.expect(&Token::RightBrace).on_err_context(
                    "parse_initializer",
                    "expected right brace to close initializer list",
                )?;
                Ok(Initializer::List(initializers))
            }
            _ => Ok(Initializer::Expr(Box::new(
                self.parse_assignment_expr()
                    .on_err_context("parse_initializer", "failed to parse assignment expression")?
                    .node,
            ))),
        }
    }

    fn parse_initializer_list(&mut self) -> Result<Vec<InitItem>, ParseError> {
        let mut initializers = Vec::new();
        loop {
            let designation = self
                .attempt(|p| p.parse_designation())
                .unwrap_or(Vec::new());
            let initializer = self.parse_initializer();
            if initializer.is_err() {
                break;
            }
            let init_item = InitItem {
                designators: designation,
                value: initializer
                    .on_err_context("parse_initializer_list", "failed to parse initializer item")?,
            };
            initializers.push(init_item);
            if self.expect(&Token::Comma).is_err() {
                break;
            }
        }
        Ok(initializers)
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
                                Err(parse_error!("Elements should be separated by commas in an argument list")
                                    .span(self.peek_span().start, self.peek_span().end))
                            }
                        )?;
                        };
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
                    let identifier = self.peek();
                    if let Some(Token::Ident(name)) = identifier {
                        base = Spanned {
                            node: Expr::Member {
                                expr: Box::new(base),
                                field: name.clone(),
                                arrow,
                            },
                            span: start.merge(&self.prev_span()),
                        };
                        self.consumes_token();
                    } else {
                        return Err(parse_error!(
                            "Postfix expression: Expected identifier, found {:?}",
                            identifier
                        ));
                    }
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
                }).or_else(|_| to_spanned(
                        Expr::SizeofExpr(Box::new(self.parse_unary_expr().on_err_context(
                            "parse_unary_expr",
                            "failed to parse unary expression in sizeof expression",
                        )?)),
                        self.prev_span(),
                ))
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
                self.expect(&Token::RightParenthesis).on_err_context(
                    "parse_unary_expr", "')' expected to end Alignof expression"
                )?;
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
        }).or_else(|_| self.parse_unary_expr())
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

    fn parse_conditional_expr(&mut self) -> Result<Spanned<Expr>, ParseError> {
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

    fn parse_assignment_expr(&mut self) -> Result<Spanned<Expr>, ParseError> {
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

    fn parse_expr(&mut self) -> Result<Spanned<Expr>, ParseError> {
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

    fn parse_pointer(&mut self) -> Vec<Spanned<Vec<Spanned<TypeQualifier>>>> {
        let mut pointers = Vec::new();
        let mut start = self.peek_span();
        while self.expect(&Token::Star).is_ok() {
            let type_qualifiers = self.parse_type_qualifiers();
            pointers.push(Spanned {
                node: type_qualifiers,
                span: start.merge(&self.prev_span()),
            });
            start = self.peek_span();
        }
        pointers
    }

    fn apply_pointer_levels(
        levels: Vec<Spanned<Vec<Spanned<TypeQualifier>>>>,
        mut inner: Declarator,
    ) -> Declarator {
        for qualifiers in levels.into_iter().rev() {
            inner = Declarator::Pointer {
                qualifiers: qualifiers.node,
                inner: Box::new(inner),
            };
        }
        inner
    }

    fn parse_array_suffix(
        &mut self,
        is_abstract: bool,
        base: Declarator,
    ) -> Result<Declarator, ParseError> {
        self.expect(&Token::LeftBracket).on_err_context(
            "parse_array_suffix",
            "failed to parse left bracket in array suffix",
        )?;
        let mut is_static = matches!(self.peek(), Some(Token::KwStatic));
        if is_static {
            self.consumes_token();
        }
        let qualifiers = self.parse_type_qualifiers();
        let size = if self.peek() == Some(&Token::Star)
            && (!is_abstract || !is_static && qualifiers.is_empty())
        {
            self.consumes_token();
            ArraySize::Vla
        } else {
            is_static = match (self.peek(), is_static) {
                (Some(Token::KwStatic), true) => {
                    return Err(parse_error!(
                        "Array suffix: Static cannot be specified more than one time"
                    ));
                }
                (Some(Token::KwStatic), ..) => {
                    self.consumes_token();
                    true
                }
                _ => is_static,
            };
            self.attempt(|p| {
                Ok(ArraySize::Fixed(Box::new(
                    p.parse_assignment_expr()
                        .on_err_context("parse_array_suffix", "failed to parse fixed array size")?
                        .node,
                )))
            })
            .unwrap_or(ArraySize::None)
        };
        self.expect(&Token::RightBracket).on_err_context(
            "parse_array_suffix",
            "failed to parse right bracket in array suffix",
        )?;
        Ok(Declarator::Array {
            inner: Box::new(base),
            size,
            is_static,
            qualifiers,
        })
    }

    fn parse_kr_suffix(&mut self) -> Result<Vec<Expr>, ParseError> {
        let start = self.peek_span();
        // K & R params
        let mut old_style_params = Vec::new();
        while self.expect(&Token::RightParenthesis).is_err()
            && let Some(Token::Ident(name)) = self.peek()
        {
            old_style_params.push(Expr::Ident(name.clone()));
            self.advance();
        }
        if !old_style_params.is_empty() {
            return Ok(old_style_params);
        }
        Err(parse_error!(
            "K&R style suffix parameters not found @ {}..{}",
            start.start,
            self.prev_span().end
        ))
    }

    fn parse_function_suffix(&mut self, base: Declarator) -> Result<Declarator, ParseError> {
        self.expect(&Token::LeftParenthesis).on_err_context(
            "parse_function_suffix",
            "failed to parse left parenthesis in function suffix",
        )?;
        if self.peek() == Some(&Token::RightParenthesis) {
            self.consumes_token();
            return Ok(Declarator::Function {
                inner: Box::new(base),
                params: Some(Vec::new()),
                old_style_params: None,
                variadic: false,
            });
        }
        if let Ok(k_r_params) = self.attempt(|p| p.parse_kr_suffix()) {
            return Ok(Declarator::Function {
                inner: Box::new(base),
                params: None,
                old_style_params: Some(k_r_params),
                variadic: false,
            });
        }
        let mut params = Vec::new();
        let mut variadic = false;
        while self.peek() != Some(&Token::RightParenthesis) {
            if self.peek() == Some(&Token::Ellipsis) {
                self.consumes_token();
                variadic = true;
                break;
            }
            let type_expr = self
                .parse_type_expr(TypeExprContext::Declaration)
                .on_err_context(
                    "parse_function_suffix",
                    "failed to parse type expression in function suffix",
                )?;
            let declarator = if let Ok(decl) = self.attempt(|p|
                p.parse_declarator(false)) {
                decl
            } else {
                let abs_decl = self.parse_declarator(true).on_err_context(
                    "parse_function_suffix",
                    "failed to parse abstract declarator in function suffix",
                )?;
                if type_expr.is_void() && params.is_empty() {
                    break; // Special case ident(void)
                }
                abs_decl
            };
            params.push(ParamDecl {
                specifiers: type_expr,
                declarator,
            });
            if self.expect(&Token::Comma).is_err() {
                break;
            }
        }
        // Parse function parameters
        self.expect(&Token::RightParenthesis).on_err_context(
            "parse_function_suffix",
            "failed to parse right parenthesis in function suffix",
        )?;
        Ok(Declarator::Function {
            inner: Box::new(base),
            params: Some(params),
            old_style_params: None,
            variadic,
        })
    }

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

    fn parse_struct_declarator(&mut self) -> Result<FieldDeclarator, ParseError> {
        if self.expect(&Token::Colon).is_ok() {
            let constant_expr = self.parse_conditional_expr().on_err_context(
                "parse_struct_declarator",
                "failed to parse constant expression in struct declarator",
            )?; // evaluable at compile-time
            return Ok(FieldDeclarator {
                declarator: None,
                bit_width: Some(Box::new(constant_expr)),
            });
        }
        let declarator = self.parse_declarator(false).on_err_context(
            "parse_struct_declarator",
            "failed to parse declarator in struct declarator",
        )?;
        let field_declarator = Ok(FieldDeclarator {
            declarator: Some(declarator),
            bit_width: if self.expect(&Token::Colon).is_ok() {
                Some(Box::new(self.parse_conditional_expr().on_err_context(
                    "parse_struct_declarator",
                    "failed to parse conditional expression in struct declarator",
                )?)) // evaluable at compile-time
            } else {
                None
            },
        });
        field_declarator
    }

    fn parse_struct_declarator_list(&mut self) -> Result<Vec<FieldDeclarator>, ParseError> {
        let mut struct_declarations = Vec::new();
        while let Ok(struct_declarator) = self.parse_struct_declarator() {
            struct_declarations.push(struct_declarator);
            if self.expect(&Token::Comma).is_err() {
                break;
            }
        }
        Ok(struct_declarations)
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

    fn parse_struct_or_union(&mut self) -> Result<StructOrUnion, ParseError> {
        let name = if let Some(Token::Ident(iden)) = self.peek() {
            Some(iden.clone())
        } else {
            None
        };
        self.consumes_token();
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

    fn parse_enum(&mut self) -> Result<EnumSpec, ParseError> {
        let name = if let Some(Token::Ident(iden)) = self.peek() {
            Some(iden.clone())
        } else {
            None
        };
        self.consumes_token();
        if self.expect(&Token::LeftBrace).is_err() {
            if name.is_some() {
                return Ok(EnumSpec {
                    name,
                    variants: None,
                });
            }
            return Err(parse_error!(
                "Enum: enums cannot be anonymous without a body, found {:?}",
                self.peek()
            )
            .span(self.peek_span().start, self.peek_span().end));
        }
        let mut variants = Vec::new();
        while self.expect(&Token::RightBrace).is_err() {
            let variant_name = match self.advance() {
                Some(SpannedToken {
                    token: Token::Ident(name),
                    ..
                }) => name,
                other => return Err(parse_error!("Enum: Expected identifier, found {:?}", other)),
            };
            let variant_value = if self.expect(&Token::Equals).is_ok() {
                Some(Box::new(
                    self.parse_conditional_expr()
                        .on_err_context(
                            "parse_enum",
                            "failed to parse conditional expression in enum variant",
                        )?
                        .node,
                )) // evaluable at compile-time
            } else {
                None
            };
            variants.push(Enumerator {
                name: variant_name,
                value: variant_value,
            });
            if self.expect(&Token::Comma).is_ok() {
                continue;
            }
        }
        Ok(EnumSpec {
            name,
            variants: Some(variants),
        })
    }

    fn parse_alignment_specifier(&mut self) -> Result<AlignmentSpecifier, ParseError> {
        self.expect(&Token::LeftParenthesis).on_err_context(
            "parse_alignment_specifier",
            "failed to parse left parenthesis in alignment specifier",
        )?;
        let try_type = self.attempt(|p| p.parse_type_name());
        let align = if try_type.is_ok() {
            Ok(AlignmentSpecifier::TypeName(Box::new(
                try_type.on_err_context(
                    "parse_alignment_specifier",
                    "failed to parse type name in alignment specifier",
                )?,
            )))
        } else {
            Ok(AlignmentSpecifier::Expr(Box::new(
                self.parse_conditional_expr()
                    .on_err_context(
                        "parse_alignment_specifier",
                        "failed to parse conditional expression in alignment specifier",
                    )?
                    .node,
            )))
        };
        self.expect(&Token::RightParenthesis).on_err_context(
            "parse_alignment_specifier",
            "failed to parse right parenthesis in alignment specifier",
        )?;
        align
    }

    fn parse_type_name(&mut self) -> Result<TypeName, ParseError> {
        let type_expr = self
            .parse_type_expr(TypeExprContext::TypeName)
            .on_err_context(
                "parse_type_name",
                "failed to parse type expression in type name",
            )?;
        let abstract_decl = self.parse_declarator(true).unwrap_or(Declarator::Abstract);
        Ok(TypeName {
            type_expr,
            derived: abstract_decl,
        })
    }

    fn parse_type_expr(&mut self, context: TypeExprContext) -> Result<TypeExpr, ParseError> {
        let start = self.peek_span().start;
        let mut builder = TypeExprBuilder::new(context);
        loop {
            match self.peek() {
                // Storage
                Some(token) if let Ok(sc) = token.as_storage_class() => {
                    builder.add_storage(sc).on_err_context(
                        "parse_type_expr",
                        "failed to parse storage class in type expression",
                    )?
                }

                // Arithmetic related
                Some(Token::KwVoid) => builder
                    .set_void()
                    .on_err_context("parse_type_expr", "failed to parse void in type expression")?,
                Some(Token::KwBool) => builder
                    .set_bool()
                    .on_err_context("parse_type_expr", "failed to parse bool in type expression")?,
                Some(Token::KwChar) => builder
                    .add_base(BaseType::Char)
                    .on_err_context("parse_type_expr", "failed to parse char in type expression")?,
                Some(Token::KwShort) => builder.add_short().on_err_context(
                    "parse_type_expr",
                    "failed to parse short in type expression",
                )?,
                Some(Token::KwInt) => builder
                    .add_base(BaseType::Int)
                    .on_err_context("parse_type_expr", "failed to parse int in type expression")?,
                Some(Token::KwLong) => builder
                    .add_long()
                    .on_err_context("parse_type_expr", "failed to parse long in type expression")?,
                Some(Token::KwFloat) => builder.add_base(BaseType::Float).on_err_context(
                    "parse_type_expr",
                    "failed to parse float in type expression",
                )?,
                Some(Token::KwDouble) => builder.add_base(BaseType::Double).on_err_context(
                    "parse_type_expr",
                    "failed to parse double in type expression",
                )?,
                Some(Token::KwSigned) => builder.add_sign(Sign::Signed).on_err_context(
                    "parse_type_expr",
                    "failed to parse signed in type expression",
                )?,
                Some(Token::KwUnsigned) => builder.add_sign(Sign::Unsigned).on_err_context(
                    "parse_type_expr",
                    "failed to parse unsigned in type expression",
                )?,
                Some(Token::KwComplex) => builder.add_complex(Complex::Complex).on_err_context(
                    "parse_type_expr",
                    "failed to parse complex in type expression",
                )?,
                Some(Token::KwImaginary) => {
                    builder.add_complex(Complex::Imaginary).on_err_context(
                        "parse_type_expr",
                        "failed to parse imaginary in type expression",
                    )?
                }

                // qualifiers
                Some(token) if let Ok(q) = token.as_type_qualifier() => builder.add_qualifier(q),

                // function specifiers
                Some(Token::KwInline) => builder
                    .add_function_specifier(FunctionSpecifier::Inline)
                    .on_err_context(
                        "parse_type_expr",
                        "failed to parse inline in type expression",
                    )?,
                Some(Token::KwNoreturn) => builder
                    .add_function_specifier(FunctionSpecifier::NoReturn)
                    .on_err_context(
                        "parse_type_expr",
                        "failed to parse noreturn in type expression",
                    )?,

                // struct/union/enum/typedef identifier
                Some(tok @ (Token::KwStruct | Token::KwUnion)) => {
                    let clone = tok.clone();
                    self.consumes_token();
                    let s = self.parse_struct_or_union().on_err_context(
                        "parse_type_expr",
                        "failed to parse struct or union in type expression",
                    )?;
                    if clone == Token::KwStruct {
                        builder
                            .set_tagged_or_named(TypeSpec::Struct(s))
                            .on_err_context(
                                "parse_type_expr",
                                "failed to parse struct in type expression",
                            )?;
                    } else {
                        builder
                            .set_tagged_or_named(TypeSpec::Union(s))
                            .on_err_context(
                                "parse_type_expr",
                                "failed to parse union in type expression",
                            )?;
                    }
                    continue;
                }
                Some(Token::KwEnum) => {
                    self.consumes_token();
                    let e = self.parse_enum().on_err_context(
                        "parse_type_expr",
                        "failed to parse enum in type expression",
                    )?;
                    builder
                        .set_tagged_or_named(TypeSpec::Enum(e))
                        .on_err_context(
                            "parse_type_expr",
                            "failed to parse enum in type expression",
                        )?;
                    continue;
                }
                Some(Token::Ident(name)) if self.env.is_typedef(name) => {
                    let name = name.clone();
                    builder
                        .set_tagged_or_named(TypeSpec::Named(name))
                        .on_err_context(
                            "parse_type_expr",
                            "failed to parse typedef in type expression",
                        )?;
                }

                // _Atomic — qualifier vs specifier decided by following '('
                Some(Token::KwAtomic) => {
                    self.consumes_token();
                    if self.expect(&Token::LeftParenthesis).is_ok() {
                        let ty = self.parse_type_name().on_err_context(
                            "parse_type_expr",
                            "failed to parse atomic type name",
                        )?; // _Atomic(type-name)
                        builder
                            .set_tagged_or_named(TypeSpec::Atomic(Box::new(ty)))
                            .on_err_context(
                                "parse_type_expr",
                                "failed to parse atomic type in type expression",
                            )?;
                        self.expect(&Token::RightParenthesis).on_err_context(
                            "parse_type_expr",
                            "failed to parse atomic type expression",
                        )?;
                        continue;
                    } else {
                        builder.add_qualifier(TypeQualifier::Atomic);
                    }
                }
                Some(Token::KwAlignas) => {
                    self.consumes_token();
                    let a = self.parse_alignment_specifier().on_err_context(
                        "parse_type_expr",
                        "failed to parse alignment specifier in type expression",
                    )?;
                    builder.set_alignment(a).on_err_context(
                        "parse_type_expr",
                        "failed to parse alignment in type expression",
                    )?;
                    continue;
                }
                _ => break,
            }
            self.advance();
        }
        builder
            .finish().map_err(|e| parse_error!("{} @ {}..{}", e,
                start, self.peek_span().end
            ).span(start, self.prev_span().end))
            .on_err_context("parse_type_expr", "failed to parse type expression")
    }

    fn parse_type_qualifiers(&mut self) -> Vec<Spanned<TypeQualifier>> {
        let mut qualifiers = Vec::new();
        let mut start = self.peek_span();
        while let Some(peek) = self.peek()
            && let Ok(tq) = peek.as_type_qualifier()
        {
            qualifiers.push(Spanned {
                node: tq,
                span: start.merge(&self.prev_span()),
            });
            self.advance();
            start = self.peek_span();
        }
        qualifiers
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

    fn parse_function_def(&mut self) -> Result<FunctionDef, ParseError> {
        Err(parse_error!(
            "Function definition parsing not implemented yet"
        ))
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        self.attempt(|p| {
            Ok(Item::FunctionDef(p.parse_function_def().on_err_context(
                "parse_item",
                "failed to parse function definition",
            )?))
        })
        .or_else(|_| Ok(Item::Declaration(
            self.parse_declaration()
                .on_err_context("parse_item", "failed to parse declaration")?,
        )))
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