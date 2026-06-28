// =============================================================================
// main.rs — pipeline: source → logos lexer → recursive-descent parser → output
// =============================================================================

mod ast;
mod criterion;
mod lexer;
mod output;
mod parser;
mod debug;

use lexer::token;
use parser::parser::Parser;
use crate::ast::Item;
use crate::lexer::token::SpannedToken;

fn main() {
    let source : &str = "int x, z = 1, c;\nint y = 2;\n";
    println!("C source:\n{}", source);

    let tokens : Vec<SpannedToken> = token::lex(source).expect("Failed to lex source.");
    println!("\nTokens:");
    for t in &tokens {
        println!("  {:?}  @ {}..{}", t.token, t.span.start, t.span.end);
    }

    let translation : Vec<Item> = Parser::new(tokens).parse_translation_unit().expect("Failed to parse declaration.");

    let dot : String = debug::dot::dump_translation_unit(&translation);
    std::fs::write("ast.dot", dot).expect("Failed to write AST to file.");

    println!("\nRust output:\n{}", output::output_translation_unit(&translation));
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::*;

    fn parse(src: &str) -> Result<Decl, String> {
        let tokens = token::lex(src)?;
        Parser::new(tokens).parse_decl()
    }

    #[test]
    fn lexes_with_logos() {
        let toks = token::lex("int x = 1;").unwrap();
        let kinds: Vec<_> = toks.iter().map(|t| t.token.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                token::Token::KwInt,
                token::Token::Ident("x".into()),
                token::Token::Equals,
                token::Token::IntLit(1),
                token::Token::SemiColon,
            ]
        );
    }

    #[test]
    fn builds_full_ast_decl() {
        let decl = parse("int x = 1;").unwrap();
        assert_eq!(decl.spec, TypeSpec::Int);
        assert_eq!(decl.declarators.len(), 1);

        let d = &decl.declarators[0];
        assert_eq!(d.declarator.ident(), Some("x"));
        assert_eq!(
            d.init,
            Some(Initializer::Expr(Box::new(Expr::IntLit(IntLit {
                value: 1,
                base: IntBase::Decimal,
                suffix: IntSuffix { unsigned: false, long: LongKind::None },
            }))))
        );
    }

    #[test]
    fn uninitialized() {
        let decl = parse("int y;").unwrap();
        assert_eq!(decl.declarators[0].init, None);
        assert_eq!(decl.declarators[0].declarator.ident(), Some("y"));
    }

    #[test]
    fn outputs_rust() {
        let decl = parse("int x = 1;").unwrap();
        assert_eq!(output::output_decl(&decl), "let x: i32 = 1;");
    }

    #[test]
    fn skips_comments_and_whitespace() {
        let decl = parse("  int   z = 42 ; // trailing comment").unwrap();
        assert_eq!(decl.declarators[0].declarator.ident(), Some("z"));
    }

    #[test]
    fn rejects_missing_semicolon() {
        assert!(parse("int x = 1").is_err());
    }

    #[test]
    fn rejects_bad_lead() {
        assert!(parse("= 1;").is_err());
    }
}
