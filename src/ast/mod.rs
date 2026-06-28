// =============================================================================
// C AST
// Covers: full expressions, statements, declarations, types
// Excludes: K&R syntax, _Generic, _Atomic, VLAs, bitfields (rare in tests)
// =============================================================================

// -----------------------------------------------------------------------------
// SPAN — source location, attach to every node for error reporting
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize, // byte offset in source
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

// Convenience wrapper — every meaningful node carries its span
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

// -----------------------------------------------------------------------------
// LITERALS
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum IntBase {
    Decimal,
    Hexadecimal,
    Octal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntLit {
    pub value: u64,
    pub base: IntBase,
    pub suffix: IntSuffix,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntSuffix {
    pub unsigned: bool,  // u / U
    pub long: LongKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LongKind {
    None,
    Long,       // l / L
    LongLong,   // ll / LL
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloatLit {
    pub value: f64,
    pub suffix: FloatSuffix,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FloatSuffix {
    Double,  // no suffix
    Float,   // f / F
    LongDouble, // l / L
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringLit {
    pub value: String,         // decoded content
    pub prefix: StringPrefix,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPrefix {
    None,    // "..."
    Wide,    // L"..."
    Utf8,    // u8"..."
    Utf16,   // u"..."
    Utf32,   // U"..."
}

// -----------------------------------------------------------------------------
// TYPES
// -----------------------------------------------------------------------------

// A fully parsed type as it appears in declarations and casts
#[derive(Debug, Clone, PartialEq)]
pub struct TypeExpr {
    pub qualifiers: Vec<TypeQualifier>,
    pub spec: TypeSpec,
    pub derived: Vec<DerivedType>, // pointer/array/function layers, outermost first
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeQualifier {
    Const,
    Volatile,
    Restrict,   // pointer contexts only
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeSpec {
    // Primitive
    Void,
    Bool,               // _Bool / stdbool.h bool
    Char,
    SignedChar,
    UnsignedChar,
    Short,
    UnsignedShort,
    Int,
    UnsignedInt,
    Long,
    UnsignedLong,
    LongLong,
    UnsignedLongLong,
    Float,
    Double,
    LongDouble,

    // Named type (typedef or tag)
    Named(String),      // e.g. size_t, uint8_t, MyStruct

    // Struct / union
    Struct(StructOrUnion),
    Union(StructOrUnion),

    // Enum
    Enum(EnumSpec),
}

// Pointer / array / function layering on top of a base type
// Example: int *(*f)(void)
//   base = Int
//   derived = [Function([void]), Pointer, Pointer]
#[derive(Debug, Clone, PartialEq)]
pub enum DerivedType {
    Pointer(Vec<TypeQualifier>),           // * const, * volatile, etc.
    Array(Option<Box<Expr>>),              // [N] or []
    Function(Vec<ParamDecl>, bool),        // (params, is_variadic)
}

// -----------------------------------------------------------------------------
// STRUCT / UNION
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct StructOrUnion {
    pub name: Option<String>,              // anonymous if None
    pub fields: Option<Vec<FieldDecl>>,   // None = forward declaration
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub spec: TypeSpec,
    pub qualifiers: Vec<TypeQualifier>,
    pub declarators: Vec<FieldDeclarator>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDeclarator {
    pub declarator: Option<Declarator>,   // None for anonymous bitfield
    // bitfields omitted — unlikely in test code
}

// -----------------------------------------------------------------------------
// ENUM
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct EnumSpec {
    pub name: Option<String>,
    pub variants: Option<Vec<Enumerator>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enumerator {
    pub name: String,
    pub value: Option<Box<Expr>>,          // explicit = value
    pub span: Span,
}

// -----------------------------------------------------------------------------
// DECLARATOR
// The part that encodes pointer/array/function on top of a type specifier.
// Parsed with the classic inside-out algorithm.
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Declarator {
    // Leaf: just a name (or abstract — no name, for param/cast types)
    Ident(String, Span),
    Abstract,                              // nameless, used in casts / params

    // Derived
    Pointer {
        qualifiers: Vec<TypeQualifier>,
        inner: Box<Declarator>,
    },
    Array {
        inner: Box<Declarator>,
        size: Option<Box<Expr>>,
        qualifiers: Vec<TypeQualifier>,    // int a[const 3] is legal
    },
    Function {
        inner: Box<Declarator>,
        params: Vec<ParamDecl>,
        variadic: bool,
    },
}

impl Declarator {
    /// Extract the declared identifier name, if any
    pub fn ident(&self) -> Option<&str> {
        match self {
            Declarator::Ident(name, _) => Some(name),
            Declarator::Abstract => None,
            Declarator::Pointer { inner, .. }
            | Declarator::Array { inner, .. }
            | Declarator::Function { inner, .. } => inner.ident(),
        }
    }
}

// -----------------------------------------------------------------------------
// PARAMETERS
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct ParamDecl {
    pub spec: TypeSpec,
    pub qualifiers: Vec<TypeQualifier>,
    pub declarator: Declarator,            // may be Abstract for unnamed params
    pub span: Span,
}

// -----------------------------------------------------------------------------
// DECLARATIONS (top-level and local)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Decl {
    pub storage: Option<StorageClass>,
    pub qualifiers: Vec<TypeQualifier>,
    pub spec: TypeSpec,
    pub declarators: Vec<InitDeclarator>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StorageClass {
    Auto,
    Register,
    Static,
    Extern,
    Typedef,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InitDeclarator {
    pub declarator: Declarator,
    pub init: Option<Initializer>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Initializer {
    Expr(Box<Expr>),                       // int x = 5;
    List(Vec<InitItem>),                   // int arr[] = {1, 2, 3};
}

#[derive(Debug, Clone, PartialEq)]
pub struct InitItem {
    pub designators: Vec<Designator>,      // [0] = or .field =
    pub value: Initializer,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Designator {
    Index(Box<Expr>),                      // [expr]
    Field(String),                         // .name
}

// -----------------------------------------------------------------------------
// EXPRESSIONS
// All operators from C11 §6.5, precedence handled by the Pratt parser at
// runtime — not encoded in the AST type itself.
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    IntLit(IntLit),
    FloatLit(FloatLit),
    CharLit(char),
    StringLit(StringLit),

    // Identifier
    Ident(String),

    // Compound literal: (Type){init}  — C99
    CompoundLit {
        ty: TypeExpr,
        init: Vec<InitItem>,
    },

    // Unary prefix
    UnaryOp {
        op: UnaryOp,
        operand: Box<Spanned<Expr>>,
    },

    // Unary postfix — kept separate because precedence/associativity differ
    PostfixOp {
        op: PostfixOp,
        operand: Box<Spanned<Expr>>,
    },

    // Binary
    BinaryOp {
        op: BinaryOp,
        lhs: Box<Spanned<Expr>>,
        rhs: Box<Spanned<Expr>>,
    },

    // Assignment (right-associative, lower precedence than most binops)
    Assign {
        op: AssignOp,
        lhs: Box<Spanned<Expr>>,
        rhs: Box<Spanned<Expr>>,
    },

    // Ternary
    Ternary {
        cond: Box<Spanned<Expr>>,
        then: Box<Spanned<Expr>>,
        els: Box<Spanned<Expr>>,
    },

    // Function call
    Call {
        callee: Box<Spanned<Expr>>,
        args: Vec<Spanned<Expr>>,
    },

    // Subscript: array[index]
    Index {
        array: Box<Spanned<Expr>>,
        index: Box<Spanned<Expr>>,
    },

    // Member access: expr.field  or  expr->field
    Member {
        expr: Box<Spanned<Expr>>,
        field: String,
        arrow: bool,               // true = ->, false = .
    },

    // Cast: (Type)expr
    Cast {
        ty: TypeExpr,
        expr: Box<Spanned<Expr>>,
    },

    // sizeof
    SizeofExpr(Box<Spanned<Expr>>),
    SizeofType(TypeExpr),

    // _Alignof  (C11)
    AlignofType(TypeExpr),

    // Comma operator: a, b  (lowest precedence)
    Comma(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
}

// -----------------------------------------------------------------------------
// OPERATORS
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,          // -
    Pos,          // +
    Not,          // !
    BitNot,       // ~
    Deref,        // *
    AddrOf,       // &
    PreInc,       // ++x
    PreDec,       // --x
}

#[derive(Debug, Clone, PartialEq)]
pub enum PostfixOp {
    PostInc,      // x++
    PostDec,      // x--
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // Arithmetic
    Add, Sub, Mul, Div, Rem,
    // Bitwise
    BitAnd, BitOr, BitXor, Shl, Shr,
    // Comparison
    Eq, Ne, Lt, Le, Gt, Ge,
    // Logical
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Assign,        // =
    AddAssign,     // +=
    SubAssign,     // -=
    MulAssign,     // *=
    DivAssign,     // /=
    RemAssign,     // %=
    BitAndAssign,  // &=
    BitOrAssign,   // |=
    BitXorAssign,  // ^=
    ShlAssign,     // <<=
    ShrAssign,     // >>=
}

// -----------------------------------------------------------------------------
// STATEMENTS
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    // Expression statement (including assignments, calls, etc.)
    Expr(Spanned<Expr>),

    // Empty statement  ;
    Empty,

    // Block  { ... }
    Block(Vec<BlockItem>),

    // Control flow
    If {
        cond: Spanned<Expr>,
        then: Box<Spanned<Stmt>>,
        els: Option<Box<Spanned<Stmt>>>,
    },
    Switch {
        expr: Spanned<Expr>,
        body: Box<Spanned<Stmt>>,
    },

    // Loops
    While {
        cond: Spanned<Expr>,
        body: Box<Spanned<Stmt>>,
    },
    DoWhile {
        body: Box<Spanned<Stmt>>,
        cond: Spanned<Expr>,
    },
    For {
        init: ForInit,
        cond: Option<Spanned<Expr>>,
        step: Option<Spanned<Expr>>,
        body: Box<Spanned<Stmt>>,
    },

    // Jump
    Return(Option<Spanned<Expr>>),
    Break,
    Continue,
    Goto(String),

    // Labels
    Label(String, Box<Spanned<Stmt>>),
    Case(Spanned<Expr>, Box<Spanned<Stmt>>),
    Default(Box<Spanned<Stmt>>),
}

// A block contains either declarations or statements, interleaved (C99+)
#[derive(Debug, Clone, PartialEq)]
pub enum BlockItem {
    Decl(Spanned<Decl>),
    Stmt(Spanned<Stmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForInit {
    Empty,
    Expr(Spanned<Expr>),
    Decl(Spanned<Decl>),     // C99: for (int i = 0; ...)
}

// -----------------------------------------------------------------------------
// FUNCTION DEFINITION
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub storage: Option<StorageClass>,
    pub qualifiers: Vec<TypeQualifier>,
    pub ret: TypeSpec,
    pub declarator: Declarator,            // encodes name + params
    pub body: Vec<BlockItem>,
    pub span: Span,
}

impl FunctionDef {
    pub fn name(&self) -> Option<&str> {
        self.declarator.ident()
    }
}

// -----------------------------------------------------------------------------
// TOP-LEVEL ITEMS
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    FunctionDef(FunctionDef),
    Decl(Spanned<Decl>),                   // global variable / typedef / extern
}