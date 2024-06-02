use super::syntax::{Flag, Instruction, Reg};

/// A segment of the program, represented by AIR instructions
pub enum Segment {
    SubProgram {
        segments: Vec<Box<Self>>,
        output_register: Option<Reg>,
    },
    Block {
        instructions: Vec<Instruction>,
        output_register: Option<Reg>,
        span: (usize, usize),
    },
}

impl Segment {
    pub fn append_inst(&mut self, instruction: Instruction, inst_span: (usize, usize)) {
        match self {
            Segment::Block {
                ref mut instructions,
                ref mut output_register,
                ref mut span,
            } => {
                match instruction.output_register() {
                    None => (),
                    Some(reg) => {
                        output_register.replace(reg);
                    }
                }
                instructions.push(instruction);
                *span = (span.0, inst_span.1);
            }
            Segment::SubProgram {
                ref mut segments,
                ref mut output_register,
            } => {
                match instruction.output_register() {
                    None => (),
                    Some(reg) => {
                        output_register.replace(reg);
                    }
                }
                if segments.is_empty() {
                    segments.push(Box::new(Segment::block_from_inst(instruction, inst_span)));
                } else {
                    segments
                        .last_mut()
                        .unwrap()
                        .append_inst(instruction, inst_span)
                }
            }
        }
    }

    pub fn block_from_inst(instruction: Instruction, span: (usize, usize)) -> Self {
        let output_register = instruction.output_register();
        Self::Block {
            instructions: vec![instruction],
            output_register,
            span,
        }
    }

    pub fn subprogram_from_inst(instruction: Instruction, span: (usize, usize)) -> Self {
        let output_register = instruction.output_register();
        Self::SubProgram {
            segments: vec![Box::new(Segment::block_from_inst(instruction, span))],
            output_register,
        }
    }

    pub fn output_register(&self) -> Option<Reg> {
        match self {
            Segment::Block {
                instructions: _,
                output_register,
                span: _,
            } => output_register.to_owned(),
            Segment::SubProgram {
                segments: _,
                output_register,
            } => output_register.to_owned(),
        }
    }

    pub fn empty_block() -> Self {
        Self::Block {
            instructions: Vec::new(),
            output_register: None,
            span: (0, 0),
        }
    }

    /// Set the span of the segment. `SubProgram`s do not keep track of their span, so this will only effect `Block`s
    pub fn set_span(&mut self, new_span: (usize, usize)) {
        match self {
            Segment::Block {
                instructions: _,
                output_register: _,
                ref mut span,
            } => *span = new_span,
            _ => (),
        }
    }

    pub fn set_output_register(&mut self, reg: Reg) {
        match self {
            Segment::Block {
                instructions: _,
                ref mut output_register,
                span: _,
            } => *output_register = Some(reg),
            Segment::SubProgram {
                segments: _,
                ref mut output_register,
            } => *output_register = Some(reg),
        }
    }
}

/// NOTE: extending a segment does update the output register, however does not update the span of a block.
impl std::iter::Extend<Instruction> for Segment {
    fn extend<T: IntoIterator<Item = Instruction>>(&mut self, iter: T) {
        match self {
            Self::Block {
                ref mut instructions,
                ref mut output_register,
                ref mut span,
            } => {
                instructions.extend(iter);
                match instructions.last() {
                    None => (),
                    Some(inst) => {
                        *output_register = inst.output_register().or(*output_register);
                    }
                }
            }
            // this case is provided for completeness, however this is likely not what you want to use
            Self::SubProgram {
                ref mut segments,
                output_register: _,
            } => match segments.last_mut() {
                None => {
                    let mut block = Segment::empty_block();
                    block.extend(iter);
                    segments.push(Box::new(block));
                }
                Some(ref mut block) => block.extend(iter),
            },
        }
    }
}

impl std::iter::IntoIterator for Segment {
    type Item = Instruction;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Block {
                instructions,
                output_register: _,
                span: _,
            } => instructions.into_iter(),
            Self::SubProgram {
                segments,
                output_register: _,
            } => segments
                .into_iter()
                .flat_map(|x| *x)
                .collect::<Vec<Self::Item>>()
                .into_iter(),
        }
    }
}

impl std::fmt::Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Segment::Block {
                instructions,
                output_register: _,
                span: _,
            } => {
                let mut indent: usize = 0;
                for instruction in instructions.iter() {
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
            Segment::SubProgram {
                segments: _,
                output_register: _,
            } => {
                todo!("implement display for subprograms")
                // likely just iterate over the blocks and print with display
            }
        }
    }
}