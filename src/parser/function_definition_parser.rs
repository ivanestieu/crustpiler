use crate::ast::function_def::FunctionDef;
use crate::parse_error;
use crate::parser::errors::ParseError;
use crate::parser::parser::Parser;

impl Parser {
    pub(super) fn parse_function_def(&mut self) -> Result<FunctionDef, ParseError> {
        Err(parse_error!(
            "Function definition parsing not implemented yet"
        ))
    }
}
