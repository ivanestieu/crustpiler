use crate::ast::decl_specifiers::TypeExpr;
use crate::ast::declarator::Declarator;
use crate::ast::enums::EnumSpec;
use crate::ast::struct_union::StructOrUnion;
use crate::lexer::token::Token;

// -----------------------------------------------------------------------------
// TYPES
// -----------------------------------------------------------------------------
pub trait AsTypeQualifier {
    fn as_type_qualifier(&self) -> Result<TypeQualifier, String>;
}

impl AsTypeQualifier for crate::lexer::token::Token {
    fn as_type_qualifier(&self) -> Result<TypeQualifier, String> {
        match self {
            Token::KwConst => Ok(TypeQualifier::Const),
            Token::KwVolatile => Ok(TypeQualifier::Volatile),
            Token::KwRestrict => Ok(TypeQualifier::Restrict),
            Token::KwAtomic => Err(
                "Atomic is ambiguous and cannot be resolved as qualifier by this function"
                    .to_string(),
            ),
            _ => Err(format!("Expected type qualifier, found {:?}", self)),
        }
    }
}

// -----------------------------------------------------------------------------
// TYPE SPECIFIER
//
// Primitive arithmetic types are NOT enumerated as fixed combinations. C builds
// them from three independent axes (sign, size, base) given in any order and in
// any number of words: `unsigned long`, `long unsigned int`, `long long`, etc.
// We accumulate the axes during parsing and store them here. Non-arithmetic
// specifiers (void, _Bool, named types, tags) stay as distinct variants.
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum TypeSpec {
    // Arithmetic types built from independent specifier axes
    Arithmetic(ArithType),

    // Non-arithmetic primitives
    Void,
    Bool, // _Bool / stdbool.h bool

    // Struct / union
    Struct(StructOrUnion),
    Union(StructOrUnion),

    // Enum
    Enum(EnumSpec),

    // Named type (typedef or tag alias)
    Named(String), // e.g. size_t, uint8_t, MyStruct

    // Atomic
    Atomic(Box<TypeName>), // eg. _Atomic(type-name)
}

/// An arithmetic type decomposed into its three independent axes.
/// `int`, `unsigned`, `long long int`, `unsigned char`, `long double` all fit.
#[derive(Debug, Clone, PartialEq)]
pub struct ArithType {
    pub sign: Option<Sign>, // None = unspecified (int→signed; char→impl-defined)
    pub size: SizeSpec,     // short / none / long / long long
    pub base: BaseType,     // int / char / float / double
    pub complex: Option<Complex>, // _Complex / _Imaginary
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sign {
    Signed,   // signed
    Unsigned, // unsigned
}

#[derive(Debug, Clone, PartialEq)]
pub enum SizeSpec {
    Short,    // short
    None,     // (no size word)
    Long,     // long
    LongLong, // long long
}

#[derive(Debug, Clone, PartialEq)]
pub enum BaseType {
    Int, // also the default base when only sign/size are written
    Char,
    Float,
    Double, // `long double` = {size: Long, base: Double}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Complex {
    Complex,
    Imaginary,
}

impl ArithType {
    pub fn to_c_string(&self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if let Some(sign) = &self.sign {
            parts.push(match sign {
                Sign::Signed => "signed",
                Sign::Unsigned => "unsigned",
            });
        }
        match self.size {
            SizeSpec::Short => parts.push("short"),
            SizeSpec::None => {}
            SizeSpec::Long => parts.push("long"),
            SizeSpec::LongLong => {
                parts.push("long");
                parts.push("long");
            }
        }
        parts.push(match self.base {
            BaseType::Int => "int",
            BaseType::Char => "char",
            BaseType::Float => "float",
            BaseType::Double => "double",
        });
        if let Some(c) = &self.complex {
            parts.push(match c {
                Complex::Complex => "_Complex",
                Complex::Imaginary => "_Imaginary",
            });
        }
        parts.join(" ")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeQualifier {
    Const,
    Volatile,
    Restrict,
    Atomic,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StorageClass {
    Auto,
    Register,
    Static,
    ThreadLocal,
    Extern,
    Typedef,
}

pub trait AsStorageClass {
    fn as_storage_class(&self) -> Result<StorageClass, String>;
}

impl AsStorageClass for crate::lexer::token::Token {
    fn as_storage_class(&self) -> Result<StorageClass, String> {
        match self {
            Token::KwAuto => Ok(StorageClass::Auto),
            Token::KwRegister => Ok(StorageClass::Register),
            Token::KwStatic => Ok(StorageClass::Static),
            Token::KwThreadLocal => Ok(StorageClass::ThreadLocal),
            Token::KwExtern => Ok(StorageClass::Extern),
            Token::KwTypedef => Ok(StorageClass::Typedef),
            _ => Err(format!("Expected storage class, found {:?}", self)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionSpecifier {
    Inline,
    NoReturn,
}

pub trait AsFunctionSpecifier {
    fn as_function_specifier(&self) -> Result<FunctionSpecifier, String>;
}
impl AsFunctionSpecifier for crate::lexer::token::Token {
    fn as_function_specifier(&self) -> Result<FunctionSpecifier, String> {
        match self {
            Token::KwInline => Ok(FunctionSpecifier::Inline),
            Token::KwNoreturn => Ok(FunctionSpecifier::NoReturn),
            other => Err(format!("Expected function specifier, found {:?}", other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeName {
    pub type_expr: TypeExpr,
    pub derived: Declarator,
}
