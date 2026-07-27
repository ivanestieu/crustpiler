pub mod ast;
mod criterion;
mod debug;
pub mod lexer;
pub mod parser;
pub mod output;
pub mod literals;

use crate::ast::ast::Item;
use crate::lexer::token;
use crate::lexer::token::SpannedToken;
use crate::parser::parser::Parser;
pub fn run(file_path : String) -> Result<(), String> {
    let source: &str = &*std::fs::read_to_string(&file_path).or_else(|e| Err(format!("Failed to read source file: {}", e)))?;
    println!("C source:\n{}", source);

    let tokens : Vec<SpannedToken> = token::lex(source).or_else(|e| Err(format!("Lexing error: {}", e)))?;
    println!("\nTokens:");
    for t in &tokens {
        println!("  {:?}  @ {}..{}", t.token, t.span.start, t.span.end);
    }

    let translation : Vec<Item> = Parser::new(tokens).parse_translation_unit().or_else(|e| Err(format!("Parsing error: {}", e)))?;

    let dot : String = debug::dot::dump_translation_unit(&translation);
    let dot_file_path = format!("{}.dot", file_path.strip_suffix(".c").unwrap_or(&file_path));
    std::fs::write(&dot_file_path, dot).expect("Failed to write AST to file.");

    println!("\nRust output:\n{}", output::output::output_translation_unit(&translation));
    Ok(())
}
