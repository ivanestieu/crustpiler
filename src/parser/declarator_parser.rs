use crate::ast::ast::Expr;
use crate::ast::decl_specifiers::TypeExprContext;
use crate::ast::declarations::{Designator, InitDeclarator, InitItem, Initializer};
use crate::ast::declarator::{ArraySize, Declarator};
use crate::ast::parameters::ParamDecl;
use crate::ast::span::Spanned;
use crate::ast::struct_union::FieldDeclarator;
use crate::ast::types::TypeQualifier;
use crate::lexer::token::{SpannedToken, Token};
use crate::parse_error;
use crate::parser::errors::{Contextualize, ParseError};
use crate::parser::parser::Parser;

trait DeclaratorMode {
    const IS_ABSTRACT: bool;
    fn direct_declarator_name() -> &'static str;
    fn declarator_name() -> &'static str;
}

struct Abstract;
impl DeclaratorMode for Abstract {
    const IS_ABSTRACT: bool = true;
    #[inline]
    fn direct_declarator_name() -> &'static str {
        "parse_direct_abstract_declarator"
    }
    #[inline]
    fn declarator_name() -> &'static str {
        "parse_abstract_declarator"
    }
}

struct Concrete;
impl DeclaratorMode for Concrete {
    const IS_ABSTRACT: bool = false;
    #[inline]
    fn direct_declarator_name() -> &'static str {
        "parse_direct_declarator"
    }
    #[inline]
    fn declarator_name() -> &'static str {
        "parse_declarator"
    }
}

impl Parser {
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
        let declarator = self.parse_declarator().on_err_context(
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

    pub(super) fn parse_struct_declarator_list(
        &mut self,
    ) -> Result<Vec<FieldDeclarator>, ParseError> {
        let mut struct_declarations = Vec::new();
        while let Ok(struct_declarator) = self.parse_struct_declarator() {
            struct_declarations.push(struct_declarator);
            if self.expect(&Token::Comma).is_err() {
                break;
            }
        }
        Ok(struct_declarations)
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

    pub(super) fn parse_initializer_list(&mut self) -> Result<Vec<InitItem>, ParseError> {
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
    pub(super) fn parse_init_declarator(&mut self) -> Result<InitDeclarator, ParseError> {
        let declarator = self.parse_declarator().on_err_context(
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

    #[inline]
    pub(super) fn parse_declarator(&mut self) -> Result<Declarator, ParseError> {
        self.parse_declarator_impl::<Concrete>()
    }

    #[inline]
    pub(super) fn parse_abstract_declarator(&mut self) -> Result<Declarator, ParseError> {
        self.parse_declarator_impl::<Abstract>()
    }

    fn parse_declarator_impl<Mode: DeclaratorMode>(&mut self) -> Result<Declarator, ParseError> {
        let pointers = self.parse_pointer();
        let inner = self
            .parse_direct_declarator_impl::<Mode>()
            .on_err_context(Mode::declarator_name(), "a declarator is expected")?;
        Ok(apply_pointer_levels(pointers, inner))
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
            let declarator = if let Ok(decl) = self.attempt(|p| p.parse_declarator()) {
                decl
            } else {
                let abs_decl = self.parse_abstract_declarator().on_err_context(
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

    fn parse_array_suffix<Mode: DeclaratorMode>(
        &mut self,
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
            && (!Mode::IS_ABSTRACT || !is_static && qualifiers.is_empty())
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

    #[inline]
    pub(super) fn parse_direct_declarator(&mut self) -> Result<Declarator, ParseError> {
        self.parse_direct_declarator_impl::<Concrete>()
    }

    #[inline]
    pub(super) fn parse_direct_abstract_declarator(&mut self) -> Result<Declarator, ParseError> {
        self.parse_direct_declarator_impl::<Abstract>()
    }

    fn parse_direct_declarator_impl<Mode: DeclaratorMode>(
        &mut self,
    ) -> Result<Declarator, ParseError> {
        let mut base = match self.peek() {
            Some(Token::Ident(name)) if !Mode::IS_ABSTRACT => {
                let ident = name.clone();
                self.consumes_token();
                Declarator::Ident(ident)
            }
            Some(Token::LeftBracket) if Mode::IS_ABSTRACT => self
                .parse_array_suffix::<Abstract>(Declarator::Abstract)
                .on_err_context(Mode::direct_declarator_name(), "expected array suffix")?,
            Some(Token::LeftParenthesis) if Mode::IS_ABSTRACT => {
                let grouped = self.attempt(|p| {
                    p.expect(&Token::LeftParenthesis)?;
                    let inner = p.parse_abstract_declarator()?;
                    if matches!(inner, Declarator::Abstract) {
                        return Err(parse_error!(
                            "{}: empty group is a function suffix, not a grouping",
                            Mode::direct_declarator_name()
                        ));
                    }
                    p.expect(&Token::RightParenthesis)?;
                    Ok(inner)
                });
                match grouped {
                    Ok(inner) => inner, // then the loop below applies trailing suffixes
                    Err(_) => self
                        .parse_function_suffix(Declarator::Abstract)
                        .on_err_context(
                            Mode::direct_declarator_name(),
                            "expected function suffix",
                        )?,
                }
            }
            Some(Token::LeftParenthesis) => {
                self.consumes_token();
                let res = self.attempt(|p| {
                    p.parse_declarator()
                        .on_err_context(Mode::direct_declarator_name(), "expected declarator")
                });
                let inner_declarator = res?;
                self.expect(&Token::RightParenthesis).on_err_context(
                    Mode::direct_declarator_name(),
                    "a right parenthesis is expected to end a declarator",
                )?;
                inner_declarator
            }
            _ if Mode::IS_ABSTRACT => Declarator::Abstract,
            other => {
                return Err(parse_error!(
                    "{}: expected identifier, found {:?} @ {}..{}",
                    Mode::direct_declarator_name(),
                    other,
                    self.peek_span().start,
                    self.peek_span().end
                )
                .span(self.peek_span().start, self.peek_span().end));
            }
        };
        loop {
            match self.peek() {
                Some(Token::LeftBracket) => {
                    base = self.parse_array_suffix::<Mode>(base).on_err_context(
                        Mode::direct_declarator_name(),
                        "expected trailing array suffix",
                    )?;
                }
                Some(Token::LeftParenthesis) => {
                    base = self.parse_function_suffix(base).on_err_context(
                        Mode::direct_declarator_name(),
                        "expected trailing function suffix",
                    )?;
                }
                _ => break,
            }
        }
        Ok(base)
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
