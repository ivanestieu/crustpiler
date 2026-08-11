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
    pub unsigned: bool, // u / U
    pub long: LongKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LongKind {
    None,
    Long,     // l / L
    LongLong, // ll / LL
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloatLit {
    pub value: f64,
    pub suffix: FloatSuffix,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FloatSuffix {
    Double,     // no suffix
    Float,      // f / F
    LongDouble, // l / L
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringLit {
    pub value: String, // decoded content
    pub prefix: StringPrefix,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPrefix {
    None,  // "..."
    Wide,  // L"..."
    Utf8,  // u8"..."
    Utf16, // u"..."
    Utf32, // U"..."
}
