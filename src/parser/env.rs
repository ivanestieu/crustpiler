use std::collections::HashSet;

/// Typedef environment — the heart of context-sensitive C parsing.
/// Empty here, but this is exactly where `typedef int Foo;` would register
/// "Foo" so a later `Foo x;` parses as a declaration, not an expression.
#[derive(Default)]
pub struct Env {
    typedefs: HashSet<String>,
}

impl Env {
    pub fn is_typedef(&self, name: &str) -> bool {
        self.typedefs.contains(name)
    }
    pub fn define_typedef(&mut self, name: &str) {
        self.typedefs.insert(name.to_string());
    }
}

