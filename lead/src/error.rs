use crate::lex::{span::Span, token::TokenType};
use miette::{Diagnostic, Report};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum LangError {
    #[error("unknown lexeme `{lexeme}`")]
    InvalidLexeme {
        #[label]
        span: Span,
        lexeme: String,
    },
    #[error("invalid character literal `{char_literal}`")]
    InvalidCharacterLiteral {
        #[label]
        span: Span,
        char_literal: String,
    },
    #[error("invalid integer literal `{num_literal}`")]
    InvalidIntegerLiteral {
        #[label]
        span: Span,
        num_literal: String,
    },
    #[error("invalid identifier name `{id_literal}`")]
    #[diagnostic(help("identifiers must begin with a letter, and can contain any other combination of english letters, digits, and underscores"))]
    InvalidIdentifier {
        #[label]
        span: Span,
        id_literal: String,
    },
    #[error(
        "invalid literal `{invalid_literal}`, expected a boolean, character, or integer literal"
    )]
    InvalidLiteral {
        span: Span,
        invalid_literal: TokenType,
    },
    #[error("`{op}` is not a valid unary operator")]
    InvalidUnaryOperator {
        #[label]
        span: Span,
        op: TokenType,
    },
    #[error("`{op}` is not a valid binary operator")]
    InvalidBinaryOperator {
        #[label]
        span: Span,
        op: TokenType,
    },
    #[error("unmatched delimiter `{expected}`, found `{found}`")]
    UnmatchedDelimiter {
        span: Span,
        expected: TokenType,
        found: TokenType,
    },
    #[error("unexpected end of file, expected `{expected}`{}",
        match found {
            None => "".to_owned(),
            Some(string) => format!(", found `{string}`")
        })]
    UnexpectedEndOfFile {
        #[label]
        span: Span,
        expected: String,
        found: Option<String>,
    },
    #[error("uninitialised variable `{name}`")]
    UninitialisedVariable {
        #[label]
        span: Span,
        name: String,
    },
    #[error("uninitialised pointer to variable `{name}`")]
    UninitialisedPointer {
        #[label]
        span: Span,
        name: String,
    },

    #[error("unexpected token `{tok}`, expected {expected}")]
    UnexpectedToken {
        #[label]
        span: Span,
        tok: TokenType,
        expected: String,
    },

    #[error("expected `{expected}`, found `{found}`")]
    ExpectedToken {
        #[label]
        span: Span,
        expected: TokenType,
        found: TokenType,
    },
    #[error("found a null value expression. Expressions must always evaluate to some value")]
    NullValueExpression {
        #[label]
        span: Span,
    },
}

impl LangError {
    pub fn with_src(self, src: String) -> Report {
        <LangError as Into<Report>>::into(self).with_source_code(src)
    }
}
