// =============================================================================
// output.rs — AST → Rust source.
// Walks the full-AST Decl, outputting one `let` per InitDeclarator.
//   int x = 1;   →   let x: i32 = 1;
// =============================================================================

use crate::ast::ast::*;

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
        TypeSpec::Arithmetic(ArithType { base: BaseType::Int, sign : Some(Sign::Unsigned), size: SizeSpec::Short }) => "u16",
        TypeSpec::Arithmetic(ArithType { base: BaseType::Int, sign : Some(Sign::Unsigned), size: SizeSpec::None }) => "u32",
        TypeSpec::Arithmetic(ArithType { base: BaseType::Int, sign : Some(Sign::Unsigned), size: SizeSpec::Long }) => "u64",
        TypeSpec::Arithmetic(ArithType { base: BaseType::Int, sign : Some(Sign::Unsigned), size: SizeSpec::LongLong }) => "u128",
        TypeSpec::Arithmetic(ArithType { base: BaseType::Int, size: SizeSpec::Short, .. }) => "i16",
        TypeSpec::Arithmetic(ArithType { base: BaseType::Int, size: SizeSpec::None, .. }) => "i32",
        TypeSpec::Arithmetic(ArithType { base: BaseType::Int, size: SizeSpec::Long, .. }) => "i64",
        TypeSpec::Arithmetic(ArithType { base: BaseType::Int, size: SizeSpec::LongLong, .. }) => "i128",
        TypeSpec::Arithmetic(ArithType { base: BaseType::Float, .. }) => "f32",
        TypeSpec::Arithmetic(ArithType { base: BaseType::Double, size: SizeSpec::Long, .. }) => "/* f80 (long double) isn't defined in rust */ f64",
        TypeSpec::Arithmetic(ArithType { base: BaseType::Double, .. }) => "f64",
        TypeSpec::Arithmetic(ArithType { base: BaseType::Char, sign : Some(Sign::Unsigned), .. }) => "/* uchar (unsigned char) isn't defined in rust */ u32",
        TypeSpec::Arithmetic(ArithType { base: BaseType::Char, .. }) => "char",
        TypeSpec::Void => "()" ,
        TypeSpec::Bool => "bool",
        _ => "unknown", // Placeholder for other types
    }
}

fn output_expr(expr: &Expr) -> String {
    match expr {
        Expr::IntLit(lit) => lit.value.to_string(),
        Expr::FloatLit(lit) => lit.value.to_string(),
        Expr::CharLit(lit) => format!("'{}'", lit),
        Expr::StringLit(lit) => lit.value.to_string(),
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