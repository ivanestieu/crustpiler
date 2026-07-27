use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub base_message : String,
    pub context : Vec<String>
}

impl ParseError {
    pub fn new(base_message : String) -> Self {
        Self { base_message, context : Default::default() }
    }

    fn push_context(mut self, call: &str) -> Self {
        self.context.push(call.to_string());
        self
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
        for context in &self.context {
            writeln!(f, "{}{}: ", "  ".repeat(ident), context)?;
            ident += 1;
        }
        writeln!(f, "{}{}", "  ".repeat(ident), self.base_message)?;
        Ok(())
    }
}

impl std::error::Error for ParseError {}

pub trait Contextualize<T> {
    fn on_err_context(self, context: &str) -> Result<T, ParseError>;
}

impl<T> Contextualize<T> for Result<T, ParseError> {
fn on_err_context(self, context: &str) -> Self {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(err.push_context(context)),
        }
    }
}

impl<T> Contextualize<T> for Result<T, String> {
    fn on_err_context(self, _context: &str) -> Result<T, ParseError> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(
                parse_error!("{}", err)
            ),
        }
    }
}