// -----------------------------------------------------------------------------
// PARAMETERS
// -----------------------------------------------------------------------------

use crate::ast::parameters::ParamDecl;
use crate::output::output::Output;

impl Output for ParamDecl {
    fn as_c_repr(&self) -> String {
        format!(
            "{} {}",
            self.specifiers.as_c_repr(),
            self.declarator.as_c_repr()
        )
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}
