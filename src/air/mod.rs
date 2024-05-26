use self::synatx::{Block, Instruction, Reg};
use crate::{
    error::LangError,
    parse::ast::{Application, Expression, Literal, OperatorType, UnaryOperator},
};

mod synatx;

static mut REGISTER_TRACKER: RegisterTracker = RegisterTracker::get_self();

struct RegisterTracker {
    next: Reg,
}

impl RegisterTracker {
    const fn get_self() -> Self {
        Self { next: Reg(0) }
    }

    fn get_next_register(&mut self) -> Reg {
        let reg = self.next;
        *self.next += 1;
        reg
    }
}

fn next_register() -> Reg {
    unsafe { REGISTER_TRACKER.get_next_register() }
}

pub trait Lowerable {
    fn lower(&self) -> Result<Block, LangError>;
}

impl Lowerable for Literal {
    fn lower(&self) -> Result<Block, LangError> {
        let reg = next_register();
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
    fn lower(&self) -> Result<Block, LangError> {
        match self {
            Application::Unary { op, expr } => {
                let mut block: Block = expr.lower()?;
                match op.ty() {
                    OperatorType::Not => {
                        block.append(Instruction::NOT(next_register(), block.output_register));
                    }
                    OperatorType::Minus => {
                        let ry: Reg = block.output_register;
                        let rx: Reg = next_register();
                        block.append(Instruction::CON(rx, 0));
                        block.append(Instruction::SUB(next_register(), rx, ry));
                    }
                    _ => unreachable!(),
                }
                Ok(block)
            }
            Application::Binary { op, left, right } => {
                let mut rx_block: Block = left.lower()?;
                let rx: Reg = rx_block.output_register;
                let ry_block: Block = right.lower()?;
                let ry: Reg = ry_block.output_register;
                rx_block.extend(ry_block);

                rx_block.append(match op.ty() {
                    OperatorType::Plus => Instruction::ADD(next_register(), rx, ry),
                    OperatorType::Minus => Instruction::SUB(next_register(), rx, ry),
                    OperatorType::Multiply => Instruction::MUL(next_register(), rx, ry),
                    OperatorType::Divide => Instruction::DIV(next_register(), rx, ry),
                    _ => todo!(),
                });
                Ok(rx_block)
            }
            _ => todo!(),
        }
    }
}

impl Lowerable for Expression {
    fn lower(&self) -> Result<Block, LangError> {
        match self {
            Expression::Literal { lit } => lit.lower(),
            Expression::App { app } => app.lower(), // incomplete implementation
            Expression::Group { expr, span: _ } => expr.lower(),
            _ => todo!(),
        }
    }
}
