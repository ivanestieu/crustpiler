// -----------------------------------------------------------------------------
// CRITERION-SPECIFIC LAYER
// Sits on top of the C AST — recognised after parsing, not during.
// -----------------------------------------------------------------------------

use crate::ast::ast::{Expr, Item};
use crate::ast::span::{Span, Spanned};
use crate::ast::statements::Stmt;

/// A full parsed .c file containing Criterion tests

#[derive(Debug)]
pub struct CriterionFile {
    pub items: Vec<Item>, // non-tests C items (helpers, etc.)
    pub suites: Vec<CriterionSuite>,
}

#[derive(Debug)]
pub struct CriterionSuite {
    pub name: String,
    pub timeout: Option<f64>,
    pub tests: Vec<CriterionTest>,
    pub span: Span,
}

#[derive(Debug)]
pub struct CriterionTest {
    pub suite: String,
    pub name: String,
    pub disabled: bool,
    pub timeout: Option<f64>,
    pub body: Vec<CriterionBodyItem>,
    pub span: Span,
}

/// Each item in a tests body is either a Criterion assertion or plain C
#[derive(Debug)]
pub enum CriterionBodyItem {
    Assertion(CriterionAssertion),
    Other(Spanned<Stmt>), // kept for context / manual review
}

#[derive(Debug)]
pub struct CriterionAssertion {
    pub kind: AssertKind,
    pub fatal: bool,                    // cr_assert = fatal, cr_expect = not
    pub args: Vec<Spanned<Expr>>,       // fully parsed, not raw strings
    pub message: Option<Spanned<Expr>>, // last string arg if present
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssertKind {
    // Boolean
    Assert, // cr_assert(expr)
    // Equality
    Eq, // cr_assert_eq(a, b)
    Ne, // cr_assert_neq(a, b)
    // Ordering
    Lt, // cr_assert_lt(a, b)
    Le, // cr_assert_leq(a, b)
    Gt, // cr_assert_gt(a, b)
    Ge, // cr_assert_geq(a, b)
    // Pointer
    Null,    // cr_assert_null(p)
    NotNull, // cr_assert_not_null(p)
    // Float
    FloatEq, // cr_assert_float_eq(a, b, eps)
    FloatNe, // cr_assert_float_neq(a, b, eps)
    // String
    StrEq, // cr_assert_str_eq(a, b)
    StrNe, // cr_assert_str_neq(a, b)
    StrLt, // cr_assert_str_lt(a, b)
    StrLe, // cr_assert_str_leq(a, b)
    StrGt, // cr_assert_str_gt(a, b)
    StrGe, // cr_assert_str_geq(a, b)
    // Memory
    MemEq, // cr_assert_arr_eq(a, b, size)
    MemNe, // cr_assert_arr_neq(a, b, size)
}

impl AssertKind {
    /// Map to the Rust macro / expression pattern used in emission
    pub fn to_rust(&self, fatal: bool) -> RustAssert {
        // cr_expect (non-fatal) maps to assert! in Rust — semantic loss noted
        match self {
            AssertKind::Assert => RustAssert::Macro("assert!"),
            AssertKind::Eq => RustAssert::Macro("assert_eq!"),
            AssertKind::Ne => RustAssert::Macro("assert_ne!"),
            AssertKind::Lt => RustAssert::Infix("<"),
            AssertKind::Le => RustAssert::Infix("<="),
            AssertKind::Gt => RustAssert::Infix(">"),
            AssertKind::Ge => RustAssert::Infix(">="),
            AssertKind::Null => RustAssert::Method("is_null()"),
            AssertKind::NotNull => RustAssert::MethodNeg("is_null()"),
            AssertKind::FloatEq => RustAssert::FloatEq,
            AssertKind::FloatNe => RustAssert::FloatNe,
            AssertKind::StrEq => RustAssert::Macro("assert_eq!"),
            AssertKind::StrNe => RustAssert::Macro("assert_ne!"),
            AssertKind::StrLt => RustAssert::Infix("<"),
            AssertKind::StrLe => RustAssert::Infix("<="),
            AssertKind::StrGt => RustAssert::Infix(">"),
            AssertKind::StrGe => RustAssert::Infix(">="),
            AssertKind::MemEq => RustAssert::Macro("assert_eq!"),
            AssertKind::MemNe => RustAssert::Macro("assert_ne!"),
        }
    }
}

/// How the assertion maps to Rust syntax
#[derive(Debug, Clone)]
pub enum RustAssert {
    Macro(&'static str),     // assert_eq!(a, b)
    Infix(&'static str),     // assert!(a < b)
    Method(&'static str),    // assert!(p.is_null())
    MethodNeg(&'static str), // assert!(!p.is_null())
    FloatEq,                 // assert!((a - b).abs() < eps)
    FloatNe,                 // assert!((a - b).abs() >= eps)
}
