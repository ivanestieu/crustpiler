// =============================================================================
// type_spec.rs — accumulate-then-validate parsing of arithmetic type specifiers
//
// C lets specifiers appear in any order and across multiple words:
//   unsigned long int   ==   long unsigned   ==   int unsigned long
// So we scan keyword-by-keyword, tally the axes, then validate the combination.
//
// This module is parser-agnostic: it works on a slice of TypeKeyword tokens so
// it is trivially unit-testable. In your real parser you would feed it from the
// token stream instead.
// =============================================================================

use crate::ast::ast::*;

/// The subset of tokens relevant to a type-specifier sequence.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeKeyword {
    Signed,
    Unsigned,
    Short,
    Long,
    Int,
    Char,
    Float,
    Double,
    Void,
    Bool,
}

/// Accumulator for the three arithmetic axes plus the non-arithmetic bases.
#[derive(Default)]
struct Acc {
    sign: Option<Sign>,
    short_count: u32,
    long_count: u32,
    base: Option<BaseAcc>,
    saw_void: bool,
    saw_bool: bool,
}

#[derive(PartialEq, Clone, Copy)]
enum BaseAcc {
    Int,
    Char,
    Float,
    Double,
}

/// Parse a sequence of type keywords into a validated TypeSpec.
/// Returns Err with a human-readable message for invalid combinations.
pub fn resolve_type_spec(keywords: &[TypeKeyword]) -> Result<TypeSpec, String> {
    let mut acc = Acc::default();

    for kw in keywords {
        match kw {
            TypeKeyword::Signed => set_sign(&mut acc, Sign::Signed)?,
            TypeKeyword::Unsigned => set_sign(&mut acc, Sign::Unsigned)?,
            TypeKeyword::Short => acc.short_count += 1,
            TypeKeyword::Long => acc.long_count += 1,
            TypeKeyword::Int => set_base(&mut acc, BaseAcc::Int)?,
            TypeKeyword::Char => set_base(&mut acc, BaseAcc::Char)?,
            TypeKeyword::Float => set_base(&mut acc, BaseAcc::Float)?,
            TypeKeyword::Double => set_base(&mut acc, BaseAcc::Double)?,
            TypeKeyword::Void => acc.saw_void = true,
            TypeKeyword::Bool => acc.saw_bool = true,
        }
    }

    // --- Non-arithmetic primitives short-circuit, with conflict checks -------
    if acc.saw_void {
        if acc.sign.is_some()
            || acc.short_count > 0
            || acc.long_count > 0
            || acc.base.is_some()
            || acc.saw_bool
        {
            return Err("'void' cannot combine with other type specifiers".into());
        }
        return Ok(TypeSpec::Void);
    }
    if acc.saw_bool {
        if acc.sign.is_some() || acc.short_count > 0 || acc.long_count > 0 || acc.base.is_some() {
            return Err("'_Bool' cannot combine with other type specifiers".into());
        }
        return Ok(TypeSpec::Bool);
    }

    // --- Resolve the size axis from short/long counts ------------------------
    let size = resolve_size(acc.short_count, acc.long_count)?;

    // --- Resolve the base, applying defaults and per-base legality -----------
    let base = match acc.base {
        Some(BaseAcc::Int) | None => BaseType::Int, // bare `unsigned`/`long` ⇒ int
        Some(BaseAcc::Char) => BaseType::Char,
        Some(BaseAcc::Float) => BaseType::Float,
        Some(BaseAcc::Double) => BaseType::Double,
    };

    // Cross-axis legality:
    validate_base_size(&base, &size)?;
    validate_base_sign(&base, &acc.sign)?;

    Ok(TypeSpec::Arithmetic(ArithType { sign: acc.sign, size, base }))
}

fn set_sign(acc: &mut Acc, sign: Sign) -> Result<(), String> {
    match &acc.sign {
        Some(existing) if *existing != sign => {
            Err("conflicting 'signed' and 'unsigned'".into())
        }
        Some(_) => Err("duplicate sign specifier".into()),
        None => {
            acc.sign = Some(sign);
            Ok(())
        }
    }
}

fn set_base(acc: &mut Acc, base: BaseAcc) -> Result<(), String> {
    match &acc.base {
        Some(existing) if *existing != base => {
            Err("conflicting base type specifiers (e.g. 'int' and 'char')".into())
        }
        Some(_) => Err("duplicate base type specifier".into()),
        None => {
            acc.base = Some(base);
            Ok(())
        }
    }
}

fn resolve_size(short_count: u32, long_count: u32) -> Result<SizeSpec, String> {
    match (short_count, long_count) {
        (0, 0) => Ok(SizeSpec::None),
        (1, 0) => Ok(SizeSpec::Short),
        (0, 1) => Ok(SizeSpec::Long),
        (0, 2) => Ok(SizeSpec::LongLong),
        (s, 0) if s > 1 => Err("'short short' is not a valid type".into()),
        (0, l) if l > 2 => Err("'long long long' is too long".into()),
        _ => Err("'short' and 'long' cannot be combined".into()),
    }
}

fn validate_base_size(base: &BaseType, size: &SizeSpec) -> Result<(), String> {
    match base {
        // char has no size variants
        BaseType::Char if *size != SizeSpec::None => {
            Err("'char' cannot be 'short' or 'long'".into())
        }
        // float has no size variants
        BaseType::Float if *size != SizeSpec::None => {
            Err("'float' cannot be 'short' or 'long'".into())
        }
        // double allows only `long double`
        BaseType::Double => match size {
            SizeSpec::None | SizeSpec::Long => Ok(()),
            _ => Err("only 'long double' is valid among sized doubles".into()),
        },
        _ => Ok(()),
    }
}

fn validate_base_sign(base: &BaseType, sign: &Option<Sign>) -> Result<(), String> {
    match base {
        // float/double have no signed/unsigned forms
        BaseType::Float | BaseType::Double if sign.is_some() => {
            Err("floating types cannot be 'signed' or 'unsigned'".into())
        }
        _ => Ok(()),
    }
}