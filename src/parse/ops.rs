use crate::error::{LangError, ERROR_TYPE_MISMATCH};

use super::ast::*;

// unary operators

pub const OP_NOT: fn(Literal) -> Result<Literal, LangError> = op_not;
pub const OP_UNARY_MINUS: fn(Literal) -> Result<Literal, LangError> = op_unary_minus;

pub fn op_not(tok: Literal) -> Result<Literal, LangError> {
    match tok {
        Literal::Boolean { val, span } => Ok(Literal::Boolean { val: !val, span }),
        _ => Err(LangError::from(
            format!("expected expression of type bool, found `{tok:?}`"),
            tok.span(),
            ERROR_TYPE_MISMATCH,
        )),
    }
}

pub fn op_unary_minus(tok: Literal) -> Result<Literal, LangError> {
    match tok {
        Literal::Number { val, span } => Ok(Literal::Number { val: -val, span }),
        _ => Err(LangError::from(
            format!("expected expression of type number, found `{tok:?}`"),
            tok.span(),
            ERROR_TYPE_MISMATCH,
        )),
    }
}

// binary operators

pub const OP_PLUS: fn(Literal, Literal) -> Result<Literal, LangError> = no_func;
pub const OP_BINARY_MINUS: fn(Literal, Literal) -> Result<Literal, LangError> = no_func;
pub const OP_DIVIDE: fn(Literal, Literal) -> Result<Literal, LangError> = no_func;
pub const OP_MULTIPLY: fn(Literal, Literal) -> Result<Literal, LangError> = no_func;
pub const OP_LESSTHAN: fn(Literal, Literal) -> Result<Literal, LangError> = no_func;
pub const OP_GREATERTHAN: fn(Literal, Literal) -> Result<Literal, LangError> = no_func;
pub const OP_LESSTHANEQUAL: fn(Literal, Literal) -> Result<Literal, LangError> = no_func;
pub const OP_GREATERTHANEQUAL: fn(Literal, Literal) -> Result<Literal, LangError> = no_func;
pub const OP_EQUAL: fn(Literal, Literal) -> Result<Literal, LangError> = no_func;
pub const OP_NOTEQUAL: fn(Literal, Literal) -> Result<Literal, LangError> = no_func;

pub fn no_func(a: Literal, b: Literal) -> Result<Literal, LangError> {
    unimplemented!()
}
