use super::*;
use crate::ast::decl_specifiers::TypeExpr;
use crate::ast::declarations::{Decl, Declaration, InitDeclarator};
use crate::ast::declarator::Declarator;
use crate::ast::span::Spanned;
use crate::ast::types::{ArithType, BaseType, Sign, SizeSpec, TypeSpec};

#[test]
fn basic_int() {
    let tokens = vec![
        SpannedToken::from(Token::KwInt),
        SpannedToken::from(Token::Ident(String::from("a"))),
        SpannedToken::from(Token::SemiColon),
    ];
    assert_eq!(
        Parser::new(tokens).parse_declaration(),
        Ok(Declaration::Normal(Spanned {
            node: Decl {
                specifiers: TypeExpr {
                    storage: None,
                    thread_local: false,
                    type_spec: TypeSpec::Arithmetic(ArithType {
                        sign: None,
                        size: SizeSpec::None,
                        base: BaseType::Int,
                        complex: None,
                    }),
                    qualifiers: vec![],
                    function_specifiers: vec![],
                    alignment: None,
                },
                declarators: vec![InitDeclarator {
                    declarator: Declarator::Ident(String::from("a")),
                    init: None,
                }],
            },
            span: Span { start: 0, end: 6 }
        })),
        "\"int a;\" did not build the expected AST"
    );
}
#[test]
fn unsigned_int() {
    let tokens = vec![
        SpannedToken::from(Token::KwUnsigned),
        SpannedToken::from(Token::KwInt),
        SpannedToken::from(Token::Ident(String::from("b"))),
        SpannedToken::from(Token::SemiColon),
    ];
    assert_eq!(
        Parser::new(tokens).parse_declaration(),
        Ok(Declaration::Normal(Spanned {
            node: Decl {
                specifiers: TypeExpr {
                    storage: None,
                    thread_local: false,
                    type_spec: TypeSpec::Arithmetic(ArithType {
                        sign: Some(Sign::Unsigned),
                        size: SizeSpec::None,
                        base: BaseType::Int,
                        complex: None,
                    }),
                    qualifiers: vec![],
                    function_specifiers: vec![],
                    alignment: None,
                },
                declarators: vec![InitDeclarator {
                    declarator: Declarator::Ident(String::from("b")),
                    init: None,
                }],
            },
            span: Span { start: 0, end: 14 }
        })),
        "\"unsigned int b;\" did not build the expected AST"
    );
}
#[test]
fn unsigned() {
    let tokens = vec![
        SpannedToken::from(Token::KwUnsigned),
        SpannedToken::from(Token::Ident(String::from("c"))),
        SpannedToken::from(Token::SemiColon),
    ];
    assert_eq!(
        Parser::new(tokens).parse_declaration(),
        Ok(Declaration::Normal(Spanned {
            node: Decl {
                specifiers: TypeExpr {
                    storage: None,
                    thread_local: false,
                    type_spec: TypeSpec::Arithmetic(ArithType {
                        sign: Some(Sign::Unsigned),
                        size: SizeSpec::None,
                        base: BaseType::Int,
                        complex: None,
                    }),
                    qualifiers: vec![],
                    function_specifiers: vec![],
                    alignment: None,
                },
                declarators: vec![InitDeclarator {
                    declarator: Declarator::Ident(String::from("c")),
                    init: None,
                }],
            },
            span: Span { start: 0, end: 11 }
        })),
        "\"unsigned c;\" did not build the expected AST"
    );
}
#[test]
fn signed_char() {
    let tokens = vec![
        SpannedToken::from(Token::KwSigned),
        SpannedToken::from(Token::KwChar),
        SpannedToken::from(Token::Ident(String::from("d"))),
        SpannedToken::from(Token::SemiColon),
    ];
    assert_eq!(
        Parser::new(tokens).parse_declaration(),
        Ok(Declaration::Normal(Spanned {
            node: Decl {
                specifiers: TypeExpr {
                    storage: None,
                    thread_local: false,
                    type_spec: TypeSpec::Arithmetic(ArithType {
                        sign: Some(Sign::Signed),
                        size: SizeSpec::None,
                        base: BaseType::Char,
                        complex: None,
                    }),
                    qualifiers: vec![],
                    function_specifiers: vec![],
                    alignment: None,
                },
                declarators: vec![InitDeclarator {
                    declarator: Declarator::Ident(String::from("d")),
                    init: None,
                }],
            },
            span: Span { start: 0, end: 13 }
        })),
        "\"signed char d;\" did not build the expected AST"
    );
}

#[test]
fn unsigned_char() {
    let tokens = vec![
        SpannedToken::from(Token::KwUnsigned),
        SpannedToken::from(Token::KwChar),
        SpannedToken::from(Token::Ident(String::from("e"))),
        SpannedToken::from(Token::SemiColon),
    ];
    assert_eq!(
        Parser::new(tokens).parse_declaration(),
        Ok(Declaration::Normal(Spanned {
            node: Decl {
                specifiers: TypeExpr {
                    storage: None,
                    thread_local: false,
                    type_spec: TypeSpec::Arithmetic(ArithType {
                        sign: Some(Sign::Unsigned),
                        size: SizeSpec::None,
                        base: BaseType::Char,
                        complex: None,
                    }),
                    qualifiers: vec![],
                    function_specifiers: vec![],
                    alignment: None,
                },
                declarators: vec![InitDeclarator {
                    declarator: Declarator::Ident(String::from("e")),
                    init: None,
                }],
            },
            span: Span { start: 0, end: 15 }
        })),
        "\"unsigned char e;\" did not build the expected AST"
    );
}

#[test]
fn short() {
    let tokens = vec![
        SpannedToken::from(Token::KwShort),
        SpannedToken::from(Token::Ident(String::from("f"))),
        SpannedToken::from(Token::SemiColon),
    ];
    assert_eq!(
        Parser::new(tokens).parse_declaration(),
        Ok(Declaration::Normal(Spanned {
            node: Decl {
                specifiers: TypeExpr {
                    storage: None,
                    thread_local: false,
                    type_spec: TypeSpec::Arithmetic(ArithType {
                        sign: None,
                        size: SizeSpec::Short,
                        base: BaseType::Int,
                        complex: None,
                    }),
                    qualifiers: vec![],
                    function_specifiers: vec![],
                    alignment: None,
                },
                declarators: vec![InitDeclarator {
                    declarator: Declarator::Ident(String::from("f")),
                    init: None,
                }],
            },
            span: Span { start: 0, end: 8 }
        })),
        "\"short f;\" did not build the expected AST"
    );
}
#[test]
fn short_int() {
    let tokens = vec![
        SpannedToken::from(Token::KwShort),
        SpannedToken::from(Token::KwInt),
        SpannedToken::from(Token::Ident(String::from("g"))),
        SpannedToken::from(Token::SemiColon),
    ];
    assert_eq!(
        Parser::new(tokens).parse_declaration(),
        Ok(Declaration::Normal(Spanned {
            node: Decl {
                specifiers: TypeExpr {
                    storage: None,
                    thread_local: false,
                    type_spec: TypeSpec::Arithmetic(ArithType {
                        sign: None,
                        size: SizeSpec::Short,
                        base: BaseType::Int,
                        complex: None,
                    }),
                    qualifiers: vec![],
                    function_specifiers: vec![],
                    alignment: None,
                },
                declarators: vec![InitDeclarator {
                    declarator: Declarator::Ident(String::from("g")),
                    init: None,
                }],
            },
            span: Span { start: 0, end: 13 }
        })),
        "\"short int g;\" did not build the expected AST"
    );
}
#[test]
fn unsigned_short() {
    let tokens = vec![
        SpannedToken::from(Token::KwUnsigned),
        SpannedToken::from(Token::KwShort),
        SpannedToken::from(Token::Ident(String::from("h"))),
        SpannedToken::from(Token::SemiColon),
    ];
    assert_eq!(
        Parser::new(tokens).parse_declaration(),
        Ok(Declaration::Normal(Spanned {
            node: Decl {
                specifiers: TypeExpr {
                    storage: None,
                    thread_local: false,
                    type_spec: TypeSpec::Arithmetic(ArithType {
                        sign: Some(Sign::Unsigned),
                        size: SizeSpec::Short,
                        base: BaseType::Int,
                        complex: None,
                    }),
                    qualifiers: vec![],
                    function_specifiers: vec![],
                    alignment: None,
                },
                declarators: vec![InitDeclarator {
                    declarator: Declarator::Ident(String::from("h")),
                    init: None,
                }],
            },
            span: Span { start: 0, end: 17 }
        })),
        "\"unsigned short h;\" did not build the expected AST"
    );
}
