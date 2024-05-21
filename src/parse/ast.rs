use crate::error::LangError;

type Span = (usize, usize);
type OpRet = Result<Literal, LangError>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Literal {
    Boolean { val: bool, span: Span },
    Char { val: char, span: Span },
    Number { val: i32, span: Span },
}

pub enum Expression {
    App { app: Application },
    Group { expr: Box<Expression>, span: Span },
    Literal { lit: Literal },
}

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

pub enum UnaryOperator {
    Not { f: fn(Literal) -> OpRet, span: Span },
    Minus { f: fn(Literal) -> OpRet, span: Span },
}

pub enum BinaryOperator {
    Placeholder {
        f: fn(Literal, Literal) -> OpRet,
        span: Span,
    },
}

trait Spans {
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
        match self {
            UnaryOperator::Not { f, span } => *span,
            UnaryOperator::Minus { f, span } => *span,
        }
    }
}

impl Spans for BinaryOperator {
    fn span(&self) -> Span {
        match self {
            BinaryOperator::Placeholder { f, span } => *span,
        }
    }
}
