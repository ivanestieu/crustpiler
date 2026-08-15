use crate::ast::decl_specifiers::{AlignmentSpecifier, TypeExpr, TypeExprBuilder, TypeExprContext};
use crate::ast::declarator::Declarator;
use crate::ast::enums::{EnumSpec, Enumerator};
use crate::ast::span::Spanned;
use crate::ast::types::{
    AsStorageClass, AsTypeQualifier, BaseType, Complex, FunctionSpecifier, Sign, TypeName,
    TypeQualifier, TypeSpec,
};
use crate::lexer::token::{SpannedToken, Token};
use crate::parse_error;
use crate::parser::errors::{Contextualize, ParseError};
use crate::parser::parser::Parser;

impl Parser {
    fn parse_enum(&mut self) -> Result<EnumSpec, ParseError> {
        let name = self.expect_identifier().ok();
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
            let variant_name = self.expect_identifier().on_err_context(
                "parse_enum",
                "expected identifier for enum variant")?;
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

    pub(super) fn parse_type_name(&mut self) -> Result<TypeName, ParseError> {
        let type_expr = self
            .parse_type_expr(TypeExprContext::TypeName)
            .on_err_context(
                "parse_type_name",
                "failed to parse type expression in type name",
            )?;
        let abstract_decl = self
            .parse_abstract_declarator()
            .unwrap_or(Declarator::Abstract);
        Ok(TypeName {
            type_expr,
            derived: abstract_decl,
        })
    }

    pub(super) fn parse_type_expr(
        &mut self,
        context: TypeExprContext,
    ) -> Result<TypeExpr, ParseError> {
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
                    .map_err(|e| {
                        parse_error!("{} @ {}..{}", e, start, self.peek_span().end)
                            .span(start, self.prev_span().end)
                    })
                    .on_err_context(
                        "parse_type_expr",
                        "failed to parse int in type expression",
                    )?,
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
                Some(Token::KwUnsigned) => builder
                    .add_sign(Sign::Unsigned)
                    .map_err(|e| {
                        parse_error!("{} @ {}..{}", e, start, self.peek_span().end)
                            .span(start, self.prev_span().end)
                    })
                    .on_err_context(
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
                    let is_struct = tok == &Token::KwStruct;
                    self.consumes_token();
                    let s = self.parse_struct_or_union().on_err_context(
                        "parse_type_expr",
                        "failed to parse struct or union in type expression",
                    )?;
                    if is_struct {
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
                    let name = self.expect_identifier()?;
                    builder
                        .set_tagged_or_named(TypeSpec::Named(name))
                        .on_err_context(
                            "parse_type_expr",
                            "failed to parse typedef in type expression",
                        )?;
                    continue;
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
            .finish()
            .map_err(|e| {
                parse_error!("{} @ {}..{}", e, start, self.peek_span().end)
                    .span(start, self.prev_span().end)
            })
            .on_err_context("parse_type_expr", "failed to parse type expression")
    }

    pub(super) fn parse_type_qualifiers(&mut self) -> Vec<Spanned<TypeQualifier>> {
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
}
