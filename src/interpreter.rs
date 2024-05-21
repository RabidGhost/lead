use crate::{
    error::{LangError, ERROR_TYPE_MISMATCH},
    parse::ast::{Application, Expression, Literal},
};

pub trait Interpretable {
    fn eval(&self) -> Result<Literal, LangError>;
}

impl Interpretable for Literal {
    fn eval(&self) -> Result<Literal, LangError> {
        Ok(*self)
    }
}

impl Interpretable for Application {
    fn eval(&self) -> Result<Literal, LangError> {
        match self {
            Self::Unary { op, expr } => op.f((**expr).eval()?),
            Self::Binary { op, left, right } => op.f((**left).eval()?, (**right).eval()?),
        }
    }
}

impl Interpretable for Expression {
    fn eval(&self) -> Result<Literal, LangError> {
        match self {
            Expression::Literal { lit } => lit.eval(),
            Expression::Group { expr, span: _ } => (**expr).eval(),
            Expression::App { app } => app.eval(),
        }
    }
}

impl TryInto<i32> for Literal {
    type Error = LangError;
    fn try_into(self) -> Result<i32, Self::Error> {
        match self {
            Self::Number { val, span: _ } => Ok(val),
            Self::Char { val: _, span } => Err(LangError::from(
                format!("expected type of `num`, found `char`"),
                span,
                ERROR_TYPE_MISMATCH,
            )),
            Self::Boolean { val: _, span } => Err(LangError::from(
                format!("expected type of `num`, found `bool`"),
                span,
                ERROR_TYPE_MISMATCH,
            )),
        }
    }
}

impl TryInto<char> for Literal {
    type Error = LangError;
    fn try_into(self) -> Result<char, Self::Error> {
        match self {
            Self::Char { val, span: _ } => Ok(val),
            Self::Number { val: _, span } => Err(LangError::from(
                format!("expected type of `char`, found `num`"),
                span,
                ERROR_TYPE_MISMATCH,
            )),
            Self::Boolean { val: _, span } => Err(LangError::from(
                format!("expected type of `char`, found `bool`"),
                span,
                ERROR_TYPE_MISMATCH,
            )),
        }
    }
}

impl TryInto<bool> for Literal {
    type Error = LangError;
    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            Self::Boolean { val, span: _ } => Ok(val),
            Self::Char { val: _, span } => Err(LangError::from(
                format!("expected type of `bool`, found `num`"),
                span,
                ERROR_TYPE_MISMATCH,
            )),
            Self::Number { val: _, span } => Err(LangError::from(
                format!("expected type of `bool`, found `num`"),
                span,
                ERROR_TYPE_MISMATCH,
            )),
        }
    }
}
