use super::span::{Span, Spans};
use crate::error::{LangError, ERROR_INVALID_KEYWORD, ERROR_INVALID_LEXEME};

pub const KEYWORDS: [&'static str; 7] = ["true", "false", "let", "if", "for", "while", "yield"];

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum TokenType {
    // Single char Tokens
    LeftParen,   // (
    RightParen,  // )
    LeftBrace,   // {
    RightBrace,  // }
    LeftSquare,  // [
    RightSquare, // ]
    Comma,       // ,
    Dot,         // .
    Minus,       // -
    Plus,        // +
    Slash,       // /
    Star,        // *
    Semicolon,   // ;

    // One or two char Tokens
    LessThan,      // <
    GreaterThan,   // >
    LessThanEq,    // <=
    GreaterThanEq, // >=
    EqEq,          // ==
    Colon,         // :
    Assign,        // :=
    Bang,          // !
    BangEq,        // !=

    // Literals
    Identifier(String),
    Char(char),
    Number(u64),
    Bool(bool),

    // Keywords
    Let,
    If,
    For,
    While,
    Yield,

    // End of file
    EOF,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Token {
    token_type: TokenType,
    span: Span,
}

impl Token {
    pub fn new(token_type: TokenType, start: usize, length: usize) -> Self {
        Self {
            token_type,
            span: Span::new((start, start + length)),
        }
    }

    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn from(string: &str, start: usize) -> Result<Self, LangError> {
        let ty = match string {
            "(" => TokenType::LeftParen,
            ")" => TokenType::RightParen,
            "{" => TokenType::LeftBrace,
            "}" => TokenType::RightBrace,
            "[" => TokenType::LeftSquare,
            "]" => TokenType::RightSquare,
            "," => TokenType::Comma,
            "." => TokenType::Dot,
            "-" => TokenType::Minus,
            "+" => TokenType::Plus,
            "*" => TokenType::Star,
            ";" => TokenType::Semicolon,
            "!" => TokenType::Bang,
            "<" => TokenType::LessThan,
            ">" => TokenType::GreaterThan,
            ":" => TokenType::Colon,
            "!=" => TokenType::BangEq,
            "==" => TokenType::EqEq,
            "<=" => TokenType::LessThanEq,
            ">=" => TokenType::GreaterThanEq,
            ":=" => TokenType::Assign,
            "/" => TokenType::Slash,
            _ => {
                return Err(LangError::from(
                    format!("invalid lexeme `{string}`"),
                    (start, start + string.len()),
                    ERROR_INVALID_LEXEME,
                ))
            }
        };

        Ok(Token {
            token_type: ty,
            span: Span::new((start, start + string.len())),
        })
    }

    pub fn from_bool(b: bool, start: usize) -> Self {
        let len: usize = match b {
            true => 4,
            false => 5,
        };
        Token {
            token_type: TokenType::Bool(b),
            span: Span::new((start, start + len)),
        }
    }

    pub fn from_num(n: u64, start: usize, stop: usize) -> Self {
        Token {
            token_type: TokenType::Number(n),
            span: Span::new((start, stop)),
        }
    }

    pub fn from_keyword(string: &str, start: usize) -> Result<Self, LangError> {
        if !KEYWORDS.contains(&string) {
            Err(LangError::from(
                format!(
                    "invalid keyword `{}`, this error should be impossible",
                    string
                ),
                Span::new((start, start + string.len())),
                ERROR_INVALID_KEYWORD,
            ))
        } else {
            let ty: TokenType = match string {
                "true" => return Ok(Token::from_bool(true, start)),
                "false" => return Ok(Token::from_bool(false, start)),
                "if" => TokenType::If,
                "let" => TokenType::Let,
                "for" => TokenType::For,
                "while" => TokenType::While,
                "yield" => TokenType::Yield,
                _ => unreachable!(),
            };
            Ok(Self {
                token_type: ty,
                span: Span::new((start, start + string.len())),
            })
        }
    }
}

impl Spans for Token {
    fn span(&self) -> Span {
        self.span
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.token_type)
    }
}
