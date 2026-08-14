use super::*;
use crate::ast::types::{ArithType, SizeSpec};

#[test]
fn basic_int() {
    let tokens = vec![
        SpannedToken {
            token: Token::KwInt,
            span: crate::lexer::token::Span { start: 0, end: 2 },
        },
        SpannedToken {
            token: Token::Ident(String::from("a")),
            span: crate::lexer::token::Span { start: 4, end: 5 },
        },
        SpannedToken {
            token: Token::SemiColon,
            span: crate::lexer::token::Span { start: 5, end: 6 },
        },
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
        SpannedToken {
            token: Token::KwUnsigned,
            span: crate::lexer::token::Span { start: 0, end: 8 },
        },
        SpannedToken {
            token: Token::KwInt,
            span: crate::lexer::token::Span { start: 9, end: 11 },
        },
        SpannedToken {
            token: Token::Ident(String::from("b")),
            span: crate::lexer::token::Span { start: 12, end: 13 },
        },
        SpannedToken {
            token: Token::SemiColon,
            span: crate::lexer::token::Span { start: 13, end: 14 },
        },
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
        SpannedToken {
            token: Token::KwUnsigned,
            span: crate::lexer::token::Span { start: 0, end: 8 },
        },
        SpannedToken {
            token: Token::Ident(String::from("c")),
            span: crate::lexer::token::Span { start: 9, end: 10 },
        },
        SpannedToken {
            token: Token::SemiColon,
            span: crate::lexer::token::Span { start: 10, end: 11 },
        },
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
        SpannedToken {
            token: Token::KwSigned,
            span: crate::lexer::token::Span { start: 0, end: 6 },
        },
        SpannedToken {
            token: Token::KwChar,
            span: crate::lexer::token::Span { start: 7, end: 10 },
        },
        SpannedToken {
            token: Token::Ident(String::from("d")),
            span: crate::lexer::token::Span { start: 11, end: 12 },
        },
        SpannedToken {
            token: Token::SemiColon,
            span: crate::lexer::token::Span { start: 12, end: 13 },
        },
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
        SpannedToken {
            token: Token::KwUnsigned,
            span: crate::lexer::token::Span { start: 0, end: 8 },
        },
        SpannedToken {
            token: Token::KwChar,
            span: crate::lexer::token::Span { start: 9, end: 12 },
        },
        SpannedToken {
            token: Token::Ident(String::from("e")),
            span: crate::lexer::token::Span { start: 13, end: 14 },
        },
        SpannedToken {
            token: Token::SemiColon,
            span: crate::lexer::token::Span { start: 14, end: 15 },
        },
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
        SpannedToken {
            token: Token::KwShort,
            span: crate::lexer::token::Span { start: 0, end: 5 },
        },
        SpannedToken {
            token: Token::Ident(String::from("f")),
            span: crate::lexer::token::Span { start: 6, end: 7 },
        },
        SpannedToken {
            token: Token::SemiColon,
            span: crate::lexer::token::Span { start: 7, end: 8 },
        },
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
        SpannedToken {
            token: Token::KwShort,
            span: crate::lexer::token::Span { start: 0, end: 5 },
        },
        SpannedToken {
            token: Token::KwInt,
            span: crate::lexer::token::Span { start: 6, end: 10 },
        },
        SpannedToken {
            token: Token::Ident(String::from("g")),
            span: crate::lexer::token::Span { start: 11, end: 12 },
        },
        SpannedToken {
            token: Token::SemiColon,
            span: crate::lexer::token::Span { start: 12, end: 13 },
        },
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
        SpannedToken {
            token: Token::KwUnsigned,
            span: crate::lexer::token::Span { start: 0, end: 8 },
        },
        SpannedToken {
            token: Token::KwShort,
            span: crate::lexer::token::Span { start: 9, end: 14 },
        },
        SpannedToken {
            token: Token::Ident(String::from("h")),
            span: crate::lexer::token::Span { start: 15, end: 16 },
        },
        SpannedToken {
            token: Token::SemiColon,
            span: crate::lexer::token::Span { start: 16, end: 17 },
        },
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
