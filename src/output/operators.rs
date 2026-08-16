// -----------------------------------------------------------------------------
// OPERATORS
// -----------------------------------------------------------------------------

use crate::lexer::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Minus,  // -
    Plus,   // +
    Not,    // !
    BitNot, // ~
    Deref,  // *
    AddrOf, // &
    PreInc, // ++x
    PreDec, // --x
}

#[derive(Debug, Clone, PartialEq)]
pub enum PostfixOp {
    PostInc, // x++
    PostDec, // x--
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Assign,       // =
    AddAssign,    // +=
    SubAssign,    // -=
    MulAssign,    // *=
    DivAssign,    // /=
    ModAssign,    // %=
    BitAndAssign, // &=
    BitOrAssign,  // |=
    BitXorAssign, // ^=
    ShlAssign,    // <<=
    ShrAssign,    // >>=
}

// Bridge token::Token -> ast::BinaryOp (kept as separate types for module decoupling)
pub trait AsBinaryOp {
    fn as_binary_op(&self) -> Result<BinaryOp, String>;
}
impl AsBinaryOp for crate::lexer::token::Token {
    fn as_binary_op(&self) -> Result<BinaryOp, String> {
        match self {
            // Multiplicative
            Token::Star => Ok(BinaryOp::Mul),
            Token::Slash => Ok(BinaryOp::Div),
            Token::Percentage => Ok(BinaryOp::Mod),
            // Additive
            Token::Plus => Ok(BinaryOp::Add),
            Token::Minus => Ok(BinaryOp::Sub),
            // Shift
            Token::LeftOp => Ok(BinaryOp::Shl),
            Token::RightOp => Ok(BinaryOp::Shr),
            // Relational
            Token::LessThan => Ok(BinaryOp::Lt),
            Token::LeOp => Ok(BinaryOp::Le),
            Token::GreaterThan => Ok(BinaryOp::Gt),
            Token::GeOp => Ok(BinaryOp::Ge),
            // Equality
            Token::EqOp => Ok(BinaryOp::Eq),
            Token::NeOp => Ok(BinaryOp::Ne),
            // Bitwise
            Token::Ampersand => Ok(BinaryOp::BitAnd),
            Token::Caret => Ok(BinaryOp::BitXor),
            Token::Pipe => Ok(BinaryOp::BitOr),
            // Logical
            Token::AndOp => Ok(BinaryOp::And),
            Token::OrOp => Ok(BinaryOp::Or),
            other => Err(format!("Expected binary operator, found {:?}", other)),
        }
    }
}

pub trait TraitBinaryOp {
    fn binding_power(&self) -> (usize, usize);
}

impl TraitBinaryOp for BinaryOp {
    fn binding_power(&self) -> (usize, usize) {
        match self {
            BinaryOp::Or => (1, 2),
            BinaryOp::And => (3, 4),
            BinaryOp::BitOr => (5, 6),
            BinaryOp::BitXor => (7, 8),
            BinaryOp::BitAnd => (9, 10),
            BinaryOp::Eq | BinaryOp::Ne => (11, 12),
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => (13, 14),
            BinaryOp::Shl | BinaryOp::Shr => (15, 16),
            BinaryOp::Add | BinaryOp::Sub => (17, 18),
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => (19, 20),
        }
    }
}

// Bridge token::Token -> ast::BinaryOp (kept as separate types for module decoupling)
pub trait AsAssignOp {
    fn as_assign_op(&self) -> Result<AssignOp, String>;
}

impl AsAssignOp for crate::lexer::token::Token {
    fn as_assign_op(&self) -> Result<AssignOp, String> {
        match self {
            Token::Equals => Ok(AssignOp::Assign),
            Token::AddAssign => Ok(AssignOp::AddAssign),
            Token::SubAssign => Ok(AssignOp::SubAssign),
            Token::MulAssign => Ok(AssignOp::MulAssign),
            Token::DivAssign => Ok(AssignOp::DivAssign),
            Token::ModAssign => Ok(AssignOp::ModAssign),
            Token::AndAssign => Ok(AssignOp::BitAndAssign),
            Token::OrAssign => Ok(AssignOp::BitOrAssign),
            Token::XorAssign => Ok(AssignOp::BitXorAssign),
            Token::LeftAssign => Ok(AssignOp::ShlAssign),
            Token::RightAssign => Ok(AssignOp::ShrAssign),
            other => Err(format!("Expected assign, found {:?}", other)),
        }
    }
}

pub trait AsUnaryOp {
    fn as_unary_op(&self) -> Result<UnaryOp, String>;
}

impl AsUnaryOp for crate::lexer::token::Token {
    fn as_unary_op(&self) -> Result<UnaryOp, String> {
        match self {
            Token::IncOp => Ok(UnaryOp::PreInc),
            Token::DecOp => Ok(UnaryOp::PreDec),
            Token::Minus => Ok(UnaryOp::Minus),
            Token::Plus => Ok(UnaryOp::Plus),
            Token::ExclamationMark => Ok(UnaryOp::Not),
            Token::Tilde => Ok(UnaryOp::BitNot),
            Token::Star => Ok(UnaryOp::Deref),
            Token::Ampersand => Ok(UnaryOp::AddrOf),
            _ => Err(format!("Expected unary operator, found {:?}", self)),
        }
    }
}
