use crate::ast::decl_specifiers::TypeExpr;
use crate::output::output::Output;

impl Output for TypeExpr {
    fn as_c_repr(&self) -> String {
        self.type_spec.as_c_repr()
    }

    fn as_rust_repr(&self) -> String {
        self.type_spec.as_rust_repr()
        // missing other fields
    }
}
