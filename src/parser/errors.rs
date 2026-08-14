use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    span: Option<(usize, usize)>,
    pub base_message: String,
    pub context: Vec<(String, String)>,
}

impl ParseError {
    pub fn new(base_message: String) -> Self {
        Self {
            base_message,
            span: None,
            context: Default::default(),
        }
    }

    fn push_context(mut self, call: &str, message: &str) -> Self {
        self.context.push((call.to_string(), message.to_string()));
        self
    }

    pub(crate) fn span(mut self, start: usize, end: usize) -> Self {
        self.span = Some((start, end));
        self
    }

    pub fn print_span(&self, source: &str) -> () {
        if self.span.is_none() {
            return;
        }
        let (start, end) = self.span.unwrap();
        let mut counter = 0;
        for line in source.lines() {
            if counter + 1 + line.len() > start {
                let span_start = start - counter;
                eprintln!(
                    "{}\n{}{} {}",
                    line,
                    " ".repeat(span_start),
                    "^".repeat(end - start),
                    self.base_message
                );

                break;
            }
            counter += line.len() + 1;
        }
    }
}
#[macro_export]
macro_rules! parse_error {
    ($($arg:tt)*) => {
        ParseError::new(format!($($arg)*))
    };
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut ident = 0;
        writeln!(f, "")?;
        for context in self.context.iter().rev() {
            writeln!(f, "{}{}: {}", "  ".repeat(ident), context.0, context.1)?;
            ident += 1;
        }
        writeln!(f, "{}{}", "  ".repeat(ident), self.base_message)?;
        Ok(())
    }
}

impl std::error::Error for ParseError {}

pub trait Contextualize<T> {
    fn on_err_context(self, callee: &str, context: &str) -> Result<T, ParseError>;
}

impl<T> Contextualize<T> for Result<T, ParseError> {
    fn on_err_context(self, callee: &str, context: &str) -> Self {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(err.push_context(callee, context)),
        }
    }
}

impl<T> Contextualize<T> for Result<T, String> {
    fn on_err_context(self, callee: &str, context: &str) -> Result<T, ParseError> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(parse_error!("{}", err).push_context(callee, context)),
        }
    }
}
