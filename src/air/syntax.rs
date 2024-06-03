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

#[derive(Clone)]
pub enum Instruction {
    ADD(Reg, Reg, Reg),
    SUB(Reg, Reg, Reg),
    MUL(Reg, Reg, Reg),
    DIV(Reg, Reg, Reg),

    /// Introduce a constant
    CON(Reg, u32),
    /// Move a value from one register to another
    MOV(Reg, Reg),

    NOT(Reg, Reg),
    /// Compare two registers, and set flags. Contains an optional info flag, designating what flag was intended to be set.
    CMP(Reg, Reg, Option<Flag>),
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
    /// Equal
    Eq,
    /// Not equal
    Ne,
    /// Less than
    Lt,
    /// Less than or equal to
    Le,
    /// Greater than
    Gt,
    /// Greater than or equal to
    Ge,
    /// Never
    Nv,
    /// Always
    Al,
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::ADD(rd, rx, ry) => writeln!(f, "ADD {rd}, {rx}, {ry}"),
            Instruction::SUB(rd, rx, ry) => writeln!(f, "SUB {rd}, {rx}, {ry}"),
            Instruction::MUL(rd, rx, ry) => writeln!(f, "MUL {rd}, {rx}, {ry}"),
            Instruction::DIV(rd, rx, ry) => writeln!(f, "DIV {rd}, {rx}, {ry}"),
            Instruction::NOT(rd, rx) => writeln!(f, "NOT {rd}, {rx}"),
            Instruction::CMP(rx, ry, _) => writeln!(f, "CMP {rx}, {ry}"),

            Instruction::CON(rd, constant) => writeln!(f, "CONST {rd}, ={constant:#x}"),
            Instruction::MOV(rd, rx) => writeln!(f, "MOV {rd}, {rx}"),

            Instruction::LBL(label) => writeln!(f, "{label}:"),
            Instruction::BRA(label) => writeln!(f, "BRA {label}"),

            Instruction::CHK(flag) => writeln!(f, "CHK {flag}",),
        }
    }
}

impl std::fmt::Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Flag::Al => "1",
                Flag::Nv => "0",
                Flag::Eq => "==",
                Flag::Ne => "!=",
                Flag::Gt => ">",
                Flag::Ge => ">=",
                Flag::Lt => "<",
                Flag::Le => "<=",
            }
        )
    }
}
