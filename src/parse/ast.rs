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
    Unit,
}

#[derive(Debug)]
pub enum Expression {
    App { app: Application },
    Group { expr: Box<Expression>, span: Span },
    Literal { lit: Literal },
    Identifier { id: String, span: Span },
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

#[derive(Debug)]
pub struct Mutate {
    pub variable: String,
    pub value: Expression,
    span: Span,
}

#[derive(Debug)]
pub struct Let {
    pub variable: String,
    pub value: Expression,
    span: Span,
}

#[derive(Debug)]
pub struct If {
    pub condition: Expression,
    pub iff: Vec<Statement>,
    span: Span,
}

#[derive(Debug)]
pub struct While {
    pub condition: Expression,
    pub body: Vec<Statement>,
    span: Span,
}

#[derive(Debug)]
pub enum Statement {
    Let(Let),
    Mutate(Mutate),
    Expr(Expression),
    If(If),
    While(While),
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
            Literal::Char { val: _, span } => *span,
            Literal::Number { val: _, span } => *span,
            Literal::Boolean { val: _, span } => *span,
            Literal::Unit => (0, 0),
        }
    }
}

impl Spans for Expression {
    fn span(&self) -> Span {
        match self {
            Expression::Literal { lit } => lit.span(),
            Expression::Group { expr: _, span } => *span,
            Expression::App { app } => app.span(),
            Expression::Identifier { id: _, span } => *span,
        }
    }
}

impl Spans for Application {
    fn span(&self) -> Span {
        match self {
            Application::Unary { op, expr } => (op.span().0, expr.span().1),
            Application::Binary { op: _, left, right } => (left.span().0, right.span().1),
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

impl Spans for Mutate {
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

    pub fn f(&self, x: Literal) -> OpRet {
        (self.f)(x)
    }

    pub fn ty(&self) -> OperatorType {
        self.ty
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

    pub fn f(&self, x: Literal, y: Literal) -> OpRet {
        (self.f)(x, y)
    }

    pub fn ty(&self) -> OperatorType {
        self.ty
    }
}

impl Mutate {
    pub fn from(variable: &Token, value: Expression) -> Self {
        let name = match variable.token_type() {
            crate::lex::token::TokenType::Identifier(name) => name.clone(),
            _ => panic!("should not be here"),
        };

        let span = (variable.span().0, value.span().1);

        Mutate {
            variable: name,
            value,
            span,
        }
    }
}

impl Let {
    pub fn from(variable: &Token, value: Expression, start: usize) -> Self {
        // this function includes a start of the span, as we dont know where the `let` was
        let name = match variable.token_type() {
            crate::lex::token::TokenType::Identifier(name) => name.clone(),
            _ => panic!("should not be here"),
        };

        let span = (start, value.span().1);

        Let {
            variable: name,
            value,
            span,
        }
    }
}

impl If {
    pub fn from(condition: Expression, body: Vec<Statement>, span: Span) -> Self {
        Self {
            condition,
            iff: body,
            span,
        }
    }
}

impl While {
    pub fn from(condition: Expression, body: Vec<Statement>, span: Span) -> Self {
        Self {
            condition,
            body,
            span,
        }
    }
}

// identifier trait

pub trait Identifier {
    fn name(&self) -> &str;
    fn span(&self) -> &str;
}

// impl display for ast

impl std::fmt::Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Char { val, span: _ } => write!(f, "{val}"),
            Literal::Boolean { val, span: _ } => write!(f, "{val}"),
            Literal::Number { val, span: _ } => write!(f, "{val}"),
            Literal::Unit => write!(f, "()"),
        }
    }
}
