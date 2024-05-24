use self::synatx::{Block, Instruction, Reg};
use crate::{
    error::LangError,
    parse::ast::{Expression, Literal},
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

trait Lowerable {
    fn lower(&self) -> Result<Option<Block>, LangError>;
}

impl Lowerable for Literal {
    fn lower(&self) -> Result<Option<Block>, LangError> {
        let reg = next_register();
        Ok(Some(match self {
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
        }))
    }
}
