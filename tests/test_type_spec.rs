#[cfg(test)]
pub mod type_spec_tests {
    use crustpiler::ast::decl_specifiers::{TypeExprBuilder, TypeExprContext};
    use crustpiler::ast::types::*;

    fn arith(spec: TypeSpec) -> ArithType {
        match spec {
            TypeSpec::Arithmetic(a) => a,
            other => panic!("expected arithmetic, got {:?}", other),
        }
    }

    #[test]
    fn bare_int() {
        let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
        builder.add_base(BaseType::Int).unwrap();
        let type_expr = builder.finish().unwrap();
        let a = arith(type_expr.type_spec);
        assert_eq!(
            a,
            ArithType {
                sign: None,
                size: SizeSpec::None,
                base: BaseType::Int,
                complex: None,
            }
        );
    }

    #[test]
    fn unsigned_defaults_to_int() {
        let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
        builder.add_sign(Sign::Unsigned).unwrap();
        let type_expr = builder.finish().unwrap();
        let a = arith(type_expr.type_spec);
        assert_eq!(a.base, BaseType::Int);
        assert_eq!(a.sign, Some(Sign::Unsigned));
    }

    #[test]
    fn order_independent() {
        let mut a_builder = TypeExprBuilder::new(TypeExprContext::Declaration);
        a_builder.add_sign(Sign::Unsigned).unwrap();
        a_builder.add_long().unwrap();
        a_builder.add_base(BaseType::Int).unwrap();
        let a = a_builder.finish().unwrap().type_spec;

        let mut b_builder = TypeExprBuilder::new(TypeExprContext::Declaration);
        b_builder.add_base(BaseType::Int).unwrap();
        b_builder.add_sign(Sign::Unsigned).unwrap();
        b_builder.add_long().unwrap();
        let b = b_builder.finish().unwrap().type_spec;

        let mut c_builder = TypeExprBuilder::new(TypeExprContext::Declaration);
        c_builder.add_long().unwrap();
        c_builder.add_base(BaseType::Int).unwrap();
        c_builder.add_sign(Sign::Unsigned).unwrap();
        let c = c_builder.finish().unwrap().type_spec;

        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn long_long() {
        let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
        builder.add_long().unwrap();
        builder.add_long().unwrap();
        let type_expr = builder.finish().unwrap();
        let a = arith(type_expr.type_spec);
        assert_eq!(a.size, SizeSpec::LongLong);
        assert_eq!(a.base, BaseType::Int);
    }

    #[test]
    fn long_double_ok() {
        let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
        builder.add_long().unwrap();
        builder.add_base(BaseType::Double).unwrap();
        let type_expr = builder.finish().unwrap();
        let a = arith(type_expr.type_spec);
        assert_eq!(a.size, SizeSpec::Long);
        assert_eq!(a.base, BaseType::Double);
    }

    #[test]
    fn canonical_spelling() {
        let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
        builder.add_long().unwrap();
        builder.add_sign(Sign::Unsigned).unwrap();
        builder.add_base(BaseType::Int).unwrap();
        let type_expr = builder.finish().unwrap();
        let a = arith(type_expr.type_spec);
        assert_eq!(a.to_c_string(), "unsigned long int");
    }

    #[test]
    fn void_alone() {
        let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
        builder.set_void().unwrap();
        let type_expr = builder.finish().unwrap();
        assert_eq!(type_expr.type_spec, TypeSpec::Void);
    }

    // --- rejection cases ----------------------------------------------------

    #[test]
    fn rejects_signed_unsigned() {
        let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
        builder.add_sign(Sign::Signed).unwrap();
        assert!(builder.add_sign(Sign::Unsigned).is_err());
    }

    #[test]
    fn rejects_triple_long() {
        let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
        builder.add_long().unwrap();
        builder.add_long().unwrap();
        builder.add_long().unwrap();
        // The validation happens during finish, not add_long
        assert!(builder.finish().is_err());
    }

    #[test]
    fn rejects_short_long() {
        let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
        builder.add_short().unwrap();
        builder.add_long().unwrap();
        assert!(builder.finish().is_err());
    }

    #[test]
    fn rejects_short_char() {
        let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
        builder.add_short().unwrap();
        builder.add_base(BaseType::Char).unwrap();
        assert!(builder.finish().is_err());
    }

    #[test]
    fn rejects_unsigned_double() {
        let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
        builder.add_sign(Sign::Unsigned).unwrap();
        builder.add_base(BaseType::Double).unwrap();
        assert!(builder.finish().is_err());
    }

    #[test]
    fn rejects_int_char() {
        let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
        builder.add_base(BaseType::Int).unwrap();
        assert!(builder.add_base(BaseType::Char).is_err());
    }

    #[test]
    fn rejects_void_with_int() {
        let mut builder = TypeExprBuilder::new(TypeExprContext::Declaration);
        builder.set_void().unwrap();
        builder.add_base(BaseType::Int).unwrap(); // This might not error immediately
        // But finish should reject the combination
        assert!(builder.finish().is_err());
    }
}
