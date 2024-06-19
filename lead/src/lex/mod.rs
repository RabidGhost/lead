pub mod span;
pub mod token;

use primes::PrimeSet;
use TSPL::{self, Parser};

use crate::error::{
    LangError, ERROR_INVALID_CHARACTER_LITERAL, ERROR_INVALID_LEXEME, ERROR_INVALID_NUMBER_LITERAL,
};
use token::{Token, TokenType, KEYWORDS};

pub struct Lexer<'l> {
    src: &'l str,
    index: usize,
}

impl<'i> TSPL::Parser<'i> for Lexer<'i> {
    fn input(&mut self) -> &'i str {
        self.src
    }

    fn index(&mut self) -> &mut usize {
        &mut self.index
    }
}

impl<'l> Lexer<'l> {
    pub fn new(src: &'l str) -> Self {
        Self { src, index: 0 }
    }

    pub fn run(&mut self) -> Result<Vec<Token>, LangError> {
        let mut buf = Vec::new();
        self.lex(&mut buf)?;
        Ok(buf)
    }

    fn lex(&mut self, buf: &mut Vec<Token>) -> Result<(), LangError> {
        self.skip_trivia();

        let mut tok: Token = Token::new(TokenType::EOF, self.index, 0); // placeholder since the compiler cannot verify tok gets initialised.
        if self.is_eof() {
            buf.push(tok);
            return Ok(());
        }

        let start = self.index;

        match self.peek_one().unwrap() {
            ' ' => {
                self.skip_spaces();
                return self.lex(buf);
            }
            '(' | ')' | '{' | '}' | '[' | ']' | ',' | '.' | '-' | '+' | '*' | ';' | '/' => {
                tok = Token::from(&self.advance_one().unwrap().to_string(), self.index)?;
            }
            '!' | '<' | '>' | ':' | '=' => {
                match self.peek_many(2) {
                    None => {
                        tok = Token::from(&self.advance_one().unwrap().to_string(), self.index)?
                    }
                    Some(string) => {
                        if string.chars().nth(1).unwrap() == '=' {
                            tok = Token::from(self.advance_many(2).unwrap(), self.index)?
                        } else {
                            tok = Token::from(&self.advance_one().unwrap().to_string(), self.index)?
                        }
                    }
                };
            }
            '\'' => {
                let ch = match self.parse_quoted_char() {
                    Ok(ch) => ch,
                    Err(e) => {
                        return Err(LangError::from(
                            e,
                            (start, self.index),
                            ERROR_INVALID_CHARACTER_LITERAL,
                        ))
                    }
                };
                tok = Token::new(TokenType::Char(ch), self.index, self.index - start);
            }
            ch if ch.is_digit(10) => {
                match self.parse_u64() {
                    Ok(n) => tok = Token::from_num(n, start, self.index),
                    Err(_) => {
                        return Err(LangError::from(
                            format!(
                                "expected number literal, found `{}`",
                                self.input()[start..self.index].to_owned()
                            ),
                            (start, self.index),
                            ERROR_INVALID_NUMBER_LITERAL,
                        ))
                    }
                };
            }
            'a'..='z' | 'A'..='Z' => {
                let name = self.take_while(|ch| Self::is_valid_identifier_char(ch));

                if KEYWORDS.contains(&name) {
                    tok = Token::from_keyword(name, start)?;
                } else {
                    tok = Token::new(TokenType::Identifier(name.to_owned()), start, name.len())
                }
            }
            ch => {
                return Err(LangError::from(
                    format!("unknown lexeme `{}`", ch),
                    (start, start + 1),
                    ERROR_INVALID_LEXEME,
                ))
            }
        }

        buf.push(tok);
        return self.lex(buf);
    }

    /// returns weather a given character is a valid non starting identifier character
    fn is_valid_identifier_char(ch: char) -> bool {
        match ch {
            '_' => true,
            ch => ch.is_ascii_alphabetic() || ch.is_digit(10),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Ty = TokenType;

    fn process(src: &str) -> Vec<TokenType> {
        let mut lexer = Lexer::new(src);
        let mut result = Vec::new();
        lexer.lex(&mut result).unwrap();

        result.iter().map(|x| x.token_type().to_owned()).collect()
    }

    #[test]
    fn keywords() {
        assert_eq!(
            vec![
                TokenType::Bool(true),
                TokenType::Bool(false),
                TokenType::Let,
                TokenType::If,
                TokenType::For,
                TokenType::While,
                TokenType::Yield,
            ],
            KEYWORDS
                .into_iter()
                .map(|keyword| {
                    let mut lexer = Lexer::new(keyword);
                    let mut tokens = Vec::new();
                    lexer.lex(&mut tokens).unwrap();
                    return tokens.first().unwrap().clone().token_type().to_owned();
                })
                .collect::<Vec<TokenType>>()
        );
    }

    #[test]
    fn simple_if() {
        let src = "if (my_var < 3) { 42 }";
        let expected: Vec<TokenType> = vec![
            Ty::If,
            Ty::LeftParen,
            Ty::Identifier("my_var".to_owned()),
            Ty::LessThan,
            Ty::Number(3),
            Ty::RightParen,
            Ty::LeftBrace,
            Ty::Number(42),
            Ty::RightBrace,
            Ty::EOF,
        ];
        assert_eq!(expected, process(src));
    }

    #[test]
    fn multiline_if() {
        let src = "let my_var := 2;\nif (my_var < 3) {\n\t42\n}";
        let expected: Vec<TokenType> = vec![
            Ty::Let,
            Ty::Identifier("my_var".to_owned()),
            Ty::Assign,
            Ty::Number(2),
            Ty::Semicolon,
            Ty::If,
            Ty::LeftParen,
            Ty::Identifier("my_var".to_owned()),
            Ty::LessThan,
            Ty::Number(3),
            Ty::RightParen,
            Ty::LeftBrace,
            Ty::Number(42),
            Ty::RightBrace,
            Ty::EOF,
        ];
        assert_eq!(expected, process(src));
    }
}
