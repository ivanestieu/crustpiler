pub mod utils;

#[cfg(test)]
pub mod test_valid_int {
    use criterion_to_rust::lexer::token::*;
    use parameterized::parameterized;
    use criterion_to_rust::literals::{IntBase, IntLit, IntSuffix, LongKind};
    use super::assert_all_eq;
    use super::utils::*;

    #[parameterized(base = {
        "0", "00", "0x0", "0X0"
    }, int_base = {
        IntBase::Decimal,
        IntBase::Octal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal
    }, name = {
        "decimal",
        "octal",
        "hex_lower",
        "hex_upper"
    })]
    fn test_zero(base : &str, int_base : IntBase, name : &str) {
        let token = lex_token(base);
        assert_eq!(token, Token::Int(IntLit {
            value: 0,
            base : int_base,
            suffix: IntSuffix { unsigned: false, long: LongKind::None },
        }), "Test {} failed for source: {}", name, base);
    }

    #[parameterized(base = {
        "12345", "0644", "0x1A3F", "0X1A3F", "0x1a3f", "0X1a3f"
    }, int_base = {
        IntBase::Decimal,
        IntBase::Octal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal
    }, int_value = {
        12345,
        420,
        0x1A3F,
        0x1A3F,
        0x1A3F,
        0x1A3F
    }, name = {
        "decimal",
        "octal",
        "binary_lower",
        "binary_upper",
        "hex_lower_2",
        "hex_upper_1"
    })]
    fn test_positive(base : &str, int_base : IntBase, int_value : u64, name : &str) {
        let token = lex_token(base);
        assert_eq!(token, Token::Int(IntLit {
            value: int_value,
            base: int_base,
            suffix: IntSuffix { unsigned: false, long: LongKind::None },
        }), "Test {} failed for source: {}", name, base);
    }

    #[parameterized(base = {
        "12345", "0644", "0x1A3F", "0X1A3F", "0x1a3f", "0X1a3f"
    }, int_base = {
        IntBase::Decimal,
        IntBase::Octal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal
    }, int_value = {
        12345,
        420,
        0x1A3F,
        0x1A3F,
        0x1A3F,
        0x1A3F
    }, name = {
        "decimal",
        "octal",
        "hex_lower_1",
        "hex_upper_2",
        "hex_lower_2",
        "hex_upper_1"
    })]
    fn test_u_suffix(base : &str, int_base : IntBase, int_value : u64, name : &str) {
        let sources = build_int_suffix(base, true, LongKind::None);
        let tokens = lex_tokens(&sources);
        assert_eq!(tokens[0], Token::Int(IntLit {
            value: int_value,
            base: int_base,
            suffix: IntSuffix { unsigned: true, long: LongKind::None },
        }), "Test {} failed for source: {}", name, sources[0]);
        assert_all_eq!(&tokens);
    }

    #[parameterized(base = {
        "12345", "0644", "0x1A3F", "0X1A3F", "0x1a3f", "0X1a3f"
    }, int_base = {
        IntBase::Decimal,
        IntBase::Octal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal
    }, int_value = {
        12345,
        420,
        0x1A3F,
        0x1A3F,
        0x1A3F,
        0x1A3F
    }, name = {
        "decimal",
        "octal",
        "hex_lower_1",
        "hex_upper_2",
        "hex_lower_2",
        "hex_upper_1"
    })]
    fn test_l_suffix() {
        let tokens = build_int_suffix("100", false, LongKind::Long);
        assert_eq!(lex_token(&tokens[0]), Token::Int(IntLit {
            value: 100,
            base: IntBase::Decimal,
            suffix: IntSuffix { unsigned: false, long: LongKind::Long },
        }));
        assert_all_eq!(lex_tokens(&tokens));
    }

    #[parameterized(base = {
        "12345", "0644", "0x1A3F", "0X1A3F", "0x1a3f", "0X1a3f"
    }, int_base = {
        IntBase::Decimal,
        IntBase::Octal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal
    }, int_value = {
        12345,
        420,
        0x1A3F,
        0x1A3F,
        0x1A3F,
        0x1A3F
    }, name = {
        "decimal",
        "octal",
        "hex_lower_1",
        "hex_upper_2",
        "hex_lower_2",
        "hex_upper_1"
    })]
    fn test_ll_suffix() {
        let tokens = build_int_suffix("100", false, LongKind::LongLong);
        assert_eq!(lex_token(&tokens[0]), Token::Int(IntLit {
            value: 100,
            base: IntBase::Decimal,
            suffix: IntSuffix { unsigned: false, long: LongKind::LongLong },
        }));
        assert_all_eq!(lex_tokens(&tokens));
    }

    #[parameterized(base = {
        "12345", "0644", "0x1A3F", "0X1A3F", "0x1a3f", "0X1a3f"
    }, int_base = {
        IntBase::Decimal,
        IntBase::Octal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal
    }, int_value = {
        12345,
        420,
        0x1A3F,
        0x1A3F,
        0x1A3F,
        0x1A3F
    }, name = {
        "decimal",
        "octal",
        "hex_lower_1",
        "hex_upper_2",
        "hex_lower_2",
        "hex_upper_1"
    })]
    fn test_ul_suffix() {
        let tokens = build_int_suffix("100", true, LongKind::Long);
        assert_eq!(lex_token(&tokens[0]), Token::Int(IntLit {
            value: 100,
            base: IntBase::Decimal,
            suffix: IntSuffix { unsigned: true, long: LongKind::Long },
        }));
        assert_all_eq!(lex_tokens(&tokens));
    }

    #[parameterized(base = {
        "12345", "0644", "0x1A3F", "0X1A3F", "0x1a3f", "0X1a3f"
    }, int_base = {
        IntBase::Decimal,
        IntBase::Octal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal
    }, int_value = {
        12345,
        420,
        0x1A3F,
        0x1A3F,
        0x1A3F,
        0x1A3F
    }, name = {
        "decimal",
        "octal",
        "hex_lower_1",
        "hex_upper_2",
        "hex_lower_2",
        "hex_upper_1"
    })]
    fn test_ull_suffix() {
        let tokens = build_int_suffix("100", true, LongKind::LongLong);
        assert_eq!(lex_token(&tokens[0]), Token::Int(IntLit {
            value: 100,
            base: IntBase::Decimal,
            suffix: IntSuffix { unsigned: true, long: LongKind::LongLong },
        }));
        assert_all_eq!(lex_tokens(&tokens));
    }
}

#[cfg(test)]
pub mod test_valid_char {
    use criterion_to_rust::lexer::token::*;
    use parameterized::parameterized;
    use crate::utils::lex_token;

    #[parameterized{base = {
        "'a'", "'B'", "'\\n'", "'\\t'", "'\\''", "'\\\\'", "'\\x41'", "'\\u1F60'",
        "'\\U0001F600'", "'\\0'", "'\\r'", "'\\v'", "'\\f'", "'\\b'", "'\\a'",
        "'\\x7F'", "'\\x00'", "'\\xFF'", "'\\U0010FFFF'", "'\\10'", "'\\100'"
    }, char_value = {
        'a', 'B', '\n', '\t', '\'', '\\', 'A', '\u{1F60}',
        '\u{1F600}', '\0', '\r', '\x0B', '\x0C', '\x08', '\x07',
        '\x7F', '\0', 0xFFu8 as char, '\u{10FFFF}', '\x08', '\x40'
    }, name = {
        "lowercase_a", "uppercase_B", "newline", "tab", "single_quote", "backslash", "hexadecimal_41", "unicode_1F60",
        "unicode_1F600", "null", "carriage_return", "vertical_tab", "form_feed", "backspace", "alert",
        "delete", "null", "max_ascii", "unicode_10FFFF", "octal_two_digit", "octal_three_digit"
    }}]
    fn tests(base : &str, char_value : char, name : &str) {
        let token = lex_token(base);
        assert_eq!(token, Token::CharLit(
            char_value,
        ), "Test {} failed for source: {}", name, base);
    }
}

#[cfg(test)]
pub mod test_invalid_char {
    use parameterized::parameterized;
    use criterion_to_rust::lexer::errors::{LexError, LexErrorKind};
    use criterion_to_rust::lexer::token::lex;

    #[parameterized{base = {
        "'\\\\n'", "'\\xFFFFFF'", "'\\u00001F60'",
    }, expected_errors = {
        LexErrorKind::BadCharLiteral { text : "\\\\n".to_string() },
        LexErrorKind::InvalidCodePoint { value : 16777215 },
        LexErrorKind::BadCharLiteral { text : "\\u00001F60".to_string() },
    }, name = {
        "invalid_escape",
        "hexadecimal_overflow",
        "small_unicode_8_digits"
    }}]
    fn tests(base : &str, expected_errors : LexErrorKind, name : &str) {
        println!("{}", lex(base).map_err(|e| e.to_string()).err().unwrap());
        assert_eq!(lex(base).map_err(|mut e| {e.span = None; e}), Err(LexError::new(expected_errors)), "Test {} failed for source: {}", name, base);
    }
}

#[cfg(test)]
pub mod test_valid_float {
    use criterion_to_rust::lexer::token::*;
    use parameterized::parameterized;
    use criterion_to_rust::literals::{FloatLit, FloatSuffix};
    use crate::assert_all_eq;
    use super::utils::*;

    #[parameterized(base = {
        "0.0", ".0", "0.", "0e0", "0E0", "0e+0", "0E+0", "0e-0", "0x0p0", "0X0P0"
    }, float_value = {
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
    }, name = {
        "decimal_point", "decimal_point_no_leading", "decimal_point_no_trailing",
        "exponent_lower", "exponent_upper", "exponent_lower_plus", "exponent_upper_plus",
        "exponent_lower_minus", "hexadecimal_lower", "hexadecimal_upper"
    })]
    fn test_zero(base : &str, float_value : f64, name : &str) {
        let token = lex_token(base);
        assert_eq!(token, Token::Float(FloatLit {
            value: float_value,
            suffix: FloatSuffix::Double,
        }), "Test {} failed for source: {}", name, base);
    }

    #[parameterized(base = {
        "1.", "3.14", ".71828", "6.022e23", "1.602E-19", "0x1.8p1", "0X1.8P1"
    }, float_value = {
        1.0, 3.14, 0.71828, 6.022e23, 1.602e-19, 3.0, 3.0
    }, name = {
        "decimal_point", "pi", "euler", "vogadro", "electron_charge", "hexadecimal_lower", "hexadecimal_upper"
    })]
    fn test_positive(base : &str, float_value : f64, name : &str) {
        let token = lex_token(base);
        assert_eq!(token, Token::Float(FloatLit {
            value: float_value,
            suffix: FloatSuffix::Double,
        }), "Test {} failed for source: {}", name, base);
    }

    #[parameterized(base = {
        "1.", "3.14", ".71828", "6.022e23", "1.602E-19", "0x1.8p1", "0X1.8P1"
    }, float_value = {
        1.0, 3.14, 0.71828, 6.022e23, 1.602e-19, 3.0, 3.0
    }, name = {
        "decimal_point", "pi", "euler", "vogadro", "electron_charge", "hexadecimal_lower", "hexadecimal_upper"
    })]
    fn test_f_suffix(base : &str, float_value : f64, name : &str) {
        let sources = build_float_suffix(base, FloatSuffix::Float);
        let tokens = lex_tokens(&sources);
        assert_eq!(tokens[0], Token::Float(FloatLit {
            value: float_value,
            suffix: FloatSuffix::Float,
        }), "Test {} failed for source: {}", name, sources[0]);
        assert_all_eq!(tokens);
    }

    #[parameterized(base = {
        "1.", "3.14", ".71828", "6.022e23", "1.602E-19", "0x1.8p1", "0X1.8P1"
    }, float_value = {
        1.0, 3.14, 0.71828, 6.022e23, 1.602e-19, 3.0, 3.0
    }, name = {
        "decimal_point", "pi", "euler", "vogadro", "electron_charge", "hexadecimal_lower", "hexadecimal_upper"
    })]
    fn test_l_suffix(base : &str, float_value : f64, name : &str) {
        let sources = build_float_suffix(base, FloatSuffix::LongDouble);
        let tokens = lex_tokens(&sources);
        assert_eq!(tokens[0], Token::Float(FloatLit {
            value: float_value,
            suffix: FloatSuffix::LongDouble,
        }), "Test {} failed for source: {}", name, sources[0]);
        assert_all_eq!(tokens);
    }
}

pub mod test_valid_string {
    use criterion_to_rust::lexer::token::*;
    use parameterized::parameterized;
    use criterion_to_rust::literals::{StringLit, StringPrefix};
    use crate::utils::lex_token;

    #[parameterized(base = {
        "\"Hello, World!\"", "\"\"", "\"\\n\"", "\"\\t\"", "\"\\\"\"", "\"\\\\\"", "\"caf\\u00e9\\t\\x41B\\0\\101\\\\\\\"\\nend\\U000000e9\""
    }, string_value = {
        "Hello, World!", "", "\n", "\t", "\"", "\\", "caf\u{e9}\t\u{41b}\u{0}A\\\"\nend\u{e9}"
    }, name = {
        "hello_world", "empty_string", "newline", "tab", "double_quote", "backslash", "complex_string"
    })]
    fn tests(base : &str, string_value : &str, name : &str) {
        let token = lex_token(base);
        assert_eq!(token, Token::StringLit( StringLit {
            value : string_value.to_string(),
            prefix : StringPrefix::None,
        }
        ), "Test {} failed for source: {}", name, base);
    }

    #[parameterized(base = {
        "u8\"Hello, World!\"", "u\"Hello, World!\"", "U\"Hello, World!\"", "L\"Hello, World!\""
    }, string_prefix = {
        StringPrefix::Utf8,
        StringPrefix::Utf16,
        StringPrefix::Utf32,
        StringPrefix::Wide
    }, name = {
        "u8_prefix", "u_prefix", "U_prefix", "L_prefix"
    })]
    fn test_string_prefix(base : &str, string_prefix : StringPrefix, name : &str) {
        let token = lex_token(base);
        assert_eq!(token, Token::StringLit( StringLit {
            value : "Hello, World!".to_string(),
            prefix : string_prefix,
        }
        ), "Test {} failed for source: {}", name, base);
    }
}