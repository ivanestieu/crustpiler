// -----------------------------------------------------------------------------
// OPERATORS
// -----------------------------------------------------------------------------

use crate::ast::operators::{AssignOp, BinaryOp, PostfixOp, UnaryOp};
use crate::output::output::Output;

impl Output for UnaryOp {
    fn as_c_repr(&self) -> String {
        match self {
            UnaryOp::Minus => "-",
            UnaryOp::Plus => "+",
            UnaryOp::Not => "!",
            UnaryOp::BitNot => "~",
            UnaryOp::Deref => "*",
            UnaryOp::AddrOf => "&",
            UnaryOp::PreInc => "++",
            UnaryOp::PreDec => "--",
        }
        .to_string()
    }

    fn as_rust_repr(&self) -> String {
        match self {
            UnaryOp::Minus => "-",
            UnaryOp::Plus => "+",
            UnaryOp::Not => "!",
            UnaryOp::BitNot => "~",
            UnaryOp::Deref => "*",
            UnaryOp::AddrOf => "&",
            UnaryOp::PreInc => "/* Rust has no prefix increment operator */",
            UnaryOp::PreDec => "/* Rust has no prefix decrement operator */",
        }
        .to_string()
    }
}

impl Output for PostfixOp {
    fn as_c_repr(&self) -> String {
        match self {
            PostfixOp::PostInc => "++",
            PostfixOp::PostDec => "--",
        }
        .to_string()
    }

    fn as_rust_repr(&self) -> String {
        String::from("/* Rust has no postfix increment operator */")
    }
}

impl Output for BinaryOp {
    fn as_c_repr(&self) -> String {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
            BinaryOp::Shl => "<<",
            BinaryOp::Shr => ">>",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        }
        .to_string()
    }
    fn as_rust_repr(&self) -> String {
        self.as_c_repr()
    }
}

impl Output for AssignOp {
    fn as_c_repr(&self) -> String {
        match self {
            AssignOp::Assign => "=",
            AssignOp::AddAssign => "+=",
            AssignOp::SubAssign => "-=",
            AssignOp::MulAssign => "*=",
            AssignOp::DivAssign => "/=",
            AssignOp::ModAssign => "%=",
            AssignOp::BitAndAssign => "&=",
            AssignOp::BitOrAssign => "|=",
            AssignOp::BitXorAssign => "^=",
            AssignOp::ShlAssign => "<<=",
            AssignOp::ShrAssign => ">>=",
        }
        .to_string()
    }

    fn as_rust_repr(&self) -> String {
        self.as_c_repr()
    }
}
