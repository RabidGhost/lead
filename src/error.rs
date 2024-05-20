// pub trait LangError: std::fmt::Debug {
//     fn report(&self);
// }

// pub mod lex_error {
//     use super::*;

pub const ERROR_INVALID_LEXEME: u32 = 1;
pub const ERROR_INVALID_CHARACTER_LITERAL: u32 = 2;
pub const ERROR_INVALID_NUMBER_LITERAL: u32 = 3;
pub const ERROR_INVALID_KEYWORD: u32 = 4;
pub const ERROR_INVALID_INDENTIFIER: u32 = 5;

pub struct LangError {
    number: u32,
    span: (usize, usize),
    message: String,
}

impl LangError {
    pub fn from_null_span(number: u32, message: String) -> Self {
        Self {
            number,
            span: (0, 0),
            message,
        }
    }

    pub fn from_message(message: String) -> Self {
        Self {
            number: 0,
            span: (0, 0),
            message,
        }
    }

    pub fn from(message: String, span: (usize, usize), number: u32) -> Self {
        Self {
            number,
            span,
            message,
        }
    }
}

impl std::fmt::Debug for LangError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[E{}] {}\n\t from {} to {}",
            self.number, self.message, self.span.0, self.span.1
        )
    }
}
