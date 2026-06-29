use logos::Logos;
use criterion_to_rust::lexer::token::{SpannedToken, Token};
use criterion_to_rust::literals::{FloatSuffix, LongKind};

fn build_int_suffix(base : &str, unsigned : bool, long_kind : LongKind) -> Vec<String> {
    let mut literals = Vec::new();
    for u in if unsigned { vec!["u", "U"] } else { vec![""] } {
        for l in match long_kind {
            LongKind::None => vec![""],
            LongKind::Long => vec!["l", "L"],
            LongKind::LongLong => vec!["ll", "LL", "lL", "Ll"],
        } {
            let suffix = format!("{}{}", u, l);
            let reversed = format!("{}{}", l, u);
            literals.push(format!("{}{}", base, suffix));
            literals.push(format!("{}{}", base, reversed));
        }
    }
    literals
}

fn build_float_suffix(base : &str, suffix : FloatSuffix) -> Vec<String> {
    let mut literals = Vec::new();
    match suffix {
        FloatSuffix::Double => {
            literals.push(format!("{}", base));
        }
        FloatSuffix::Float => {
            literals.push(format!("{}f", base));
            literals.push(format!("{}F", base));
        }
        FloatSuffix::LongDouble => {
            literals.push(format!("{}l", base));
            literals.push(format!("{}L", base));
        }
    }
    literals
}

fn lex_token(src: &str) -> Token {
    let mut lex = Token::lexer(src).spanned();
    let (result, range) = lex.next().expect("No token found");
    match result {
        Ok(token) => token,
        Err(()) => {
            panic!(
                "lex error: unrecognized input at bytes {}..{} ({:?})",
                range.start,
                range.end,
                &src[range.clone()]
            );
        }
    }
}

fn lex_tokens(vec: &Vec<String>) -> Vec<Token> {
    vec.into_iter().map(|s| lex_token(&s)).collect()
}

#[macro_export]
macro_rules! assert_all_eq {
        ($vec:expr) => {
            let first = &$vec[0];
            for (i, item) in $vec.iter().enumerate() {
                assert_eq!(first, item, "Item at index {} does not match the first item.", i);
            }
        };
    }

#[cfg(test)]
pub mod test_lex_int_literals {
    use criterion_to_rust::lexer::token::*;
    use parameterized::parameterized;
    use criterion_to_rust::literals::{IntBase, IntLit, IntSuffix, LongKind};
    use super::{build_int_suffix, lex_token, lex_tokens};

    #[parameterized(base = {
        "0", "00", "0b0", "0B0", "0x0", "0X0"
    }, int_base = {
        IntBase::Decimal,
        IntBase::Octal,
        IntBase::Binary,
        IntBase::Binary,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal
    }, name = {
        "decimal",
        "octal",
        "binary_lower",
        "binary_upper",
        "hex_lower",
        "hex_upper"
    })]
    fn test_zero(base : &str, int_base : IntBase, name : &str) {
        let token = lex_token(base);
        assert_eq!(token, Token::Int(IntLit {
            value: 0,
            base : int_base,
            suffix: IntSuffix { unsigned: false, long: LongKind::None },
        }), "Failed on test case: {}", name);
    }

    #[parameterized(base = {
        "12345", "0644", "0b1010", "0B1010", "0x1A3F", "0X1A3F", "0x1a3f", "0X1a3f"
    }, int_base = {
        IntBase::Decimal,
        IntBase::Octal,
        IntBase::Binary,
        IntBase::Binary,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal
    }, int_value = {
        12345,
        420,
        10,
        10,
        6719,
        6719,
        6719,
        6719
    }, name = {
        "decimal",
        "octal",
        "binary_lower",
        "binary_upper",
        "hex_lower_1",
        "hex_upper_2",
        "hex_lower_2",
        "hex_upper_1"
    })]
    fn test_positive_integer(base : &str, int_base : IntBase, int_value : u64, name : &str) {
        let token = lex_token(base);
        assert_eq!(token, Token::Int(IntLit {
            value: int_value,
            base: int_base,
            suffix: IntSuffix { unsigned: false, long: LongKind::None },
        }), "Failed on test case: {}", name);
    }

    #[parameterized(base = {
        "12345", "0644", "0b1010", "0B1010", "0x1A3F", "0X1A3F", "0x1a3f", "0X1a3f"
    }, int_base = {
        IntBase::Decimal,
        IntBase::Octal,
        IntBase::Binary,
        IntBase::Binary,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal
    }, int_value = {
        12345,
        420,
        10,
        10,
        6719,
        6719,
        6719,
        6719
    }, name = {
        "decimal",
        "octal",
        "binary_lower",
        "binary_upper",
        "hex_lower_1",
        "hex_upper_2",
        "hex_lower_2",
        "hex_upper_1"
    })]
    fn test_integer_u_suffix(base : &str, int_base : IntBase, int_value : u64, name : &str) {
        let tokens = build_int_suffix(base, true, LongKind::None);
        assert_eq!(lex_token(&tokens[0]), Token::Int(IntLit {
            value: int_value,
            base: int_base,
            suffix: IntSuffix { unsigned: true, long: LongKind::None },
        }), "Failed on test case: {}", name);
        assert_all_eq!(lex_tokens(&tokens));
    }

    #[parameterized(base = {
        "12345", "0644", "0b1010", "0B1010", "0x1A3F", "0X1A3F", "0x1a3f", "0X1a3f"
    }, int_base = {
        IntBase::Decimal,
        IntBase::Octal,
        IntBase::Binary,
        IntBase::Binary,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal
    }, int_value = {
        12345,
        420,
        10,
        10,
        6719,
        6719,
        6719,
        6719
    }, name = {
        "decimal",
        "octal",
        "binary_lower",
        "binary_upper",
        "hex_lower_1",
        "hex_upper_2",
        "hex_lower_2",
        "hex_upper_1"
    })]
    fn test_integer_l_suffix() {
        let tokens = build_int_suffix("100", false, LongKind::Long);
        assert_eq!(lex_token(&tokens[0]), Token::Int(IntLit {
            value: 100,
            base: IntBase::Decimal,
            suffix: IntSuffix { unsigned: false, long: LongKind::Long },
        }));
        assert_all_eq!(lex_tokens(&tokens));
    }

    #[parameterized(base = {
        "12345", "0644", "0b1010", "0B1010", "0x1A3F", "0X1A3F", "0x1a3f", "0X1a3f"
    }, int_base = {
        IntBase::Decimal,
        IntBase::Octal,
        IntBase::Binary,
        IntBase::Binary,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal
    }, int_value = {
        12345,
        420,
        10,
        10,
        6719,
        6719,
        6719,
        6719
    }, name = {
        "decimal",
        "octal",
        "binary_lower",
        "binary_upper",
        "hex_lower_1",
        "hex_upper_2",
        "hex_lower_2",
        "hex_upper_1"
    })]
    fn test_integer_ll_suffix() {
        let tokens = build_int_suffix("100", false, LongKind::LongLong);
        assert_eq!(lex_token(&tokens[0]), Token::Int(IntLit {
            value: 100,
            base: IntBase::Decimal,
            suffix: IntSuffix { unsigned: false, long: LongKind::LongLong },
        }));
        assert_all_eq!(lex_tokens(&tokens));
    }

    #[parameterized(base = {
        "12345", "0644", "0b1010", "0B1010", "0x1A3F", "0X1A3F", "0x1a3f", "0X1a3f"
    }, int_base = {
        IntBase::Decimal,
        IntBase::Octal,
        IntBase::Binary,
        IntBase::Binary,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal
    }, int_value = {
        12345,
        420,
        10,
        10,
        6719,
        6719,
        6719,
        6719
    }, name = {
        "decimal",
        "octal",
        "binary_lower",
        "binary_upper",
        "hex_lower_1",
        "hex_upper_2",
        "hex_lower_2",
        "hex_upper_1"
    })]
    fn test_integer_ul_suffix() {
        let tokens = build_int_suffix("100", true, LongKind::Long);
        assert_eq!(lex_token(&tokens[0]), Token::Int(IntLit {
            value: 100,
            base: IntBase::Decimal,
            suffix: IntSuffix { unsigned: true, long: LongKind::Long },
        }));
        assert_all_eq!(lex_tokens(&tokens));
    }

    #[parameterized(base = {
        "12345", "0644", "0b1010", "0B1010", "0x1A3F", "0X1A3F", "0x1a3f", "0X1a3f"
    }, int_base = {
        IntBase::Decimal,
        IntBase::Octal,
        IntBase::Binary,
        IntBase::Binary,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal,
        IntBase::Hexadecimal
    }, int_value = {
        12345,
        420,
        10,
        10,
        6719,
        6719,
        6719,
        6719
    }, name = {
        "decimal",
        "octal",
        "binary_lower",
        "binary_upper",
        "hex_lower_1",
        "hex_upper_2",
        "hex_lower_2",
        "hex_upper_1"
    })]
    fn test_integer_ull_suffix() {
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
pub mod test_char_literals {
    use parameterized::parameterized;
}

#[cfg(test)]
pub mod test_float_literals {
    use criterion_to_rust::lexer::token::*;
    use parameterized::parameterized;
    use criterion_to_rust::literals::{FloatLit, FloatSuffix};
    use super::{lex_token, lex_tokens};

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
        }), "Failed on test case: {}", name);
    }

    #[parameterized(base = {
        "1.0", "3.14", "2.71828", "6.022e23", "1.602E-19", "0x1.8p1", "0X1.8P1"
    }, float_value = {
        1.0, 3.14, 2.71828, 6.022e23, 1.602e-19, 3.0, 3.0
    }, name = {
        "decimal_point", "pi", "euler", "avogadro", "electron_charge", "hexadecimal_lower", "hexadecimal_upper"
    })]
    fn test_positive_float(base : &str, float_value : f64, name : &str) {
        let token = lex_token(base);
        assert_eq!(token, Token::Float(FloatLit {
            value: float_value,
            suffix: FloatSuffix::Double,
        }), "Failed on test case: {}", name);
    }
}
