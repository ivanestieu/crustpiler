// =============================================================================
// parser.rs — handwritten recursive descent, producing the full-AST shapes.
//
// Grammar slice:
//     decl       := storage? qualifier* type init_decl ("," init_decl)* ";"
//     init_decl  := ident ("=" initializer)?
//     initializer:= expr
//     expr       := int_literal
// =============================================================================

use std::vec;
use itertools::Itertools;
use crate::ast::ast::{Expr, GenericAssoc, Item};
use crate::ast::decl_specifiers::{TypeExprBuilder, TypeExprContext};
use crate::ast::declarations::{Decl, Declaration, InitDeclarator, Initializer, StaticAssert};
use crate::ast::declarator::{ArraySize, Declarator};
use crate::ast::operators::{AsAssignOp, AsBinaryOp, AsUnaryOp, TraitBinaryOp, UnaryOp};
use crate::ast::parameters::ParamDecl;
use crate::ast::span::{Span, Spanned};
use crate::ast::struct_union::{FieldDecl, FieldDeclarator, StructMember, StructOrUnion};
use crate::ast::types::{AsStorageClass, AsTypeQualifier, BaseType, Complex, FunctionSpecifier, Sign, TypeExpr, TypeQualifier, TypeSpec};
use crate::lexer::token::{SpannedToken, Token};
use crate::parser::env::Env;

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
    env: Env,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, pos: 0, env: Env::default() }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|st : &SpannedToken | &st.token)
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
        let tok : Option<SpannedToken> = self.tokens.get(self.pos).cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn consumes_token(&mut self) -> () {
        self.advance();
    }

    fn expect(&mut self, expected: &Token) -> Result<SpannedToken, String> {
        match self.peek() {
            Some(t) if t == expected => Ok(self.advance().unwrap()),
            other => Err(format!("expected {:?}, found {:?}", expected, other)),
        }
    }

    fn at_eof(&self) -> bool {
        if self.pos >= self.tokens.len() {
            true
        } else {
            false
        }
    }


    /// decl := storage? qualifier* type init_decl ("," init_decl)* ";"
    pub fn parse_decl(&mut self) -> Result<Decl, String> {
        let type_expr = self.parse_type_expr(TypeExprContext::Declaration)?;

        // One or more comma-separated declarators: `int a, b = 1, c;`
        let mut declarators : Vec<InitDeclarator> = vec![self.parse_init_declarator()?];
        while matches!(self.peek(), Some(Token::Comma)) {
            self.advance();
            declarators.push(self.parse_init_declarator()?);
        }

        Ok(Decl { specifiers : type_expr, declarators })
    }
    fn parse_declarator(&mut self) -> Result<Declarator, String> {
        let pointers = self.parse_pointer();
        let inner = self.parse_direct_declarator()?;
        Ok(Self::apply_pointer_levels(pointers, inner))
    }

    fn parse_direct_declarator(&mut self) -> Result<Declarator, String> {
        let mut base = match self.peek() {
            Some(Token::Ident(_)) => {
                let SpannedToken { token: Token::Ident(name), .. } = self.advance().unwrap()
                else {
                    // should be unreachable
                    return Err(format!("Expected identifier, found {:?}", self.peek().unwrap()))
                };
                Declarator::Ident(name)
            }
            Some(Token::LeftParenthesis) => {
                self.advance();
                let inner_declarator = self.parse_declarator()?;
                self.expect(&Token::RightParenthesis)?;
                inner_declarator
            }
            other => return Err(format!("expected identifier, found {:?}", other)),
        };
        loop {
            match self.peek() {
                Some(Token::LeftBracket) => {base =self.parse_array_suffix(base)?;},
                Some(Token::LeftParenthesis) => {base = self.parse_function_suffix(base)?;},
                _ => break,
            }
        }
        Ok(base)
    }

    fn parse_abstract_declarator(&mut self) -> Result<Declarator, String> {
        /*let pointer = self.parse_pointer();
        let direct_abstract_declarator = self.parse_direct_abstract_declarator();
        if pointer.is_empty() && direct_abstract_declarator.is_err() {
            return Err("Expected abstract declarator, found nothing".to_string());
        }
        Ok(pointer && direct_abstract_declarator)*/
        Err("Direct Abstract declarator cannot be parsed".to_string())
    }

    fn parse_direct_abstract_declarator(&mut self) -> Result<Decl, String> {
        /*let lhs = match self.peek() {
            Some(Token::LeftParenthesis) => {
                self.advance();
                if let Some(abstract_declarator) = self.parse_abstract_declarator() {
                    self.expect(Token::RightParenthesis)?;
                    Some(abstract_declarator)
                } else {
                    None
                }
            }
            Some(Token::LeftBracket) => {

            }
            other => return Err(format!("Expected '(' or '[', found {:?}", other)),
        }*/
        Err("Direct Abstract declarator cannot be parsed".to_string())
    }

    /// init_decl := ident ("=" initializer)?
    fn parse_init_declarator(&mut self) -> Result<InitDeclarator, String> {
        // The declared name.
        let name = match self.advance() {
            Some(SpannedToken { token: Token::Ident(name), .. }) => name,
            other => return Err(format!("expected identifier, found {:?}", other)),
        };
        let declarator = Declarator::Ident(name);

        // Optional initializer.
        let init = if matches!(self.peek(), Some(Token::Equals)) {
            self.consumes_token();
                Some(
                    Initializer::Expr(
                        Box::new(self.parse_expr()?.node)
                    )
                )
        } else {
            None
        };

        Ok(InitDeclarator { declarator, init })
    }

    fn spanned_and_consume(&mut self, expr: Expr) -> Result<Spanned<Expr>, String> {
        let res = Ok(Spanned {node: expr, span : self.prev_span()});
        self.advance();
        res
    }

    fn parse_primary_expr(&mut self) -> Result<Spanned<Expr>, String> {
        match self.peek() {
            // Identifier
            Some(Token::Ident(s)) => self.spanned_and_consume(Expr::Ident(s.clone())),
            // Constant
            Some(Token::Int(n)) => self.spanned_and_consume(Expr::IntLit(n.clone())),
            Some(Token::Float(f)) => self.spanned_and_consume(Expr::FloatLit(f.clone())),
            Some(Token::CharLit(c)) => self.spanned_and_consume(Expr::CharLit(c.clone())),
            // (Enum constants)
            // String
            Some(Token::StringLit(s)) => self.spanned_and_consume(Expr::StringLit(s.clone())),
            Some(Token::KwFuncName) => self.spanned_and_consume(Expr::FuncName("".to_string())),
            // Nested in parentheses Expr
            Some(Token::LeftParenthesis) => {
                let start = self.peek_span();
                self.consumes_token();
                let inner_expr = self.parse_expr()?;
                self.expect(&Token::RightParenthesis)?;
                Ok(Spanned { node: inner_expr.node, span: start.merge(&self.prev_span()) })
            }
            // Generic Selection
            Some(Token::KwGeneric) => {
                let start = self.peek_span();
                self.consumes_token();
                self.expect(&Token::LeftParenthesis)?;
                let controlling = self.parse_assignment_expr()?;
                self.expect(&Token::Comma)?;
                let mut associated = Vec::new();
                loop {
                    match self.peek() {
                        Some(Token::RightParenthesis) => {self.consumes_token(); break},
                        Some(Token::Comma) => {self.consumes_token(); continue},
                        Some(Token::KwDefault) => {
                            self.advance();
                            self.expect(&Token::Colon)?;
                            associated.push(GenericAssoc { type_expr: None, value: self.parse_assignment_expr()?});
                        }
                        _ => {
                            if let Ok(type_name) = self.parse_type_expr(TypeExprContext::TypeName) {
                                self.expect(&Token::Colon)?;
                                associated.push(GenericAssoc { type_expr: Some(type_name), value: self.parse_assignment_expr()?});
                            }
                            else {
                                return Err(format!("expected identifier, found {:?}", self.peek()));
                            }
                        }
                    }
                };
                Ok(Spanned { node: Expr::Generic {
                    controlling: Box::new(controlling),
                    associated
                }, span: start.merge(&self.prev_span()) })
            }
            other => Err(format!("expected expression, found {:?}", other)),
        }
    }

    fn parse_binary_expr(&mut self, min_bp : Option<usize>) -> Result<Spanned<Expr>, String> {
        let mut lhs = self.parse_primary_expr()?;
        while let Some(token) = self.peek() && let Ok(op) = token.as_binary_op() {
            let (left_bp, right_bp) = op.binding_power();
            if left_bp < min_bp.unwrap_or(0) {
                break;
            }

            self.consumes_token();
            let rhs = self.parse_binary_expr(Some(right_bp))?;
            let span = lhs.span.merge(&rhs.span);
            lhs = Spanned { node: Expr::BinaryOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }, span
            };
        }
        Ok(lhs)
    }

    fn parse_unary_expr(&mut self) -> Result<Spanned<Expr>, String> {
        let start = self.peek_span();
        let to_spanned = |expr : Expr, end : Span| Ok(Spanned {  node: expr , span: start.merge(&end)});
        // parse_postfix_expr
        match self.peek() {
            Some(Token::IncOp) => {
                self.advance();
                to_spanned(Expr::UnaryOp { op: UnaryOp::PreInc, operand: Box::new(self.parse_unary_expr()?)}, self.prev_span())
            }
            Some(Token::DecOp) => {
                self.advance();
                to_spanned(Expr::UnaryOp { op: UnaryOp::PreDec, operand: Box::new(self.parse_unary_expr()?)}, self.prev_span())
            }
            Some(tok @ Token::Minus)
            | Some(tok @ Token::Plus)
            | Some(tok @ Token::ExclamationMark)
            | Some(tok @ Token::Tilde)
            | Some(tok @ Token::Star)
            | Some(tok @ Token::Ampersand) => {
                let unary_op = tok.as_unary_op()?;
                self.advance();
                to_spanned(Expr::UnaryOp {op : unary_op, operand: Box::new(self.parse_cast_expr()?)}, self.prev_span())
            }
            Some(Token::KwSizeof) => {
                self.advance();
                if self.expect(&Token::LeftParenthesis).is_ok() {
                    to_spanned(Expr::SizeofType(self.parse_type_expr(TypeExprContext::TypeName)?), self.prev_span())
                } else {
                    to_spanned(Expr::SizeofExpr(Box::new(self.parse_unary_expr()?)), self.prev_span())
                }
            }
            Some(Token::KwAlignof) => {
                self.advance();
                self.expect(&Token::LeftParenthesis)?;
                to_spanned(Expr::SizeofType(self.parse_type_expr(TypeExprContext::TypeName)?), self.prev_span())
            }
            other => Err(format!("expected unary expression, found {:?}", other)),
        }
    }

    fn parse_cast_expr(&mut self) -> Result<Spanned<Expr>, String> {
        let start = self.peek_span();
        match self.peek() {
            Some(Token::LeftParenthesis) => {
                self.advance();
                Ok(Spanned {
                    node: Expr::Cast {
                        ty: self.parse_type_expr(TypeExprContext::TypeName)?,
                        expr: Box::new(self.parse_cast_expr()?)
                    },
                    span: start.merge(&self.prev_span())
                })
            }
            _ => self.parse_unary_expr()
        }
    }

    fn parse_conditional_expr(&mut self) -> Result<Spanned<Expr>, String> {
        let lhs = self.parse_binary_expr(None)?;
        let middle: Spanned<Expr> = match self.peek() {
            Some(Token::InterrogationMark) => {
                self.advance();
                self.parse_expr()?
            }
            _ => return Ok(lhs),
        };
        self.expect(&Token::Colon)?;
        let rhs = self.parse_conditional_expr()?;
        let span = lhs.span.merge(&rhs.span);
        Ok(Spanned { node: Expr::Ternary {cond : Box::new(lhs), then : Box::new(middle), els : Box::new(rhs)}, span })
    }

    fn parse_assignment_expr(&mut self) -> Result<Spanned<Expr>, String> {
        let mut lhs = self.parse_conditional_expr()?;
        if !matches!(lhs, Spanned { node : Expr::BinaryOp {..}, .. }) {
            return Ok(lhs);
        }
        while let Some(token) = self.peek() && let op = token.as_assign_op()? {
            self.consumes_token();
            let rhs = self.parse_assignment_expr()?;
            let span = lhs.span.merge(&rhs.span);
            lhs = Spanned { node: Expr::Assign {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }, span
            };
        }
        Ok(lhs)
    }

    fn parse_expr(&mut self) -> Result<Spanned<Expr>, String> {
        let start = self.peek_span();
        let mut lhs = self.parse_assignment_expr()?;
        while self.expect(&Token::Comma).is_ok() {
            let rhs = self.parse_assignment_expr()?;
            lhs = Spanned { node: Expr::Comma(Box::new(lhs),Box::new(rhs)), span : start.merge(&self.prev_span())}
        }
        Ok(lhs)
    }

    fn parse_pointer(&mut self) -> Vec<Spanned<Vec<Spanned<TypeQualifier>>>> {
        let mut pointers = Vec::new();
        let mut start = self.peek_span();
        while self.expect(&Token::Star).is_ok() {
            let type_qualifiers = self.parse_type_qualifiers();
            pointers.push(Spanned { node: type_qualifiers, span : start.merge(&self.prev_span()) });
            start = self.peek_span();
        }
        pointers
    }

    fn apply_pointer_levels(levels: Vec<Spanned<Vec<Spanned<TypeQualifier>>>>, mut inner: Declarator) -> Declarator {
        for qualifiers in levels.into_iter().rev() {
            inner = Declarator::Pointer {
                qualifiers : qualifiers.node,
                inner: Box::new(inner),
            };
        }
        inner
    }

    fn parse_array_suffix(&mut self, base : Declarator) -> Result<Declarator, String> {
        self.expect(&Token::LeftBracket)?;
        let mut is_static = false;
        let mut size = ArraySize::None;
        let mut qualifiers = Vec::new();
        is_static = matches!(self.peek(), Some(Token::KwStatic));
        if is_static {
            self.consumes_token();
        }
        qualifiers = self.parse_type_qualifiers();
        if self.peek() == Some(&Token::Star) {
            self.consumes_token();
            size = ArraySize::Vla;
        } else {
            is_static = match (self.peek(), is_static) {
                (Some(Token::KwStatic), true) => return Err("Static cannot be specified more than one time".to_string()),
                (Some(Token::KwStatic), ..) => {self.consumes_token(); true},
                _ => is_static,
            };
            size = ArraySize::Fixed(Box::new(self.parse_assignment_expr()?.node));
        }
        self.expect(&Token::RightBracket)?;
        Ok(Declarator::Array {
            inner : Box::new(base),
            size,
            is_static,
            qualifiers,
        })
    }

    fn parse_function_suffix(&mut self, base : Declarator) -> Result<Declarator, String> {
        self.expect(&Token::LeftParenthesis)?;
        let mut params = Vec::new();
        let mut variadic = false;
        let mut void = false;
        if self.peek() == Some(&Token::KwVoid) {
            void = true;
        }
        // Identifiers only not implemented at the point
        loop {
            if self.peek() == Some(&Token::RightParenthesis) {
                break;
            }
            if self.peek() == Some(&Token::Ellipsis) {
                self.consumes_token();
                variadic = true;
                break;
            }
            let type_expr = self.parse_type_expr(TypeExprContext::Declaration)?;
            let declarator = if let Ok(decl) = self.parse_declarator() {
                decl
            } else {
                self.parse_abstract_declarator()?
            };
            params.push(ParamDecl {
                specifiers : type_expr,
                declarator
            });
            if self.peek() == Some(&Token::Comma) {
                self.consumes_token();
                continue;
            }
            break
        }
        // Parse function parameters
        self.expect(&Token::RightParenthesis)?;
        Ok(Declarator::Function {
            inner: Box::new(base),
            params : Vec::new(),
            variadic : false,
        })
    }


    fn parse_static_assert_declaration(&mut self) -> Result<StaticAssert, String> {
        self.expect(&Token::KwStaticAssert)?;
        self.expect(&Token::LeftParenthesis)?;
        let cond = self.parse_conditional_expr()?; // evaluable at compile-time
        self.expect(&Token::Comma)?;
        let message = match self.peek() {
            Some(Token::StringLit(s)) => {
                let slit = s.clone();
                self.advance();
                self.expect(&Token::RightParenthesis)?;
                self.expect(&Token::SemiColon)?;
                slit
            }
            other => return Err(format!("expected string literal, found {:?}", other)),
        };
        Ok(StaticAssert {cond : Box::new(cond.node), message})
    }

    fn parse_struct_declarator(&mut self) -> Result<FieldDeclarator, String> {
        if self.expect(&Token::Colon).is_ok() {
            let constant_expr = self.parse_conditional_expr()?; // evaluable at compile-time
            return Ok(FieldDeclarator { declarator: None, bit_width: Some(Box::new(constant_expr)) })
        }
        let declarator = self.parse_declarator()?;
        Ok(FieldDeclarator {declarator : Some(declarator), bit_width:
            if self.expect(&Token::Colon).is_ok() {
                Some(Box::new(self.parse_conditional_expr()?)) // evaluable at compile-time
            } else {
                None
            }
        })
    }

    fn parse_struct_declarator_list(&mut self) -> Result<Vec<FieldDeclarator>, String> {
        let mut struct_declarations = Vec::new();
        while let Ok(struct_declarator) = self.parse_struct_declarator() {
            struct_declarations.push(struct_declarator);
            if self.expect(&Token::Comma).is_ok() {
                break;
            }
        }
        Ok(struct_declarations)
    }

    fn parse_struct_declaration(&mut self) -> Result<StructMember, String> {
        if let Ok(static_assert) = self.parse_static_assert_declaration() {
            return Ok(StructMember::StaticAssert(static_assert));
        }
        let specifiers= self.parse_type_expr(TypeExprContext::StructUnionField)?;
        Ok(StructMember::Field(FieldDecl { type_expr: specifiers, declarators :
            if self.expect(&Token::SemiColon).is_ok() {
                    Vec::new()
            } else {
                self.parse_struct_declarator_list()?
            }
        }))
    }

    fn parse_struct_or_union(&mut self) -> Result<StructOrUnion, String> {
        let name= if let Some(Token::Ident(iden)) = self.peek() {
            Some(iden.clone())
        } else {
            None
        };
        if self.expect(&Token::LeftBrace).is_err() {
            return Ok(StructOrUnion { name, fields : None})
        }
        let mut fields = Vec::new();
        while let Ok(field) = self.parse_struct_declaration() {
            fields.push(field);
        }
        self.expect(&Token::RightBrace)?;
        Ok(StructOrUnion { name, fields : Some(fields) })
    }

    fn parse_type_expr(&mut self, context : TypeExprContext) -> Result<TypeExpr, String> {
        let mut builder = TypeExprBuilder::new(context);
        loop {
            match self.peek() {
                // Storage
                Some(token) if let Ok(sc) = token.as_storage_class() => builder.add_storage(sc)?,

                // Arithmetic related
                Some(Token::KwVoid) => builder.set_void()?,
                Some(Token::KwBool) => builder.set_bool()?,
                Some(Token::KwChar) => builder.add_base(BaseType::Char)?,
                Some(Token::KwShort) => builder.add_short()?,
                Some(Token::KwInt) => builder.add_base(BaseType::Int)?,
                Some(Token::KwLong) => builder.add_long()?,
                Some(Token::KwFloat) => builder.add_base(BaseType::Float)?,
                Some(Token::KwDouble) => builder.add_base(BaseType::Double)?,
                Some(Token::KwSigned) => builder.add_sign(Sign::Signed)?,
                Some(Token::KwUnsigned) => builder.add_sign(Sign::Unsigned)?,
                Some(Token::KwComplex) => builder.add_complex(Complex::Complex)?,
                Some(Token::KwImaginary) => builder.add_complex(Complex::Imaginary)?,

                // qualifiers
                Some(token) if let Ok(q) = token.as_type_qualifier() => builder.add_qualifier(q),

                // function specifiers
                Some(Token::KwInline) => builder.add_function_specifier(FunctionSpecifier::Inline)?,
                Some(Token::KwNoreturn) => builder.add_function_specifier(FunctionSpecifier::NoReturn)?,

                // struct/union/enum/_Alignas/_Atomic(T)/typedef identifier
                Some(tok @ Token::KwStruct) | Some(tok @ Token::KwUnion) => {
                    let clone = tok.clone();
                    self.consumes_token();
                    let s = self.parse_struct_or_union()?;
                    if clone == Token::KwStruct {
                        builder.set_tagged_or_named(TypeSpec::Struct(s))?;
                    } else {
                        builder.set_tagged_or_named(TypeSpec::Union(s))?;
                    }
                }
                /*Some(Token::KwEnum) => {
                    let e = self.parse_enum()?;
                    builder.set_tagged_or_named(TypeSpec::Enum(e))?;
                }
                /*Some(Token::Ident(name)) if self.env.is_typedef(name) => {
                    let name = name.clone();
                    builder.set_tagged_or_named(TypeSpec::Named(name))?;
                }*/
                // _Atomic — qualifier vs specifier decided by following '('
                Some(Token::KwAtomic) => {
                    self.advance();
                    if self.peek() == Some(&Token::LeftParenthesis) {
                        let ty = self.parse_atomic_type_specifier()?; // _Atomic(type-name)
                        builder.set_tagged_or_named(TypeSpec::Atomic(Box::new(ty)))?;
                    } else {
                        builder.add_qualifier(TypeQualifier::Atomic);
                    }
                }
                Some(Token::KwAlignas) => {
                    let a = self.parse_alignment_specifier()?;
                    builder.set_alignment(a)?;
                }*/
                _ => break,
            }
            self.advance();
        };
        builder.finish()
    }

    fn parse_type_qualifiers(&mut self) -> Vec<Spanned<TypeQualifier>> {
        let mut qualifiers = Vec::new();
        let mut start = self.peek_span();
        while let Some(peek) = self.peek() && let Ok(tq) = peek.as_type_qualifier() {
            qualifiers.push(Spanned { node: tq, span: start.merge(&self.prev_span()) });
            self.advance();
            start = self.peek_span();
        }
        qualifiers
    }

    /// translation_unit := item* EOF
    pub fn parse_translation_unit(&mut self) -> Result<Vec<Item>, String> {
        let mut items : Vec<Item> = Vec::new();
        while !self.at_eof() {
            items.push(self.parse_item()?);
        }
        Ok(items)
    }

    /// item := function_def | declaration
    fn parse_item(&mut self) -> Result<Item, String> {
        // For now, only parse declarations; function definitions are not yet supported.
        let start_span : Span = self.peek_span();
        let decl : Decl = self.parse_decl()?;
        let end_span : Span = self.prev_span();
        let span : Span = start_span.merge(&end_span);
        Ok(Item::Declaration(Declaration::Normal(Spanned { node: decl, span })))
    }
}

// Bridge token::Span -> ast::Span (kept as separate types for module decoupling)
trait IntoAstSpan {
    fn into_ast(self) -> Span;
}
impl IntoAstSpan for crate::lexer::token::Span {
    fn into_ast(self) -> Span {
        Span { start: self.start, end: self.end }
    }
}
