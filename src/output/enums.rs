// -----------------------------------------------------------------------------
// ENUM
// -----------------------------------------------------------------------------

use crate::ast::enums::{EnumSpec, Enumerator};
use crate::output::output::Output;
use itertools::Itertools;

impl Output for EnumSpec {
    fn as_c_repr(&self) -> String {
        let mut output = String::from("enum ");
        if self.name.is_some() {
            output += self.name.as_ref().unwrap();
            output += " ";
        };
        output += "{\n";
        if self.variants.is_some() {
            output += &self
                .variants
                .as_ref()
                .unwrap()
                .iter()
                .map(|v| v.as_c_repr())
                .join(",\n");
        };
        output += "}";
        output
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}

impl Output for Enumerator {
    fn as_c_repr(&self) -> String {
        format!(
            "    {}{}",
            self.name,
            if self.value.is_some() {
                format!(" = {}", self.value.as_ref().unwrap().as_c_repr())
            } else {
                String::new()
            }
        )
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}
