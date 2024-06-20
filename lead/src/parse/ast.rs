use crate::error::LangError;
use crate::lex::{span::*, token::Token};

type Statements = Vec<Statement>;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Literal {
    Boolean { val: bool, span: Span },
    Char { val: char, span: Span },
    Number { val: i32, span: Span },
}

#[derive(Debug, Clone)]
pub enum Expression {
    App {
        app: Application,
    },
    Group {
        expr: Box<Expression>,
        span: Span,
    },
    Literal {
        lit: Literal,
    },
    Identifier(Identifier),
    Array {
        elements: Vec<Box<Expression>>,
        span: Span,
    },
    Index {
        variable: Identifier,
        index: Box<Expression>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub struct Identifier {
    id: String,
    span: Span,
}

impl Identifier {
    pub fn new(name: String, span: impl Spans) -> Self {
        Self {
            id: name,
            span: span.span(),
        }
    }

    pub fn name(&self) -> &str {
        &self.id
    }

    pub fn borrow_name(&self) -> &String {
        &self.id
    }
}

impl Spans for Identifier {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone)]
pub enum Application {
    Unary {
        op: OperatorType,
        expr: Box<Expression>,
        span: Span,
    },
    Binary {
        op: OperatorType,
        left: Box<Expression>,
        right: Box<Expression>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub struct Mutate {
    pub variable: String,
    pub value: Expression,
    span: Span,
}

#[derive(Debug, Clone)]
pub struct Let {
    pub variable: String,
    pub value: Expression,
    span: Span,
}

impl Spans for Let {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone)]
pub struct If {
    pub condition: Expression,
    pub iff: Statements,
    span: Span,
}

#[derive(Debug, Clone)]
pub struct While {
    pub condition: Expression,
    pub body: Statements,
    span: Span,
}

impl Spans for While {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let(Let),
    Mutate(Mutate),
    Expr(Expression),
    If(If),
    While(While),
    Yield(Expression),
}

impl Spans for Statement {
    fn span(&self) -> Span {
        match self {
            Self::Let(r#let) => r#let.span(),
            Self::Mutate(mutate) => mutate.span(),
            Self::Expr(expr) => expr.span(),
            Self::If(r#if) => r#if.span(),
            Self::While(r#while) => r#while.span(),
            Self::Yield(r#yield) => r#yield.span(),
        }
    }
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

impl Spans for Literal {
    fn span(&self) -> Span {
        match self {
            Literal::Char { val: _, span } => *span,
            Literal::Number { val: _, span } => *span,
            Literal::Boolean { val: _, span } => *span,
        }
    }
}

impl Spans for Expression {
    fn span(&self) -> Span {
        match self {
            Expression::Literal { lit } => lit.span(),
            Expression::Group { expr: _, span } => *span,
            Expression::App { app } => app.span(),
            Expression::Identifier(identifier) => identifier.span(),
            Expression::Array { elements: _, span } => *span,
            Expression::Index {
                variable: _,
                index: _,
                span,
            } => *span,
        }
    }
}

impl Spans for Application {
    fn span(&self) -> Span {
        match self {
            Application::Unary {
                op: _,
                expr: _,
                span,
            } => *span,
            Application::Binary {
                op: _,
                left: _,
                right: _,
                span,
            } => *span,
        }
    }
}

impl Spans for Mutate {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spans for If {
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

impl Application {
    /// Create an Application from a unary operator and expression
    pub fn from_unary(tok: &Token, op: OperatorType, expr: Expression) -> Self {
        let span = Span::superspan(&tok.span(), &expr);
        Self::Unary {
            op,
            expr: Box::new(expr),
            span,
        }
    }

    /// Create an Application from a binary operator and two expressions
    pub fn from_binary(op: OperatorType, left: Expression, right: Expression) -> Self {
        let span = Span::superspan(&left.span(), &right.span());
        Self::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
            span,
        }
    }
}

// impl UnaryOperator {
//     pub fn from(tok: &Token, ty: OperatorType) -> Self {
//         Self {
//             ty,
//             f: match ty {
//                 OperatorType::Minus => OP_UNARY_MINUS,
//                 OperatorType::Not => OP_NOT,
//                 _ => panic!("unchecked UnaryOperator::from()"),
//             },
//             span: tok.span(),
//         }
//     }

//     pub fn f(&self, x: Literal) -> OpRet {
//         (self.f)(x)
//     }

//     pub fn ty(&self) -> OperatorType {
//         self.ty
//     }
// }

// impl BinaryOperator {
//     pub fn from(tok: &Token, ty: OperatorType) -> Self {
//         Self {
//             ty,
//             f: match ty {
//                 OperatorType::Plus => OP_PLUS,
//                 OperatorType::Minus => OP_BINARY_MINUS,
//                 OperatorType::Divide => OP_DIVIDE,
//                 OperatorType::Multiply => OP_MULTIPLY,
//                 OperatorType::LessThan => OP_LESSTHAN,
//                 OperatorType::GreaterThan => OP_GREATERTHAN,
//                 OperatorType::LessThanEq => OP_LESSTHANEQUAL,
//                 OperatorType::GreaterThanEq => OP_GREATERTHANEQUAL,
//                 OperatorType::Equal => OP_EQUAL,
//                 OperatorType::NotEqual => OP_NOTEQUAL,
//                 _ => panic!("unchecked UnaryOperator::from()"),
//             },
//             span: tok.span(),
//         }
//     }

//     pub fn f(&self, x: Literal, y: Literal) -> OpRet {
//         (self.f)(x, y)
//     }

//     pub fn ty(&self) -> OperatorType {
//         self.ty
//     }
// }

impl Mutate {
    pub fn from(variable: &Token, value: Expression) -> Self {
        let name = match variable.token_type() {
            crate::lex::token::TokenType::Identifier(name) => name.clone(),
            _ => panic!("should not be here"),
        };

        let span = Span::superspan(&variable.span(), &value.span());

        Mutate {
            variable: name,
            value,
            span,
        }
    }
}

impl Let {
    pub fn from(variable: &Token, value: Expression, start: impl Spans) -> Result<Self, LangError> {
        // this function includes a start of the span, as we dont know where the `let` was
        let name = match variable.token_type() {
            crate::lex::token::TokenType::Identifier(name) => name.clone(),
            ty => {
                return Err(LangError::InvalidIdentifier {
                    span: variable.span(),
                    id_literal: format!("{ty}"),
                })
            }
        };

        let span = Span::superspan(&start.span(), &value.span());

        Ok(Let {
            variable: name,
            value,
            span,
        })
    }
}

impl If {
    pub fn from(condition: Expression, body: Statements, span: Span) -> Self {
        Self {
            condition,
            iff: body,
            span,
        }
    }
}

impl While {
    pub fn from(condition: Expression, body: Statements, span: Span) -> Self {
        Self {
            condition,
            body,
            span,
        }
    }
}

// impl display for ast

impl std::fmt::Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Char { val, span: _ } => write!(f, "{val}"),
            Literal::Boolean { val, span: _ } => write!(f, "{val}"),
            Literal::Number { val, span: _ } => write!(f, "{val}"),
        }
    }
}
