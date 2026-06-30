use logos::Logos;
use criterion_to_rust::lexer::token::Token;
use criterion_to_rust::literals::{FloatSuffix, LongKind};

fn build_int_suffix(base: &str, unsigned: bool, long_kind: LongKind) -> Vec<String> {
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

fn build_float_suffix(base: &str, suffix: FloatSuffix) -> Vec<String> {
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