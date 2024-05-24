use std::fmt::write;

#[derive(Clone, Copy)]
pub struct Reg(pub u32);

impl std::ops::Deref for Reg {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Reg {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}", **self)
    }
}

pub struct Block {
    instructions: Vec<Instruction>,
    output_register: Reg,
    span: (usize, usize),
}

impl Block {
    pub fn new(instructions: &[Instruction], output_register: Reg, span: (usize, usize)) -> Self {
        Self {
            instructions: instructions.to_vec(),
            output_register,
            span,
        }
    }
}

#[derive(Clone)]
pub enum Instruction {
    ADD(Reg, Reg, Reg),
    SUB(Reg, Reg, Reg),
    MUL(Reg, Reg, Reg),
    DIV(Reg, Reg, Reg),

    /// introduce a constant
    CON(Reg, u32),

    NOT(Reg, Reg),

    CMP(Reg, Reg),
    CHK(Flag),

    LBL(String),
    BRA(String),
}

#[derive(Copy, Clone)]
pub enum Flag {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}
