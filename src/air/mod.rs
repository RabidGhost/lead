use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{LangError, ERROR_NULL_VARIABLE_EXPRESSION, ERROR_UNINITIALISED_VARIABLE},
    lex::span::Spans,
    parse::ast::{
        Application, Expression, If, Let, Literal, Mutate, OperatorType, Statement, While,
    },
};
use lead_vm::air::{Flag, Instruction, Reg};
use segment::Segment;

mod segment;

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
            Expression::Identifier { id, span } => {
                // set the output register to be the variable register, and give back a block with no instructions
                let variable_register = state.variable_register(&id, self)?;
                let mut block = Segment::empty_block();
                block.set_span(*span);
                block.set_output_register(variable_register.to_owned());
                Ok(block)
            }
            Expression::Array {
                elements: _,
                span: _,
            } => {
                unimplemented!("no air impementaion for arrays exists yet")
            }
            Expression::Index {
                variable: _,
                index: _,
                span: _,
            } => unimplemented!("no air impementaion for array indexing exists yet"),
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
