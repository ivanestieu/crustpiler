// decl_specifiers.rs
//
// ONE big specifier structure, collected by ONE shared builder, with context
// rules applied at add time.

pub(crate) use crate::ast::declarations::{AlignmentSpecifier};
use crate::ast::types::{ArithType, BaseType, Complex, FunctionSpecifier, Sign, SizeSpec, StorageClass, TypeQualifier, TypeSpec};

// -----------------------------------------------------------------------------
// The unified structure every declaration-ish context produces.
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct TypeExpr {
    pub storage: Option<StorageClass>,
    pub thread_local: bool,               // _Thread_local, tracked separately so it can co-occur with static/extern
    pub type_spec: TypeSpec,
    pub qualifiers: Vec<TypeQualifier>,
    pub function_specifiers: Vec<FunctionSpecifier>,
    pub alignment: Option<AlignmentSpecifier>,
}

impl TypeExpr {
    pub(crate) fn is_void(&self) -> bool {
        self.type_spec == TypeSpec::Void
    }
}

// -----------------------------------------------------------------------------
// Context — where the specifiers appear. Drives which are legal.
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TypeExprContext {
    /// declaration_specifiers: everything allowed.
    #[default]
    Declaration,
    /// struct/union member: no storage class, no function specifier.
    /// (_Alignas IS permitted on a member in C11.)
    StructUnionField,
    /// type_name (cast, sizeof, _Alignof, compound literal, _Generic):
    /// only type-specifiers and type-qualifiers.
    TypeName,
    /// parameter_declaration: like a declaration, but `register` is the only
    /// storage class allowed, and no function specifiers / _Alignas.
    Parameter,
}

impl TypeExprContext {
    fn validate_storage_class(&self, &sc : &StorageClass) -> Result<(), String> {
        match self {
            TypeExprContext::StructUnionField =>
                 Err("a storage-class specifier is not allowed on a struct/union member".into()),
            TypeExprContext::TypeName =>
                 Err("a storage-class specifier is not allowed in a type name".into()),
            TypeExprContext::Parameter =>
                match sc {
                    StorageClass::ThreadLocal =>
                        Err("`_Thread_local` is not allowed on a parameter".into()),
                    StorageClass::Register => Ok(()),
                    _ => Err(format!(
                        "only `register` is allowed as a storage class on a parameter, found `{:?}`",
                        sc
                        ))
                }
            _ => Ok(())
        }
    }

    fn validate_function_specifiers(&self) -> Result<(), String> {
        match self {
            TypeExprContext::StructUnionField => Err("a function specifier (`inline`/`_Noreturn`) is not allowed on a struct/union member".into()),
            TypeExprContext::TypeName => Err("a function specifier is not allowed in a type name".into()),
            TypeExprContext::Parameter => Err("a function specifier is not allowed on a parameter".into()),
            _ => Ok(())
        }
    }

    fn validate_alignment(&self) -> Result<(), String> {
        match self {
            TypeExprContext::TypeName => Err("`_Alignas` is not allowed in a type name".into()),
            TypeExprContext::Parameter => Err("`_Alignas` is not allowed on a parameter".into()),
            _ => Ok(()),
        }
    }

}

// -----------------------------------------------------------------------------
// The shared builder — accumulate specifiers in any order, validate keywords
// depending on context, resolve on finish
// -----------------------------------------------------------------------------

#[derive(Default, Debug)]
pub struct TypeExprBuilder {
    context: TypeExprContext,
    // storage
    storage: Option<StorageClass>,
    thread_local: bool,

    // arithmetic axes
    sign: Option<Sign>,
    short_count: u32,
    long_count: u32,
    base: Option<BaseType>,
    complex: Option<Complex>,
    saw_void: bool,
    saw_bool: bool,

    // non-arithmetic type specifier (struct/union/enum/typedef/_Atomic(T))
    tagged_or_named: Option<TypeSpec>,

    qualifiers: Vec<TypeQualifier>,
    function_specifiers: Vec<FunctionSpecifier>,
    alignment: Option<AlignmentSpecifier>,
}

impl TypeExprBuilder {
    pub fn new(context: TypeExprContext) -> Self { TypeExprBuilder { context, .. Default::default() } }

    // ── storage class ──────────────────────────────────────────────────────
    pub fn add_storage(&mut self, sc: StorageClass) -> Result<(), String> {
        self.context.validate_storage_class(&sc)?;
        if sc == StorageClass::ThreadLocal {
            if self.thread_local {
                return Err("duplicate `_Thread_local`".into());
            }
            self.thread_local = true;
            return Ok(());
        }
        match &self.storage {
            None => { self.storage = Some(sc); Ok(()) }
            Some(existing) => Err(format!(
                "cannot combine storage-class specifiers `{:?}` and `{:?}`",
                existing, sc
            )),
        }
    }

    // ── arithmetic axes ────────────────────────────────────────────────────
    pub fn add_sign(&mut self, s: Sign) -> Result<(), String> {
        self.reject_if_tagged("a sign specifier")?;
        match &self.sign {
            Some(existing) if *existing != s => Err("conflicting `signed` and `unsigned`".into()),
            Some(_) => Err("duplicate sign specifier".into()),
            None => { self.sign = Some(s); Ok(()) }
        }
    }
    pub fn add_short(&mut self) -> Result<(), String> {
        self.reject_if_tagged("`short`")?;
        self.short_count += 1; Ok(())
    }
    pub fn add_long(&mut self) -> Result<(), String> {
        self.reject_if_tagged("`long`")?;
        self.long_count += 1; Ok(())
    }
    pub fn add_base(&mut self, b: BaseType) -> Result<(), String> {
        self.reject_if_tagged("a type specifier")?;
        match &self.base {
            Some(existing) if *existing != b => Err("conflicting base type specifiers".into()),
            Some(_) => Err("duplicate base type specifier".into()),
            None => { self.base = Some(b); Ok(()) }
        }
    }
    pub fn add_complex(&mut self, c: Complex) -> Result<(), String> {
        self.reject_if_tagged("`_Complex`/`_Imaginary`")?;
        match &self.complex {
            Some(_) => Err("duplicate `_Complex`/`_Imaginary`".into()),
            None => { self.complex = Some(c); Ok(()) }
        }
    }
    pub fn set_void(&mut self) -> Result<(), String> {
        self.reject_if_tagged("`void`")?;
        self.saw_void = true; Ok(())
    }
    pub fn set_bool(&mut self) -> Result<(), String> {
        self.reject_if_tagged("`_Bool`")?;
        self.saw_bool = true; Ok(())
    }

    // ── tagged / named type specifier ──────────────────────────────────────
    pub fn set_tagged_or_named(&mut self, spec: TypeSpec) -> Result<(), String> {
        if self.has_any_arith_axis() {
            return Err("cannot combine a struct/union/enum/typedef name with \
                        arithmetic type specifiers".into());
        }
        if self.tagged_or_named.is_some() {
            return Err("more than one type specifier".into());
        }
        self.tagged_or_named = Some(spec);
        Ok(())
    }

    // ── qualifiers / fn-spec / alignment ───────────────────────────────────
    pub fn add_qualifier(&mut self, q: TypeQualifier) {
        if !self.qualifiers.contains(&q) { self.qualifiers.push(q); }
    }
    pub fn add_function_specifier(&mut self, fs: FunctionSpecifier) -> Result<(), String> {
        self.context.validate_function_specifiers()?;
        if !self.function_specifiers.contains(&fs) { self.function_specifiers.push(fs); }
        Ok(())
    }
    pub fn set_alignment(&mut self, a: AlignmentSpecifier) -> Result<(), String> {
        self.context.validate_alignment()?;
        if self.alignment.is_some() { return Err("multiple `_Alignas` specifiers".into()); }
        self.alignment = Some(a); Ok(())
    }

    // ── helpers ────────────────────────────────────────────────────────────
    fn has_any_arith_axis(&self) -> bool {
        self.sign.is_some() || self.short_count > 0 || self.long_count > 0
            || self.base.is_some() || self.complex.is_some()
            || self.saw_void || self.saw_bool
    }
    fn reject_if_tagged(&self, what: &str) -> Result<(), String> {
        if self.tagged_or_named.is_some() {
            Err(format!("cannot combine {} with a struct/union/enum/typedef name", what))
        } else { Ok(()) }
    }

    // ── finish: resolve type ─────────────────────
    pub fn finish(self) -> Result<TypeExpr, String> {
        let type_spec = self.resolve_type_spec();
        if type_spec.is_err()
            && self.storage.is_none()
            && self.thread_local == false
            && self.qualifiers.is_empty()
            && self.function_specifiers.is_empty()
            && self.alignment.is_none()
        {
            Err("Invalid TypeExpr: all fields cannot be empty.".into())
        } else {
            Ok(TypeExpr {
                storage: self.storage,
                thread_local: self.thread_local,
                type_spec: type_spec.ok().unwrap_or(TypeSpec::Arithmetic(ArithType {
                    sign: self.sign,
                    size: SizeSpec::None,
                    base: BaseType::Int,
                    complex: self.complex,
                })),
                qualifiers: self.qualifiers,
                function_specifiers: self.function_specifiers,
                alignment: self.alignment,
            })
        }
    }

    fn resolve_type_spec(&self) -> Result<TypeSpec, String> {
        if let Some(spec) = &self.tagged_or_named {
            return Ok(spec.clone());
        }
        if self.saw_void {
            if self.has_other_than(true) {
                return Err("`void` cannot combine with other type specifiers".into());
            }
            return Ok(TypeSpec::Void);
        }
        if self.saw_bool {
            if self.has_other_than(false) {
                return Err("`_Bool` cannot combine with other type specifiers".into());
            }
            return Ok(TypeSpec::Bool);
        }
        // Arithmetic
        let size = resolve_size(self.short_count, self.long_count)?;
        let base = self.base.clone().unwrap_or(BaseType::Int);
        validate_base_size(&base, &size)?;
        validate_base_sign(&base, &self.sign)?;
        validate_base_complex(&base, &self.complex)?;
        if self.base.is_none()
            && self.sign.is_none()
            && self.complex.is_none()
            && size == SizeSpec::None
        {
            return Err("no type specifier found; expected one of: `void`, `_Bool`, `_Complex`, `_Imaginary`, `char`, `short`, `int`, `long`, `float`, `double`, or a struct/union/enum/typedef name".into());
        }
        Ok(TypeSpec::Arithmetic(ArithType {
            sign: self.sign.clone(),
            size,
            base,
            complex: self.complex.clone(),
        }))
    }

    fn has_other_than(&self, checking_void: bool) -> bool {
        let arith = self.sign.is_some() || self.short_count > 0 || self.long_count > 0
            || self.base.is_some() || self.complex.is_some();
        if checking_void { arith || self.saw_bool } else { arith || self.saw_void }
    }
}

// ── arithmetic validation (shared helpers) ───────────────────────────────────

fn resolve_size(short_count: u32, long_count: u32) -> Result<SizeSpec, String> {
    match (short_count, long_count) {
        (0, 0) => Ok(SizeSpec::None),
        (1, 0) => Ok(SizeSpec::Short),
        (0, 1) => Ok(SizeSpec::Long),
        (0, 2) => Ok(SizeSpec::LongLong),
        (s, 0) if s > 1 => Err("`short short` is not a valid type".into()),
        (0, l) if l > 2 => Err("`long long long` is too long".into()),
        _ => Err("`short` and `long` cannot be combined".into()),
    }
}

fn validate_base_size(base: &BaseType, size: &SizeSpec) -> Result<(), String> {
    match base {
        BaseType::Char if *size != SizeSpec::None => Err("`char` cannot be `short` or `long`".into()),
        BaseType::Float if *size != SizeSpec::None => Err("`float` cannot be `short` or `long`".into()),
        BaseType::Double => match size {
            SizeSpec::None | SizeSpec::Long => Ok(()),
            _ => Err("only `long double` is valid among sized doubles".into()),
        },
        _ => Ok(()),
    }
}

fn validate_base_sign(base: &BaseType, sign: &Option<Sign>) -> Result<(), String> {
    match base {
        BaseType::Float | BaseType::Double if sign.is_some() =>
            Err("floating types cannot be `signed` or `unsigned`".into()),
        _ => Ok(()),
    }
}

fn validate_base_complex(base: &BaseType, complex: &Option<Complex>) -> Result<(), String> {
    match (base, complex) {
        (BaseType::Int | BaseType::Char, Some(_)) =>
            Err("`_Complex`/`_Imaginary` require a floating base type".into()),
        _ => Ok(()),
    }
}