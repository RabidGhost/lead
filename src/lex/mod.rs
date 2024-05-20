pub mod token;

use TSPL::{self, Parser};

use std::error::Error;

use crate::error::{
    LangError, ERROR_INVALID_CHARACTER_LITERAL, ERROR_INVALID_INDENTIFIER,
    ERROR_INVALID_NUMBER_LITERAL,
};
use token::{Token, TokenType, KEYWORDS};

pub struct Lexer<'l> {
    src: &'l str,
    index: usize,
    tokens: Vec<Token>,
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
    fn lex(&mut self, buf: &mut Vec<Token>) -> Result<(), LangError> {
        self.skip_trivia();

        let mut tok: Token = Token::new(TokenType::EOF, self.index, 0); // placeholder since the compiler cannot verify tok gets initialised.
        if self.is_eof() {
            buf.push(tok);
            return Ok(());
        }

        let start = self.index;

        match self.peek_one().unwrap() {
            ' ' | '(' | ')' | '{' | '}' | '[' | ']' | ',' | '.' | '-' | '+' | '*' | ';' | '/' => {
                tok = Token::from(&self.advance_one().unwrap().to_string(), self.index)?;
            }
            '!' | '<' | '>' | ':' | '=' => {
                match self.advance_many(2) {
                    None => {
                        tok = Token::from(&self.advance_one().unwrap().to_string(), self.index)?
                    }
                    Some(string) => {
                        if string.chars().nth(2).unwrap() == '=' {
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
                        self.consume(keyword);
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

            _ => todo!(),
        }

        buf.push(tok);
        return self.lex(buf);
    }

    // checks if the next characters are a valid keyword, and also not part of a longer identifier
    fn starts_with_keyword(&mut self, keyword: &str) -> bool {
        self.starts_with(keyword)
            && !Self::is_valid_identifier(
                self.peek_many(keyword.len() + 1)
                    .unwrap_or_else(|| " ")
                    .chars()
                    .last()
                    .unwrap(),
            )
    }

    fn is_valid_identifier(ch: char) -> bool {
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

//     pub fn run(mut self) -> Result<Vec<Token>, Box<dyn Error>> {
//         loop {
//             let next = match self.src.chars().nth(self.offset) {
//                 Some(ch) => ch,
//                 None => {
//                     let _ = &mut self.add_token(TokenType::EOF)?;
//                     //self.tokens.iter().filter(|&&tk| tk != Space).collect()

//                     return Ok(self.tokens);
//                 } // add EOF to our tokens, then return them
//             };

//             if next == '\n' {
//                 self.line += 1;
//                 self.col = 0;
//             } else {
//                 self.col += 1;
//             }

//             self.buf.push(next);
//             while !self.buf.is_empty() {
//                 match self.lex()? {
//                     None => break,
//                     Some(token_type) => self.add_token(token_type)?,
//                 }
//             }

//             self.offset += 1;
//         }
//     }

//     fn eat_comment(&mut self) {
//         todo!("implement eat comment")
//     }

//     fn lex(&mut self) -> Result<Option<TokenType>, Box<dyn Error>> {
//         // we are only concered with matching anything from the start of the buffer.
//         // if the token does not fill the entire buffer, we get its length and only remove that from the buffer,
//         // then continue lexing
//         Ok(Some(
//             match self
//                 .buf
//                 .get(0)
//                 .ok_or::<Box<dyn Error>>("Buffer Empty on Lex".into())?
//             {
//                 ' ' => TokenType::Space,
//                 '(' => TokenType::LeftParen,
//                 ')' => TokenType::RightParen,
//                 '{' => TokenType::LeftBrace,
//                 '}' => TokenType::RightBrace,
//                 '[' => TokenType::LeftSquare,
//                 ']' => TokenType::RightSquare,
//                 ',' => TokenType::Comma,
//                 '.' => TokenType::Dot,
//                 '-' => TokenType::Minus,
//                 '+' => TokenType::Plus,
//                 '*' => TokenType::Star,
//                 ';' => TokenType::Semicolon,
//                 'A'..='Z' | 'a'..='z' => {
//                     match self.lex_keywords()? {
//                         Some(tok) => return Ok(Some(tok)),
//                         None => (),
//                     }

//                     match self.lex_identifier()? {
//                         Some(tok) => return Ok(Some(tok)),
//                         None => return Ok(None),
//                     }
//                 }
//                 '0'..='9' => match self.lex_number()? {
//                     Some(tok) => tok,
//                     None => return Ok(None),
//                 },

//                 // potential two char tokens
//                 _ => {
//                     let fst = self.buf[0];
//                     let snd = match self.buf.get(1) {
//                         Some(ch) => ch,
//                         None => return Ok(None),
//                     };

//                     println!("{:?}", self.buf);

//                     match fst {
//                         '/' => match snd {
//                             '/' => {
//                                 self.eat_comment();
//                                 return Ok(None);
//                             }
//                             _ => TokenType::Slash,
//                         },
//                         '!' => match snd {
//                             '=' => TokenType::BangEq,
//                             _ => TokenType::Bang,
//                         },
//                         '<' => match snd {
//                             '=' => TokenType::LessThanEq,
//                             _ => TokenType::LessThan,
//                         },
//                         '>' => match snd {
//                             '=' => TokenType::GreaterThanEq,
//                             _ => TokenType::GreaterThan,
//                         },
//                         ':' => match snd {
//                             '=' => TokenType::Assign,
//                             _ => TokenType::Colon,
//                         },
//                         '=' => match snd {
//                             '=' => TokenType::EqEq,
//                             ch => {
//                                 self.error(LangError::from_message(format!(
//                                     "invalid lexeme: {ch}"
//                                 )));
//                                 self.reset();
//                                 return Ok(None);
//                             }
//                         },
//                         _ => {
//                             return Ok(None);
//                             //todo!()
//                         }
//                     }
//                 }
//             },
//         ))
//     }

//     fn lex_keywords(&mut self) -> Result<Option<TokenType>, Box<dyn Error>> {
//         if self.peek() == Some(' ') {
//             Ok(Some(
//                 match &self.buf.iter().clone().collect::<String>()[..] {
//                     "let" => TokenType::Let,
//                     "if" => TokenType::If,
//                     "for" => TokenType::For,
//                     "while" => TokenType::While,
//                     "true" => TokenType::True,
//                     "false" => TokenType::False,
//                     _ => return Ok(None),
//                 },
//             ))
//         } else {
//             Ok(None)
//         }
//     }

//     fn lex_identifier(&mut self) -> Result<Option<TokenType>, Box<dyn Error>> {
//         if match self.peek() {
//             None => return Err("unexpected end of file".into()),
//             Some(ch) => !(ch.is_alphanumeric() || ch == '_'),
//         } {
//             if Self::is_valid_identifier(&self.buf[..]) {
//                 Ok(Some(TokenType::Identifier(
//                     self.buf.iter().clone().collect::<String>(),
//                 )))
//             } else {
//                 panic!("Temp panic")
//             }
//         } else {
//             Ok(None)
//         }
//     }

//     fn lex_number(&mut self) -> Result<Option<TokenType>, Box<dyn Error>> {
//         if Self::is_valid_number(&vec![self.peek().unwrap()][..]) {
//             return Ok(None);
//         } else {
//             let no_underscores: Vec<char> = self
//                 .buf
//                 .iter()
//                 .clone()
//                 .filter(|&&ch| ch != '_')
//                 .map(|&ch| ch)
//                 .collect();
//             if Self::is_valid_number(&no_underscores[..]) {
//                 Ok(Some(TokenType::Number(
//                     no_underscores.iter().clone().collect::<String>().parse()?,
//                     no_underscores.iter().clone().collect::<String>(),
//                 )))
//             } else {
//                 Ok(None) //for now. this should probably be some kind of parse error depending on the circumstance
//             }
//         }
//     }

//     fn add_token(&mut self, token_type: TokenType) -> Result<(), Box<dyn Error>> {
//         let size = token_type.size();
//         let token: Token = Token::new(token_type, self.line, self.col - size, size);
//         self.buf.drain(0..size);
//         self.tokens.push(token);
//         Ok(())
//     }

//     fn error(&mut self, err: LangError) {
//         self.errors.push(err);
//     }

//     fn reset(&mut self) {
//         // we need to reset the lexer after encountering invalid lexemes, so we can continue lexing
//         self.buf.clear()
//     }

//     fn peek(&self) -> Option<char> {
//         // peek one char ahead
//         self.src.chars().nth(self.offset + 1)
//     }

//     fn is_valid_identifier(string: &[char]) -> bool {
//         string
//             .iter()
//             .map(|&ch| ch.is_ascii_alphanumeric() || ch == '_')
//             .reduce(|x, y| x && y)
//             .unwrap()
//             && string.iter().nth(0).unwrap().is_ascii_alphabetic()
//     }

//     fn is_valid_number(string: &[char]) -> bool {
//         // numbers are integers, and may be seperated by any number of underscores for readability
//         // underscore are filtered out before this step
//         string
//             .iter()
//             .map(|&ch| ch.is_ascii_digit())
//             .reduce(|x, y| x && y)
//             .unwrap()
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     //#[test]
//     fn lex_simple_1() {
//         let src: &str = r#"1 + 1;"#;
//         let lexer = Lexer::new(src);
//         let tokens = lexer.run().unwrap();
//         let mut token_string = String::new();
//         for tok in tokens {
//             token_string.push_str(&format!("{tok}, ")[..])
//         }
//         assert_eq!(
//             r#"Number(1, "1"), Plus, Number(1, "1"), Semicolon, EOF"#,
//             token_string
//         )
//     }
// }
