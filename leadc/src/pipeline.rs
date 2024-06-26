use lead::{
    air::{
        air::{Inst, Instruction},
        generate_program, GenerationState,
    },
    lex::{token::Token, Lexer},
    parse::{ast::Statement, LangParser},
};

use crate::cli::RunArgs;

use lead_vm::{Machine, Message, VMFlags, DEFAULT_VERBOSITY};
use miette::{Diagnostic, Result};
use std::{
    fs::read_to_string,
    io::{Read, Stdin},
    path::PathBuf,
    sync::mpsc::channel,
    thread,
};
use thiserror::Error;

impl Into<VMFlags> for RunArgs {
    fn into(self) -> VMFlags {
        let verbosity = {
            if self.quiet {
                0
            } else if self.verbose {
                2
            } else if self.very_verbose {
                3
            } else {
                DEFAULT_VERBOSITY
            }
        };

        VMFlags {
            memory_size: self.memory_size,
            verbosity,
        }
    }
}

#[derive(Clone)]
pub enum Pipeline {
    Text(String, Option<RunArgs>),
    Tokens(String, Option<RunArgs>, Vec<Token>),
    SyntaxTree(String, Option<RunArgs>, Vec<Statement>),
    IntermediateRepr(String, Option<RunArgs>, Vec<Inst>),
}

impl std::fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(_, _) => write!(f, "Text"),
            Self::Tokens(_, _, _) => write!(f, "Tokens"),
            Self::SyntaxTree(_, _, _) => write!(f, "SyntaxTree"),
            Self::IntermediateRepr(_, _, _) => write!(f, "Intermediate Representation"),
        }
    }
}

#[derive(Error, Debug, Diagnostic)]
pub enum PipelineError {
    #[error("Error reading file into text\n\t{}", 0)]
    ReadError(String),
    #[error("`Token`s can only be constructed from `Text`, found `{}`", 0)]
    InvalidLex(String),
    #[error("Syntax trees can only be constructed from Tokens, found `{}`", 0)]
    InvalidParse(String),
    #[error(
        "Assembly Intermediate Representation (AIR) can only be built from `SyntaxTree`, found {}",
        0
    )]
    InvalidBuild(String),
    #[error(
        "Only Assembly Intermediate Representation (AIR) can be run, found `{}`",
        0
    )]
    InvalidRun(String),
    #[error(
        "Can only add arguments to a pipeline before running has begun, found {}",
        0
    )]
    InvalidWithArgs(String),
    #[error("Invalid utf8 in stream")]
    InvalidUTF8Input,
    #[error("Error reading from stdin: {}", 0)]
    ErrorReadFromStdin(String),
    #[error("{} can only be created from {}", 0, 1)]
    InvalidInto(String, String),
}

impl Pipeline {
    pub fn with_run_args(self, args: RunArgs) -> Result<Self> {
        match self {
            Pipeline::Text(src, _) => Ok(Pipeline::Text(src, Some(args))),
            Pipeline::Tokens(src, _, tokens) => Ok(Pipeline::Tokens(src, Some(args), tokens)),
            Pipeline::SyntaxTree(src, _, ast) => Ok(Pipeline::SyntaxTree(src, Some(args), ast)),
            _ => Err(PipelineError::InvalidWithArgs(format!("{self:?}")).into()),
        }
    }

    pub fn lex(self) -> Result<Self> {
        match self {
            Pipeline::Text(src, args) => {
                let mut lexer = Lexer::new(&src);
                return Ok(Self::Tokens(
                    src.clone(),
                    args,
                    lexer.run().map_err(|err| err.with_src(src.clone()))?,
                ));
            }
            _ => Err(PipelineError::InvalidLex(format!("{self:?}")).into()),
        }
    }

    pub fn parse(self) -> Result<Self> {
        match self {
            Self::Tokens(src, args, tokens) => {
                let mut parser: LangParser = LangParser::new(&tokens);
                let ast: Vec<Statement> = parser
                    .parse_statement(Vec::new())
                    .map_err(|e| e.with_src(src.clone()))?;
                Ok(Self::SyntaxTree(src.clone(), args, ast))
            }
            _ => Err(PipelineError::InvalidParse(format!("{self:?}")).into()),
        }
    }

    pub fn build(self) -> Result<Self> {
        match self {
            Self::SyntaxTree(src, args, ast) => {
                let mut gen_state: GenerationState = GenerationState::new();
                // this is not efficient at the moment
                let air: Vec<Inst> = generate_program(&mut gen_state, ast)
                    .map_err(|err| err.with_src(src.clone()))?
                    .into_iter()
                    .flatten()
                    .collect();
                Ok(Pipeline::IntermediateRepr(src.clone(), args, air))
            }
            _ => Err(PipelineError::InvalidBuild(format!("{self:?}")).into()),
        }
    }

    pub fn run(self) -> Result<()> {
        match self {
            Self::IntermediateRepr(_, args, instructions) => {
                let (sndr, rcvr) = channel();
                let vm_flags = match args {
                    Some(args) => args.into(),
                    None => VMFlags::none(),
                };
                let instructions = instructions
                    .iter()
                    .map(|inst| inst.clone().instruction()) // should check if this clone is nessessary?
                    .collect();
                let mut vm = Machine::new(instructions, sndr, vm_flags);
                let vm_thread = thread::spawn(move || vm.run());

                loop {
                    match rcvr.recv() {
                        Ok(msg) => match msg {
                            Message::Yield(val) => println!("{val}"),
                            Message::Done => {
                                vm_thread.join().expect("could not join the vm thread");
                                break;
                            }
                        },
                        Err(e) => {
                            vm_thread.join().expect("could not join the vm threaed");
                            eprintln!("{e}");
                            return Ok(());
                        }
                    }
                }
                Ok(())
            }
            _ => Err(PipelineError::InvalidRun(format!("{self:?}")).into()),
        }
    }
}

impl TryFrom<PathBuf> for Pipeline {
    type Error = PipelineError;
    fn try_from(value: PathBuf) -> std::result::Result<Self, Self::Error> {
        Ok(Pipeline::Text(
            read_to_string(value.as_path())
                .map_err(|e| PipelineError::ReadError(format!("{:?}", e)))?,
            None,
        ))
    }
}

impl TryFrom<&mut Stdin> for Pipeline {
    type Error = PipelineError;
    fn try_from(value: &mut Stdin) -> std::result::Result<Self, Self::Error> {
        let mut buf: Vec<u8> = Vec::new();
        value
            .read_to_end(&mut buf)
            .map_err(|err| PipelineError::ErrorReadFromStdin(format!("{err}")))?;
        let input: String = String::from_utf8(buf).map_err(|_| PipelineError::InvalidUTF8Input)?;
        Ok(Pipeline::Text(input, None))
    }
}

impl Into<Vec<Statement>> for Pipeline {
    fn into(self) -> Vec<Statement> {
        match self {
            Self::SyntaxTree(_, _, ast) => ast,
            _ => panic!("invalid into"),
        }
    }
}

impl Into<Vec<Token>> for Pipeline {
    fn into(self) -> Vec<Token> {
        match self {
            Self::Tokens(_, _, tokens) => tokens,
            _ => panic!("invalid into"),
        }
    }
}

impl TryInto<Vec<Instruction>> for Pipeline {
    type Error = PipelineError;
    fn try_into(self) -> std::result::Result<Vec<Instruction>, Self::Error> {
        match self {
            Self::IntermediateRepr(_, _, instructions) => Ok(instructions
                .iter()
                .map(|inst| inst.clone().instruction())
                .collect()),
            _ => Err(PipelineError::InvalidInto(
                "Vec<Inst>".to_owned(),
                "IntermediateRepr".to_owned(),
            )),
        }
    }
}

impl TryInto<Vec<Inst>> for Pipeline {
    type Error = PipelineError;
    fn try_into(self) -> std::result::Result<Vec<Inst>, Self::Error> {
        match self {
            Self::IntermediateRepr(_, _, instructions) => Ok(instructions),
            _ => Err(PipelineError::InvalidInto(
                "Vec<Inst>".to_owned(),
                "IntermediateRepr".to_owned(),
            )),
        }
    }
}
