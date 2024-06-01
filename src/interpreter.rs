use std::collections::HashMap;

use crate::{
    error::{LangError, ERROR_TYPE_MISMATCH, ERROR_UNKNOWN_VARIABLE},
    parse::ast::{Application, Expression, Literal, Statement},
};

pub struct GlobalAlloc {
    variables: HashMap<String, Literal>,
}

pub trait LangAlloc {
    fn allocate(&mut self, name: String, var: Literal);
    fn is_allocated(&mut self, name: &String) -> bool;
    fn fetch(&self, name: &String) -> Option<Literal>;
}

impl LangAlloc for GlobalAlloc {
    fn allocate(&mut self, name: String, var: Literal) {
        self.variables.insert(name, var);
    }

    fn is_allocated(&mut self, name: &String) -> bool {
        self.variables.contains_key(name)
    }

    fn fetch(&self, name: &String) -> Option<Literal> {
        self.variables.get(name).cloned()
    }
}

impl GlobalAlloc {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// for debugging and testing
    pub fn insert(&mut self, name: String, var: Literal) {
        self.allocate(name, var)
    }
}

pub trait Interpretable {
    fn eval(&self, alloc: &mut impl LangAlloc) -> Result<Literal, LangError>;
}

impl Interpretable for Literal {
    fn eval(&self, alloc: &mut impl LangAlloc) -> Result<Literal, LangError> {
        Ok(*self)
    }
}

impl Interpretable for Application {
    fn eval(&self, alloc: &mut impl LangAlloc) -> Result<Literal, LangError> {
        match self {
            Self::Unary { op, expr } => op.f((**expr).eval(alloc)?),
            Self::Binary { op, left, right } => op.f((**left).eval(alloc)?, (**right).eval(alloc)?),
        }
    }
}

impl Interpretable for Expression {
    fn eval(&self, alloc: &mut impl LangAlloc) -> Result<Literal, LangError> {
        match self {
            Expression::Literal { lit } => lit.eval(alloc),
            Expression::Group { expr, span: _ } => (**expr).eval(alloc),
            Expression::App { app } => app.eval(alloc),
            Expression::Identifier { id, span } => alloc.fetch(id).ok_or(LangError::from(
                format!("no such variable `{}`", id),
                *span,
                ERROR_UNKNOWN_VARIABLE,
            )),
            _ => todo!("implement interpret for indentifiers"),
        }
    }
}

impl Interpretable for Statement {
    fn eval(&self, alloc: &mut impl LangAlloc) -> Result<Literal, LangError> {
        match self {
            Statement::Expr(expr) => expr.eval(alloc),
            Statement::Let(assign) => {
                let expr = assign.value.eval(alloc)?;
                alloc.allocate(assign.variable.clone(), expr);
                Ok(Literal::Unit)
            }
            Statement::Mutate(assign) => {
                if alloc.is_allocated(&assign.variable) {
                    let expr = assign.value.eval(alloc)?;
                    alloc.allocate(assign.variable.clone(), expr);
                    Ok(Literal::Unit)
                } else {
                    todo!("lang error here")
                }

                // this may not be correct
            }
            Statement::If(iff) => {
                let cond: bool = iff.condition.eval(alloc)?.try_into()?;
                if cond {
                    return eval_statements(alloc, &iff.iff);
                }
                Ok(Literal::Unit)
            }
            Statement::While(whilee) => {
                let mut end_state: Literal = Literal::Unit;
                while whilee.condition.eval(alloc)?.try_into()? {
                    end_state = eval_statements(alloc, &whilee.body)?;
                }
                Ok(end_state)
            }
        }
    }
}

fn eval_statements(
    alloc: &mut impl LangAlloc,
    statments: &Vec<Statement>,
) -> Result<Literal, LangError> {
    let mut end_state: Literal = Literal::Unit;
    for statement in statments {
        end_state = statement.eval(alloc)?
    }
    return Ok(end_state);
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
            Self::Unit => Err(LangError::from(
                format!("expected type of `num`, found `()`"),
                (0, 0),
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
            Self::Unit => Err(LangError::from(
                format!("expected type of `char`, found `()`"),
                (0, 0),
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
            Self::Unit => Err(LangError::from(
                format!("expected type of `bool`, found `()`"),
                (0, 0),
                ERROR_TYPE_MISMATCH,
            )),
        }
    }
}
