use crate::{
    error::LangError,
    lex::{
        span::{Span, Spans},
        token::{Token, TokenType},
    },
};

use self::ast::{
    Application, Expression, Identifier, If, Let, Literal, Mutate, OperatorType, Statement, While,
};

pub mod ast;

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

    pub fn parse_statement(
        &mut self,
        mut buf: Vec<Statement>,
    ) -> Result<Vec<Statement>, LangError> {
        if self.is_eof() {
            return Ok(buf);
        }

        loop {
            let statement = match self.peek_one().unwrap().token_type() {
                TokenType::EOF => break,
                TokenType::RightBrace => {
                    //self.consume(TokenType::RightBrace)?;
                    break;
                }

                TokenType::Identifier(_) => {
                    //self.advance_one();
                    match self.peek_nth(2)?.token_type() {
                        TokenType::Assign => Statement::Mutate(self.parse_mutate()?),
                        _ => Statement::Expr(self.parse_expr()?),
                    }
                }
                // raw expression
                TokenType::Number(_)
                | TokenType::Bool(_)
                | TokenType::Char(_)
                | TokenType::LeftParen
                | TokenType::Bang
                | TokenType::Minus => Statement::Expr(self.parse_expr()?),

                // keywords
                TokenType::Let => Statement::Let(self.parse_let()?),
                TokenType::While | TokenType::If => self.parse_wif()?,
                TokenType::Yield => self.parse_yield()?,
                _ => {
                    let tok = self.peek_one()?;
                    return Err(LangError::UnexpectedToken {
                        span: tok.span(),
                        tok: tok.ty(),
                        expected: "statement".to_string(),
                    });
                }
            };

            buf.push(statement);
        }
        return Ok(buf);
    }

    fn parse_yield(&mut self) -> Result<Statement, LangError> {
        self.consume(TokenType::Yield)?;
        let statement = Statement::Yield(self.parse_expr()?);
        self.consume(TokenType::Semicolon)?;
        Ok(statement)
    }

    pub fn parse_wif(&mut self) -> Result<Statement, LangError> {
        let start = self.advance_one().unwrap();
        let ty = start.token_type();

        let condition = self.parse_expr()?;

        let lb_span = self.consume(TokenType::LeftBrace)?.span();

        let body: Vec<Statement> = self.parse_statement(Vec::new())?;

        let span = Span::together([
            start.span(),
            condition.span(),
            lb_span,
            self.consume(TokenType::RightBrace)?.span(),
        ]);

        Ok(match ty {
            TokenType::If => Statement::If(If::from(condition, body, span)),
            TokenType::While => Statement::While(While::from(condition, body, span)),
            _ => {
                unreachable!()
            }
        })
    }

    pub fn parse_let(&mut self) -> Result<Let, LangError> {
        let start = self.consume(TokenType::Let)?;
        let (variable, expr) = self.parse_assign()?;
        Let::from(variable, expr, start)
    }

    pub fn parse_mutate(&mut self) -> Result<Mutate, LangError> {
        let (variable, expr) = self.parse_assign()?;
        Ok(Mutate::from(variable, expr))
    }

    pub fn parse_assign(&mut self) -> Result<(&Token, Expression), LangError> {
        let variable = self.advance_one().ok_or(LangError::UnexpectedEndOfFile {
            span: Span::new((self.index - 1, self.index)),
            expected: "identifier".to_string(),
            found: None,
        })?;
        self.consume(TokenType::Assign)?;
        let value = self.parse_expr()?;
        self.consume(TokenType::Semicolon)?;
        Ok((variable, value))
    }

    pub fn parse_expr(&mut self) -> Result<Expression, LangError> {
        if self.is_eof() {
            return Err(LangError::UnexpectedEndOfFile {
                span: Span::new((self.index - 1, self.index)),
                expected: "expression".to_owned(),
                found: None,
            });
        }

        let expr: Expression = match self.peek_one()?.token_type() {
            // literals and binary applications
            TokenType::Number(_) | TokenType::Bool(_) | TokenType::Char(_) => {
                let left = Expression::Literal {
                    lit: self.parse_literal()?,
                };

                self.parse_partial(left)?
            }
            // unary operators
            TokenType::Minus | TokenType::Bang => {
                let tok = self.peek_one().unwrap().clone();
                let op = self.parse_unary_operator()?;
                let expr = self.parse_expr()?;
                Expression::App {
                    app: Application::from_unary(&tok, op, expr),
                }
            }
            // grouping
            TokenType::LeftParen => {
                let mut span = self.advance_one().unwrap().span();
                let expr = self.parse_expr()?;
                match self.peek_one()?.ty() {
                    TokenType::RightParen => span.join(self.advance_one().unwrap()),
                    ty => {
                        return Err(LangError::UnmatchedDelimiter {
                            span: self.peek_one()?.span(),
                            expected: TokenType::RightParen,
                            found: ty,
                        });
                    }
                };
                Expression::Group {
                    expr: Box::new(expr),
                    span,
                }
            }
            // identifier or array index
            TokenType::Identifier(name) => {
                let identifier = Identifier::new((*name).clone(), self.advance_one().unwrap());
                match self.peek_one()?.token_type() {
                    TokenType::LeftSquare => {
                        self.consume(TokenType::LeftSquare)?;
                        let index = Box::new(self.parse_expr()?);
                        let span =
                            Span::superspan(&identifier, self.consume(TokenType::RightSquare)?);
                        Expression::Index {
                            variable: identifier,
                            index,
                            span,
                        }
                    }
                    _ => Expression::Identifier(identifier),
                }
            }
            TokenType::LeftSquare => self.parse_array()?,
            tok => {
                dbg!(tok);
                todo!()
            }
        };

        return self.parse_partial(expr);
    }

    fn parse_partial(&mut self, left: Expression) -> Result<Expression, LangError> {
        if self.is_eof() || self.is_line_end() {
            return Ok(left); // the expression
        }

        match self.peek_one()?.token_type() {
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
                let right = self.parse_expr()?;
                return Ok(Expression::App {
                    app: Application::from_binary(op, left, right),
                });
            }
            _ => return Ok(left),
        }
    }

    /*
     <expression> ::= <literal> | <app> | <grouping>
     <app>        ::= <expression> <operator> <expression> | <operator> <expression>
     <grouping>   ::= "(" <expression> ")"
    */

    fn parse_literal(&mut self) -> Result<Literal, LangError> {
        let tok = self.peek_one()?;
        let literal = match tok.token_type() {
            TokenType::Bool(b) => Literal::from_bool(tok, *b),
            TokenType::Char(c) => Literal::from_char(tok, *c),
            TokenType::Number(n) => Literal::from_number(tok, (*n).try_into().unwrap()), // todo: fix this panic
            _ => {
                return Err(LangError::InvalidLiteral {
                    span: tok.span(),
                    invalid_literal: tok.ty(),
                });
            }
        };
        self.advance_one();
        Ok(literal)
    }

    fn parse_unary_operator(&mut self) -> Result<OperatorType, LangError> {
        let tok = self.peek_one()?;
        let op = match tok.token_type() {
            TokenType::Bang => OperatorType::Not,
            TokenType::Minus => OperatorType::Minus,
            _ => {
                return Err(LangError::InvalidUnaryOperator {
                    span: tok.span(),
                    op: tok.ty(),
                });
            }
        };
        self.advance_one();
        return Ok(op);
    }

    fn parse_binary_operator(&mut self) -> Result<OperatorType, LangError> {
        let tok = self.peek_one().unwrap();
        let op = match tok.token_type() {
            TokenType::Minus => OperatorType::Minus,
            TokenType::Plus => OperatorType::Plus,
            TokenType::Slash => OperatorType::Divide,
            TokenType::Star => OperatorType::Multiply,
            TokenType::LessThan => OperatorType::LessThan,
            TokenType::GreaterThan => OperatorType::GreaterThan,
            TokenType::LessThanEq => OperatorType::LessThanEq,
            TokenType::GreaterThanEq => OperatorType::GreaterThanEq,
            TokenType::EqEq => OperatorType::Equal,
            TokenType::BangEq => OperatorType::NotEqual,
            _ => {
                return Err(LangError::InvalidBinaryOperator {
                    span: tok.span(),
                    op: tok.ty(),
                });
            }
        };
        self.advance_one();
        return Ok(op);
    }

    fn parse_array(&mut self) -> Result<Expression, LangError> {
        let mut elements: Vec<Box<Expression>> = Vec::new();
        let left_square = self.consume(TokenType::LeftSquare)?;
        while *self.peek_one()?.token_type() != TokenType::RightSquare {
            if *self.peek_one()?.token_type() == TokenType::Comma {
                self.consume(TokenType::Comma)?;
            }
            elements.push(Box::new(self.parse_expr()?));
        }
        let right_square = self.consume(TokenType::RightSquare)?;
        Ok(Expression::Array {
            elements,
            span: Span::superspan(left_square, right_square),
        })
    }

    /// checks if their are no tokens remaining, or the current token is `EOF`.
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

    /// consume one token of a given type, erroring if it is not found
    fn consume(&mut self, ty: TokenType) -> Result<Token, LangError> {
        match Parser::peek_one(self) {
            None => Err(LangError::UnexpectedEndOfFile {
                span: Span::new((self.index, self.index)),
                expected: format!("{}", ty),
                found: Some("end of file".to_owned()),
            }),
            Some(tok) if *tok.token_type() == ty => {
                self.advance_one();
                return Ok(tok.clone());
            }
            Some(tok) => Err(LangError::ExpectedToken {
                span: Span::new((self.index, self.index + 1)),
                expected: ty,
                found: tok.ty(),
            }),
        }
    }

    fn peek_one(&mut self) -> Result<&Token, LangError> {
        Parser::peek_one(self).ok_or(LangError::UnexpectedEndOfFile {
            span: Span::new((self.index, self.index + 1)),
            expected: "EOF".to_owned(),
            found: None,
        })
    }

    fn peek_nth(&mut self, count: usize) -> Result<&Token, LangError> {
        Ok(self
            .peek_many(count)
            .ok_or(LangError::UnexpectedEndOfFile {
                span: Span::new((self.index, self.index + 1)),
                expected: "EOF".to_owned(),
                found: None,
            })?
            .get(count - 1)
            .unwrap())
    }

    // fn advance_one(&mut self) -> Result<&Token, LangError> {

    // }
}
