use crate::ast::function_def::FunctionDef;
use crate::output::output::Output;
use itertools::Itertools;

// -----------------------------------------------------------------------------
// FUNCTION DEFINITION
// -----------------------------------------------------------------------------
impl Output for FunctionDef {
    fn as_c_repr(&self) -> String {
        format!("{} {}\n{}{{\n{}\n}}",
            self.ret.as_c_repr(),
            self.declarator.as_c_repr(),
            self.old_style_params.iter().map(|p| p.as_c_repr()).join("\n"),
            self.body.iter().map(|b| b.as_c_repr()).join("\n")
        )
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}
