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

//     // pub struct BufFullOnPushErr {
//     //     string: String,
//     //     line: usize,
//     //     col: usize,
//     //     length: usize,
//     // }

//     // impl BufFullOnPushErr {
//     //     pub fn new(string: String, line: usize, col: usize, length: usize) -> Self {
//     //         Self { string, line, col, length }
//     //     }
//     // }

//     #[derive(Debug)]
//     pub struct InvalidLexemeErr {
//         string: String,
//         line: usize,
//         col: usize,
//         length: usize,
//     }

//     impl InvalidLexemeErr {
//         pub fn new(string: String, line: usize, col: usize, length: usize) -> Self {
//             Self {
//                 string,
//                 line,
//                 col,
//                 length,
//             }
//         }
//     }
// }

// pub mod parse_error {
//     //use crate::parse::ast::{LangExpression, LangType};
//     use crate::lex::token::Token;

//     use super::LangError;

//     #[derive(Debug)]
//     pub struct MixedTypeExpressionError {
//         left: LangType,
//         right: LangType,
//     }
//     // will need more detail in future

//     impl MixedTypeExpressionError {
//         pub fn new(left: LangType, right: LangType) -> Self {
//             Self { left, right }
//         }
//     }

//     impl LangError for MixedTypeExpressionError {
//         fn report(&self) {
//             todo!()
//         }
//     }

//     #[derive(Debug)]
//     pub struct NoTokensOnParseErr;
//     #[derive(Debug)]
//     pub struct InvalidLiteral {
//         literal: Token,
//     }

//     impl InvalidLiteral {
//         pub fn new(literal: Token) -> Self {
//             InvalidLiteral { literal }
//         }
//     }

//     impl LangError for InvalidLiteral {
//         fn report(&self) {
//             todo!()
//         }
//     }

//     impl LangError for NoTokensOnParseErr {
//         fn report(&self) {
//             todo!()
//         }
//     }

//     #[derive(Debug)]
//     pub struct InvalidUnaryOperatorErr {
//         op: Token,
//     }

//     impl InvalidUnaryOperatorErr {
//         pub fn new(op: Token) -> Self {
//             Self { op }
//         }
//     }

//     impl LangError for InvalidUnaryOperatorErr {
//         fn report(&self) {
//             todo!()
//         }
//     }

//     // use std::marker::PhantomData;

//     // use crate::lex::token::Token;
//     // use crate::parse::ast::Boolean;

//     // struct BooleanExpressionParseErr<T: Boolean> {
//     //     tokens: Vec<Token>,
//     //     line: usize,
//     //     col: usize,
//     //     length: usize,
//     //     _ast_type: PhantomData<T>,
//     // }
// }

// // impl LangError for lex_error::BufFullOnPushErr {
// //     fn report(&self) {
// //         todo!()
// //     }
// // }

// impl LangError for lex_error::InvalidLexemeErr {
//     fn report(&self) {
//         todo!()
//     }
// }
