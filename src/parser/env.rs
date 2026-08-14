use std::collections::HashSet;

/// Typedef environment — the heart of context-sensitive C parsing.
/// Empty here, but this is exactly where `typedef int Foo;` would register
/// "Foo" so a later `Foo x;` parses as a declaration, not an expression.
#[derive(Default)]
pub struct Env {
    scopes: Vec<Scope>,
}
#[derive(Default, Clone)]
struct Scope {
    typedefs: HashSet<String>,
    shadowed: HashSet<String>,
}

impl Env {
    pub fn new() -> Env {
        Self {
            scopes: vec![Scope::default()],
        }
    }

    pub fn push(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
        if self.scopes.is_empty() {
            panic!("File scope popped!")
        }
    }

    pub fn define_typedef(&mut self, name: String) -> () {
        let current_scope = self.scopes.last_mut().unwrap();
        current_scope.typedefs.insert(name);
    }

    pub fn shadow(&mut self, name: String) -> () {
        let current_scope = self.scopes.last_mut().unwrap();
        current_scope.shadowed.insert(name);
    }

    pub fn is_typedef(&self, name: &String) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.shadowed.contains(name) {
                return false;
            }
            if scope.typedefs.contains(name) {
                return true;
            }
        }
        false
    }
}
