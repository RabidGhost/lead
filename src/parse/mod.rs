use crate::{
    error::{
        LangError, ERROR_EXPECTED, ERROR_INVALID_LITERAL, ERROR_INVALID_OPERATOR,
        ERROR_UNEXPECTED_END_OF_FILE, ERROR_UNMATCHED_DELIMITER,
    },
    lex::token::{Token, TokenType},
};

use self::ast::{
    Application, Expression, If, Let, Literal, Mutate, OperatorType, Statement, While,
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
                    dbg!(self.peek_one().unwrap());
                    todo!()
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
        let start = self.index;
        let ty = self.advance_one().unwrap().token_type();
        //self.consume(TokenType::If)?;
        let condition = self.parse_expr()?;
        self.consume(TokenType::LeftBrace)?;

        let body: Vec<Statement> = self.parse_statement(Vec::new())?;
        self.consume(TokenType::RightBrace)?;

        Ok(match ty {
            TokenType::If => Statement::If(If::from(condition, body, (start, self.index))),
            TokenType::While => Statement::While(While::from(condition, body, (start, self.index))),
            _ => {
                unreachable!()
            }
        })
    }

    pub fn parse_let(&mut self) -> Result<Let, LangError> {
        let start = self.index;
        self.consume(TokenType::Let)?;
        let variable = self.advance_one().ok_or(LangError::from(
            "expected identifier".to_owned(),
            (self.index - 1, self.index),
            ERROR_UNEXPECTED_END_OF_FILE,
        ))?;
        self.consume(TokenType::Assign)?;
        let value = self.parse_expr()?;
        let assign = Let::from(variable, value, start);
        self.consume(TokenType::Semicolon)?;
        Ok(assign)
    }

    pub fn parse_mutate(&mut self) -> Result<Mutate, LangError> {
        let variable = self.advance_one().ok_or(LangError::from(
            "expected identifier".to_owned(),
            (self.index - 1, self.index),
            ERROR_UNEXPECTED_END_OF_FILE,
        ))?;
        self.consume(TokenType::Assign)?;
        let value = self.parse_expr()?;
        let assign = Mutate::from(variable, value);
        self.consume(TokenType::Semicolon)?;
        Ok(assign)
    }

    pub fn parse_expr(&mut self) -> Result<Expression, LangError> {
        if self.is_eof() {
            return Err(LangError::from(
                "expected expression".to_owned(),
                (self.index, self.index),
                ERROR_UNEXPECTED_END_OF_FILE,
            ));
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
                let start = self.index;
                self.advance_one();
                let expr = self.parse_expr()?;
                match self.peek_one()?.token_type() {
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

    fn parse_unary_operator(&mut self) -> Result<OperatorType, LangError> {
        let tok = self.peek_one()?;
        let op = match tok.token_type() {
            TokenType::Bang => OperatorType::Not,
            TokenType::Minus => OperatorType::Minus,
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
            None => Err(LangError::from(
                format!("expected `{:?}`, found end of file", ty),
                (self.index, self.index),
                ERROR_UNEXPECTED_END_OF_FILE,
            )),
            Some(tok) => {
                if *tok.token_type() == ty {
                    self.advance_one();
                    return Ok(tok.clone());
                } else {
                    Err(LangError::from(
                        format!("expected `{:?}`, found `{}`", ty, tok),
                        (self.index, self.index),
                        ERROR_EXPECTED,
                    ))
                }
            }
        }
    }

    fn peek_one(&mut self) -> Result<&Token, LangError> {
        Ok(Parser::peek_one(self).ok_or(LangError::from(
            "expected EOF".to_owned(),
            (self.index, self.index + 1),
            ERROR_UNEXPECTED_END_OF_FILE,
        ))?)
    }

    fn peek_nth(&mut self, count: usize) -> Result<&Token, LangError> {
        Ok(self
            .peek_many(count)
            .ok_or(LangError::from(
                "expected EOF".to_owned(),
                (self.index, self.index + count),
                ERROR_UNEXPECTED_END_OF_FILE,
            ))?
            .get(count - 1)
            .unwrap())
    }
}
