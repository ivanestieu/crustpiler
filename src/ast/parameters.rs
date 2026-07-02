// -----------------------------------------------------------------------------
// PARAMETERS
// -----------------------------------------------------------------------------

use crate::ast::decl_specifiers::TypeExpr;
use crate::ast::declarator::Declarator;

#[derive(Debug, Clone, PartialEq)]
pub struct ParamDecl {
    pub specifiers: TypeExpr,
    pub declarator: Declarator,            // may be Abstract for unnamed params
}

