pub const ERROR_INVALID_LEXEME: u32 = 1;
pub const ERROR_INVALID_CHARACTER_LITERAL: u32 = 2;
pub const ERROR_INVALID_NUMBER_LITERAL: u32 = 3;
pub const ERROR_INVALID_KEYWORD: u32 = 4;
pub const ERROR_INVALID_INDENTIFIER: u32 = 5;
pub const ERROR_UNEXPECTED_END_OF_STREAM: u32 = 6;
pub const ERROR_INVALID_LITERAL: u32 = 7;
pub const ERROR_INVALID_OPERATOR: u32 = 8;
pub const ERROR_TYPE_MISMATCH: u32 = 9;
pub const ERROR_UNMATCHED_DELIMITER: u32 = 10;
pub const ERROR_UNEXPECTED_END_OF_FILE: u32 = 11;
pub const ERROR_UNKNOWN_VARIABLE: u32 = 12;
pub const ERROR_EXPECTED: u32 = 13;
pub const ERROR_UNINITIALISED_VARIABLE: u32 = 14;
pub const ERROR_NULL_VARIABLE_EXPRESSION: u32 = 15;

use crate::lex::span::{Span, Spans};

#[derive(Clone)]
pub struct LangError {
    number: u32,
    span: Span,
    message: String,
}

impl LangError {
    pub fn from(message: String, span: impl Spans, number: u32) -> Self {
        Self {
            number,
            span: span.span(),
            message,
        }
    }
}

impl std::fmt::Debug for LangError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[E{}] {}\n\t from {} to {}",
            self.number,
            self.message,
            self.span.span().0,
            self.span.span().1
        )
    }
}
