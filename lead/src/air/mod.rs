use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::LangError,
    lex::span::*,
    parse::ast::{
        Application, Expression, Identifier, If, Let, Literal, Mutate, OperatorType, Statement,
        While,
    },
};
use air::{Flag, Inst, Instruction, Mode, Reg};
use block::Block;

pub mod air;
mod block;

/// Word size in bytes. This does not modify behavior program wide
const WORD_SIZE: usize = 4;

// temp pub struct
#[derive(Debug)]
pub struct GenerationState {
    next_reg: Reg,
    variables: HashMap<String, Reg>,
    /// The variable pointers in a program.
    pointers: HashMap<String, usize>,
    /// the address of the next place in memory to store arrays and strings.
    next_mem_addr: usize,
}

impl GenerationState {
    pub fn new() -> Self {
        Self {
            next_reg: Reg(0),
            variables: HashMap::new(),
            pointers: HashMap::new(),
            next_mem_addr: 0,
        }
    }

    fn next_register(&mut self) -> Reg {
        let reg = self.next_reg;
        (*self.next_reg) += 1;
        reg
    }

    fn next_mem_addr(&mut self) -> usize {
        let addr = self.next_mem_addr;
        self.next_mem_addr += 4; // the current word size in bytes
        addr
    }

    /// initialise a variable in the program. Returns the register it was allocated to
    fn initialise_variable(&mut self, variable: String, register: Reg) {
        self.variables.insert(variable, register);
    }

    /// initialise a pointer to a variable in the program.
    fn initialise_pointer(&mut self, variable: String, pointer: usize) {
        self.pointers.insert(variable, pointer);
    }

    /// checks if a variable is already initialised
    fn variable_exists(&mut self, variable: String) -> bool {
        self.variables.contains_key(&variable)
    }

    fn variable_register(
        &mut self,
        variable: &String,
        span: impl Spans,
    ) -> Result<&Reg, LangError> {
        match self.variables.get(variable) {
            Some(reg) => Ok(reg),
            None => Err(LangError::UninitialisedVariable {
                span: span.span(),
                name: variable.to_owned(),
            }),
        }
    }

    fn deref_pointer(&self, variable: &String, span: impl Spans) -> Result<usize, LangError> {
        match self.pointers.get(variable) {
            Some(pointer) => Ok(*pointer),
            None => Err(LangError::UninitialisedPointer {
                span: span.span(),
                name: variable.to_owned(),
            }),
        }
    }
}

/// Generate a nested unoptimised program.
pub fn generate_program(
    state: &mut GenerationState,
    statements: Vec<Statement>,
) -> Result<Vec<Block>, LangError> {
    let mut segments = Vec::new();
    for statement in statements {
        segments.push(statement.lower(state)?)
    }
    Ok(segments)
}

pub trait Lowerable {
    fn lower(&self, state: &mut GenerationState) -> Result<Block, LangError>;
}

impl Lowerable for Literal {
    fn lower(&self, state: &mut GenerationState) -> Result<Block, LangError> {
        let reg = state.next_register();
        Ok(match self {
            Literal::Char { val, span } => {
                Block::new(Inst::new(Instruction::CON(reg, *val as u32), *span))
            }
            Literal::Number { val, span } => {
                Block::new(Inst::new(Instruction::CON(reg, *val as u32), *span))
            }
            Literal::Boolean { val, span } => {
                Block::new(Inst::new(Instruction::CON(reg, *val as u32), *span))
            }
        })
    }
}

impl Lowerable for Application {
    fn lower(&self, state: &mut GenerationState) -> Result<Block, LangError> {
        match self {
            Application::Unary { op, expr, span } => {
                let mut block: Block = expr.lower(state)?;
                match op {
                    OperatorType::Not => {
                        block.append_inst(Inst::new(
                            Instruction::NOT(
                                state.next_register(),
                                block.output_register_unchecked(),
                            ),
                            *span,
                        ));
                    }
                    OperatorType::Minus => {
                        let rx: Reg = state.next_register();
                        let block_output_register = block.output_register_unchecked();
                        block.append_inst(Inst::new(Instruction::CON(rx, 0), *span));
                        block.append_inst(Inst::new(
                            Instruction::SUB(state.next_register(), rx, block_output_register),
                            *span,
                        ));
                    }
                    _ => unreachable!(),
                }
                Ok(block)
            }
            Application::Binary {
                op,
                left,
                right,
                span,
            } => {
                let mut rx_block: Block = left.lower(state)?;
                let rx: Reg = rx_block.output_register_unchecked();
                let ry_block: Block = right.lower(state)?;
                let ry: Reg = ry_block.output_register_unchecked();

                rx_block.extend(ry_block);

                rx_block.append_inst(Inst::new(
                    match op {
                        OperatorType::Plus => Instruction::ADD(state.next_register(), rx, ry),
                        OperatorType::Minus => Instruction::SUB(state.next_register(), rx, ry),
                        OperatorType::Multiply => Instruction::MUL(state.next_register(), rx, ry),
                        OperatorType::Divide => Instruction::DIV(state.next_register(), rx, ry),
                        OperatorType::LessThan => Instruction::CMP(rx, ry, Some(Flag::Lt)),
                        OperatorType::LessThanEq => Instruction::CMP(rx, ry, Some(Flag::Le)),
                        OperatorType::GreaterThan => Instruction::CMP(rx, ry, Some(Flag::Gt)),
                        OperatorType::GreaterThanEq => Instruction::CMP(rx, ry, Some(Flag::Ge)),
                        OperatorType::NotEqual => Instruction::CMP(rx, ry, Some(Flag::Ne)),
                        OperatorType::Equal => Instruction::CMP(rx, ry, Some(Flag::Eq)),
                        _ => unreachable!("OperatorType::Not is a unary operator "),
                    },
                    *span,
                ));
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
            Expression::Identifier(identifier) => identifier.lower(state),
            Expression::Array {
                elements: array_elements,
                span,
            } => {
                let mut array_initialisation = Block::empty();
                let reg_index = state.next_register();
                let offset = state.next_register();
                array_initialisation.append_inst(Inst::new(
                    Instruction::CON(reg_index, state.next_mem_addr() as u32),
                    *span,
                ));
                array_initialisation
                    .append_inst(Inst::new(Instruction::CON(offset, WORD_SIZE as u32), *span));

                for element in array_elements {
                    let element_expr: Block = element.lower(state)?;
                    array_initialisation.extend(element_expr);
                    array_initialisation.append_inst(Inst::new(
                        Instruction::STR(
                            array_initialisation.output_register().unwrap(),
                            reg_index,
                            Mode::PostOffset(offset),
                        ),
                        element.span(),
                    ))
                }

                // should store the base address of the array in the register of the variable.
                // this might mean moving array intialisation to a statement.

                Ok(array_initialisation)
            }
            Expression::Index {
                variable,
                index: index_expr,
                span,
            } => {
                let base_addr = state.deref_pointer(variable.borrow_name(), span)?;
                let r_base_addr = state.next_register();
                let mut block: Block = Block::new(Inst::new(
                    Instruction::CON(r_base_addr, base_addr as u32),
                    *span,
                ));

                block.extend(index_expr.lower(state)?);

                let r_index_output = block.output_register_unchecked();
                let r_word_size = state.next_register();
                block.append_inst(Inst::new(
                    Instruction::CON(r_word_size, WORD_SIZE as u32),
                    index_expr.span(),
                ));
                let r_index = state.next_register();
                block.append_inst(Inst::new(
                    Instruction::MUL(r_index, r_index_output, r_word_size),
                    index_expr.span(),
                ));

                let r_data = state.next_register();

                block.append_inst(Inst::new(
                    Instruction::LDR(r_data, r_base_addr, Mode::Offset(r_index)),
                    *span,
                ));

                // this shouldnt be needed
                Ok(block)
            }
        }
    }
}

impl Lowerable for Statement {
    fn lower(&self, state: &mut GenerationState) -> Result<Block, LangError> {
        match self {
            Statement::Expr(expr) => expr.lower(state),
            Statement::Let(r#let) => r#let.lower(state),
            Statement::Mutate(mutate) => mutate.lower(state),
            Statement::If(r#if) => r#if.lower(state),
            Statement::While(r#while) => r#while.lower(state),
            Statement::Yield(expr) => {
                let mut expr_block = expr.lower(state)?;
                match expr_block.output_register() {
                    None => (),
                    Some(reg) => {
                        // todo, change this to use the yield instructions span.
                        expr_block.append_inst(Inst::new(Instruction::YLD(reg), expr.span()))
                    }
                };
                Ok(expr_block)
            }
        }
    }
}

impl Lowerable for If {
    fn lower(&self, state: &mut GenerationState) -> Result<Block, LangError> {
        // the condition should contain an AST `CMP` instruction, which will contain a flag hint.
        let mut if_block: Block = self.condition.lower(state)?;

        // we check the latest flag hint, and if it exists, generate a `CHK` instruction for it
        // if no flag hint exists, then the we generate a `CHK Nv` (check never) instruction
        if_block.append_inst(Inst::new(
            Instruction::CHK(if_block.latest_flag_hint().unwrap_or(Flag::Nv)),
            self.condition.span(),
        ));

        let mut if_label = Uuid::new_v4().as_hyphenated().to_string();
        if_label.push_str("-if");

        if_block.append_inst(Inst::new(Instruction::BRA(if_label.clone()), self.span()));

        let mut inner_block: Block = Block::new(Inst::new(Instruction::LBL(if_label), self.span()));

        for statement in self.iff.iter() {
            inner_block.extend(statement.lower(state)?);
        }

        if_block.extend(inner_block);

        Ok(if_block)
    }
}

impl Lowerable for While {
    fn lower(&self, state: &mut GenerationState) -> Result<Block, LangError> {
        let label_uuid = Uuid::new_v4().as_hyphenated().to_string();
        let check_condition_label = format!("{}-check-condition", label_uuid.clone());

        let mut while_block = Block::new(Inst::new(
            Instruction::LBL(check_condition_label.clone()),
            self.condition.span(),
        ));

        while_block.extend(self.condition.lower(state)?);

        // let loop_label = format!("{}-loop", label_uuid.clone()); // no need for a loop label
        let break_label = format!("{}-break", label_uuid);

        while_block.append_inst(Inst::new(
            Instruction::CHK(
                while_block
                    .latest_flag_hint()
                    .map_or(Flag::Nv, |flag| flag.negate()),
            ),
            self.condition.span(),
        ));

        // append a branch to break if the condition is unsuccessful.

        while_block.append_inst(Inst::new(
            Instruction::BRA(break_label.clone()),
            self.condition.span(),
        ));

        // the block inside the {}.
        let mut inner_block: Block = Block::empty();

        for statement in self.body.iter() {
            inner_block.extend(statement.lower(state)?);
        }

        // add a jump back to the condition check
        inner_block.append_inst(Inst::new(
            Instruction::BRA(check_condition_label),
            self.condition.span(),
        ));

        // add the inner instructions to the whole loop body
        while_block.extend(inner_block);

        // finally add the break label

        while_block.append_inst(Inst::new(
            Instruction::LBL(break_label),
            self.condition.span(),
        ));

        Ok(while_block)
    }
}

impl Lowerable for Let {
    fn lower(&self, state: &mut GenerationState) -> Result<Block, LangError> {
        Ok(match self.value {
            Expression::Array {
                elements: _,
                span: _,
            } => {
                let base_mem_addr = state.next_mem_addr; // this doesnt increment the address unlike the method call
                state.initialise_pointer(self.variable.clone(), base_mem_addr);
                self.value.lower(state)?
            }
            _ => {
                let block: Block = self.value.lower(state)?;

                match block.output_register() {
                    // this doesn't require an instruction
                    Some(reg) => state.initialise_variable(self.variable.clone(), reg),
                    None => {
                        return Err(LangError::NullValueExpression {
                            span: self.value.span(),
                        })
                    }
                }
                block
            }
        })
    }
}

impl Lowerable for Mutate {
    fn lower(&self, state: &mut GenerationState) -> Result<Block, LangError> {
        let mut block: Block = self.value.lower(state)?;

        let variable_register = state.variable_register(&self.variable, self.span())?;

        match block.output_register() {
            Some(reg) => {
                block.append_inst(Inst::new(
                    Instruction::MOV(variable_register.to_owned(), reg),
                    self.span(),
                ));
            }
            None => {
                return Err(LangError::NullValueExpression {
                    span: self.value.span(),
                })
            }
        }

        Ok(block)
    }
}

impl Lowerable for Identifier {
    fn lower(&self, state: &mut GenerationState) -> Result<Block, LangError> {
        // set the output register to be the variable register, and give back a block with no instructions
        let variable_register = state.variable_register(self.borrow_name(), self)?;
        let mut block = Block::empty();
        block.set_output_register(Some(variable_register.to_owned()));
        Ok(block)
    }
}
