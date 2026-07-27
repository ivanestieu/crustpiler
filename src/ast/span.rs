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

    pub fn to_string(&self) -> String {
        format!("{}..{}", self.start, self.end)
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
