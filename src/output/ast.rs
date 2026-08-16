// =============================================================================
// C AST
// Covers: full expressions, statements, declarations, types
// =============================================================================
use crate::ast::declarations::{Declaration, InitItem};
use crate::ast::function_def::FunctionDef;
use crate::ast::operators::{AssignOp, BinaryOp, PostfixOp, UnaryOp};
use crate::ast::span::Spanned;
use crate::ast::types::TypeName;
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
        type_name: TypeName,
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
        arrow: bool, // true = ->, false = .
    },

    // Cast: (Type)expr
    Cast {
        type_name: TypeName,
        expr: Box<Spanned<Expr>>,
    },

    // sizeof
    SizeofExpr(Box<Spanned<Expr>>),
    SizeofType(TypeName),

    // _Alignof
    AlignofType(TypeName),

    // _Generic
    Generic {
        controlling: Box<Spanned<Expr>>,
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
    FunctionDef(FunctionDef),
    Declaration(Declaration), // global variable / typedef / extern
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericAssoc {
    pub type_name: Option<TypeName>,
    pub value: Spanned<Expr>,
}
