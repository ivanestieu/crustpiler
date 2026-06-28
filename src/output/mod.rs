// =============================================================================
// output.rs — AST → Rust source.
// Walks the full-AST Decl, outputting one `let` per InitDeclarator.
//   int x = 1;   →   let x: i32 = 1;
// =============================================================================

use crate::ast::*;

pub fn output_decl(decl: &Decl) -> String {
    let ty = output_type(&decl.spec);

    // One Rust `let` line per declarator (handles `int a, b;` once commas exist).
    decl.declarators
        .iter()
        .map(|d| output_init_declarator(d, ty))
        .collect::<Vec<_>>()
        .join("\n")
}

fn output_init_declarator(d: &InitDeclarator, ty: &str) -> String {
    let name = d.declarator.ident().unwrap_or("_");
    match &d.init {
        Some(Initializer::Expr(e)) => format!("let {}: {} = {};", name, ty, output_expr(e)),
        None => format!("let {}: {};", name, ty),
        _ => format!("let {}: {};", name, ty), // Placeholder for other initializers
    }
}

fn output_type(spec: &TypeSpec) -> &'static str {
    match spec {
        TypeSpec::Int => "i32",
        _ => "unknown", // Placeholder for other types
    }
}

fn output_expr(expr: &Expr) -> String {
    match expr {
        Expr::IntLit(lit) => lit.value.to_string(),
        _ => "/* expr */".to_string(), // Placeholder for other expressions
    }
}

pub fn output_translation_unit(p0: &Vec<Item>) -> String {
    p0.iter().filter(|item| matches!(item, Item::Decl(_))).map(|item| {
        match item {
            Item::Decl(decl) => output_decl(&decl.node),
            _ => String::new(),
        }
    }).collect::<Vec<_>>().join("\n")
}