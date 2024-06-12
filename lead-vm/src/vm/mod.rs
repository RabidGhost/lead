use crate::air::{Flag, Instruction, Reg};
use std::collections::HashMap;
use std::sync::mpsc::Sender;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Eq)]
pub enum Message {
    Yield(u32),
    Done,
}

pub struct Machine {
    instructions: Vec<Instruction>,
    registers: HashMap<u32, u32>,
    yield_callback: Sender<Message>,
    /// program counter
    pc: usize,
    flags: Flags,
}

impl Machine {
    pub fn new(instructions: Vec<Instruction>, yield_sender: Sender<Message>) -> Self {
        Self {
            instructions,
            registers: HashMap::new(),
            yield_callback: yield_sender,
            pc: 0,
            flags: Flags::empty(),
        }
    }

    pub fn run(&mut self) {
        while self.step() {}
        self.yield_callback.send(Message::Done).expect("oh oh!");
    }

    /// Take one step through the program, returning false when the program has terminated, true otherwise
    fn step(&mut self) -> bool {
        match self.instructions.get(self.pc) {
            None => false,
            Some(instruction) => {
                self.process(&instruction.clone());
                self.advance(1);
                true
            }
        }
    }

    /// Advance the program counter by `n` steps.
    fn advance(&mut self, count: usize) {
        self.pc += count;
    }

    fn process(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::ADD(rd, rx, ry) => self.store(rd, &((self.get(rx)) + self.get(ry))),
            Instruction::SUB(rd, rx, ry) => self.store(rd, &((self.get(rx)) - self.get(ry))),
            Instruction::MUL(rd, rx, ry) => self.store(rd, &((self.get(rx)) * self.get(ry))),
            Instruction::DIV(rd, rx, ry) => self.store(rd, &((self.get(rx)) / self.get(ry))),
            Instruction::CMP(rx, ry, _) => self.set_flags(rx, ry),
            Instruction::CON(rd, val) => self.store(rd, val),
            Instruction::MOV(rd, rx) => self.store(rd, &self.get(rx)),
            Instruction::NOT(rd, rx) => self.store(rd, &!self.get(rx)),
            Instruction::BRA(label) => self.branch(label),
            Instruction::YLD(rx) => self.yield_register(rx),
            Instruction::LBL(_) => (),
            Instruction::CHK(flag) => {
                if !self.flags.contains(*flag) {
                    self.advance(1)
                }
            }
        }
    }

    /// Get the value in a register, unchecked.
    fn get(&self, reg: &Reg) -> u32 {
        *self.registers.get(&(*reg)).unwrap()
    }

    /// Store a value in a register
    fn store(&mut self, reg: &Reg, val: &u32) {
        self.registers.insert(**reg, *val);
    }

    /// Yield a value in a register from the program. This passes the value to the yield callback
    fn yield_register(&mut self, reg: &Reg) {
        let val: u32 = self.get(reg);
        self.yield_callback
            .send(Message::Yield(val))
            .expect("oh no!") // this requires better handling
    }

    /// Branch to a label, panics if the label doesn't exist
    fn branch(&mut self, label: &str) {
        match self.find_label(label) {
            None => panic!("expected label {label} to exist in the program"),
            Some(idx) => self.pc = idx,
        }
    }

    /// Find the index of the first label with the specified name if it exists
    fn find_label(&self, label: &str) -> Option<usize> {
        self.instructions
            .iter()
            .enumerate()
            .map(|(i, inst)| match inst {
                Instruction::LBL(lbl) if lbl == label => Some(i),
                _ => None,
            })
            .find(|x| x.is_some())
            .unwrap_or(None)
    }

    fn set_flags(&mut self, rx: &Reg, ry: &Reg) {
        let x = self.get(rx);
        let y = self.get(ry);

        if x == y {
            self.flags.set(Flag::Eq)
        }
        if x != y {
            self.flags.set(Flag::Ne)
        }
        if x < y {
            self.flags.set(Flag::Lt)
        }
        if x <= y {
            self.flags.set(Flag::Le)
        }
        if x > y {
            self.flags.set(Flag::Gt)
        }
        if x >= y {
            self.flags.set(Flag::Ge)
        }
    }
}

struct Flags(u16);

impl Flags {
    fn empty() -> Self {
        // Always is always set
        Self(1)
    }

    fn set(&mut self, flag: Flag) {
        *self = Self(self.0 & (1 << (flag as u8)));
    }

    fn contains(&self, flag: Flag) -> bool {
        if flag == Flag::Nv {
            false
        } else {
            (self.0 & (1 << (flag as u8))) == 1
        }
    }
}
