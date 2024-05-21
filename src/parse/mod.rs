use crate::{
    error::{
        LangError, ERROR_INVALID_LITERAL, ERROR_INVALID_OPERATOR, ERROR_UNEXPECTED_END_OF_FILE,
        ERROR_UNEXPECTED_END_OF_STREAM, ERROR_UNMATCHED_DELIMITER,
    },
    lex::token::{Token, TokenType},
};

use self::ast::{Application, BinaryOperator, Expression, Literal, OperatorType, UnaryOperator};
use self::ops::*;

pub mod ast;
mod ops;

pub trait Parser<'i, T: 'i> {
    fn input(&mut self) -> &'i [T];
    fn index(&mut self) -> &mut usize;

    fn peek_one(&mut self) -> Option<&'i T> {
        self.input().get(*self.index())
    }

    fn peek_many(&mut self, count: usize) -> Option<&'i [T]> {
        let idx: usize = *self.index();

        if idx + count <= self.input().len() {
            Some(&self.input()[idx..idx + count])
        } else {
            None
        }
    }

    fn advance_one(&mut self) -> Option<&'i T> {
        let t = self.peek_one()?;
        *self.index() += 1;
        return Some(t);
    }

    fn advance_many(&mut self, count: usize) -> Option<&'i [T]> {
        let ts = self.peek_many(count)?;
        *self.index() += count;
        return Some(ts);
    }

    fn is_eof(&mut self) -> bool {
        *self.index() >= self.input().len()
    }

    fn take_while(&mut self, mut f: impl FnMut(&T) -> bool) -> &'i [T] {
        let start = *self.index();

        while let Some(t) = self.peek_one() {
            if f(t) {
                self.advance_one();
            } else {
                break;
            }
        }
        let end = *self.index();
        return &self.input()[start..end];
    }

    fn skip_trivia(&mut self)
    where
        T: HasTrivia,
    {
        while let Some(t) = self.peek_one() {
            if (*t).is_trivia() {
                self.advance_one();
            } else {
                break;
            }
        }
    }

    // fn consume(&mut self, slice: &[T]) -> Result<(), ()> {

    // }
}

pub trait HasTrivia {
    fn is_trivia(&self) -> bool;
}

pub struct LangParser<'i> {
    src: &'i [Token],
    index: usize,
}

impl<'i> Parser<'i, Token> for LangParser<'i> {
    fn input(&mut self) -> &'i [Token] {
        self.src
    }

    fn index(&mut self) -> &mut usize {
        &mut self.index
    }
}

impl<'i> LangParser<'i> {
    pub fn new(src: &'i [Token]) -> Self {
        Self { src, index: 0 }
    }

    pub fn parse(&mut self) -> Result<Expression, LangError> {
        if self.is_eof() {
            return Err(LangError::from(
                "expected expression".to_owned(),
                (self.index, self.index),
                ERROR_UNEXPECTED_END_OF_FILE,
            ));
        }

        let expr: Expression = match self.peek_one().unwrap().token_type() {
            // literals and binary applications
            TokenType::Number(_) | TokenType::Bool(_) | TokenType::Char(_) => {
                let left = Expression::Literal {
                    lit: self.parse_literal()?,
                };

                self.parse_partial(left)?
            }

            // unary operators
            TokenType::Minus | TokenType::Bang => {
                let op = self.parse_unary_operator()?;
                let expr = self.parse()?;
                Expression::App {
                    app: Application::Unary {
                        op,
                        expr: Box::new(expr),
                    },
                }
            }

            // grouping
            TokenType::LeftParen => {
                let start = self.index;
                self.advance_one();
                let expr = self.parse()?;
                match self.peek_one() {
                    None => {
                        return Err(LangError::from(
                            "expected unary operator, found end of file".to_owned(),
                            (self.index, self.index),
                            ERROR_UNEXPECTED_END_OF_STREAM,
                        ))
                    }
                    Some(tok) => match tok.token_type() {
                        TokenType::RightParen => {
                            self.advance_one();
                        }
                        _ => {
                            return Err(LangError::from(
                                "expected closing delimiter `)`, found `{tok}`".to_owned(),
                                (self.index, self.index),
                                ERROR_UNMATCHED_DELIMITER,
                            ))
                        }
                    },
                };
                Expression::Group {
                    expr: Box::new(expr),
                    span: (start, self.index),
                }
            }
            TokenType::Identifier(name) => Expression::Identifier {
                id: (*name).clone(),
                span: self.advance_one().unwrap().span(),
            },
            _ => todo!(),
        };

        return self.parse_partial(expr);
    }

    fn parse_partial(&mut self, left: Expression) -> Result<Expression, LangError> {
        if self.is_eof() || self.is_line_end() {
            return Ok(left); // the expression
        }

        match self.peek_one() {
            None => unreachable!(),
            Some(tok) => match tok.token_type() {
                TokenType::Minus
                | TokenType::Plus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::LessThan
                | TokenType::GreaterThan
                | TokenType::LessThanEq
                | TokenType::GreaterThanEq
                | TokenType::EqEq
                | TokenType::Bang
                | TokenType::BangEq => {
                    // we are parsing a binary expression
                    let op = self.parse_binary_operator()?;
                    let right = self.parse()?;
                    return Ok(Expression::App {
                        app: Application::Binary {
                            op,
                            left: Box::new(left),
                            right: Box::new(right),
                        },
                    });
                }
                _ => return Ok(left),
            },
        }
    }

    /*
     <expression> ::= <literal> | <app> | <grouping>
     <app>        ::= <expression> <operator> <expression> | <operator> <expression>
     <grouping>   ::= "(" <expression> ")"
    */

    fn parse_literal(&mut self) -> Result<Literal, LangError> {
        match self.peek_one() {
            None => {
                return Err(LangError::from(
                    "expected literal, found end of file".to_owned(),
                    (self.index, self.index),
                    ERROR_UNEXPECTED_END_OF_STREAM,
                ))
            }
            Some(tok) => {
                let literal = match tok.token_type() {
                    TokenType::Bool(b) => Literal::from_bool(tok, *b),
                    TokenType::Char(c) => Literal::from_char(tok, *c),
                    TokenType::Number(n) => Literal::from_number(tok, (*n).try_into().unwrap()), // todo: fix this panic
                    _ => {
                        return Err(LangError::from(
                            format!("expected literal, found `{tok}`"),
                            tok.span(),
                            ERROR_INVALID_LITERAL,
                        ))
                    }
                };
                self.advance_one();
                Ok(literal)
            }
        }
    }

    fn parse_unary_operator(&mut self) -> Result<UnaryOperator, LangError> {
        match self.peek_one() {
            None => {
                return Err(LangError::from(
                    "expected unary operator, found end of file".to_owned(),
                    (self.index, self.index),
                    ERROR_UNEXPECTED_END_OF_STREAM,
                ))
            }
            Some(tok) => {
                let op = match tok.token_type() {
                    TokenType::Bang => UnaryOperator::from(tok, OperatorType::Not),
                    TokenType::Minus => UnaryOperator::from(tok, OperatorType::Minus),
                    _ => {
                        return Err(LangError::from(
                            format!("`{tok}` is not a valid unary operator"),
                            tok.span(),
                            ERROR_INVALID_OPERATOR,
                        ))
                    }
                };
                self.advance_one();
                return Ok(op);
            }
        }
    }

    fn parse_binary_operator(&mut self) -> Result<BinaryOperator, LangError> {
        let tok = self.peek_one().unwrap();
        let op = match tok.token_type() {
            TokenType::Minus => BinaryOperator::from(tok, OperatorType::Minus),
            TokenType::Plus => BinaryOperator::from(tok, OperatorType::Plus),
            TokenType::Slash => BinaryOperator::from(tok, OperatorType::Divide),
            TokenType::Star => BinaryOperator::from(tok, OperatorType::Multiply),
            TokenType::LessThan => BinaryOperator::from(tok, OperatorType::LessThan),
            TokenType::GreaterThan => BinaryOperator::from(tok, OperatorType::GreaterThan),
            TokenType::LessThanEq => BinaryOperator::from(tok, OperatorType::LessThanEq),
            TokenType::GreaterThanEq => BinaryOperator::from(tok, OperatorType::GreaterThanEq),
            TokenType::EqEq => BinaryOperator::from(tok, OperatorType::Equal),
            TokenType::BangEq => BinaryOperator::from(tok, OperatorType::NotEqual),
            _ => {
                return Err(LangError::from(
                    format!("`{tok}` is not a valid binary operator"),
                    tok.span(),
                    ERROR_INVALID_OPERATOR,
                ));
            }
        };
        self.advance_one();
        return Ok(op);
    }

    /// checks if their are no tokens remaining, or the current token is EOF.
    fn is_eof(&mut self) -> bool {
        match self.input().get(self.index) {
            None => true,
            Some(tok) => match tok.token_type() {
                TokenType::EOF => true,
                _ => false,
            },
        }
    }

    /// checks if the current token is `;`.
    fn is_line_end(&mut self) -> bool {
        match self.input().get(self.index) {
            None => false,
            Some(tok) => match tok.token_type() {
                TokenType::EOF => true,
                _ => false,
            },
        }
    }
}
