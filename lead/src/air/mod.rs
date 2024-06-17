use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{LangError, ERROR_NULL_VARIABLE_EXPRESSION, ERROR_UNINITIALISED_VARIABLE},
    lex::span::Spans,
    parse::ast::{
        Application, Expression, Identifier, If, Let, Literal, Mutate, OperatorType, Statement,
        While,
    },
};
use lead_vm::air::{Flag, Instruction, Mode, Reg};
use segment::Segment;

mod segment;

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
            None => Err(LangError::from(
                format!("uninitialised variable `{}`", variable),
                span,
                ERROR_UNINITIALISED_VARIABLE,
            )),
        }
    }

    fn deref_pointer(&self, variable: &String, span: impl Spans) -> Result<usize, LangError> {
        match self.pointers.get(variable) {
            Some(pointer) => Ok(*pointer),
            None => Err(LangError::from(
                format!("uninitialised pointer to variable `{}`", variable),
                span,
                ERROR_UNINITIALISED_VARIABLE,
            )),
        }
    }
}

/// Generate a nested unoptimised program.
pub fn generate_program(
    state: &mut GenerationState,
    statements: Vec<Statement>,
) -> Result<Vec<Segment>, LangError> {
    let mut segments = Vec::new();
    for statement in statements {
        segments.push(statement.lower(state)?)
    }
    Ok(segments)
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
        })
    }
}

impl Lowerable for Application {
    fn lower(&self, state: &mut GenerationState) -> Result<Segment, LangError> {
        match self {
            Application::Unary { op, expr, span } => {
                let mut block: Segment = expr.lower(state)?;
                let block_output_register = block
                    .output_register()
                    .expect("expected expr to have Some() output register");
                match op {
                    OperatorType::Not => {
                        block.append_inst(
                            Instruction::NOT(state.next_register(), block_output_register),
                            *span,
                        );
                    }
                    OperatorType::Minus => {
                        let rx: Reg = state.next_register();
                        block.append_inst(Instruction::CON(rx, 0), self.span());
                        block.append_inst(
                            Instruction::SUB(state.next_register(), rx, block_output_register),
                            *span,
                        );
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
                let mut rx_block: Segment = left.lower(state)?;
                let rx: Reg = rx_block
                    .output_register()
                    .expect("expected expr to have Some() output register");
                let ry_block: Segment = right.lower(state)?;
                let ry: Reg = ry_block
                    .output_register()
                    .expect("expected expr to have Some() output register");
                rx_block
                    .span_unchecked_mut()
                    .join(ry_block.span_unchecked());
                rx_block.extend(ry_block);

                rx_block.append_inst(
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
            Expression::Identifier(identifier) => identifier.lower(state),
            Expression::Array {
                elements: array_elements,
                span,
            } => {
                let mut array_initialisation = Segment::empty_block();
                let reg_index = state.next_register();
                let offset = state.next_register();
                array_initialisation.append_inst(
                    Instruction::CON(reg_index, state.next_mem_addr() as u32),
                    *span,
                );
                array_initialisation.append_inst(Instruction::CON(offset, WORD_SIZE as u32), *span);
                // let mut index = 0;

                for element in array_elements {
                    let val: Segment = element.lower(state)?;
                    array_initialisation.append_segment(val);
                    array_initialisation.append_inst(
                        Instruction::STR(
                            array_initialisation.output_register().unwrap(),
                            reg_index,
                            Mode::PostOffset(offset),
                        ),
                        *span,
                    )
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
                let mut block: Segment = Segment::subprogram_from_inst(
                    Instruction::CON(r_base_addr, base_addr as u32),
                    *span,
                );

                block.append_segment(index_expr.lower(state)?);
                let r_index_output = block.output_register().unwrap();
                let r_word_size = state.next_register();
                block.append_inst(
                    Instruction::CON(r_word_size, WORD_SIZE as u32),
                    index_expr.span(),
                );
                let r_index = state.next_register();
                block.append_inst(
                    Instruction::MUL(r_index, r_index_output, r_word_size),
                    index_expr.span(),
                );

                let r_data = state.next_register();

                block.append_inst_as_block(
                    Instruction::LDR(r_data, r_base_addr, Mode::Offset(r_index)),
                    *span,
                );
                block.set_output_register(r_data);
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
            Statement::If(r#if) => r#if.lower(state),
            Statement::While(r#while) => r#while.lower(state),
            Statement::Yield(expr) => {
                let mut expr_block = expr.lower(state)?;
                match expr_block.output_register() {
                    None => (),
                    Some(reg) => {
                        expr_block.append_inst(Instruction::YLD(reg), expr_block.span_unchecked())
                    }
                };
                Ok(expr_block)
            }
        }
    }
}

impl Lowerable for If {
    fn lower(&self, state: &mut GenerationState) -> Result<Segment, LangError> {
        let condition: Segment = self.condition.lower(state)?;
        // the condition should contain an AST `CMP` instruction, which will contain a flag hint.
        let mut if_segment = Segment::subprogram_from_segment(condition);
        // we check the latest flag hint, and if it exists, generate a `CHK` instruction for it
        // if no flag hint exists, then the we generate a `CHK Nv` (check never) instruction
        if_segment.append_inst(
            Instruction::CHK(match if_segment.latest_flag_hint() {
                Some(flag) => flag,
                None => Flag::Nv,
            }),
            self.condition.span(),
        );

        let mut if_label = Uuid::new_v4().as_hyphenated().to_string();
        if_label.push_str("-if");

        let branch_if: Segment =
            Segment::block_from_inst(Instruction::BRA(if_label.clone()), self.span());

        let mut sub_program_if: Segment =
            Segment::subprogram_from_inst(Instruction::LBL(if_label), self.span());

        for statement in self.iff.iter() {
            sub_program_if.append_segment(statement.lower(state)?);
        }

        if_segment.append_segment(branch_if);
        if_segment.append_segment(sub_program_if);

        Ok(if_segment)
    }
}

impl Lowerable for While {
    fn lower(&self, state: &mut GenerationState) -> Result<Segment, LangError> {
        let label_uuid = Uuid::new_v4().as_hyphenated().to_string();
        let check_condition_label = format!("{}-check-condition", label_uuid.clone());
        let mut check_condition_block: Segment = Segment::block_from_inst(
            Instruction::LBL(check_condition_label.clone()),
            self.condition.span(),
        );

        check_condition_block.extend(self.condition.lower(state)?);

        let loop_label = format!("{}-loop", label_uuid.clone());
        let break_label = format!("{}-break", label_uuid);

        check_condition_block.append_inst(
            Instruction::CHK(match check_condition_block.latest_flag_hint() {
                Some(flag) => flag,
                None => Flag::Nv,
            }),
            self.condition.span(),
        );
        check_condition_block.append_inst(Instruction::BRA(loop_label), self.condition.span());
        check_condition_block
            .append_inst(Instruction::BRA(break_label.clone()), self.condition.span());

        let mut while_loop: Segment = Segment::subprogram_from_segment(check_condition_block);

        for statement in self.body.iter() {
            while_loop.append_segment(statement.lower(state)?);
        }

        // add a jump back to the condition check
        while_loop.append_inst_as_block(
            Instruction::BRA(check_condition_label),
            self.condition.span(),
        );

        // finally add the break label
        while_loop.append_inst_as_block(Instruction::LBL(break_label), self.condition.span());

        Ok(while_loop)
    }
}

impl Lowerable for Let {
    fn lower(&self, state: &mut GenerationState) -> Result<Segment, LangError> {
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
                block
            }
        })
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

impl Lowerable for Identifier {
    fn lower(&self, state: &mut GenerationState) -> Result<Segment, LangError> {
        // set the output register to be the variable register, and give back a block with no instructions
        let variable_register = state.variable_register(self.borrow_name(), self)?;
        let mut block = Segment::empty_block();
        block.set_span(self.span());
        block.set_output_register(variable_register.to_owned());
        Ok(block)
    }
}
