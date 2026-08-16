// -----------------------------------------------------------------------------
// DECLARATOR
// The part that encodes pointer/array/function on top of a type specifier.
// Parsed with the classic inside-out algorithm.
// -----------------------------------------------------------------------------
use crate::ast::ast::Expr;
use crate::ast::parameters::ParamDecl;
use crate::ast::span::Spanned;
use crate::ast::types::TypeQualifier;

#[derive(Debug, Clone, PartialEq)]
pub enum ArraySize {
    None, // []
    Vla,  // [*]
    Fixed(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declarator {
    // Leaf: just a name (or abstract — no name, for param/cast types)
    Ident(String),
    Abstract, // nameless, used in casts / params

    // Derived
    Pointer {
        qualifiers: Vec<Spanned<TypeQualifier>>,
        inner: Box<Declarator>,
    },
    Array {
        inner: Box<Declarator>,
        qualifiers: Vec<Spanned<TypeQualifier>>, // int a[const 3] is legal
        is_static: bool,
        size: ArraySize,
    },
    Function {
        inner: Box<Declarator>,
        params: Option<Vec<ParamDecl>>,
        old_style_params: Option<Vec<Expr>>,
        variadic: bool,
    },
}

impl Declarator {
    /// Extract the declared identifier name, if any
    pub fn ident(&self) -> Option<&str> {
        match self {
            Declarator::Ident(name) => Some(name),
            Declarator::Abstract => None,
            Declarator::Pointer { inner, .. }
            | Declarator::Array { inner, .. }
            | Declarator::Function { inner, .. } => inner.ident(),
        }
    }
}
