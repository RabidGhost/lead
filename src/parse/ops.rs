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

pub const OP_PLUS: fn(Literal, Literal) -> Result<Literal, LangError> = op_plus;
pub const OP_BINARY_MINUS: fn(Literal, Literal) -> Result<Literal, LangError> = op_minus;
pub const OP_DIVIDE: fn(Literal, Literal) -> Result<Literal, LangError> = op_divide;
pub const OP_MULTIPLY: fn(Literal, Literal) -> Result<Literal, LangError> = op_multiply;
pub const OP_LESSTHAN: fn(Literal, Literal) -> Result<Literal, LangError> = op_lessthan;
pub const OP_GREATERTHAN: fn(Literal, Literal) -> Result<Literal, LangError> = op_greaterthan;
pub const OP_LESSTHANEQUAL: fn(Literal, Literal) -> Result<Literal, LangError> = op_lessthanequal;
pub const OP_GREATERTHANEQUAL: fn(Literal, Literal) -> Result<Literal, LangError> =
    op_greaterthanequal;
pub const OP_EQUAL: fn(Literal, Literal) -> Result<Literal, LangError> = op_equal;
pub const OP_NOTEQUAL: fn(Literal, Literal) -> Result<Literal, LangError> = op_notequal;

fn op_plus(left: Literal, right: Literal) -> Result<Literal, LangError> {
    let l: i32 = left.try_into()?;
    let r: i32 = right.try_into()?;

    Ok(Literal::Number {
        val: l + r,
        span: (0, 0),
    })
}

fn op_minus(left: Literal, right: Literal) -> Result<Literal, LangError> {
    let l: i32 = left.try_into()?;
    let r: i32 = right.try_into()?;

    Ok(Literal::Number {
        val: l - r,
        span: (0, 0),
    })
}

fn op_multiply(left: Literal, right: Literal) -> Result<Literal, LangError> {
    let l: i32 = left.try_into()?;
    let r: i32 = right.try_into()?;

    Ok(Literal::Number {
        val: l * r,
        span: (0, 0),
    })
}

fn op_divide(left: Literal, right: Literal) -> Result<Literal, LangError> {
    let l: i32 = left.try_into()?;
    let r: i32 = right.try_into()?;

    Ok(Literal::Number {
        val: l / r,
        span: (0, 0),
    })
}

fn op_greaterthan(left: Literal, right: Literal) -> Result<Literal, LangError> {
    let l: i32 = left.try_into()?;
    let r: i32 = right.try_into()?;

    Ok(Literal::Boolean {
        val: l > r,
        span: (0, 0),
    })
}

fn op_lessthan(left: Literal, right: Literal) -> Result<Literal, LangError> {
    let l: i32 = left.try_into()?;
    let r: i32 = right.try_into()?;

    Ok(Literal::Boolean {
        val: l < r,
        span: (0, 0),
    })
}

fn op_greaterthanequal(left: Literal, right: Literal) -> Result<Literal, LangError> {
    let l: i32 = left.try_into()?;
    let r: i32 = right.try_into()?;

    Ok(Literal::Boolean {
        val: l >= r,
        span: (0, 0),
    })
}

fn op_lessthanequal(left: Literal, right: Literal) -> Result<Literal, LangError> {
    let l: i32 = left.try_into()?;
    let r: i32 = right.try_into()?;

    Ok(Literal::Boolean {
        val: l <= r,
        span: (0, 0),
    })
}

fn op_equal(left: Literal, right: Literal) -> Result<Literal, LangError> {
    let l: bool = left.try_into()?;
    let r: bool = right.try_into()?;

    Ok(Literal::Boolean {
        val: l == r,
        span: (0, 0),
    })
}

fn op_notequal(left: Literal, right: Literal) -> Result<Literal, LangError> {
    let l: bool = left.try_into()?;
    let r: bool = right.try_into()?;

    Ok(Literal::Boolean {
        val: l != r,
        span: (0, 0),
    })
}

pub fn _no_func(_a: Literal, _b: Literal) -> Result<Literal, LangError> {
    unimplemented!()
}
