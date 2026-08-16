// -----------------------------------------------------------------------------
// DECLARATIONS (top-level and local)
// -----------------------------------------------------------------------------

use crate::ast::decl_specifiers::AlignmentSpecifier;
use crate::ast::declarations::{
    Decl, Declaration, Designator, InitDeclarator, InitItem, Initializer, StaticAssert,
};
use crate::ast::span::Spanned;
use crate::output::output::Output;
use itertools::Itertools;

impl Output for AlignmentSpecifier {
    fn as_c_repr(&self) -> String {
        match self {
            AlignmentSpecifier::TypeName(type_name) => type_name.as_c_repr(),
            AlignmentSpecifier::Expr(expr) => expr.as_c_repr(),
        }
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}

impl Output for StaticAssert {
    fn as_c_repr(&self) -> String {
        format!(
            "_Static_assert({}, {});",
            self.cond.as_c_repr(),
            self.message.value.escape_debug().to_string()
        )
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}

impl Output for Declaration {
    fn as_c_repr(&self) -> String {
        match self {
            Declaration::Normal(Spanned { node: decl, .. }) => format!("{};", decl.as_c_repr()),
            Declaration::StaticAssert(Spanned {
                node: static_assert,
                ..
            }) => static_assert.as_c_repr(),
        }
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}

impl Output for Decl {
    fn as_c_repr(&self) -> String {
        format!(
            "{} {}",
            self.specifiers.as_c_repr(),
            self.declarators
                .iter()
                .map(|d| d.as_c_repr())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
    fn as_rust_repr(&self) -> String {
        let spec = self.specifiers.as_rust_repr();
        let mut decl: Vec<String> = vec![];
        let print_vec = |v: &Vec<String>| {
            format!("{}", {
                if v.len() == 1 {
                    format!("{}", v[0])
                } else {
                    format!("({})", v.join(", "))
                }
            })
        };
        let mut declarators_iter = self.declarators.iter().peekable();
        while declarators_iter.peek().is_some() {
            let mut declarators: Vec<String> = vec![];
            let mut specs: Vec<String> = vec![];
            declarators_iter
                .peeking_take_while(|d| d.init.is_none())
                .for_each(|d| {
                    declarators.push(d.declarator.ident().unwrap().to_string());
                    specs.push(spec.clone());
                });
            if !declarators.is_empty() {
                decl.push(format!(
                    "let {} : {};",
                    print_vec(&declarators),
                    print_vec(&specs)
                ));
            }
            specs.clear();
            declarators.clear();
            let mut inits: Vec<String> = vec![];
            declarators_iter
                .peeking_take_while(|d| d.init.is_some())
                .for_each(|d| {
                    declarators.push(d.declarator.ident().unwrap().to_string());
                    specs.push(spec.clone());
                    inits.push(d.init.as_ref().unwrap().as_rust_repr());
                });
            if !declarators.is_empty() {
                decl.push(format!(
                    "let {} : {} = {};",
                    print_vec(&declarators),
                    print_vec(&specs),
                    print_vec(&inits)
                ));
            }
        }
        decl.join("\n")
    }
}

impl Output for InitDeclarator {
    fn as_c_repr(&self) -> String {
        let name = self.declarator.ident().unwrap_or("_");
        match &self.init {
            Some(Initializer::Expr(e)) => format!("{} = {};", name, e.as_rust_repr()),
            None => format!("{};", name),
            _ => format!("{};", name), // Placeholder for other initializers
        }
    }

    fn as_rust_repr(&self) -> String {
        let name = self.declarator.ident().unwrap_or("_");
        match &self.init {
            Some(Initializer::Expr(e)) => format!("{} = {};", name, e.as_rust_repr()),
            None => format!("let {};", name),
            _ => format!("let {};", name), // Placeholder for other initializers
        }
    }
}

impl Output for Initializer {
    fn as_c_repr(&self) -> String {
        match self {
            Initializer::Expr(e) => e.as_c_repr(),
            Initializer::List(list) => format!(
                "{{{}}}",
                list.iter().map(|init| init.as_c_repr()).join(", ")
            ),
        }
    }

    fn as_rust_repr(&self) -> String {
        match self {
            Initializer::Expr(e) => e.as_rust_repr(),
            Initializer::List(list) => format!(
                "{{{}}}",
                list.iter().map(|init| init.as_rust_repr()).join(", ")
            ),
        }
    }
}

impl Output for InitItem {
    fn as_c_repr(&self) -> String {
        format!(
            "{} = {}",
            self.designators.iter().map(|d| d.as_c_repr()).join(""),
            self.value.as_c_repr()
        )
    }

    fn as_rust_repr(&self) -> String {
        format!(
            "{} = {}",
            self.designators.iter().map(|d| d.as_rust_repr()).join(""),
            self.value.as_rust_repr()
        )
    }
}

impl Output for Designator {
    fn as_c_repr(&self) -> String {
        match self {
            Designator::Index(e) => format!("[{}]", e.as_c_repr()),
            Designator::Field(f) => format!(".{}", f),
        }
    }

    fn as_rust_repr(&self) -> String {
        match self {
            Designator::Index(e) => format!("[{}]", e.as_rust_repr()),
            Designator::Field(f) => format!(".{}", f),
        }
    }
}
