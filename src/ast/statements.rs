// -----------------------------------------------------------------------------
// STATEMENTS
// -----------------------------------------------------------------------------

use crate::ast::ast::Expr;
use crate::ast::declarations::Declaration;
use crate::ast::span::Spanned;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    // Expression statement (including assignments, calls, etc.)
    Expr(Spanned<Expr>),
    // Empty statement  ;
    Empty, // Still considered as expression Statement in ANSI C grammar

    // Labeled
    Label(String, Box<Spanned<Stmt>>),
    Case(Spanned<Expr>, Box<Spanned<Stmt>>), // Expr must be constant_expression
    Default(Box<Spanned<Stmt>>),

    // Block  { ... }
    Block(Vec<BlockItem>), // compound, block, block list as one type

    // Selection
    If {
        cond: Spanned<Expr>,
        then: Box<Spanned<Stmt>>,
        els: Option<Box<Spanned<Stmt>>>,
    },
    Switch {
        expr: Spanned<Expr>,
        body: Box<Spanned<Stmt>>,
    },

    // Iteration
    While {
        cond: Spanned<Expr>,
        body: Box<Spanned<Stmt>>,
    },
    DoWhile {
        body: Box<Spanned<Stmt>>,
        cond: Spanned<Expr>,
    },
    For {
        init: ForInit,
        cond: Option<Spanned<Expr>>,
        step: Option<Spanned<Expr>>,
        body: Box<Spanned<Stmt>>,
    },

    // Jump
    Return(Option<Spanned<Expr>>),
    Break,
    Continue,
    Goto(String), // verify String --> Ident ?
}

// A block contains either declarations or statements, interleaved (C99+)
#[derive(Debug, Clone, PartialEq)]
pub enum BlockItem {
    Decl(Spanned<Declaration>),
    Stmt(Spanned<Stmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForInit {
    Empty, // for(;...
    Expr(Spanned<Expr>), // for(i = 1;...
    Decl(Spanned<Declaration>), // for(int i = 1;...
}
