#[cfg(test)]
mod tests;

use lead::air::air::{Flag, Instruction, Mode, Reg};
use log::{debug, info};
use std::collections::HashMap;
use std::sync::mpsc::Sender;

pub const DEFAULT_MEMORY_SIZE: usize = 256;
pub const DEFAULT_VERBOSITY: u8 = 1;

#[derive(Debug, Clone, Copy)]
pub struct VMFlags {
    pub memory_size: usize,
    /// the logging verbosity. 0 for quiet, 1 for normal, 2 for verbose, 3 for very verbose.
    pub verbosity: u8,
}

pub enum Verbosity {
    Quiet = 0,
    Normal = 1,
    Verbose = 2,
    VeryVerbose = 3,
}

impl VMFlags {
    pub const fn none() -> Self {
        Self {
            memory_size: DEFAULT_MEMORY_SIZE,
            verbosity: DEFAULT_VERBOSITY,
        }
    }

    pub const fn new(memory_size: usize, verbosity: u8) -> Self {
        Self {
            memory_size,
            verbosity,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Message {
    Yield(u32),
    Done,
}

pub struct Machine {
    instructions: Vec<Instruction>,
    registers: HashMap<u32, u32>,
    memory: Vec<u8>,
    yield_callback: Sender<Message>,
    /// program counter
    pc: usize,
    flags: Flags,
    vm_flags: VMFlags,
}

impl Machine {
    pub fn new(
        instructions: Vec<Instruction>,
        yield_sender: Sender<Message>,
        vm_flags: VMFlags,
    ) -> Self {
        Self {
            instructions,
            memory: vec![0; vm_flags.memory_size],
            registers: HashMap::new(),
            yield_callback: yield_sender,
            pc: 0,
            flags: Flags::empty(),
            vm_flags,
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
        if self.log_is_very_verbose() {
            debug!("advancing by {count}")
        }
        self.pc += count;
    }

    fn process(&mut self, instruction: &Instruction) {
        if self.log_is_verbose() {
            debug!("processing instruction: {instruction}")
        }

        match instruction {
            Instruction::ADD(rd, rx, ry) => self.save(rd, &((self.get(rx)) + self.get(ry))),
            Instruction::SUB(rd, rx, ry) => self.save(rd, &((self.get(rx)) - self.get(ry))),
            Instruction::MUL(rd, rx, ry) => self.save(rd, &((self.get(rx)) * self.get(ry))),
            Instruction::DIV(rd, rx, ry) => self.save(rd, &((self.get(rx)) / self.get(ry))),
            Instruction::CMP(rx, ry, _) => self.set_flags(rx, ry),
            Instruction::CON(rd, val) => self.save(rd, val),
            Instruction::MOV(rd, rx) => self.save(rd, &self.get(rx)),
            Instruction::NOT(rd, rx) => self.save(rd, &!self.get(rx)),
            Instruction::BRA(label) => self.branch(label),
            Instruction::YLD(rx) => self.yield_register(rx),
            Instruction::LBL(_) => (),
            Instruction::CHK(flag) => {
                if !self.flags.contains(*flag) {
                    self.advance(1)
                }
            }
            Instruction::STR(data, addr, mode) => self.store(addr, &self.get(data), mode),
            Instruction::LDR(rd, addr, mode) => {
                let data = &self.load(addr, mode);
                self.save(rd, data)
            }
        }
    }

    /// Get the value in a register, unchecked.
    fn get(&self, reg: &Reg) -> u32 {
        let val = *self.registers.get(&(*reg)).unwrap();

        if self.log_is_very_verbose() {
            debug!("getting {reg}, got {val}")
        }
        val
    }

    /// Save a value in a register
    fn save(&mut self, reg: &Reg, val: &u32) {
        if self.log_is_very_verbose() {
            debug!("saving {reg} with value {val}")
        }
        self.registers.insert(**reg, *val);
    }

    fn store(&mut self, rd: &Reg, value: &u32, mode: &Mode) {
        let bytes = value.to_be_bytes();

        let addr = match mode {
            Mode::None | Mode::PostOffset(_) => self.get(rd) as usize,
            Mode::Offset(r_ofst) => (self.get(rd) + self.get(&r_ofst)) as usize,
            Mode::PreOffset(r_ofst) => {
                let addr = self.get(rd) + self.get(&r_ofst);
                self.save(rd, &addr);
                addr as usize
            }
        };

        for (i, byte) in bytes.iter().enumerate() {
            let mem: &mut u8 = self
                .memory
                .get_mut(addr + i)
                .expect("error handling needed in vm");
            *mem = *byte;
        }

        match mode {
            Mode::PostOffset(r_ofst) => {
                let addr = self.get(rd) + self.get(&r_ofst);
                self.save(rd, &addr);
            }
            _ => (),
        }
    }

    // consider a storeb variant

    fn load(&mut self, rd: &Reg, mode: &Mode) -> u32 {
        let mut bytes: [u8; 4] = [0; 4];

        let addr = match mode {
            Mode::None | Mode::PostOffset(_) => self.get(rd) as usize,
            Mode::Offset(r_ofst) => (self.get(rd) + self.get(&r_ofst)) as usize,
            Mode::PreOffset(r_ofst) => {
                let addr = self.get(rd) + self.get(&r_ofst);
                self.save(rd, &addr);
                addr as usize
            }
        };

        for i in 0..4 {
            bytes[i] = self.memory[addr + i];
        }

        if let Mode::PostOffset(r_ofst) = mode {
            let addr = self.get(rd) + self.get(&r_ofst);
            self.save(rd, &addr);
        }

        u32::from_be_bytes(bytes)
    }

    /// Yield a value in a register from the program. This passes the value to the yield callback
    fn yield_register(&mut self, reg: &Reg) {
        let val: u32 = self.get(reg);
        if self.log_is_normal() {
            debug!("yielding {val}")
        }

        self.yield_callback
            .send(Message::Yield(val))
            .expect("oh no!") // this requires better handling
    }

    /// Branch to a label, panics if the label doesn't exist
    fn branch(&mut self, label: &str) {
        match self.find_label(label) {
            None => panic!("expected label {label} to exist in the program"),
            Some(idx) => {
                if self.log_is_verbose() {
                    debug!("branching to {label}, pc = {idx}")
                }
                self.pc = idx
            }
        }
    }

    /// Find the index of the first label with the specified name if it exists
    fn find_label(&self, label: &str) -> Option<usize> {
        self.instructions
            .iter()
            .enumerate()
            .find(|(_, inst)| **inst == Instruction::LBL(label.to_string()))
            .map(|(i, _)| i)
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

        if self.log_is_verbose() {
            debug!("flags set: {:#08b}", self.flags.0)
        }
    }

    /// return weather the vm is logging in at least normal mode
    fn log_is_normal(&self) -> bool {
        self.vm_flags.verbosity >= Verbosity::Normal as u8
    }

    /// return weather the vm is logging in at least verbose mode
    fn log_is_verbose(&self) -> bool {
        self.vm_flags.verbosity >= Verbosity::Verbose as u8
    }
    /// return weather the vm is logging in very verbose mode
    fn log_is_very_verbose(&self) -> bool {
        self.vm_flags.verbosity >= Verbosity::VeryVerbose as u8
    }
}

struct Flags(u16);

impl Flags {
    fn empty() -> Self {
        // Always is always set
        Self(1)
    }

    fn set(&mut self, flag: Flag) {
        *self = Self(self.0 | (1 << (flag as u8)));
    }

    fn contains(&self, flag: Flag) -> bool {
        debug!("checking if flag {flag} is set: self is {:#010b}", self.0);

        if flag == Flag::Nv {
            false
        } else {
            (self.0 >> (flag as u8) & 1) == 1
        }
    }
}
