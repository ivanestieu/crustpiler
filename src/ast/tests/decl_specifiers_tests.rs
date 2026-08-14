use super::*;
use crate::ast::ast::Expr;
use crate::ast::declarator::Declarator;
use crate::ast::types::TypeName;
use parameterized::parameterized;

#[macro_export]
macro_rules! tagged_builder {
    () => {
        TypeExprBuilder {
            tagged_or_named: Some(TypeSpec::Void),
            ..Default::default()
        }
    };
}

#[parameterized(context = {
    TypeExprContext::Declaration,
    TypeExprContext::TypeName,
    TypeExprContext::StructUnionField,
    TypeExprContext::Parameter,
}, expected = {
    Ok(()),
    Err(String::from("function specifiers (`inline`/`_Noreturn`) are not allowed in a type name")),
    Err(String::from("function specifiers (`inline`/`_Noreturn`) are not allowed on a struct/union member")),
    Err(String::from("function specifiers (`inline`/`_Noreturn`) are not allowed on a parameter")),
})]
fn test_context_validate_function_specifiers(
    context: TypeExprContext,
    expected: Result<(), String>,
) {
    assert_eq!(
        context.validate_function_specifiers(),
        expected,
        "validate_function_specifiers failed for context {:?}",
        context
    );
}

#[parameterized(context = {
    TypeExprContext::Declaration,
    TypeExprContext::StructUnionField,
    TypeExprContext::TypeName,
    TypeExprContext::Parameter,
}, expected = {
    Ok(()),
    Ok(()),
    Err(String::from("`_Alignas` is not allowed in a type name")),
    Err(String::from("`_Alignas` is not allowed on a parameter")),
})]
fn test_context_validate_alignment(context: TypeExprContext, expected: Result<(), String>) {
    assert_eq!(
        context.validate_alignment(),
        expected,
        "validate_function_specifiers failed for context {:?}",
        context
    );
}

#[test]
fn test_new_default_context() {
    let builder = TypeExprBuilder::new(Default::default());
    assert_eq!(
        builder,
        TypeExprBuilder {
            context: TypeExprContext::Declaration,
            storage: None,
            thread_local: false,
            sign: None,
            short_count: 0,
            long_count: 0,
            base: None,
            complex: None,
            saw_void: false,
            saw_bool: false,
            tagged_or_named: None,
            qualifiers: vec![],
            function_specifiers: vec![],
            alignment: None,
        },
        "TypeExprBuilder::new(Default::default()) failed."
    );
}

#[test]
fn test_new_type_name() {
    let builder = TypeExprBuilder::new(TypeExprContext::TypeName);
    assert_eq!(
        builder.context,
        TypeExprContext::TypeName,
        "TypeExprBuilder::new(/* context */) with context TypeExprContext::TypeName failed."
    );
}

#[test]
fn test_add_storage_valid() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_storage(StorageClass::Static)
        .expect("add_storage failed to add storage class.");
    assert_eq!(
        builder.storage,
        Some(StorageClass::Static),
        "add_storage failed to set storage class."
    );
}

#[parameterized(context = {
    TypeExprContext::TypeName,
    TypeExprContext::StructUnionField,
    TypeExprContext::Parameter,
}, error_msg = {
    "storage-class specifiers are not allowed in a type name",
    "storage-class specifiers are not allowed on a struct/union member",
    "only `register` is allowed as storage-class specifier on a parameter, found `Static`",
}, additional_msg = {
    "",
    "",
    " except for `register`",
})]
fn test_add_storage_invalid_context(
    context: TypeExprContext,
    error_msg: &'static str,
    additional_msg: &'static str,
) {
    let mut builder = TypeExprBuilder::new(context);
    let result = builder.add_storage(StorageClass::Static);
    assert_eq!(
        result,
        Err(String::from(error_msg)),
        "add_storage should fail when context is {:?}{}.",
        context,
        additional_msg
    );
    assert_eq!(
        builder.storage, None,
        "add_storage should not set storage class when it fails."
    );
}

#[test]
fn test_validate_storage_parameter_register() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Parameter);
    let result = builder.add_storage(StorageClass::Register);
    assert_eq!(
        result,
        Ok(()),
        "add_storage should accept `register` when context is TypeExprContext::Parameter."
    );
    assert_eq!(
        builder.storage,
        Some(StorageClass::Register),
        "add_storage failed to set storage class to `register`."
    );
}

#[test]
fn test_add_thread_local_valid() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_storage(StorageClass::ThreadLocal)
        .expect("add_thread_local failed to add thread-local specifier.");
    assert_eq!(
        builder.thread_local, true,
        "add_thread_local failed to set thread_local to true."
    );
}

#[test]
fn test_add_thread_local_double() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_storage(StorageClass::ThreadLocal)
        .expect("add_thread_local failed to add thread-local specifier.");
    assert_eq!(
        builder.thread_local, true,
        "add_storage failed to set thread_local to true."
    );
    assert_eq!(
        builder.add_storage(StorageClass::ThreadLocal),
        Err(String::from("duplicate `_Thread_local`")),
        "add_storage should fail when thread_local is already true."
    )
}

#[test]
fn test_add_storage_multiple() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_storage(StorageClass::Static)
        .expect("add_thread_local failed to add thread-local specifier.");
    assert_eq!(
        builder.storage,
        Some(StorageClass::Static),
        "add_storage failed to set storage class to `Static`."
    );
    assert_eq!(
        builder.add_storage(StorageClass::Extern),
        Err(String::from(
            "cannot combine storage-class specifiers `Static` and `Extern`",
        )),
        "add_storage should fail when trying to add a second storage class."
    )
}

#[test]
fn test_reject_if_tagged_valid() {
    let builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    assert_eq!(
        builder.reject_if_tagged("tests valid"),
        Ok(()),
        "reject_if_tagged_valid failed to set tagged_or_named to Tagged."
    );
}

#[test]
fn test_reject_if_tagged_invalid() {
    let builder = tagged_builder!();
    assert_eq!(
        builder.reject_if_tagged("`tests invalid`"),
        Err(String::from(
            "cannot combine `tests invalid` with a struct/union/enum/typedef name",
        )),
        "reject_if_tagged_valid failed to set tagged_or_named to Tagged."
    );
}

#[test]
fn test_add_sign_valid() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_sign(Sign::Signed)
        .expect("add_sign failed to add sign specifier.");
    assert_eq!(
        builder.sign,
        Some(Sign::Signed),
        "add_sign failed to set sign to Signed."
    );
}

#[test]
fn test_add_sign_double() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_sign(Sign::Signed)
        .expect("add_sign failed to add sign specifier.");
    assert_eq!(
        builder.sign,
        Some(Sign::Signed),
        "add_sign failed to set sign to Signed."
    );
    assert_eq!(
        builder.add_sign(Sign::Signed),
        Err(String::from("duplicate sign specifier")),
        "add_sign should fail when sign is already set."
    )
}

#[test]
fn test_add_sign_different() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_sign(Sign::Signed)
        .expect("add_sign failed to add sign specifier.");
    assert_eq!(
        builder.sign,
        Some(Sign::Signed),
        "add_sign failed to set sign to Signed."
    );
    assert_eq!(
        builder.add_sign(Sign::Unsigned),
        Err(String::from("conflicting `signed` and `unsigned`")),
        "add_sign should fail when sign is already set."
    )
}

#[test]
fn test_add_short_valid() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_short()
        .expect("add_short failed to add short specifier.");
    assert_eq!(
        builder.short_count, 1,
        "add_short failed to increment short_count."
    );
}

#[test]
fn test_add_short_multiple() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_short()
        .expect("add_short failed to add short specifier.");
    assert_eq!(
        builder.short_count, 1,
        "add_short failed to increment short_count."
    );
    builder
        .add_short()
        .expect("add_short failed to add a second short specifier.");
    assert_eq!(
        builder.short_count, 2,
        "add_short failed to increment short_count."
    );
    builder
        .add_short()
        .expect("add_short failed to add a third short specifier.");
    assert_eq!(
        builder.short_count, 3,
        "add_short failed to increment short_count."
    );
}

#[test]
fn test_add_short_invalid() {
    let mut builder = tagged_builder!();
    assert_eq!(
        builder.add_short(),
        Err(String::from(
            "cannot combine `short` with a struct/union/enum/typedef name",
        )),
        "add_short should fail when tagged_or_named is set."
    );
}

#[test]
fn test_add_long_valid() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_long()
        .expect("add_long failed to add long specifier.");
    assert_eq!(
        builder.long_count, 1,
        "add_long failed to increment long_count."
    );
}

#[test]
fn test_add_long_multiple() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_long()
        .expect("add_long failed to add long specifier.");
    assert_eq!(
        builder.long_count, 1,
        "add_long failed to increment long_count."
    );
    builder
        .add_long()
        .expect("add_long failed to add a second long specifier.");
    assert_eq!(
        builder.long_count, 2,
        "add_long failed to increment long_count."
    );
    builder
        .add_long()
        .expect("add_long failed to add a third long specifier.");
    assert_eq!(
        builder.long_count, 3,
        "add_long failed to increment long_count."
    );
}

#[test]
fn test_add_long_invalid() {
    let mut builder = tagged_builder!();
    assert_eq!(
        builder.add_long(),
        Err(String::from(
            "cannot combine `long` with a struct/union/enum/typedef name",
        )),
        "add_long should fail when tagged_or_named is set."
    );
}

#[test]
fn test_set_void_valid() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .set_void()
        .expect("set_void failed to set void specifier.");
    assert_eq!(
        builder.saw_void, true,
        "set_void failed to set saw_void to true."
    );
}

#[test]
fn test_set_void_multiple() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .set_void()
        .expect("set_void failed to set void specifier.");
    assert_eq!(
        builder.saw_void, true,
        "set_void failed to set saw_void to true."
    );
    assert_eq!(
        builder.set_void(),
        Err(String::from("duplicate `void`")),
        "set_void should fail when saw_void is already true."
    )
}

#[test]
fn test_set_void_invalid() {
    let mut builder = tagged_builder!();
    assert_eq!(
        builder.set_void(),
        Err(String::from(
            "cannot combine `void` with a struct/union/enum/typedef name",
        )),
        "set_void should fail when tagged_or_named is set."
    );
}

#[test]
fn test_set_bool_valid() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .set_bool()
        .expect("set_bool failed to set bool specifier.");
    assert_eq!(
        builder.saw_bool, true,
        "set_bool failed to set saw_bool to true."
    );
}

#[test]
fn test_set_bool_multiple() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .set_bool()
        .expect("set_bool failed to set bool specifier.");
    assert_eq!(
        builder.saw_bool, true,
        "set_bool failed to set saw_bool to true."
    );
    assert_eq!(
        builder.set_bool(),
        Err(String::from("duplicate `_Bool`")),
        "set_bool should fail when saw_bool is already true."
    )
}

#[test]
fn test_set_bool_invalid() {
    let mut builder = tagged_builder!();
    assert_eq!(
        builder.set_bool(),
        Err(String::from(
            "cannot combine `_Bool` with a struct/union/enum/typedef name",
        )),
        "set_bool should fail when tagged_or_named is set."
    );
}

#[test]
fn test_add_base_valid() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_base(BaseType::Int)
        .expect("add_base failed to add base type.");
    assert_eq!(
        builder.base,
        Some(BaseType::Int),
        "add_base failed to set base type."
    );
}

#[test]
fn test_add_base_double() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_base(BaseType::Int)
        .expect("add_base failed to add base type.");
    assert_eq!(
        builder.base,
        Some(BaseType::Int),
        "add_base failed to set base type."
    );
    assert_eq!(
        builder.add_base(BaseType::Int),
        Err(String::from("duplicate base type specifier")),
        "add_base should fail when base type is already set."
    )
}

#[test]
fn test_add_base_different() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_base(BaseType::Int)
        .expect("add_base failed to add base type.");
    assert_eq!(
        builder.base,
        Some(BaseType::Int),
        "add_base failed to set base type."
    );
    assert_eq!(
        builder.add_base(BaseType::Float),
        Err(String::from("conflicting base type specifiers")),
        "add_base should fail when base type is already set."
    )
}

#[test]
fn test_add_base_invalid() {
    let mut builder = tagged_builder!();
    assert_eq!(
        builder.add_base(BaseType::Float),
        Err(String::from(
            "cannot combine a type specifier with a struct/union/enum/typedef name",
        )),
    )
}

#[test]
fn test_add_complex_valid() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_complex(Complex::Complex)
        .expect("add_complex failed to add complex type.");
    assert_eq!(
        builder.complex,
        Some(Complex::Complex),
        "add_complex failed to set complex type."
    );
}

#[test]
fn test_add_complex_double() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_complex(Complex::Imaginary)
        .expect("add_complex failed to add complex type.");
    assert_eq!(
        builder.complex,
        Some(Complex::Imaginary),
        "add_complex failed to set complex type."
    );
    assert_eq!(
        builder.add_complex(Complex::Complex),
        Err(String::from("duplicate `_Complex`/`_Imaginary`")),
        "add_complex should fail when complex type is already set."
    )
}

#[test]
fn test_add_complex_invalid() {
    let mut builder = tagged_builder!();
    assert_eq!(
        builder.add_complex(Complex::Complex),
        Err(String::from(
            "cannot combine `_Complex`/`_Imaginary` with a struct/union/enum/typedef name",
        )),
    )
}

#[parameterized(builder = {
    TypeExprBuilder::new(TypeExprContext::Declaration),
    TypeExprBuilder{
        sign: Some(Sign::Signed),
        ..Default::default()
    },
    TypeExprBuilder{
        short_count: 1,
        ..Default::default()
    },
    TypeExprBuilder{
        long_count: 1,
        ..Default::default()
    },
    TypeExprBuilder{
        base: Some(BaseType::Int),
        ..Default::default()
    },
    TypeExprBuilder{
        complex: Some(Complex::Complex),
        ..Default::default()
    },
    TypeExprBuilder{
        saw_void: true,
        ..Default::default()
    },
    TypeExprBuilder{
        saw_bool: true,
        ..Default::default()
    }
}, expected = {
    false,
    true,
    true,
    true,
    true,
    true,
    true,
    true,
}, test_name = {
    "Default builder with no arithmetic specifiers",
    "builder with sign specifier",
    "builder with short specifier",
    "builder with long specifier",
    "builder with base type set",
    "builder with complex specifier",
    "builder with void specifier",
    "builder with bool specifier",
})]
fn test_has_any_arith_axis(builder: TypeExprBuilder, expected: bool, test_name: &'static str) {
    assert_eq!(
        builder.has_any_arith_axis(),
        expected,
        "has_any_arith_axis failed for test case: {}",
        test_name
    );
}

#[test]
fn test_set_tagged_or_named_valid() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .set_tagged_or_named(TypeSpec::Void)
        .expect("set_tagged_or_named failed to set tagged_or_named.");
    assert_eq!(
        builder.tagged_or_named,
        Some(TypeSpec::Void),
        "set_tagged_or_named failed to set tagged_or_named."
    );
}

#[test]
fn test_set_tagged_or_named_double() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .set_tagged_or_named(TypeSpec::Void)
        .expect("set_tagged_or_named failed to set tagged_or_named.");
    assert_eq!(
        builder.tagged_or_named,
        Some(TypeSpec::Void),
        "first set_tagged_or_named failed to set tagged_or_named."
    );
    assert_eq!(
        builder.set_tagged_or_named(TypeSpec::Named(String::from("TypeDef"))),
        Err(String::from("more than one type specifier")),
        "set_tagged_or_named should fail when tagged_or_named is already set."
    )
}

#[test]
fn test_set_tagged_or_named_invalid() {
    let mut builder = TypeExprBuilder {
        sign: Some(Sign::Signed),
        ..Default::default()
    };
    assert_eq!(
        builder.set_tagged_or_named(TypeSpec::Named(String::from("TypeDef"))),
        Err(String::from(
            "cannot combine a struct/union/enum/typedef name with arithmetic type specifiers"
        )),
        "set_tagged_or_named should fail when tagged_or_named is already set."
    )
}

#[test]
fn test_add_qualifier_valid() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder.add_qualifier(TypeQualifier::Const);
    assert_eq!(
        builder.qualifiers,
        vec![TypeQualifier::Const],
        "add_qualifier failed to add qualifier."
    );
}

#[test]
fn test_add_qualifier_double() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder.add_qualifier(TypeQualifier::Const);
    assert_eq!(
        builder.qualifiers,
        vec![TypeQualifier::Const],
        "first add_qualifier failed to add qualifier."
    );
    builder.add_qualifier(TypeQualifier::Const);
    assert_eq!(
        builder.qualifiers,
        vec![TypeQualifier::Const],
        "add_qualifier should not change qualifier vector."
    );
}

#[test]
fn test_add_qualifier_different() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder.add_qualifier(TypeQualifier::Const);
    assert_eq!(
        builder.qualifiers,
        vec![TypeQualifier::Const],
        "first add_qualifier failed to add qualifier."
    );
    builder.add_qualifier(TypeQualifier::Volatile);
    assert_eq!(
        builder.qualifiers,
        vec![TypeQualifier::Const, TypeQualifier::Volatile],
        "second add_qualifier failed to add qualifier."
    );
}

#[test]
fn test_add_function_specifier_valid() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_function_specifier(FunctionSpecifier::Inline)
        .expect("add_function_specifier failed to add function specifier.");
    assert_eq!(
        builder.function_specifiers,
        vec![FunctionSpecifier::Inline],
        "add_function_specifier failed to add function_specifier."
    );
}

#[test]
fn test_add_function_specifier_double() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_function_specifier(FunctionSpecifier::Inline)
        .expect("add_function_specifier failed to add function specifier.");
    assert_eq!(
        builder.function_specifiers,
        vec![FunctionSpecifier::Inline],
        "first add_function_specifier failed to add function_specifier."
    );
    builder
        .add_function_specifier(FunctionSpecifier::Inline)
        .expect("add_function_specifier failed to add function specifier.");
    assert_eq!(
        builder.function_specifiers,
        vec![FunctionSpecifier::Inline],
        "add_function_specifier should not change function specifier vector."
    );
}

#[test]
fn test_add_function_specifier_different() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .add_function_specifier(FunctionSpecifier::NoReturn)
        .expect("add_function_specifier failed to add function specifier.");
    assert_eq!(
        builder.function_specifiers,
        vec![FunctionSpecifier::NoReturn],
        "first add_function_specifier failed to add function_specifier."
    );
    builder
        .add_function_specifier(FunctionSpecifier::Inline)
        .expect("add_function_specifier failed to add function specifier.");
    assert_eq!(
        builder.function_specifiers,
        vec![FunctionSpecifier::NoReturn, FunctionSpecifier::Inline],
        "second add_function_specifier failed to add function_specifier."
    );
}

#[test]
fn test_add_function_specifier_invalid_context() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Parameter);
    assert_eq!(
        builder.add_function_specifier(FunctionSpecifier::Inline),
        Err(String::from(
            "function specifiers (`inline`/`_Noreturn`) are not allowed on a parameter"
        )),
    )
}

#[test]
fn test_set_alignment_valid() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .set_alignment(AlignmentSpecifier::Expr(Box::new(Expr::Ident(
            String::from("alignment"),
        ))))
        .expect("set_alignment failed to add function specifier.");
    assert_eq!(
        builder.alignment,
        Some(AlignmentSpecifier::Expr(Box::new(Expr::Ident(
            String::from("alignment")
        )))),
        "set_alignment failed to add function_specifier."
    );
}

#[test]
fn test_set_alignment_multiple() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
    builder
        .set_alignment(AlignmentSpecifier::Expr(Box::new(Expr::Ident(
            String::from("alignment"),
        ))))
        .expect("set_alignment failed to add function specifier.");
    assert_eq!(
        builder.alignment,
        Some(AlignmentSpecifier::Expr(Box::new(Expr::Ident(
            String::from("alignment")
        )))),
        "first set_alignment failed to add function_specifier."
    );
    assert_eq!(
        builder.set_alignment(AlignmentSpecifier::Expr(Box::new(Expr::Ident(
            String::from("alignment the second")
        )))),
        Err(String::from("multiple `_Alignas` specifiers")),
        "set_alignment should fail when alignment is already set."
    );
}

#[test]
fn test_set_alignment_invalid_context() {
    let mut builder = TypeExprBuilder::new(TypeExprContext::TypeName);
    assert_eq!(
        builder.set_alignment(AlignmentSpecifier::TypeName(Box::new(TypeName {
            type_expr: TypeExpr {
                storage: None,
                thread_local: false,
                type_spec: TypeSpec::Void,
                qualifiers: vec![],
                function_specifiers: vec![],
                alignment: None,
            },
            derived: Declarator::Abstract,
        }))),
        Err(String::from("`_Alignas` is not allowed in a type name")),
        "set_alignment should fail when context is TypeName"
    )
}

#[parameterized(parameters = {
    (0, 0),
    (1, 0),
    (0, 1),
    (0, 2),
    (2, 0),
    (0, 3),
    (1, 1)
}, expected = {
    Ok(SizeSpec::None),
    Ok(SizeSpec::Short),
    Ok(SizeSpec::Long),
    Ok(SizeSpec::LongLong),
    Err(String::from("`short short` is too short")),
    Err(String::from("`long long long` is too long")),
    Err(String::from("`short` and `long` cannot be combined")),
}, test_name = {
    "no size specifiers",
    "short",
    "long",
    "long long",
    "too short",
    "too long",
    "short and long combined"
}

)]
fn test_resolve_size(
    parameters: (u32, u32),
    expected: Result<SizeSpec, String>,
    test_name: &'static str,
) {
    let result = resolve_size(parameters.0, parameters.1);
    assert_eq!(
        result, expected,
        "resolve_size failed for test case: {}",
        test_name
    );
}

#[parameterized(parameters = {
    (BaseType::Int, SizeSpec::None),
    (BaseType::Char, SizeSpec::None),
    (BaseType::Double, SizeSpec::None),
    (BaseType::Float, SizeSpec::None),
    (BaseType::Double, SizeSpec::Long),
    (BaseType::Char, SizeSpec::Short),
    (BaseType::Float, SizeSpec::Short),
    (BaseType::Double, SizeSpec::Short),
}, expected = {
    Ok(()),
    Ok(()),
    Ok(()),
    Ok(()),
    Ok(()),
    Err(String::from("`char` cannot be `short` or `long`")),
    Err(String::from("`float` cannot be `short` or `long`")),
    Err(String::from("only `long double` is valid among sized doubles")),
}, test_name = {
    "int with no size specifier",
    "char with no size specifier",
    "double with no size specifier",
    "float with no size specifier",
    "long double",
    "short char",
    "short float",
    "short double"
})]
fn test_validate_base_size(
    parameters: (BaseType, SizeSpec),
    expected: Result<(), String>,
    test_name: &'static str,
) {
    let result = validate_base_size(&parameters.0, &parameters.1);
    assert_eq!(
        result, expected,
        "validate_base_size failed for test case: {}",
        test_name
    );
}

#[parameterized(parameters = {
    (BaseType::Int, Some(Sign::Signed)),
    (BaseType::Char, Some(Sign::Unsigned)),
    (BaseType::Float, Some(Sign::Signed)),
    (BaseType::Double, Some(Sign::Unsigned)),
    (BaseType::Float, None),
}, expected = {
    Ok(()),
    Ok(()),
    Err(String::from("floating types cannot be `signed` or `unsigned`")),
    Err(String::from("floating types cannot be `signed` or `unsigned`")),
    Ok(()),
}, test_name = {
    "int with signed",
    "char with unsigned",
    "float with signed",
    "double with unsigned",
    "float with no sign"
}
)]
fn test_validate_base_sign(
    parameters: (BaseType, Option<Sign>),
    expected: Result<(), String>,
    test_name: &'static str,
) {
    let result = validate_base_sign(&parameters.0, &parameters.1);
    assert_eq!(
        result, expected,
        "validate_base_sign failed for test case: {}",
        test_name
    );
}

#[parameterized(parameters = {
    (BaseType::Float, Some(Complex::Complex)),
    (BaseType::Double, Some(Complex::Imaginary)),
    (BaseType::Int, Some(Complex::Complex)),
    (BaseType::Char, Some(Complex::Imaginary)),
    (BaseType::Float, None),
}, expected = {
    Ok(()),
    Ok(()),
    Err(String::from("`_Complex`/`_Imaginary` require a floating base type")),
    Err(String::from("`_Complex`/`_Imaginary` require a floating base type")),
    Ok(()),
}, test_name = {
    "float with complex",
    "double with imaginary",
    "int with complex",
    "char with imaginary",
    "float with no complex nor imaginary"
}
)]
fn test_validate_base_complex(
    parameters: (BaseType, Option<Complex>),
    expected: Result<(), String>,
    test_name: &'static str,
) {
    let result = validate_base_complex(&parameters.0, &parameters.1);
    assert_eq!(
        result, expected,
        "validate_base_sign failed for test case: {}",
        test_name
    );
}

#[parameterized(builder = {
    TypeExprBuilder {
        saw_void: true,
        ..Default::default()
    },
    TypeExprBuilder {
        saw_void: true,
        sign: Some(Sign::Unsigned),
        ..Default::default()
    },
    TypeExprBuilder {
        saw_void: true,
        short_count: 1,
        ..Default::default()
    },
    TypeExprBuilder {
        saw_void: true,
        long_count: 1,
        ..Default::default()
    },
    TypeExprBuilder {
        saw_void: true,
        base: Some(BaseType::Int),
        ..Default::default()
    },
    TypeExprBuilder {
        saw_void: true,
        complex: Some(Complex::Complex),
        ..Default::default()
    },
    TypeExprBuilder {
        saw_void: true,
        saw_bool: true,
        ..Default::default()
    },
}, expected = {
    false,
    true,
    true,
    true,
    true,
    true,
    true,
}, test_name = {
    "valid",
    "unsigned",
    "short",
    "long",
    "int",
    "_Complex",
    "_Bool"
})]
fn test_is_void_only(builder: TypeExprBuilder, expected: bool, test_name: &'static str) {
    assert_eq!(
        builder.is_void_only(),
        expected,
        "is_void_only test case {} failed",
        test_name
    )
}

#[parameterized(builder = {
    TypeExprBuilder {
        saw_bool: true,
        ..Default::default()
    },
    TypeExprBuilder {
        saw_bool: true,
        sign: Some(Sign::Unsigned),
        ..Default::default()
    },
    TypeExprBuilder {
        saw_bool: true,
        short_count: 1,
        ..Default::default()
    },
    TypeExprBuilder {
        saw_bool: true,
        long_count: 1,
        ..Default::default()
    },
    TypeExprBuilder {
        saw_bool: true,
        base: Some(BaseType::Int),
        ..Default::default()
    },
    TypeExprBuilder {
        saw_bool: true,
        complex: Some(Complex::Complex),
        ..Default::default()
    },
    TypeExprBuilder {
        saw_bool: true,
        saw_void: true,
        ..Default::default()
    },
}, expected = {
    false,
    true,
    true,
    true,
    true,
    true,
    true,
}, test_name = {
    "valid",
    "unsigned",
    "short",
    "long",
    "int",
    "_Complex",
    "void"
})]
fn test_is_bool_only(builder: TypeExprBuilder, expected: bool, test_name: &'static str) {
    assert_eq!(
        builder.is_bool_only(),
        expected,
        "is_bool_only test case {} failed",
        test_name
    )
}

#[parameterized(builder = {

}, expected = {

}, test_name = {

})]
fn test_resolve_type_spec(
    builder: TypeExprBuilder,
    expected: Result<TypeSpec, String>,
    test_name: &'static str,
) {
    assert_eq!(builder.resolve_type_spec(), expected, "{}", test_name)
}
