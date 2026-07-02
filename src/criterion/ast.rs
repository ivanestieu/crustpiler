// -----------------------------------------------------------------------------
// CRITERION-SPECIFIC LAYER
// Sits on top of the C AST — recognised after parsing, not during.
// -----------------------------------------------------------------------------

use crate::ast::ast::{Expr, Item};
use crate::ast::statements::Stmt;

/// A full parsed .c file containing Criterion tests
#[derive(Debug)]
pub struct CriterionFile {
    pub items: Vec<Item>,                  // non-test C items (helpers, etc.)
    pub suites: Vec<CriterionSuite>,
}

#[derive(Debug)]
pub struct CriterionSuite {
    pub name: String,
    pub timeout: Option<f64>,
    pub tests: Vec<CriterionTest>,
    pub span: crate::ast::span::Span,
}

#[derive(Debug)]
pub struct CriterionTest {
    pub suite: String,
    pub name: String,
    pub disabled: bool,
    pub timeout: Option<f64>,
    pub body: Vec<CriterionBodyItem>,
    pub span: crate::ast::span::Span,
}

/// Each item in a test body is either a Criterion assertion or plain C
#[derive(Debug)]
pub enum CriterionBodyItem {
    Assertion(CriterionAssertion),
    Other(crate::ast::span::Spanned<Stmt>),                  // kept for context / manual review
}

#[derive(Debug)]
pub struct CriterionAssertion {
    pub kind: AssertKind,
    pub fatal: bool,                       // cr_assert = fatal, cr_expect = not
    pub args: Vec<crate::ast::span::Spanned<Expr>>,          // fully parsed, not raw strings
    pub message: Option<crate::ast::span::Spanned<Expr>>,    // last string arg if present
    pub span: crate::ast::span::Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssertKind {
    Assert,
    Eq, Ne,
    Lt, Le, Gt, Ge,
    Null, NotNull,
    FloatEq, FloatNe,
    StrEq, StrNe, StrLt, StrLe, StrGt, StrGe,
    MemEq, MemNe,
}
