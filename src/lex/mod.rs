pub mod token;

use TSPL::{self, Parser};

use crate::error::{
    LangError, ERROR_INVALID_CHARACTER_LITERAL, ERROR_INVALID_INDENTIFIER,
    ERROR_INVALID_NUMBER_LITERAL,
};
use token::{Token, TokenType, KEYWORDS};

pub struct Lexer<'l> {
    src: &'l str,
    index: usize,
    errors: Vec<LangError>,
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
        Self {
            src,
            index: 0,
            errors: Vec::new(),
        }
    }

    pub fn run(&mut self) -> Result<Vec<Token>, Vec<LangError>> {
        let mut buf = Vec::new();
        match self.lex(&mut buf) {
            Ok(_) => (),
            Err(e) => return Err(vec![e]),
        }
        return Ok(buf);
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
            _ => {
                let mut found_keyword = false;
                // check keywords
                for keyword in KEYWORDS {
                    if self.starts_with_keyword(keyword) {
                        self.consume(keyword).unwrap();
                        tok = Token::from_keyword(keyword, start)?;
                        found_keyword = true;
                        break;
                    }
                }
                if !found_keyword {
                    let identifier = self.parse_identifier()?;
                    let identifier_length = identifier.len();
                    tok = Token::new(TokenType::Identifier(identifier), start, identifier_length);
                }
            }
        }

        buf.push(tok);
        return self.lex(buf);
    }

    // checks if the next characters are a valid keyword, and also not part of a longer identifier
    fn starts_with_keyword(&mut self, keyword: &str) -> bool {
        self.starts_with(keyword)
            && !Self::is_valid_identifier_char(
                self.input().chars().nth(keyword.len()).unwrap_or(' '),
            )
    }

    fn is_valid_identifier_char(ch: char) -> bool {
        match ch {
            '_' => true,
            ch => ch.is_alphabetic() || ch.is_digit(10),
        }
    }

    fn parse_identifier(&mut self) -> Result<String, LangError> {
        self.skip_trivia();
        if match self.peek_one() {
            Some(ch) => ch.is_alphabetic(),
            None => false,
        } {
            Ok(self
                .take_while(|c| c.is_ascii_alphanumeric() || "_".contains(c))
                .to_owned())
        } else {
            Err(LangError::from(
                "expected valid identifier".to_owned(),
                (self.index, self.index + 1),
                ERROR_INVALID_INDENTIFIER,
            ))
        }
    }
}
