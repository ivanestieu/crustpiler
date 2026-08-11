// -----------------------------------------------------------------------------
// DECLARATIONS (top-level and local)
// -----------------------------------------------------------------------------

use crate::ast::ast::Expr;
use crate::ast::decl_specifiers::TypeExpr;
use crate::ast::declarator::Declarator;
use crate::ast::span::Spanned;
use crate::ast::types::TypeName;
use crate::literals::StringLit;

#[derive(Debug, Clone, PartialEq)]
pub enum AlignmentSpecifier {
    Type(Box<TypeName>),
    Expr(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticAssert {
    pub cond: Box<Expr>,    // the constant_expression
    pub message: StringLit, // the STRING_LITERAL (mandatory in C11)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Normal(Spanned<Decl>),
    StaticAssert(Spanned<StaticAssert>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Decl {
    pub specifiers: TypeExpr,
    pub declarators: Vec<InitDeclarator>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct InitDeclarator {
    pub declarator: Declarator,
    pub init: Option<Initializer>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Initializer {
    Expr(Box<Expr>),     // int x = 5;
    List(Vec<InitItem>), // int arr[] = {1, 2, 3};
}

#[derive(Debug, Clone, PartialEq)]
pub struct InitItem {
    pub designators: Vec<Designator>, // [0] = or .field =
    pub value: Initializer,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Designator {
    Index(Box<Expr>), // [expr]
    Field(String),    // .name
}
