#[derive(Debug, Clone, Copy)]
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

trait Segment {
    fn flatten(&self) -> &Vec<Instruction>;
}

pub struct Block {
    instructions: Vec<Instruction>,
    pub output_register: Reg,
    pub flag_set: Option<Flag>,
    pub span: (usize, usize),
}

impl Segment for Block {
    fn flatten(&self) -> &Vec<Instruction> {
        self.instructions.as_ref()
    }
}

pub struct SubProgram {
    blocks: Vec<Block>,
    pub output_register: Reg,
    pub flag_set: Option<Flag>,
    pub span: (usize, usize),
}

// impl Segment for SubProgram {
//     fn flatten(&self) -> &Vec<Instruction> {
//         &self
//             .blocks
//             .into_iter()
//             .flat_map(|block| block.flatten())
//             .map(|x| x.to_owned())
//             .collect()
//     }
// }

impl Block {
    pub fn new(instructions: &[Instruction], output_register: Reg, span: (usize, usize)) -> Self {
        Self {
            instructions: instructions.to_vec(),
            output_register,
            flag_set: None,
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

    pub fn set_flag(&mut self, flag: Flag) {
        self.flag_set = Some(flag);
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
    /// move a value from one register to another
    MOV(Reg, Reg),

    NOT(Reg, Reg),

    CMP(Reg, Reg),
    CHK(Flag),

    LBL(String),
    BRA(String),
}

impl Instruction {
    pub fn output_register(&self) -> Option<Reg> {
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
            Instruction::MOV(rd, rx) => writeln!(f, "MOV {rd}, {rx}"),

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

impl std::iter::Extend<Instruction> for Block {
    fn extend<T: IntoIterator<Item = Instruction>>(&mut self, iter: T) {
        self.instructions.extend(iter)
    }
}

impl std::iter::IntoIterator for Block {
    type Item = Instruction;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.instructions.into_iter()
    }
}
