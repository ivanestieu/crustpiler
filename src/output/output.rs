// =============================================================================
// output.rs — AST → Rust source.
// Walks the full-AST Decl, outputting one `let` per InitDeclarator.
//   int x = 1;   →   let x: i32 = 1;
// =============================================================================

use crate::ast::ast::*;
use crate::ast::declarations::{
    Decl, Declaration, Designator, InitDeclarator, InitItem, Initializer,
};
use crate::ast::declarator::Declarator;
use crate::ast::operators::{AssignOp, BinaryOp, PostfixOp, UnaryOp};
use crate::ast::types::{ArithType, BaseType, Sign, SizeSpec, TypeExpr, TypeName, TypeSpec};
use itertools::Itertools;

pub trait Output {
    fn as_c_repr(&self) -> String;
    fn as_rust_repr(&self) -> String;
}

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

impl Output for Designator {
    fn as_c_repr(&self) -> String {
        todo!()
    }

    fn as_rust_repr(&self) -> String {
        match self {
            Designator::Index(e) => format!("[{}]", e.as_rust_repr()),
            Designator::Field(f) => format!(".{}", f),
        }
    }
}

impl Output for InitItem {
    fn as_c_repr(&self) -> String {
        todo!()
    }

    fn as_rust_repr(&self) -> String {
        format!(
            "{} = {}",
            self.designators.iter().map(|d| d.as_rust_repr()).join(""),
            self.value.as_rust_repr()
        )
    }
}

impl Output for Initializer {
    fn as_c_repr(&self) -> String {
        todo!()
    }

    fn as_rust_repr(&self) -> String {
        match self {
            Initializer::Expr(e) => e.as_rust_repr(),
            Initializer::List(list) => format!(
                "{{{}}}",
                list.iter().map(|init| init.as_rust_repr()).join(", ")
            ),
        }
    }
}

impl Output for Decl {
    fn as_c_repr(&self) -> String {
        format!(
            "{} {}",
            self.specifiers.as_c_repr(),
            self.declarators
                .iter()
                .map(|d| d.as_c_repr())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
    fn as_rust_repr(&self) -> String {
        let spec = self.specifiers.as_rust_repr();
        let mut decl: Vec<String> = vec![];
        let print_vec = |v: &Vec<String>| {
            format!("{}", {
                if v.len() == 1 {
                    format!("{}", v[0])
                } else {
                    format!("({})", v.join(", "))
                }
            })
        };
        let mut declarators_iter = self.declarators.iter().peekable();
        while declarators_iter.peek().is_some() {
            let mut declarators: Vec<String> = vec![];
            let mut specs: Vec<String> = vec![];
            declarators_iter
                .peeking_take_while(|d| d.init.is_none())
                .for_each(|d| {
                    declarators.push(d.declarator.ident().unwrap().to_string());
                    specs.push(spec.clone());
                });
            if !declarators.is_empty() {
                decl.push(format!(
                    "let {} : {};",
                    print_vec(&declarators),
                    print_vec(&specs)
                ));
            }
            specs.clear();
            declarators.clear();
            let mut inits: Vec<String> = vec![];
            declarators_iter
                .peeking_take_while(|d| d.init.is_some())
                .for_each(|d| {
                    declarators.push(d.declarator.ident().unwrap().to_string());
                    specs.push(spec.clone());
                    inits.push(d.init.as_ref().unwrap().as_rust_repr());
                });
            if !declarators.is_empty() {
                decl.push(format!(
                    "let {} : {} = {};",
                    print_vec(&declarators),
                    print_vec(&specs),
                    print_vec(&inits)
                ));
            }
        }
        decl.join("\n")
    }
}

impl Output for InitDeclarator {
    fn as_c_repr(&self) -> String {
        let name = self.declarator.ident().unwrap_or("_");
        match &self.init {
            Some(Initializer::Expr(e)) => format!("{} = {};", name, e.as_rust_repr()),
            None => format!("{};", name),
            _ => format!("{};", name), // Placeholder for other initializers
        }
    }

    fn as_rust_repr(&self) -> String {
        let name = self.declarator.ident().unwrap_or("_");
        match &self.init {
            Some(Initializer::Expr(e)) => format!("{} = {};", name, e.as_rust_repr()),
            None => format!("let {};", name),
            _ => format!("let {};", name), // Placeholder for other initializers
        }
    }
}

impl Output for ArithType {
    fn as_c_repr(&self) -> String {
        todo!()
    }

    fn as_rust_repr(&self) -> String {
        match self {
            ArithType {
                base: BaseType::Int,
                sign: Some(Sign::Unsigned),
                size: SizeSpec::Short,
                ..
            } => "u16",
            ArithType {
                base: BaseType::Int,
                sign: Some(Sign::Unsigned),
                size: SizeSpec::None,
                ..
            } => "u32",
            ArithType {
                base: BaseType::Int,
                sign: Some(Sign::Unsigned),
                size: SizeSpec::Long,
                ..
            } => "u64",
            ArithType {
                base: BaseType::Int,
                sign: Some(Sign::Unsigned),
                size: SizeSpec::LongLong,
                ..
            } => "u128",
            ArithType {
                base: BaseType::Int,
                size: SizeSpec::Short,
                ..
            } => "i16",
            ArithType {
                base: BaseType::Int,
                size: SizeSpec::None,
                ..
            } => "i32",
            ArithType {
                base: BaseType::Int,
                size: SizeSpec::Long,
                ..
            } => "i64",
            ArithType {
                base: BaseType::Int,
                size: SizeSpec::LongLong,
                ..
            } => "i128",
            ArithType {
                base: BaseType::Float,
                ..
            } => "f32",
            ArithType {
                base: BaseType::Double,
                size: SizeSpec::Long,
                ..
            } => "/* f80 (long double) isn't defined in rust */ f64",
            ArithType {
                base: BaseType::Double,
                ..
            } => "f64",
            ArithType {
                base: BaseType::Char,
                sign: Some(Sign::Unsigned),
                ..
            } => "/* uchar (unsigned char) isn't defined in rust */ u32",
            ArithType {
                base: BaseType::Char,
                ..
            } => "char",
        }
        .to_string()
    }
}

impl Output for TypeSpec {
    fn as_c_repr(&self) -> String {
        match self {
            TypeSpec::Arithmetic(arith) => arith.as_c_repr(),
            TypeSpec::Void => "void".to_string(),
            TypeSpec::Bool => "_Bool".to_string(),
            _ => "/* type */".to_string(), // Placeholder for other types
        }
    }

    fn as_rust_repr(&self) -> String {
        match self {
            TypeSpec::Arithmetic(arith) => arith.as_rust_repr(),
            TypeSpec::Void => "()".to_string(),
            TypeSpec::Bool => "_Bool".to_string(),
            TypeSpec::Named(name) => name.clone(),
            _ => "unknown".to_string(), // Placeholder for other types
        }
    }
}

impl Output for TypeExpr {
    fn as_c_repr(&self) -> String {
        self.type_spec.as_c_repr()
    }

    fn as_rust_repr(&self) -> String {
        self.type_spec.as_rust_repr()
    }
}

// Walk an abstract declarator inside-out, wrapping `base` in each derived layer.
// Abstract / Ident is the leaf (no wrapping). Pointer → `*mut T` (or `*const T`
// if const-qualified). Array/Function are rendered best-effort.
fn wrap_declarator_rust(d: &Declarator, base: String) -> String {
    match d {
        Declarator::Abstract | Declarator::Ident(_) => base,
        Declarator::Pointer { inner, qualifiers } => {
            // choose *const vs *mut from the pointer's own qualifiers
            let is_const = qualifiers
                .iter()
                .any(|q| format!("{:?}", q).contains("Const"));
            let ptr = if is_const {
                format!("*const {}", wrap_declarator_rust(inner, base))
            } else {
                format!("*mut {}", wrap_declarator_rust(inner, base))
            };
            ptr
        }
        Declarator::Array { inner, size, .. } => {
            use crate::ast::declarator::ArraySize;
            let elem = wrap_declarator_rust(inner, base);
            match size {
                ArraySize::Fixed(n) => format!("[{}; {}]", elem, n.as_rust_repr()),
                _ => format!("*mut {} /* unsized array */", elem),
            }
        }
        Declarator::Function { inner, .. } => {
            // function type in a type-name → a fn pointer; best-effort
            format!(
                "fn() -> {} /* fn type */",
                wrap_declarator_rust(inner, base)
            )
        }
    }
}

impl Output for TypeName {
    fn as_c_repr(&self) -> String {
        // base spec, then the declarator's C spelling is best-effort here
        todo!()
    }

    fn as_rust_repr(&self) -> String {
        let base = self.type_expr.as_rust_repr();
        wrap_declarator_rust(&self.derived, base)
    }
}

impl Output for Expr {
    fn as_c_repr(&self) -> String {
        todo!()
    }

    fn as_rust_repr(&self) -> String {
        match self {
            // Literals
            Expr::IntLit(lit) => lit.value.to_string(),
            Expr::FloatLit(lit) => lit.value.to_string(),
            Expr::StringLit(lit) => lit.value.escape_default().to_string(),
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
pub fn output_translation_unit(p0: &Vec<Item>) -> String {
    p0.iter()
        .filter_map(|item| match item {
            Item::Declaration(decl) => match decl {
                Declaration::Normal(d) => Some(d.node.as_rust_repr()),
                Declaration::StaticAssert(_) => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}
