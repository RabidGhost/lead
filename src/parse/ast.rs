// use crate::{error::{parse_error::MixedTypeExpressionError, LangError}, lex::token::{Token, TokenType}};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Literal {
    Boolean(bool),
    Char(char),
    Number(u64),
}

pub enum Expression {
    App { app: Application },
    Group { expr: Box<Expression> },
    Literal { lit: Box<Literal> },
}

enum Application {
    Unary {
        op: Operator,
        expr: Box<Expression>,
    },
    Binary {
        op: Operator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

enum Operator {
    Unary(Box<dyn Fn(Literal) -> Literal>),
    Binary(Box<dyn Fn(Literal, Literal) -> Literal>),
}
