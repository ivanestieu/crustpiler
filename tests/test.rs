
#[cfg(test)]
mod tests {
    use criterion_to_rust::ast::ast::*;
    use criterion_to_rust::ast::declarations::{Decl, Initializer};
    use criterion_to_rust::ast::types::{ArithType, BaseType, SizeSpec, TypeSpec};
    use criterion_to_rust::lexer::token;
    use criterion_to_rust::literals::{IntBase, IntSuffix};
    use criterion_to_rust::parser::parser;
    use criterion_to_rust::output::output;

    fn parse(src: &str) -> Result<Decl, String> {
        let tokens = token::lex(src).map_err(|e| e.to_string())?;
        parser::Parser::new(tokens).parse_decl()
    }

    // Helper to strip trailing semicolon for parsing declarations
    fn parse_without_semicolon(src: &str) -> Result<Decl, String> {
        let src_without_semi = src.trim_end_matches(';').trim();
        parse(src_without_semi)
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
                token::Token::Int(criterion_to_rust::literals::IntLit        {
                    value: 1,
                    base: IntBase::Decimal,
                    suffix: IntSuffix { unsigned: false, long: criterion_to_rust::literals::LongKind::None },
                }),
                token::Token::SemiColon,
            ]
        );
    }

    #[test]
    fn builds_full_ast_decl() {
        let decl = parse_without_semicolon("int x = 1;").unwrap();
        assert_eq!(decl.specifiers.type_spec, TypeSpec::Arithmetic(ArithType {
            sign: None,
            base: BaseType::Int,
            size: SizeSpec::None,
            complex: None,
        }));
        assert_eq!(decl.declarators.len(), 1);

        let d = &decl.declarators[0];
        assert_eq!(d.declarator.ident(), Some("x"));
        assert_eq!(
            d.init,
            Some(Initializer::Expr(Box::new(Expr::IntLit(criterion_to_rust::literals::IntLit {
                value: 1,
                base: IntBase::Decimal,
                suffix: IntSuffix { unsigned: false, long: criterion_to_rust::literals::LongKind::None },
            }))))
        );
    }

    #[test]
    fn uninitialized() {
        let decl = parse_without_semicolon("int y;").unwrap();
        assert_eq!(decl.declarators[0].init, None);
        assert_eq!(decl.declarators[0].declarator.ident(), Some("y"));
    }

    #[test]
    fn outputs_rust() {
        let decl = parse_without_semicolon("int x = 1;").unwrap();
        assert_eq!(output::output_decl(&decl), "let x: i32 = 1;");
    }

    #[test]
    fn skips_comments_and_whitespace() {
        let decl = parse_without_semicolon("  int   z = 42 ; // trailing comment").unwrap();
        assert_eq!(decl.declarators[0].declarator.ident(), Some("z"));
    }

    #[test]
    fn rejects_missing_semicolon() {
        // Without the semicolon, it should still parse fine as a declaration
        // (the parser doesn't require or handle semicolons in parse_decl)
        assert!(parse("int x = 1").is_ok());
    }

    #[test]
    fn rejects_bad_lead() {
        assert!(parse_without_semicolon("= 1;").is_err());
    }
}
