// -----------------------------------------------------------------------------
// STRUCT / UNION
// -----------------------------------------------------------------------------

use crate::ast::ast::Expr;
use crate::ast::declarations::StaticAssert;
use crate::ast::declarator::Declarator;
use crate::ast::span::Spanned;
use crate::ast::types::TypeExpr;

#[derive(Debug, Clone, PartialEq)]
pub struct StructOrUnion {
    pub name: Option<String>,              // anonymous if None
    pub fields: Option<Vec<StructMember>>, // None = forward declaration
}

#[derive(Debug, Clone, PartialEq)]
pub enum StructMember {
    Field(FieldDecl),
    StaticAssert(StaticAssert),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub type_expr: TypeExpr,
    pub declarators: Vec<FieldDeclarator>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDeclarator {
    pub declarator: Option<Declarator>, // None for anonymous bitfield
    pub bit_width: Option<Box<Spanned<Expr>>>,
}
