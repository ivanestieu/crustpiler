use crate::ast::decl_specifiers::TypeExpr;
// =============================================================================
// C AST
// Covers: full expressions, statements, declarations, types
// =============================================================================
use crate::ast::declarations::{Decl, Declaration, InitItem};
use crate::ast::function_def::FunctionDef;
use crate::ast::operators::{AssignOp, BinaryOp, PostfixOp, UnaryOp};
use crate::ast::span::Spanned;
use crate::literals::{FloatLit, IntLit, StringLit};

// -----------------------------------------------------------------------------
// EXPRESSIONS
// All operators from C11;
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    IntLit(IntLit),
    FloatLit(FloatLit),
    StringLit(StringLit),
    FuncName(String),
    CharLit(char),

    // Identifier
    Ident(String),

    // Compound literal: (Type){init}
    CompoundLit {
        ty: TypeExpr,
        init: Vec<InitItem>,
    },

    // Unary prefix
    UnaryOp {
        op: UnaryOp,
        operand: Box<Spanned<Expr>>,
    },

    // Unary postfix — kept separate because precedence/associativity differ
    PostfixOp {
        op: PostfixOp,
        operand: Box<Spanned<Expr>>,
    },

    // Binary
    BinaryOp {
        op: BinaryOp,
        lhs: Box<Spanned<Expr>>,
        rhs: Box<Spanned<Expr>>,
    },

    // Assignment (right-associative, lower precedence than most binops)
    Assign {
        op: AssignOp,
        lhs: Box<Spanned<Expr>>,
        rhs: Box<Spanned<Expr>>,
    },

    // Ternary
    Ternary {
        cond: Box<Spanned<Expr>>,
        then: Box<Spanned<Expr>>,
        els: Box<Spanned<Expr>>,
    },

    // Function call
    Call {
        callee: Box<Spanned<Expr>>,
        args: Vec<Spanned<Expr>>,
    },

    // Subscript: array[index]
    Index {
        array: Box<Spanned<Expr>>,
        index: Box<Spanned<Expr>>,
    },

    // Member access: expr.field  or  expr->field
    Member {
        expr: Box<Spanned<Expr>>,
        field: String,
        arrow: bool,               // true = ->, false = .
    },

    // Cast: (Type)expr
    Cast {
        ty: TypeExpr,
        expr: Box<Spanned<Expr>>,
    },

    // sizeof
    SizeofExpr(Box<Spanned<Expr>>),
    SizeofType(TypeExpr),

    // _Alignof
    AlignofType(TypeExpr),

    // _Generic
    Generic {
        controlling : Box<Spanned<Expr>>,
        associated: Vec<GenericAssoc>,
    },

    // Comma operator: a, b  (lowest precedence)
    Comma(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
}


// -----------------------------------------------------------------------------
// TOP-LEVEL ITEMS
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    FunctionDef(Spanned<FunctionDef>),
    Declaration(Declaration),                   // global variable / typedef / extern
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericAssoc {
    pub type_expr: Option<TypeExpr>,
    pub value: Spanned<Expr>,
}
