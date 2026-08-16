use itertools::Itertools;
// -----------------------------------------------------------------------------
// DECLARATOR
// The part that encodes pointer/array/function on top of a type specifier.
// Parsed with the classic inside-out algorithm.
// -----------------------------------------------------------------------------
use crate::ast::declarator::{ArraySize, Declarator};
use crate::output::output::Output;

impl Output for ArraySize {
    fn as_c_repr(&self) -> String {
        match self {
            ArraySize::None => String::from(""),
            ArraySize::Vla => String::from("*"),
            ArraySize::Fixed(expr) => expr.as_c_repr(),
        }
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}

impl Output for Declarator {
    fn as_c_repr(&self) -> String {
        match self {
            Declarator::Ident(name) => name.clone(),
            Declarator::Abstract => String::new(),
            Declarator::Pointer { qualifiers, inner } => {
                let mut pointer = qualifiers.iter().map(|sq| sq.node.as_c_repr()).join(" ");
                pointer += "*";
                pointer += &inner.as_c_repr();
                pointer
            }
            Declarator::Array {
                inner,
                qualifiers,
                is_static,
                size,
            } => {
                let mut array = inner.as_c_repr();
                array += "[";
                if *is_static {
                    array += "static";
                }
                array += &qualifiers.iter().map(|sq| sq.node.as_c_repr()).join(" ");
                array += &size.as_c_repr();
                array += "]";
                array
            }
            Declarator::Function {
                inner,
                params,
                old_style_params,
                variadic,
            } => {
                let mut function = inner.as_c_repr();
                function += "(";
                if params.is_some() {
                    function += &params
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|p| p.as_c_repr())
                        .join(", ");
                };
                if *variadic {
                    function += "...";
                }
                function += ")";
                if old_style_params.is_some() {
                    function += "\n";
                    function += &old_style_params
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|p| p.as_c_repr())
                        .join(";\n");
                }
                function
            }
        }
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}
