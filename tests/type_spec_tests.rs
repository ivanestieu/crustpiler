#[cfg(test)]
pub mod type_spec_tests {
    use criterion_to_rust::ast::ast::*;
    use criterion_to_rust::ast::type_spec::*;

    fn arith(spec: TypeSpec) -> ArithType {
        match spec {
            TypeSpec::Arithmetic(a) => a,
            other => panic!("expected arithmetic, got {:?}", other),
        }
    }

    #[test]
    fn bare_int() {
        let a = arith(resolve_type_spec(&[TypeKeyword::Int]).unwrap());
        assert_eq!(a, ArithType { sign: None, size: SizeSpec::None, base: BaseType::Int });
    }

    #[test]
    fn unsigned_defaults_to_int() {
        let a = arith(resolve_type_spec(&[TypeKeyword::Unsigned]).unwrap());
        assert_eq!(a.base, BaseType::Int);
        assert_eq!(a.sign, Some(Sign::Unsigned));
    }

    #[test]
    fn order_independent() {
        let a = resolve_type_spec(&[TypeKeyword::Unsigned, TypeKeyword::Long, TypeKeyword::Int]).unwrap();
        let b = resolve_type_spec(&[TypeKeyword::Int, TypeKeyword::Unsigned, TypeKeyword::Long]).unwrap();
        let c = resolve_type_spec(&[TypeKeyword::Long, TypeKeyword::Int, TypeKeyword::Unsigned]).unwrap();
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn long_long() {
        let a = arith(resolve_type_spec(&[TypeKeyword::Long, TypeKeyword::Long]).unwrap());
        assert_eq!(a.size, SizeSpec::LongLong);
        assert_eq!(a.base, BaseType::Int);
    }

    #[test]
    fn long_double_ok() {
        let a = arith(resolve_type_spec(&[TypeKeyword::Long, TypeKeyword::Double]).unwrap());
        assert_eq!(a.size, SizeSpec::Long);
        assert_eq!(a.base, BaseType::Double);
    }

    #[test]
    fn canonical_spelling() {
        let a = arith(resolve_type_spec(&[TypeKeyword::Long, TypeKeyword::Unsigned, TypeKeyword::Int]).unwrap());
        assert_eq!(a.to_c_string(), "unsigned long int");
    }

    #[test]
    fn void_alone() {
        assert_eq!(resolve_type_spec(&[TypeKeyword::Void]).unwrap(), TypeSpec::Void);
    }

    // --- rejection cases ----------------------------------------------------

    #[test]
    fn rejects_signed_unsigned() {
        assert!(resolve_type_spec(&[TypeKeyword::Signed, TypeKeyword::Unsigned]).is_err());
    }

    #[test]
    fn rejects_triple_long() {
        assert!(resolve_type_spec(&[TypeKeyword::Long, TypeKeyword::Long, TypeKeyword::Long]).is_err());
    }

    #[test]
    fn rejects_short_long() {
        assert!(resolve_type_spec(&[TypeKeyword::Short, TypeKeyword::Long]).is_err());
    }

    #[test]
    fn rejects_short_char() {
        assert!(resolve_type_spec(&[TypeKeyword::Short, TypeKeyword::Char]).is_err());
    }

    #[test]
    fn rejects_unsigned_double() {
        assert!(resolve_type_spec(&[TypeKeyword::Unsigned, TypeKeyword::Double]).is_err());
    }

    #[test]
    fn rejects_int_char() {
        assert!(resolve_type_spec(&[TypeKeyword::Int, TypeKeyword::Char]).is_err());
    }

    #[test]
    fn rejects_void_with_int() {
        assert!(resolve_type_spec(&[TypeKeyword::Void, TypeKeyword::Int]).is_err());
    }
}
