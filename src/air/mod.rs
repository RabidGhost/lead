use std::collections::HashMap;

use self::synatx::{Block, Instruction, Reg};
use crate::{
    error::LangError,
    parse::ast::{
        Application, Expression, Let, Literal, Mutate, OperatorType, Statement, UnaryOperator,
    },
};

mod synatx;

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
}

pub trait Lowerable {
    fn lower(&self, state: &mut GenerationState) -> Result<Block, LangError>;
}

impl Lowerable for Literal {
    fn lower(&self, state: &mut GenerationState) -> Result<Block, LangError> {
        let reg = state.next_register();
        Ok(match self {
            Literal::Char { val, span } => {
                Block::new(&[Instruction::CON(reg, *val as u32)], reg, *span)
            }
            Literal::Number { val, span } => {
                Block::new(&[Instruction::CON(reg, *val as u32)], reg, *span)
            }
            Literal::Boolean { val, span } => {
                Block::new(&[Instruction::CON(reg, *val as u32)], reg, *span)
            }
            Literal::Unit => todo!("implement unit literal"),
        })
    }
}

impl Lowerable for Application {
    fn lower(&self, state: &mut GenerationState) -> Result<Block, LangError> {
        match self {
            Application::Unary { op, expr } => {
                let mut block: Block = expr.lower(state)?;
                match op.ty() {
                    OperatorType::Not => {
                        block.append(Instruction::NOT(
                            state.next_register(),
                            block.output_register,
                        ));
                    }
                    OperatorType::Minus => {
                        let ry: Reg = block.output_register;
                        let rx: Reg = state.next_register();
                        block.append(Instruction::CON(rx, 0));
                        block.append(Instruction::SUB(state.next_register(), rx, ry));
                    }
                    _ => unreachable!(),
                }
                Ok(block)
            }
            Application::Binary { op, left, right } => {
                let mut rx_block: Block = left.lower(state)?;
                let rx: Reg = rx_block.output_register;
                let ry_block: Block = right.lower(state)?;
                let ry: Reg = ry_block.output_register;
                rx_block.extend(ry_block);

                rx_block.append(match op.ty() {
                    OperatorType::Plus => Instruction::ADD(state.next_register(), rx, ry),
                    OperatorType::Minus => Instruction::SUB(state.next_register(), rx, ry),
                    OperatorType::Multiply => Instruction::MUL(state.next_register(), rx, ry),
                    OperatorType::Divide => Instruction::DIV(state.next_register(), rx, ry),
                    _ => todo!(),
                });
                Ok(rx_block)
            }
        }
    }
}

impl Lowerable for Expression {
    fn lower(&self, state: &mut GenerationState) -> Result<Block, LangError> {
        match self {
            Expression::Literal { lit } => lit.lower(state),
            Expression::App { app } => app.lower(state),
            Expression::Group { expr, span: _ } => expr.lower(state),
            _ => todo!(),
        }
    }
}

impl Lowerable for Statement {
    fn lower(&self, state: &mut GenerationState) -> Result<Block, LangError> {
        match self {
            Statement::Expr(expr) => expr.lower(state),
            Statement::Let(assign) => assign.lower(state),
            _ => todo!(),
        }
    }
}

impl Lowerable for Let {
    fn lower(&self, state: &mut GenerationState) -> Result<Block, LangError> {
        let block: Block = self.value.lower(state)?;

        // this doesnt require any instruction,
        state.initialise_variable(self.variable.clone(), block.output_register);

        Ok(block)
    }
}
