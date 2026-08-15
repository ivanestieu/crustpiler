// -----------------------------------------------------------------------------
// FUNCTION DEFINITION
// -----------------------------------------------------------------------------

use crate::ast::decl_specifiers::TypeExpr;
use crate::ast::declarations::Declaration;
use crate::ast::declarator::Declarator;
use crate::ast::statements::BlockItem;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub ret: TypeExpr,
    pub declarator: Declarator, // encodes name + params
    pub old_style_params: Vec<Declaration>,
    pub body: Vec<BlockItem>,
}

impl FunctionDef {
    pub fn name(&self) -> Option<&str> {
        self.declarator.ident()
    }
}
