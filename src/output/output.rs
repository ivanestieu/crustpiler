// =============================================================================
// output.rs — AST → Rust source.
// Walks the full-AST Decl, outputting one `let` per InitDeclarator.
//   int x = 1;   →   let x: i32 = 1;
// =============================================================================

use itertools::Itertools;
use crate::ast::ast::*;
use crate::ast::declarations::Declaration;
use crate::ast::declarator::Declarator;

pub trait Output {
    fn as_c_repr(&self) -> String;
    fn as_rust_repr(&self) -> String;
}

impl Output for Vec<Item> {
    fn as_c_repr(&self) -> String {
        self.iter()
            .map(|item| match item {
                Item::Declaration(decl) => decl.as_c_repr(),
                Item::FunctionDef(def) => def.as_c_repr(),
            })
            .join("\n")
    }

    fn as_rust_repr(&self) -> String {
        self.iter()
            .filter_map(|item| match item {
                Item::Declaration(decl) => match decl {
                    Declaration::Normal(d) => Some(d.node.as_rust_repr()),
                    Declaration::StaticAssert(_) => None,
                },
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// Walk an abstract declarator inside-out, wrapping `base` in each derived layer.
// Abstract / Ident is the leaf (no wrapping). Pointer → `*mut T` (or `*const T`
// if const-qualified). Array/Function are rendered best-effort.
pub(crate) fn wrap_declarator_rust(d: &Declarator, base: String) -> String {
    match d {
        Declarator::Abstract | Declarator::Ident(_) => base,
        Declarator::Pointer { inner, qualifiers } => {
            // choose *const vs *mut from the pointer's own qualifiers
            let is_const = qualifiers
                .iter()
                .any(|q| format!("{:?}", q).contains("Const"));
            let ptr = if is_const {
                format!("*{}", wrap_declarator_rust(inner, base))
            } else {
                format!("*mut {}", wrap_declarator_rust(inner, base))
            };
            ptr
        }
        Declarator::Array { inner, size, .. } => {
            use crate::ast::declarator::ArraySize;
            let elem = wrap_declarator_rust(inner, base);
            match size {
                ArraySize::Fixed(n) => format!("[{}; {}]", elem, n.as_rust_repr()),
                _ => format!("*mut {} /* unsized array */", elem),
            }
        }
        Declarator::Function { inner, .. } => {
            // function type in a type-name → a fn pointer; best-effort
            format!(
                "fn() -> {} /* fn type */",
                wrap_declarator_rust(inner, base)
            )
        }
    }
}
