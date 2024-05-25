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
    pub output_register: Reg,
    pub span: (usize, usize),
}

impl Block {
    pub fn new(instructions: &[Instruction], output_register: Reg, span: (usize, usize)) -> Self {
        Self {
            instructions: instructions.to_vec(),
            output_register,
            span,
        }
    }

    /// append a new instruction at the end of the block
    pub fn append(&mut self, instruction: Instruction) {
        match instruction.output_register() {
            None => (),
            Some(reg) => self.output_register = reg,
        }
        self.instructions.push(instruction);
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

impl Instruction {
    fn output_register(&self) -> Option<Reg> {
        Some(match self {
            Self::ADD(r, _, _) => *r,
            Self::SUB(r, _, _) => *r,
            Self::MUL(r, _, _) => *r,
            Self::DIV(r, _, _) => *r,
            Self::CON(r, _) => *r,
            Self::NOT(r, _) => *r,
            _ => return None,
        })
    }
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

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::ADD(rd, rx, ry) => writeln!(f, "ADD {rd}, {rx}, {ry}"),
            Instruction::SUB(rd, rx, ry) => writeln!(f, "SUB {rd}, {rx}, {ry}"),
            Instruction::MUL(rd, rx, ry) => writeln!(f, "MUL {rd}, {rx}, {ry}"),
            Instruction::DIV(rd, rx, ry) => writeln!(f, "DIV {rd}, {rx}, {ry}"),
            Instruction::NOT(rd, rx) => writeln!(f, "NOT {rd}, {rx}"),
            Instruction::CMP(rx, ry) => writeln!(f, "CMP {rx}, {ry}"),

            Instruction::CON(rd, constant) => writeln!(f, "CONST {rd}, ={constant:#x}"),

            Instruction::LBL(label) => writeln!(f, "{label}:"),
            Instruction::BRA(label) => writeln!(f, "BRA {label}"),

            Instruction::CHK(_) => todo!(),
        }
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut indent: usize = 0;
        for instruction in self.instructions.iter() {
            match instruction {
                Instruction::LBL(_) => {
                    write!(f, "{}{instruction}", "\t".repeat(indent))?;
                    indent += 1;
                }
                _ => write!(f, "{instruction}")?,
            }
        }
        Ok(())
    }
}
