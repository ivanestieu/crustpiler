use crate::ast::types::{
    ArithType, BaseType, Complex, FunctionSpecifier, Sign, SizeSpec, StorageClass, TypeName,
    TypeQualifier, TypeSpec,
};
use crate::output::output::Output;

// -----------------------------------------------------------------------------
// TYPE SPECIFIER
// -----------------------------------------------------------------------------
impl Output for TypeSpec {
    fn as_c_repr(&self) -> String {
        match self {
            TypeSpec::Arithmetic(arith) => arith.as_c_repr(),
            TypeSpec::Void => String::from("void"),
            TypeSpec::Bool => String::from("_Bool"),
            TypeSpec::Struct(struct_) => format!("struct {}", struct_.as_c_repr()),
            TypeSpec::Union(union) => format!("union {}", union.as_c_repr()),
            TypeSpec::Enum(enum_) => enum_.as_c_repr(),
            TypeSpec::Named(name) => name.clone(),
            TypeSpec::Atomic(atomic) => atomic.as_c_repr(),
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

impl Output for ArithType {
    fn as_c_repr(&self) -> String {
        match (&self.size, &self.sign, &self.complex) {
            (SizeSpec::None, Some(sign), None) => format!("{} {}",
                sign.as_c_repr(),
                self.base.as_c_repr()
            ),
            (size_spec, Some(sign), None) => format!("{} {} {}",
                sign.as_c_repr(),
                size_spec.as_c_repr(),
                self.base.as_c_repr()
            ),
            (SizeSpec::None, None, None) => self.base.as_c_repr(),
            (size_spec, None, None) => format!("{} {}",
                size_spec.as_c_repr(),
                self.base.as_c_repr()
            ),
            (SizeSpec::None, None, Some(complex)) => format!("{} {}",
                complex.as_c_repr(),
                self.base.as_c_repr()
            ),
            _ => panic!("this state should not exists.")
        }
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

impl Output for Sign {
    fn as_c_repr(&self) -> String {
        match self {
            Sign::Signed => "signed",
            Sign::Unsigned => "unsigned"
        }.to_string()
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}

impl Output for SizeSpec {
    fn as_c_repr(&self) -> String {
        match self {
            SizeSpec::Short => "short",
            SizeSpec::None => "",
            SizeSpec::Long => "long",
            SizeSpec::LongLong => "long long",
        }.to_string()
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}

impl Output for BaseType {
    fn as_c_repr(&self) -> String {
       match self {
           BaseType::Int => "int",
           BaseType::Char => "char",
           BaseType::Float => "float",
           BaseType::Double => "double"
       }.to_string()
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}

impl Output for Complex {
    fn as_c_repr(&self) -> String {
        match self {
            Complex::Complex => "_Complex",
            Complex::Imaginary => "_Imaginary",
        }
        .to_string()
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}

impl Output for TypeQualifier {
    fn as_c_repr(&self) -> String {
        match self {
            TypeQualifier::Const => "const",
            TypeQualifier::Volatile => "volatile",
            TypeQualifier::Restrict => "restrict",
            TypeQualifier::Atomic => "_Atomic",
        }
        .to_string()
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}

impl Output for StorageClass {
    fn as_c_repr(&self) -> String {
        match self {
            StorageClass::Auto => "auto",
            StorageClass::Register => "register",
            StorageClass::Static => "static",
            StorageClass::ThreadLocal => "_Thread_local",
            StorageClass::Extern => "extern",
            StorageClass::Typedef => "typedef",
        }
        .to_string()
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}

impl Output for FunctionSpecifier {
    fn as_c_repr(&self) -> String {
        match self {
            FunctionSpecifier::Inline => "inline",
            FunctionSpecifier::NoReturn => "_Noreturn",
        }
        .to_string()
    }

    fn as_rust_repr(&self) -> String {
        todo!()
    }
}

impl Output for TypeName {
    fn as_c_repr(&self) -> String {
        // base spec, then the declarator's C spelling is best-effort here
        format!(
            "{} {}",
            self.type_expr.as_c_repr(),
            self.derived.as_c_repr()
        )
    }

    fn as_rust_repr(&self) -> String {
        let base = self.type_expr.as_rust_repr();
        crate::output::output::wrap_declarator_rust(&self.derived, base)
    }
}
