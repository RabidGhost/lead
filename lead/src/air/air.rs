use crate::lex::span::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// A wrapper type on `Instruction` containing additional span information.
#[derive(Clone)]
pub struct Inst {
    pub instruction: Instruction,
    pub span: Span,
}

impl Inst {
    pub fn new(instruction: Instruction, span: impl Spans) -> Self {
        Self {
            instruction,
            span: span.span(),
        }
    }

    pub fn output_register(&self) -> Option<Reg> {
        self.instruction.output_register()
    }

    pub fn instruction(self) -> Instruction {
        self.instruction
    }
}

impl Spans for Inst {
    fn span(&self) -> Span {
        self.span
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

    // not really sure what the point of this was
    NOT(Reg, Reg),

    /// Compare two registers, and set flags. Contains an optional info flag, designating what flag was intended to be set.
    CMP(Reg, Reg, Option<Flag>),
    CHK(Flag),

    /// Store a register in memory, at a memory address given by rx, with a memmory addressing mode
    STR(Reg, Reg, Mode),
    /// Read to a register from memory, at a memory address given by rx, with a memmory addressing mode
    LDR(Reg, Reg, Mode),

    LBL(String),
    ///
    BRA(String),
    /// Yield a register. This returns the value in the register, and continues executing.
    YLD(Reg),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    None,
    Offset(Reg),
    PreOffset(Reg),
    PostOffset(Reg),
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

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Flag {
    /// Always
    Al,
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
}

impl Flag {
    /// Negate a flag, returning a flag such that CHK FLAGB successedes exactly when FLAGA fails
    pub fn negate(&self) -> Self {
        match self {
            Flag::Al => Flag::Nv,
            Flag::Eq => Flag::Ne,
            Flag::Ge => Flag::Lt,
            Flag::Gt => Flag::Le,
            Flag::Le => Flag::Gt,
            Flag::Lt => Flag::Ge,
            Flag::Ne => Flag::Eq,
            Flag::Nv => Flag::Al,
        }
    }
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
            Instruction::YLD(rx) => writeln!(f, "YLD {rx}"),

            Instruction::STR(rd, adr, mode) => match mode {
                Mode::None => writeln!(f, "STR {rd}, [{adr}]"),
                Mode::Offset(ofst) => writeln!(f, "STR {rd}, [{adr}, {ofst}]"),
                Mode::PreOffset(ofst) => writeln!(f, "STR {rd}, [{adr}, {ofst}]!"),
                Mode::PostOffset(ofst) => writeln!(f, "STR {rd}, [{adr}], {ofst}"),
            },
            Instruction::LDR(rd, adr, mode) => match mode {
                Mode::None => writeln!(f, "LDR {rd}, [{adr}]"),
                Mode::Offset(ofst) => writeln!(f, "LDR {rd}, [{adr}, {ofst}]"),
                Mode::PreOffset(ofst) => writeln!(f, "LDR {rd}, [{adr}, {ofst}]!"),
                Mode::PostOffset(ofst) => writeln!(f, "LDR {rd}, [{adr}], {ofst}"),
            },
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
