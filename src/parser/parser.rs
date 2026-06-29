// =============================================================================
// parser.rs — handwritten recursive descent, producing the full-AST shapes.
//
// Handwritten (not chumsky) because C is not context-free: resolving whether
// an identifier names a type requires a mutable typedef environment threaded
// through the parser. That environment lives here as `Env`, ready to grow.
//
// Grammar slice:
//     decl       := storage? qualifier* type init_decl ("," init_decl)* ";"
//     init_decl  := ident ("=" initializer)?
//     initializer:= expr
//     expr       := int_literal
// =============================================================================

use env::Env;
use crate::ast::ast::*;
use crate::ast::type_spec::*;
use crate::parser::env;
use crate::lexer::token::{SpannedToken, Token};

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
        let start_span : Span = self.peek_span();

        let storage: Option<StorageClass> = self.parse_storage_class();
        let qualifiers : Vec<TypeQualifier> = self.parse_qualifiers();
        let spec : TypeSpec = self.parse_type()?;

        // One or more comma-separated declarators: `int a, b = 1, c;`
        let mut declarators : Vec<InitDeclarator> = vec![self.parse_init_declarator()?];
        while matches!(self.peek(), Some(Token::Comma)) {
            self.advance();
            declarators.push(self.parse_init_declarator()?);
        }

        let semi: SpannedToken = self.expect(&Token::SemiColon)?;
        let span : Span = start_span.merge(&semi.span.into_ast());

        Ok(Decl { storage, qualifiers, spec, declarators, span })
    }

    /// init_decl := ident ("=" initializer)?
    fn parse_init_declarator(&mut self) -> Result<InitDeclarator, String> {
        // The declared name.
        let (name, name_span) = match self.advance() {
            Some(SpannedToken { token: Token::Ident(name), span }) => (name, span.into_ast()),
            other => return Err(format!("expected identifier, found {:?}", other)),
        };
        let declarator = Declarator::Ident(name, name_span.clone());

        // Optional initializer.
        let (init, end_span) = if matches!(self.peek(), Some(Token::Equals)) {
            self.consumes_token();
            (
                Some(
                    Initializer::Expr(
                        Box::new(self.parse_expr()?)
                    )
                ),
                self.prev_span()
            )
        } else {
            (None, name_span.clone())
        };

        let span: Span = name_span.merge(&end_span);
        Ok(InitDeclarator { declarator, init, span })
    }

    /// initializer's expr := int_literal
    fn parse_expr(&mut self) -> Result<Expr, String> {
        match self.advance() {
            Some(SpannedToken { token: Token::Int(n), .. }) => {
                // Lower the raw i64 into the structured IntLit of the full AST.
                Ok(Expr::IntLit(n))
            }
            Some(SpannedToken { token: Token::Float(f), .. }) => {
                Ok(Expr::FloatLit(f))
            }
            Some(SpannedToken { token: Token::CharLit(c), .. }) => {
                Ok(Expr::CharLit(c))
            }
            Some(SpannedToken { token: Token::StringLit(s), .. }) => {
                Ok(Expr::StringLit(s))
            }
            other => Err(format!("expected expression, found {:?}", other)),
        }
    }

    fn parse_type(&mut self) -> Result<TypeSpec, String> {
        let mut kws = Vec::new();
        loop {
            match self.peek() {
                Some(Token::KwVoid) => { kws.push(TypeKeyword::Void); self.advance(); },
                Some(Token::KwBool) => { kws.push(TypeKeyword::Bool); self.advance(); },
                Some(Token::KwChar) => { kws.push(TypeKeyword::Char); self.advance(); },
                Some(Token::KwShort) => { kws.push(TypeKeyword::Short); self.advance(); },
                Some(Token::KwInt) => { kws.push(TypeKeyword::Int); self.advance(); },
                Some(Token::KwLong) => { kws.push(TypeKeyword::Long); self.advance(); },
                Some(Token::KwFloat) => { kws.push(TypeKeyword::Float); self.advance(); },
                Some(Token::KwDouble) => { kws.push(TypeKeyword::Double); self.advance(); },
                Some(Token::KwSigned) => { kws.push(TypeKeyword::Signed); self.advance(); },
                Some(Token::KwUnsigned) => { kws.push(TypeKeyword::Unsigned); self.advance(); },
                _ => break,
            }
        }
        // (struct/union/enum/typedef-name handled separately before this point)
        resolve_type_spec(&kws)
    }

    fn parse_storage_class(&mut self) -> Option<StorageClass> {
        // No storage-class keywords tokenized in this slice; always None.
        // Wired here so adding `static`/`extern`/`typedef` tokens is a one-liner.
        None
    }

    fn parse_qualifiers(&mut self) -> Vec<TypeQualifier> {
        // No qualifier keywords tokenized yet; returns empty.
        Vec::new()
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
        Ok(Item::Decl(Spanned { node: decl, span }))
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