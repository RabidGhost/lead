use crate::error::LangError;
use crate::lex::token::Token;

use super::{
    OP_BINARY_MINUS, OP_DIVIDE, OP_EQUAL, OP_GREATERTHAN, OP_GREATERTHANEQUAL, OP_LESSTHAN,
    OP_LESSTHANEQUAL, OP_MULTIPLY, OP_NOT, OP_NOTEQUAL, OP_PLUS, OP_UNARY_MINUS,
};

type Span = (usize, usize);
type OpRet = Result<Literal, LangError>;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Literal {
    Boolean { val: bool, span: Span },
    Char { val: char, span: Span },
    Number { val: i32, span: Span },
}

#[derive(Debug)]
pub enum Expression {
    App { app: Application },
    Group { expr: Box<Expression>, span: Span },
    Literal { lit: Literal },
}

#[derive(Debug)]
pub enum Application {
    Unary {
        op: UnaryOperator,
        expr: Box<Expression>,
    },
    Binary {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

#[derive(Debug)]
pub struct UnaryOperator {
    ty: OperatorType,
    f: fn(Literal) -> OpRet,
    span: Span,
}

#[derive(Debug)]
pub struct BinaryOperator {
    ty: OperatorType,
    f: fn(Literal, Literal) -> OpRet,
    span: Span,
}

#[derive(Clone, Copy, Debug)]
pub enum OperatorType {
    Plus,
    Minus,
    Divide,
    Multiply,
    LessThan,
    GreaterThan,
    LessThanEq,
    GreaterThanEq,
    Equal,
    Not,
    NotEqual,
}

// spans for ast

pub trait Spans {
    fn span(&self) -> Span;
}

impl Spans for Literal {
    fn span(&self) -> Span {
        match self {
            Literal::Char { val, span } => *span,
            Literal::Number { val, span } => *span,
            Literal::Boolean { val, span } => *span,
        }
    }
}

impl Spans for Expression {
    fn span(&self) -> Span {
        match self {
            Expression::Literal { lit } => lit.span(),
            Expression::Group { expr, span } => *span,
            Expression::App { app } => app.span(),
        }
    }
}

impl Spans for Application {
    fn span(&self) -> Span {
        match self {
            Application::Unary { op, expr } => (op.span().0, expr.span().1),
            Application::Binary { op, left, right } => (left.span().0, right.span().1),
        }
    }
}

impl Spans for UnaryOperator {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spans for BinaryOperator {
    fn span(&self) -> Span {
        self.span
    }
}

// impl ast

impl Literal {
    /// create a literal given a value and token
    pub fn from_bool(tok: &Token, val: bool) -> Self {
        Self::Boolean {
            val,
            span: tok.span(),
        }
    }

    pub fn from_char(tok: &Token, val: char) -> Self {
        Self::Char {
            val,
            span: tok.span(),
        }
    }

    pub fn from_number(tok: &Token, val: i32) -> Self {
        Self::Number {
            val,
            span: tok.span(),
        }
    }
}

impl UnaryOperator {
    pub fn from(tok: &Token, ty: OperatorType) -> Self {
        Self {
            ty,
            f: match ty {
                OperatorType::Minus => OP_UNARY_MINUS,
                OperatorType::Not => OP_NOT,
                _ => panic!("unchecked UnaryOperator::from()"),
            },
            span: tok.span(),
        }
    }
}

impl BinaryOperator {
    pub fn from(tok: &Token, ty: OperatorType) -> Self {
        Self {
            ty,
            f: match ty {
                OperatorType::Plus => OP_PLUS,
                OperatorType::Minus => OP_BINARY_MINUS,
                OperatorType::Divide => OP_DIVIDE,
                OperatorType::Multiply => OP_MULTIPLY,
                OperatorType::LessThan => OP_LESSTHAN,
                OperatorType::GreaterThan => OP_GREATERTHAN,
                OperatorType::LessThanEq => OP_LESSTHANEQUAL,
                OperatorType::GreaterThanEq => OP_GREATERTHANEQUAL,
                OperatorType::Equal => OP_EQUAL,
                OperatorType::NotEqual => OP_NOTEQUAL,
                _ => panic!("unchecked UnaryOperator::from()"),
            },
            span: tok.span(),
        }
    }
}

// impl display for ast

impl std::fmt::Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Char { val, span } => write!(f, "{val}"),
            Literal::Boolean { val, span } => write!(f, "{val}"),
            Literal::Number { val, span } => write!(f, "{val}"),
        }
    }
}
