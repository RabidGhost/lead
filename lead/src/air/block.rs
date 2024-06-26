use super::air::{Flag, Inst, Instruction, Reg};

/// A Block of the program, represented by AIR instructions
#[derive(Clone)]
pub struct Block {
    instructions: Vec<Inst>,
    output_register: Option<Reg>,
}

impl Block {
    pub fn new(instruction: Inst) -> Self {
        let output_register = instruction.output_register();
        Self {
            instructions: vec![instruction],
            output_register,
        }
    }

    pub fn empty() -> Self {
        Self {
            instructions: Vec::new(),
            output_register: None,
        }
    }

    pub fn append_inst(&mut self, instruction: Inst) {
        self.output_register = instruction.output_register().or(self.output_register);
        self.instructions.push(instruction);
    }

    pub fn from_instructions(instructions: Vec<Inst>) -> Self {
        let output_register: Option<Reg> = instructions
            .iter()
            .rev()
            .map(|inst| inst.output_register())
            .find(|r| r.is_some())
            .map(|x| x.unwrap());

        Self {
            instructions,
            output_register,
        }
    }

    pub fn output_register(&self) -> Option<Reg> {
        self.output_register
    }

    pub fn output_register_unchecked(&self) -> Reg {
        self.output_register.expect("unchecked output register")
    }

    pub fn set_output_register(&mut self, reg: Option<Reg>) {
        self.output_register = reg;
    }

    /// Returns the most recent flag hint if one exists, otherwise `None`
    pub fn latest_flag_hint(&self) -> Option<Flag> {
        self.instructions.iter().rev().fold(None, |acc, inst| {
            if acc.is_some() {
                return acc;
            } else {
                match inst.instruction {
                    Instruction::CMP(_, _, Some(flag_hint)) => return Some(flag_hint),
                    _ => None,
                }
            }
        })
    }

    pub fn instructions(&self) -> &Vec<Inst> {
        &self.instructions
    }

    /// Return a mutable reference to the latest *instruction* (not Inst) that furfils the predicate
    pub fn get_latest_mut(
        &mut self,
        predicate: impl Fn(&Instruction) -> bool,
    ) -> Option<&mut Instruction> {
        self.instructions
            .iter_mut()
            .rfind(|x| predicate(x.instruction_borrow()))
            .map_or(None, |inst| Some(inst.instruction_mut()))
    }
}

impl std::iter::Extend<Inst> for Block {
    fn extend<T: IntoIterator<Item = Inst>>(&mut self, iter: T) {
        self.instructions.extend(iter);
        self.output_register = self
            .instructions
            .iter()
            .rev()
            .find(|inst| inst.output_register().is_some())
            .map(|x| x.output_register().unwrap());
    }
}

impl std::iter::IntoIterator for Block {
    type Item = Inst;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.instructions.into_iter()
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for inst in self.instructions.iter() {
            write!(f, "{inst}")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for Inst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.instruction)
    }
}
