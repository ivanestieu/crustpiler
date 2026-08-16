// -----------------------------------------------------------------------------
// STRUCT / UNION
// -----------------------------------------------------------------------------

use crate::ast::struct_union::{FieldDecl, FieldDeclarator, StructMember, StructOrUnion};
use crate::output::output::Output;
use itertools::Itertools;

impl Output for StructOrUnion {
    fn as_c_repr(&self) -> String {
        match (&self.name, &self.fields) {
            (Some(name), Some(fields)) => format!(
                "{} {{\n{}\n}}",
                name,
                fields.iter().map(|m| m.as_c_repr()).join("\n")
            ),
            (Some(name), None) => name.clone(),
            (None, Some(fields)) => format!(
                "{{\n{}\n}}",
                fields.iter().map(|m| m.as_c_repr()).join("\n")
            ),
            _ => panic!("This state should not exists."), // Make this state impossible at compile time
        }
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}

impl Output for StructMember {
    fn as_c_repr(&self) -> String {
        match self {
            StructMember::Field(field) => format!("{};", field.as_c_repr()),
            StructMember::StaticAssert(static_assert) => static_assert.as_c_repr(),
        }
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}

impl Output for FieldDecl {
    fn as_c_repr(&self) -> String {
        format!("{} {}",
            self.type_expr.as_c_repr(),
            self.declarators.iter().map(|fd| fd.as_c_repr()).join(", ")
        )
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}

impl Output for FieldDeclarator {
    fn as_c_repr(&self) -> String {
        match (&self.declarator, &self.bit_width) {
            (Some(declarator), Some(bit_width)) => format!(
                "{} : {}",
                declarator.as_c_repr(),
                bit_width.node.as_c_repr()
            ),
            (Some(declarator), None) => declarator.as_c_repr(),
            (None, Some(bit_width)) => format!(
                ": {}",
                bit_width.node.as_c_repr()
            ),
            _ => panic!("This state should not exists."), // Make this state impossible at compile time
        }
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}
