// -----------------------------------------------------------------------------
// ENUM
// -----------------------------------------------------------------------------

use crate::ast::ast::Expr;
use crate::ast::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct EnumSpec {
    pub name: Option<String>,
    pub variants: Option<Vec<Enumerator>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enumerator {
    pub name: String,
    pub value: Option<Box<Expr>>,          // explicit = value
    pub span: Span,
}

