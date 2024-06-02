use std::collections::HashMap;

use self::syntax::{Block, Instruction, Reg};
use crate::{
    error::{LangError, ERROR_NULL_VARIABLE_EXPRESSION, ERROR_UNINITIALISED_VARIABLE},
    parse::ast::{
        Application, Expression, Let, Literal, Mutate, OperatorType, Spans, Statement,
        UnaryOperator,
    },
};
use segment::Segment;

mod segment;
mod syntax;

// temp pub struct
#[derive(Debug)]
pub struct GenerationState {
    next_reg: Reg,
    variables: HashMap<String, Reg>,
}

impl GenerationState {
    pub fn new() -> Self {
        Self {
            next_reg: Reg(0),
            variables: HashMap::new(),
        }
    }

    fn next_register(&mut self) -> Reg {
        let reg = self.next_reg;
        (*self.next_reg) += 1;
        reg
    }

    /// initialise a variable in the program. Returns the register it was allocated to
    fn initialise_variable(&mut self, variable: String, register: Reg) {
        self.variables.insert(variable, register);
    }

    /// checks if a variable is already initialised
    fn variable_exists(&mut self, variable: String) -> bool {
        self.variables.contains_key(&variable)
    }

    fn variable_register(
        &mut self,
        variable: &String,
        span: (usize, usize),
    ) -> Result<&Reg, LangError> {
        match self.variables.get(variable) {
            Some(reg) => Ok(reg),
            None => Err(LangError::from(
                format!("uninitialised variable `{}`", variable),
                span,
                ERROR_UNINITIALISED_VARIABLE,
            )),
        }
    }
}

pub trait Lowerable {
    fn lower(&self, state: &mut GenerationState) -> Result<Segment, LangError>;
}

impl Lowerable for Literal {
    fn lower(&self, state: &mut GenerationState) -> Result<Segment, LangError> {
        let reg = state.next_register();
        Ok(match self {
            Literal::Char { val, span } => {
                Segment::block_from_inst(Instruction::CON(reg, *val as u32), *span)
            }
            Literal::Number { val, span } => {
                Segment::block_from_inst(Instruction::CON(reg, *val as u32), *span)
            }
            Literal::Boolean { val, span } => {
                Segment::block_from_inst(Instruction::CON(reg, *val as u32), *span)
            }
            Literal::Unit => todo!("implement unit literal"),
        })
    }
}

impl Lowerable for Application {
    fn lower(&self, state: &mut GenerationState) -> Result<Segment, LangError> {
        match self {
            Application::Unary { op, expr } => {
                let mut block: Segment = expr.lower(state)?;
                let block_output_register = block
                    .output_register()
                    .expect("expected expr to have Some() output register");
                match op.ty() {
                    OperatorType::Not => {
                        block.append_inst(
                            Instruction::NOT(state.next_register(), block_output_register),
                            self.span(),
                        );
                    }
                    OperatorType::Minus => {
                        let rx: Reg = state.next_register();
                        block.append_inst(Instruction::CON(rx, 0), self.span());
                        block.append_inst(
                            Instruction::SUB(state.next_register(), rx, block_output_register),
                            self.span(),
                        );
                    }
                    _ => unreachable!(),
                }
                Ok(block)
            }
            Application::Binary { op, left, right } => {
                let mut rx_block: Segment = left.lower(state)?;
                let rx: Reg = rx_block
                    .output_register()
                    .expect("expected expr to have Some() output register");
                let ry_block: Segment = right.lower(state)?;
                let ry: Reg = ry_block
                    .output_register()
                    .expect("expected expr to have Some() output register");
                rx_block.extend(ry_block);

                rx_block.append_inst(
                    match op.ty() {
                        OperatorType::Plus => Instruction::ADD(state.next_register(), rx, ry),
                        OperatorType::Minus => Instruction::SUB(state.next_register(), rx, ry),
                        OperatorType::Multiply => Instruction::MUL(state.next_register(), rx, ry),
                        OperatorType::Divide => Instruction::DIV(state.next_register(), rx, ry),
                        _ => todo!(),
                    },
                    self.span(),
                );
                Ok(rx_block)
            }
        }
    }
}

impl Lowerable for Expression {
    fn lower(&self, state: &mut GenerationState) -> Result<Segment, LangError> {
        match self {
            Expression::Literal { lit } => lit.lower(state),
            Expression::App { app } => app.lower(state),
            Expression::Group { expr, span: _ } => expr.lower(state),
            Expression::Identifier { id, span } => {
                // set the output register to be the variable register, and give back a block with no instructions
                let variable_register = state.variable_register(&id, self.span())?;
                let mut block = Segment::empty_block();
                block.set_span(*span);
                block.set_output_register(variable_register.to_owned());
                Ok(block)
            }
        }
    }
}

impl Lowerable for Statement {
    fn lower(&self, state: &mut GenerationState) -> Result<Segment, LangError> {
        match self {
            Statement::Expr(expr) => expr.lower(state),
            Statement::Let(r#let) => r#let.lower(state),
            Statement::Mutate(mutate) => mutate.lower(state),
            // Statement::If(r#if) => r#if.lower(),
            _ => todo!(),
        }
    }
}

impl Lowerable for Let {
    fn lower(&self, state: &mut GenerationState) -> Result<Segment, LangError> {
        let block: Segment = self.value.lower(state)?;

        match block.output_register() {
            // this doesn't require an instruction
            Some(reg) => state.initialise_variable(self.variable.clone(), reg),
            None => {
                return Err(LangError::from(
                    "let statement expression evaluates to nothing".to_owned(),
                    self.value.span(),
                    ERROR_NULL_VARIABLE_EXPRESSION,
                ))
            }
        }
        Ok(block)
    }
}

impl Lowerable for Mutate {
    fn lower(&self, state: &mut GenerationState) -> Result<Segment, LangError> {
        let mut block: Segment = self.value.lower(state)?;

        let variable_register = state.variable_register(&self.variable, self.span())?;

        match block.output_register() {
            Some(reg) => {
                block.append_inst(
                    Instruction::MOV(variable_register.to_owned(), reg),
                    self.span(),
                );
            }
            None => {
                return Err(LangError::from(
                    "let statement expression evaluates to nothing".to_owned(),
                    self.value.span(),
                    ERROR_NULL_VARIABLE_EXPRESSION,
                ))
            }
        }

        Ok(block)
    }
}
