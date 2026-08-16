use crate::ast::ast::Expr;
use crate::ast::decl_specifiers::TypeExprContext;
use crate::ast::declarations::Declaration;
use crate::ast::function_def::FunctionDef;
use crate::ast::span::{Span, Spanned};
use crate::ast::statements::{BlockItem, ForInit, Stmt};
use crate::lexer::token::{SpannedToken, Token};
use crate::parse_error;
use crate::parser::errors::{Contextualize, ParseError};
use crate::parser::parser::Parser;

impl Parser {
    fn parse_labeled_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.peek_span().start;
        match self.peek() {
            Some(Token::Ident(_)) => {
                let Ok(identifier) = self.expect_identifier() else { unreachable!() };
                self.expect(&Token::Colon)
                    .on_err_context("parse_labeled_statement", "a colon is expected between label and statement")?;
                Ok(Stmt::Label(identifier, Box::new(self.parse_statement()?)))
            }
            Some(Token::KwCase) => {
                self.consumes_token();
                let expr = self.parse_conditional_expr()?; // expr must be evaluable at compile time
                self.expect(&Token::Colon)
                    .on_err_context("parse_labeled_statement", "a colon is expected between label and statement")?;
                Ok(Stmt::Case(expr, Box::new(self.parse_statement()?)))
            }
            Some(Token::KwDefault) => {
                self.consumes_token();
                self.expect(&Token::Colon)
                    .on_err_context("parse_labeled_statement", "a colon is expected between label and statement")?;
                Ok(Stmt::Default(Box::new(self.parse_statement()?)))
            }
            other => Err(parse_error!(
                "a labeled statement must start with and identifier or the keywords `case` or `default`: found {:?} @ {}..{}",
                other,
                start,
                self.peek_span().end
            ).span(start, self.peek_span().end))
        }
    }

    fn parse_expression_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            Some(Token::SemiColon) => {
                self.consumes_token();
                Ok(Stmt::Empty)
            }
            _ => {
                let result = Ok(Stmt::Expr(self.parse_expr()?));
                self.expect(&Token::SemiColon).on_err_context(
                    "parse_expression_statmeent",
                    "an expression statement must end with a `;`",
                )?;
                result
            }
        }
    }

    fn parse_condition(&mut self, keyword: &'static str) -> Result<Spanned<Expr>, ParseError> {
        self.expect(&Token::LeftParenthesis).on_err_context(
            "parse_condition",
            format!("`(` is expected after keyword {}", keyword).as_str(),
        )?;
        let cond = self.parse_expr().on_err_context(
            "parse_condition",
            format!("an expression is expected in the {} condition", keyword).as_str(),
        )?;
        self.expect(&Token::RightParenthesis).on_err_context(
            "parse_condition",
            format!("`)` is expected after keyword {}", keyword).as_str(),
        )?;
        Ok(cond)
    }

    fn parse_selection_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            Some(Token::KwIf) => {
                self.consumes_token();
                let cond = self
                    .parse_condition("if")
                    .on_err_context("parse_selection_statement", "failed to parse if condition")?;
                let then = self.parse_statement().on_err_context(
                    "parse_selection_statement",
                    "a statement is expected after if condition",
                )?;
                let els = if self.expect(&Token::KwElse).is_ok() {
                    Some(Box::new(self.parse_statement().on_err_context(
                        "parse_selection_statement",
                        "a statement is expected after keyword else",
                    )?))
                } else {
                    None
                };
                Ok(Stmt::If {
                    cond,
                    then: Box::new(then),
                    els,
                })
            }
            Some(Token::KwSwitch) => {
                self.consumes_token();
                let expr = self.parse_condition("switch").on_err_context(
                    "parse_selection_statement",
                    "failed to parse switch expression",
                )?;
                let body = self.parse_statement().on_err_context(
                    "parse_selection_statement",
                    "a statement is expected after switch condition",
                )?;
                Ok(Stmt::Switch {
                    expr,
                    body: Box::new(body),
                })
            }
            other => Err(parse_error!(
                "Expected a selection statement (`if`/`switch`), found: {:?} @ {}..{}",
                other,
                self.peek_span().start,
                self.peek_span().end
            )
            .span(self.peek_span().start, self.peek_span().end)),
        }
    }

    fn parse_for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::KwFor).on_err_context(
            "parse_for_statement",
            "for statements must start with `for` keyword",
        )?;
        self.expect(&Token::LeftParenthesis).on_err_context(
            "parse_for_statement",
            "`for` keyword must be followed by `(`",
        )?;
        let for_init = if let Ok(stmt) = self.parse_expression_statement() {
            match stmt {
                Stmt::Expr(expr) => ForInit::Expr(expr),
                Stmt::Empty => ForInit::Empty,
                _ => unreachable!(),
            }
        } else {
            ForInit::Decl(
                self.parse_declaration()
                    .on_err_context("parse_for_statement", "failed to parse for initialize")?,
            )
        };
        let cond_stmt = self
            .parse_expression_statement()
            .on_err_context("parse_for_statment", "failed to parse for condition")?;
        let cond = match cond_stmt {
            Stmt::Expr(expr) => Some(expr),
            Stmt::Empty => None,
            _ => unreachable!(),
        };
        let step = self.parse_expr().ok();
        self.expect(&Token::RightParenthesis).on_err_context(
            "parse_for_statement",
            "`for` initializer, conditional, and optional step must be followed by `)`",
        )?;
        let body = self.parse_statement().on_err_context(
            "parse_for_statement",
            "`for` body (even empty: ';') is mandatory",
        )?;
        Ok(Stmt::For {
            init: for_init,
            cond,
            step,
            body: Box::new(body),
        })
    }

    fn parse_iteration_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            Some(Token::KwWhile) => {
                self.consumes_token();
                let cond = self.parse_condition("while").on_err_context(
                    "parse_iteration_statement",
                    "failed to parse while condition",
                )?;
                let body = self.parse_statement().on_err_context(
                    "parse_selection_statement",
                    "a statement is expected after switch condition",
                )?;
                Ok(Stmt::While {
                    cond,
                    body: Box::new(body),
                })
            }
            Some(Token::KwDo) => {
                self.consumes_token();
                let body = self.parse_statement().on_err_context(
                    "parse_selection_statement",
                    "a statement is expected after switch condition",
                )?;
                self.expect(&Token::KwWhile).on_err_context(
                    "parse_iteration_statement",
                    "`while` is expected after a `do` statement",
                )?;
                let cond = self.parse_condition("do ... while").on_err_context(
                    "parse_iteration_statement",
                    "failed to parse do ... while condition",
                )?;
                self.expect(&Token::SemiColon).on_err_context(
                    "parse_iteration_statement",
                    "a `;` must end a do ... while statement",
                )?;
                Ok(Stmt::DoWhile {
                    body: Box::new(body),
                    cond,
                })
            }
            Some(&Token::KwFor) => self.parse_for_statement().on_err_context(
                "parse_iteration_statement",
                "failed to parse `for` statement",
            ),
            other => Err(parse_error!(
                "Expected an iteration statement (`do`/`while`/`for`), found: {:?} @ {}..{}",
                other,
                self.peek_span().start,
                self.peek_span().end
            )
            .span(self.peek_span().start, self.peek_span().end)),
        }
    }

    fn parse_jump_statement(&mut self) -> Result<Stmt, ParseError> {
        let jump = match self.peek() {
            Some(Token::KwGoto) => {
                self.consumes_token();
                let identifier = self.expect_identifier().on_err_context(
                    "parse_jump_statement",
                    "expected identifier after `goto` keyword"
                )?;
                Stmt::Goto(identifier)
            }
            Some(Token::KwContinue) => {
                self.consumes_token();
                Stmt::Continue
            }
            Some(Token::KwBreak) => {
                self.consumes_token();
                Stmt::Break
            }
            Some(Token::KwReturn) => {
                self.consumes_token();
                Stmt::Return(self.parse_expr().ok())
            }
            other => Err(parse_error!(
                "Expected an jump statement (`goto`/`continue`/`break`/`return`), found: {:?} @ {}..{}",
                other,
                self.peek_span().start,
                self.peek_span().end
            )
            .span(self.peek_span().start, self.peek_span().end))?
        };
        self.expect(&Token::SemiColon).on_err_context(
            "parse_jump_statement",
            "expected semi colon to end jump statement",
        )?;
        Ok(jump)
    }

    fn parse_statement(&mut self) -> Result<Spanned<Stmt>, ParseError> {
        let start = self.peek_span().start;
        let mut stmt = self.attempt(|p| p.parse_labeled_statement());
        if stmt.is_err() {
            stmt = self.parse_compound_statement().map(|cs| Stmt::Block(cs));
        }
        if stmt.is_err() {
            stmt = self.parse_expression_statement();
        }
        if stmt.is_err() {
            stmt = self.parse_selection_statement();
        }
        if stmt.is_err() {
            stmt = self.parse_iteration_statement();
        }
        if stmt.is_err() {
            stmt = self.parse_jump_statement();
        }
        Ok(Spanned {
            node: stmt?,
            span: Span {
                start,
                end: self.prev_span().end,
            },
        })
    }

    fn parse_block_item(&mut self) -> Result<BlockItem, ParseError> {
        let start = self.peek_span().start;
        self.attempt(|p| Ok(BlockItem::Decl(p.parse_declaration()?)))
            .or_else(|_| Ok(BlockItem::Stmt(self.parse_statement()?)))
    }

    fn parse_compound_statement(&mut self) -> Result<Vec<BlockItem>, ParseError> {
        self.expect(&Token::LeftBrace).on_err_context(
            "parse_compound_statement",
            "a compound statement should start with '{'",
        )?;
        let mut result = vec![];
        while self.expect(&Token::RightBrace).is_err() {
            result.push(self.parse_block_item()?);
        }
        Ok(result)
    }

    pub(super) fn parse_function_definition(&mut self) -> Result<FunctionDef, ParseError> {
        let declaration_specifiers = self
            .parse_type_expr(TypeExprContext::Declaration)
            .on_err_context(
                "parse_function_definition",
                "declaration specifiers are mandatory at the beginning of a function definition",
            )?;
        let declarator = self.parse_declarator()
            .on_err_context("parse_function_definition", "a declarator should follow the declaration specifiers at the beginning of a function definition")?;

        let mut declarations: Vec<Declaration> = if let Ok(ok) = self.parse_declaration() {
            vec![ok]
        } else {
            vec![]
        };
        while let Ok(decl) = self.attempt(|p| p.parse_declaration()) {
            declarations.push(decl);
        }

        let compound_statement = self.parse_compound_statement()
            .on_err_context("parse_function_definition", "compound statement (body) of the function is mandatory when parsing a function definition")?;

        Ok(FunctionDef {
            ret: declaration_specifiers,
            declarator,
            old_style_params: declarations,
            body: compound_statement,
        })
    }
}
