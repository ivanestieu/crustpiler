// -----------------------------------------------------------------------------
// STATEMENTS
// -----------------------------------------------------------------------------

use crate::ast::span::Spanned;
use crate::ast::statements::{BlockItem, ForInit, Stmt};
use crate::output::output::Output;
use itertools::Itertools;

impl Output for Stmt {
    fn as_c_repr(&self) -> String {
        match self {
            Stmt::Expr(expr) => format!("{};", expr.node.as_c_repr()),
            Stmt::Empty => String::from(";"),
            Stmt::Label(ident, stmt) => format!("{}:\n{}", ident, stmt.node.as_c_repr()),
            Stmt::Case(expr, stmt) => {
                format!("case {}:\n{}", expr.node.as_c_repr(), stmt.node.as_c_repr())
            }
            Stmt::Default(stmt) => format!("default:\n{}", stmt.node.as_c_repr()),
            Stmt::Block(block_items) => block_items.iter().map(|b| b.as_c_repr()).join("\n"),
            Stmt::If { cond, then, els } => {
                let mut if_ = format!(
                    "if ({})\n{{\n{}\n}}",
                    cond.node.as_c_repr(),
                    then.node.as_c_repr()
                );
                if let Some(els) = els {
                    if_ += &format!("\nelse\n{{\n{}\n}}", els.node.as_c_repr());
                };
                if_
            }
            Stmt::Switch { expr, body } => {
                format!(
                    "switch ({})\n{{\n{}\n}}",
                    expr.node.as_c_repr(),
                    body.node.as_c_repr()
                )
            }
            Stmt::While { cond, body } => {
                format!(
                    "while ({})\n{{\n{}\n}}",
                    cond.node.as_c_repr(),
                    body.node.as_c_repr()
                )
            }
            Stmt::DoWhile { body, cond } => {
                format!(
                    "do \n{{\n{}\n}} while ({});",
                    body.node.as_c_repr(),
                    cond.node.as_c_repr()
                )
            }
            Stmt::For {
                init,
                cond,
                step,
                body,
            } => {
                format!(
                    "for ({} {} {})\n{{\n{}\n}}",
                    init.as_c_repr(),
                    if cond.is_some() {
                        cond.as_ref().unwrap().node.as_c_repr()
                    } else {
                        String::from(";")
                    },
                    if step.is_some() {
                        step.as_ref().unwrap().node.as_c_repr()
                    } else {
                        String::from(";")
                    },
                    body.node.as_c_repr()
                )
            }
            Stmt::Return(Some(expr)) => format!("return {};", expr.node.as_c_repr()),
            Stmt::Return(_) => String::from("return;"),
            Stmt::Break => String::from("break;"),
            Stmt::Continue => String::from("continue;"),
            Stmt::Goto(ident) => format!("goto {};", ident),
        }
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}

impl Output for BlockItem {
    fn as_c_repr(&self) -> String {
        match self {
            BlockItem::Decl(declaration) => declaration.as_c_repr(),
            BlockItem::Stmt(Spanned { node: stmt, .. }) => stmt.as_c_repr(),
        }
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}

impl Output for ForInit {
    fn as_c_repr(&self) -> String {
        match self {
            ForInit::Empty => String::new(),
            ForInit::Expr(Spanned { node: expr, .. }) => format!("{};", expr.as_c_repr()),
            ForInit::Decl(declaration) => declaration.as_c_repr(),
        }
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}
