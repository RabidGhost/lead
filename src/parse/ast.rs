use crate::{error::{parse_error::MixedTypeExpressionError, LangError}, lex::token::{Token, TokenType}};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LangType {
    Boolean,
    Char,
    Number
}

pub trait LangExpression {
    // fn eval()
    // fn compile?()
    fn type_check(&self) -> Result<(), Box<dyn LangError>>;

    fn lang_type(&self) -> LangType;
}

// Boolean
// struct BooleanValue {
//     token: Token //must be "true" or "false". Is validated
// }


#[derive(Debug, PartialEq, Eq)]
pub enum Expression {
    ExprLiteral(Literal),
    ExprGrouping(Grouping),
    ExprUnary(UnaryApplication),
    ExprBinary(BinaryApplication),
}

impl LangExpression for Expression {
    fn type_check(&self) -> Result<(), Box<dyn LangError>> {
        todo!()
    }

    fn lang_type(&self) -> LangType {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Grouping {
    expression: Box<Expression>,
}

impl Grouping {
    pub fn new(expression: Expression) -> Self {
        Self { expression: Box::new(expression) }
    }
}

impl LangExpression for Grouping {
    fn type_check(&self) -> Result<(), Box<dyn LangError>> {
        self.expression.type_check()
    }

    fn lang_type(&self) -> LangType {
        self.expression.lang_type()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct BinaryOperator {
    operator: Token,
    ty: LangType,
}

impl BinaryOperator {
    pub fn new(operator: Token, ty: LangType) -> Self {
        Self { operator, ty }
    }
}

impl LangExpression for BinaryOperator {
    fn type_check(&self) -> Result<(), Box<dyn LangError>> {
        match self.operator.token_type() {
            TokenType::EqEq | TokenType::BangEq => { // this is missing "&&" and "||"
                if self.ty == LangType::Boolean { // these should be able to apply to most things, not just bools
                    Ok(())
                } else {
                    Err(Box::new(MixedTypeExpressionError::new(LangType::Boolean, self.ty)))
                }
            },
            TokenType::LessThan | TokenType::LessThanEq | TokenType::GreaterThan
            | TokenType::GreaterThanEq | TokenType::Plus | TokenType::Minus
            | TokenType::Star | TokenType::Slash => {
                if self.ty == LangType::Boolean {
                    Ok(())
                } else {
                    Err(Box::new(MixedTypeExpressionError::new(LangType::Number, self.ty)))
                }
            },
            _ => todo!(),
        }
    }

    fn lang_type(&self) -> LangType {
        self.ty
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct UnaryOperator {
    operator: Token,
    ty: LangType,
}

impl UnaryOperator {
    pub fn new(operator: Token, ty: LangType) -> Self {
        UnaryOperator { operator, ty }
    }
}

impl LangExpression for UnaryOperator {
    fn type_check(&self) -> Result<(), Box<dyn LangError>> {
        match self.operator.token_type() {
            TokenType::Bang => {
                if self.ty == LangType::Boolean {
                    Ok(())
                } else {
                    Err(Box::new(MixedTypeExpressionError::new(LangType::Boolean, self.ty)))
                }
            },
            TokenType::Minus => {
                if self.ty == LangType::Number {
                    Ok(())
                } else {
                    Err(Box::new(MixedTypeExpressionError::new(LangType::Number, self.ty)))
                }
            }
            _ => todo!()
            
        }
    }

    fn lang_type(&self) -> LangType {
        self.ty
    }
}

#[derive(Debug, PartialEq, Eq)]
struct BinaryApplication {
    operator: BinaryOperator,
    left: Box<Expression>,
    right: Box<Expression>,
}

#[derive(Debug, PartialEq, Eq)]
struct UnaryApplication {
    operator: UnaryOperator,
    expression: Box<Expression>,
}

impl UnaryApplication {
    fn new(operator: UnaryOperator, expression: Expression) -> Self {
        UnaryApplication { operator, expression: Box::new(expression) }
    }
}

impl LangExpression for UnaryApplication {
    fn type_check(&self) -> Result<(), Box<dyn LangError>> {
        if self.operator.ty == (*self.expression).lang_type() {
            Ok(())
        } else {
            Err(Box::new(
                MixedTypeExpressionError::new(self.operator.ty, self.expression.lang_type())
            ))
        }
    }

    fn lang_type(&self) -> LangType {
        self.operator.ty
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Literal {
    literal: Token,
    ty: LangType,
}

impl Literal {
    pub fn new(literal: Token, ty: LangType) -> Literal {
        Literal { literal, ty}
    }
}

impl LangExpression for Literal {
    fn type_check(&self) -> Result<(), Box<dyn LangError>> {
        Ok(()) // raw literals always pass type checks.
    }

    fn lang_type(&self) -> LangType {
        self.ty
    }
}