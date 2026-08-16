use itertools::Itertools;
use crate::ast::ast::{Expr, GenericAssoc, Item};
use crate::output::output::Output;

// -----------------------------------------------------------------------------
// EXPRESSIONS
// All operators from C11;
// -----------------------------------------------------------------------------
impl Output for Expr {
    fn as_c_repr(&self) -> String {
        match self {
            // Literals
            Expr::IntLit(lit) => lit.value.to_string(),
            Expr::FloatLit(lit) => lit.value.to_string(),
            Expr::StringLit(lit) => format!("\"{}\"", lit.value.escape_debug().to_string()),
            Expr::FuncName(name) => name.clone(),
            Expr::CharLit(lit) => format!("'{}'", lit.escape_debug()),

            // Identifier
            Expr::Ident(name) => name.clone(),

            // Compound literal: (Type){init}
            Expr::CompoundLit { type_name, init} => format!("({}){{{}}}", type_name.as_c_repr(), init.iter().map(|i| i.as_c_repr()).join(", ")),
            // Unary prefix
            Expr::UnaryOp { op, operand } => {
                format!("{}{}", op.as_c_repr(), &operand.node.as_c_repr())
            }

            // Unary postfix — kept separate because precedence/associativity differ
            Expr::PostfixOp { op, operand } => {
                format!("{}{}", &operand.node.as_c_repr(), op.as_c_repr())
            }

            // Binary
            Expr::BinaryOp { lhs, op, rhs } => format!(
                "{} {} {}",
                &lhs.node.as_c_repr(),
                op.as_c_repr(),
                &rhs.node.as_c_repr()
            ),
            // Assignment (right-associative, lower precedence than most binary ops)
            Expr::Assign { op, lhs, rhs } => format!(
                "{} {} {}",
                &lhs.node.as_c_repr(),
                op.as_c_repr(),
                &rhs.node.as_c_repr()
            ),
            // Ternary
            Expr::Ternary {cond, then, els } => format!("{} ? {} : {}",
                cond.node.as_c_repr(),
                then.node.as_c_repr(),
                els.node.as_c_repr()
            ),
            // Function call
            Expr::Call { callee, args} => format!("{}({})",
                callee.node.as_c_repr(),
                args.iter().map(|arg| arg.node.as_c_repr()).join(", ")
            ),
            // Subscript: array[index]
            Expr::Index { array, index } => format!("{}[{}]",
                array.node.as_c_repr(),
                index.node.as_c_repr()
            ),
            // Member access: expr.field  or  expr->field
            Expr::Member { expr, field, arrow } => format!("{}{}{}",
                expr.node.as_c_repr(),
                if *arrow { "->" } else { "." },
                field
            ),
            // Cast: (Type)expr
            Expr::Cast { type_name, expr } => format!(
                "({}){}",
                &expr.node.as_c_repr(),
                type_name.as_c_repr()
            ),

            // sizeof
            Expr::SizeofExpr(e) => format!("sizeof {}", &e.node.as_c_repr()),
            Expr::SizeofType(ty) => format!("sizeof({})", ty.as_c_repr()),
            // _Alignof
            Expr::AlignofType(ty) => format!("_Alignof({})", ty.as_c_repr()),
            // _Generic
            Expr::Generic { controlling, associated } => format!("_Generic({}, {})",
                controlling.node.as_c_repr(),
                associated.iter().map(|a| a.as_c_repr()).join(", ")
            ),
            // Comma operator: a, b  (lowest precedence)
            Expr::Comma(left, right) => format!(
                "{}, {}",
                left.node.as_c_repr(),
                right.node.as_c_repr()
            ),
        }
    }

    fn as_rust_repr(&self) -> String {
        match self {
            // Literals
            Expr::IntLit(lit) => lit.value.to_string(),
            Expr::FloatLit(lit) => lit.value.to_string(),
            Expr::StringLit(lit) => lit.value.escape_debug().to_string(),
            Expr::FuncName(name) => name.clone(),
            Expr::CharLit(lit) => format!("'{}'", lit.escape_debug()),

            // Identifier
            Expr::Ident(name) => name.clone(),

            // Compound literal: (Type){init}
            // Unary prefix
            Expr::UnaryOp { op, operand } => {
                format!("{}{}", op.as_rust_repr(), &operand.node.as_rust_repr())
            }

            // Unary postfix — kept separate because precedence/associativity differ
            Expr::PostfixOp { op, operand } => {
                format!("{}{}", &operand.node.as_rust_repr(), op.as_rust_repr())
            }

            // Binary
            Expr::BinaryOp { lhs, op, rhs } => format!(
                "{} {} {}",
                &lhs.node.as_rust_repr(),
                op.as_rust_repr(),
                &rhs.node.as_rust_repr()
            ),
            // Assignment (right-associative, lower precedence than most binary ops)
            Expr::Assign { op, lhs, rhs } => format!(
                "{} {} {}",
                &lhs.node.as_rust_repr(),
                op.as_rust_repr(),
                &rhs.node.as_rust_repr()
            ),
            // Ternary
            // Function call
            // Subscript: array[index]
            // Member access: expr.field  or  expr->field
            // Cast: (Type)expr
            Expr::Cast { type_name, expr } => format!(
                "({} as {})",
                &expr.node.as_rust_repr(),
                type_name.as_rust_repr()
            ),

            // sizeof
            Expr::SizeofExpr(e) => format!("std::mem::size_of_val(&{})", &e.node.as_rust_repr()),
            // _Alignof
            // _Generic
            // Comma operator: a, b  (lowest precedence)
            Expr::Comma(left, right) => format!(
                "{}, {}",
                &left.node.as_rust_repr(),
                &right.node.as_rust_repr()
            ),
            _ => "/* expr */".to_string(), // Placeholder for other expressions
        }
    }
}

// -----------------------------------------------------------------------------
// TOP-LEVEL ITEMS
// -----------------------------------------------------------------------------

impl Output for Item {
    fn as_c_repr(&self) -> String {
        match self {
            Item::FunctionDef(definition) => definition.as_c_repr(),
            Item::Declaration(declaration) => declaration.as_c_repr(),
        }
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}

impl Output for GenericAssoc {
    fn as_c_repr(&self) -> String {
        if self.type_name.is_some() {
            format!(
                "{} {}",
                self.type_name.as_ref().unwrap().as_c_repr(),
                self.value.node.as_c_repr()
            )
        } else {
            self.value.node.as_c_repr()
        }
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}
