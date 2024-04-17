mod token;

use std::{collections::VecDeque, error::Error, iter::Peekable, str::Chars};

use crate::error::{lex_error::*, LangError};
use token::{Token, TokenType};

pub struct Lexer {
    src: String,
    buf: Vec<char>,
    offset: usize,
    line: usize,
    col: usize,
    tokens: Vec<Token>,
    errors: Vec<Box<dyn LangError>>,
}

impl Lexer {
    pub fn new(src: &str) -> Self {
        Lexer {
            src: src.to_string(),
            buf: Vec::new(),
            offset: 0,
            line: 0,
            col: 0,
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn run(mut self) -> Result<Vec<Token>, Box<dyn Error>> {
        loop {
            let next = match self.src.chars().nth(self.offset) {
                Some(ch) => ch,
                None => {
                    let _ = &mut self.add_token(TokenType::EOF)?;
                    //self.tokens.iter().filter(|&&tk| tk != Space).collect()

                    return Ok(self.tokens);
                } // add EOF to our tokens, then return them
            };

            if next == '\n' {
                self.line += 1;
                self.col = 0;
            } else {
                self.col += 1;
            }

            self.buf.push(next);
            while !self.buf.is_empty() {
                match self.lex()? {
                    None => break,
                    Some(token_type) => self.add_token(token_type)?,
                }
            }

            self.offset += 1;
        }

        todo!()
    }

    fn eat_comment(&mut self) {
        todo!()
    }

    fn lex(&mut self) -> Result<Option<TokenType>, Box<dyn Error>> {
        // we are only concered with matching anything from the start of the buffer.
        // if the token does not fill the entire buffer, we get its length and only remove that from the buffer,
        // then continue lexing
        Ok(Some(
            match self
                .buf
                .get(0)
                .ok_or::<Box<dyn Error>>("Buffer Empty on Lex".into())?
            {
                ' ' => TokenType::Space,
                '(' => TokenType::LeftParen,
                ')' => TokenType::RightParen,
                '{' => TokenType::LeftBrace,
                '}' => TokenType::RightBrace,
                '[' => TokenType::LeftSquare,
                ']' => TokenType::RightSquare,
                ',' => TokenType::Comma,
                '.' => TokenType::Dot,
                '-' => TokenType::Minus,
                '+' => TokenType::Plus,
                '*' => TokenType::Star,
                ';' => TokenType::Semicolon,
                'A'..='Z' | 'a'..='z' => {
                    match self.lex_keywords()? {
                        Some(tok) => return Ok(Some(tok)),
                        None => (),
                    }

                    match self.lex_identifier()? {
                        Some(tok) => return Ok(Some(tok)),
                        None => return Ok(None),
                    }
                }
                '0'..='9' => match self.lex_number()? {
                    Some(tok) => tok,
                    None => return Ok(None),
                },

                // potential two char tokens
                _ => {
                    let fst = self.buf[0];
                    let snd = match self.buf.get(1) {
                        Some(ch) => ch,
                        None => return Ok(None),
                    };

                    println!("{:?}", self.buf);

                    match fst {
                        '/' => match snd {
                            '/' => {
                                self.eat_comment();
                                return Ok(None);
                            }
                            _ => TokenType::Slash,
                        },
                        '!' => match snd {
                            '=' => TokenType::BangEq,
                            _ => TokenType::Bang,
                        },
                        '<' => match snd {
                            '=' => TokenType::LessThanEq,
                            _ => TokenType::LessThan,
                        },
                        '>' => match snd {
                            '=' => TokenType::GreaterThanEq,
                            _ => TokenType::GreaterThan,
                        },
                        ':' => match snd {
                            '=' => TokenType::Assign,
                            _ => TokenType::Colon,
                        },
                        '=' => match snd {
                            '=' => TokenType::EqEq,
                            _ => {
                                self.error(InvalidLexemeErr::new(
                                    self.src.lines().nth(self.line).unwrap().to_string(),
                                    self.line,
                                    self.col,
                                    2,
                                ));
                                self.reset();
                                return Ok(None);
                            }
                        },
                        _ => {
                            return Ok(None);
                            //todo!()
                        }
                    }
                }
            },
        ))
    }

    fn lex_keywords(&mut self) -> Result<Option<TokenType>, Box<dyn Error>> {
        if self.peek() == Some(' ') {
            Ok(Some(
                match &self.buf.iter().clone().collect::<String>()[..] {
                    "let" => TokenType::Let,
                    "if" => TokenType::If,
                    "for" => TokenType::For,
                    "while" => TokenType::While,
                    "true" => TokenType::True,
                    "false" => TokenType::False,
                    _ => return Ok(None),
                },
            ))
        } else {
            Ok(None)
        }
    }

    fn lex_identifier(&mut self) -> Result<Option<TokenType>, Box<dyn Error>> {
        if match self.peek() {
            None => return Err("unexpected end of file".into()),
            Some(ch) => !(ch.is_alphanumeric() || ch == '_'),
        } {
            if Self::is_valid_identifier(&self.buf[..]) {
                Ok(Some(TokenType::Identifier(
                    self.buf.iter().clone().collect::<String>(),
                )))
            } else {
                panic!("Temp panic")
            }
        } else {
            Ok(None)
        }
    }

    fn lex_number(&mut self) -> Result<Option<TokenType>, Box<dyn Error>> {
        if Self::is_valid_number(&vec![self.peek().unwrap()][..]) {
            return Ok(None);
        } else {
            let no_underscores: Vec<char> = self
                .buf
                .iter()
                .clone()
                .filter(|&&ch| ch != '_')
                .map(|&ch| ch)
                .collect();
            if Self::is_valid_number(&no_underscores[..]) {
                Ok(Some(TokenType::Number(
                    no_underscores.iter().clone().collect::<String>().parse()?,
                    no_underscores.iter().clone().collect::<String>(),
                )))
            } else {
                Ok(None) //for now. this should probably be some kind of parse error depending on the circumstance
            }
        }
    }

    fn add_token(&mut self, token_type: TokenType) -> Result<(), Box<dyn Error>> {
        let size = token_type.size();
        let token: Token = Token::new(token_type, self.line, self.col - size, size);
        self.buf.drain(0..size);
        self.tokens.push(token);
        Ok(())
    }

    fn error(&mut self, err: impl LangError + 'static) {
        self.errors.push(Box::new(err));
    }

    fn reset(&mut self) {
        // we need to reset the lexer after encountering invalid lexemes, so we can continue lexing
        self.buf.clear()
    }

    fn peek(&self) -> Option<char> {
        // peek one char ahead
        self.src.chars().nth(self.offset + 1)
    }

    fn is_valid_identifier(string: &[char]) -> bool {
        string
            .iter()
            .map(|&ch| ch.is_ascii_alphanumeric() || ch == '_')
            .reduce(|x, y| x && y)
            .unwrap()
            && string.iter().nth(0).unwrap().is_ascii_alphabetic()
    }

    fn is_valid_number(string: &[char]) -> bool {
        // numbers are integers, and may be seperated by any number of underscores for readability
        // underscore are filtered out before this step
        string
            .iter()
            .map(|&ch| ch.is_ascii_digit())
            .reduce(|x, y| x && y)
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_simple_1() {
        let src: &str = r#"1 + 1;"#;
        let lexer = Lexer::new(src);
        let tokens = lexer.run().unwrap();
        let mut token_string = String::new();
        for tok in tokens {
            token_string.push_str(&format!("{tok}, ")[..])
        }
        assert_eq!(
            r#"Number(1, "1"), Plus, Number(1, "1"), Semicolon, EOF"#,
            token_string
        )
    }
}
