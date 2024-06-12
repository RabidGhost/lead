use crate::parse::ast::Statement;
use air::{generate_program, GenerationState};
use clap::{Parser, Subcommand};
use error::LangError;
use lead_vm::{
    air::Instruction,
    vm::{Machine, Message},
};
use lex::{token::Token, token::TokenType, Lexer};
use miette::{Diagnostic, Result};
use parse::LangParser;
use std::{fs::read_to_string, path::PathBuf, sync::mpsc::channel, thread};
use thiserror::Error;

mod air;
mod error;
mod lex;
mod parse;

#[derive(Parser)]
#[command(about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// run in the interpreter from a file
    Run {
        file: PathBuf,
    },
    Build {
        file: PathBuf,
    },
    Lex {
        file: PathBuf,
    },
    Parse {
        file: PathBuf,
    },
    Repl,
}

enum Pipeline {
    Text(String),
    Tokens(String, Vec<Token>),
    SyntaxTree(String, Vec<Statement>),
    IntermediateRepr(String, Vec<Instruction>),
}

impl std::fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(_) => write!(f, "Text"),
            Self::Tokens(_, _) => write!(f, "Tokens"),
            Self::SyntaxTree(_, _) => write!(f, "SyntaxTree"),
            Self::IntermediateRepr(_, _) => write!(f, "Intermediate Representation"),
        }
    }
}

#[derive(Error, Debug, Diagnostic)]
enum PipelineError {
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
}

impl Pipeline {
    fn new(src: String) -> Self {
        Self::Text(src)
    }

    fn lex(self) -> Result<Self> {
        match self {
            Pipeline::Text(src) => {
                let mut lexer = Lexer::new(&src);
                return Ok(Self::Tokens(
                    src.clone(),
                    lexer.run().map_err(|err| err.with_src(src.clone()))?,
                ));
            }
            _ => Err(PipelineError::InvalidLex(format!("{self:?}")).into()),
        }
    }

    fn parse(self) -> Result<Self> {
        match self {
            Self::Tokens(src, tokens) => {
                let mut parser: LangParser = LangParser::new(&tokens);
                let ast: Vec<Statement> = parser
                    .parse_statement(Vec::new())
                    .map_err(|e| e.with_src(src.clone()))?;
                Ok(Self::SyntaxTree(src.clone(), ast))
            }
            _ => Err(PipelineError::InvalidParse(format!("{self:?}")).into()),
        }
    }

    fn build(self) -> Result<Self> {
        match self {
            Self::SyntaxTree(src, ast) => {
                let mut gen_state: GenerationState = GenerationState::new();
                // this is not efficient at the moment
                let air: Vec<Instruction> = generate_program(&mut gen_state, ast)?
                    .into_iter()
                    .flat_map(|segment| {
                        segment
                            .clone()
                            .flatten()
                            .into_iter()
                            .map(|x| x.to_owned())
                            .collect::<Vec<Instruction>>()
                    })
                    .collect();
                Ok(Pipeline::IntermediateRepr(src.clone(), air))
            }
            _ => Err(PipelineError::InvalidBuild(format!("{self:?}")).into()),
        }
    }

    fn run(self) -> Result<()> {
        match self {
            Self::IntermediateRepr(_, instructions) => {
                let (sndr, rcvr) = channel();
                let mut vm = Machine::new(instructions, sndr);
                let vm_thread = thread::spawn(move || vm.run());

                loop {
                    match rcvr.recv() {
                        Ok(msg) => match msg {
                            Message::Yield(val) => println!("{val}"),
                            Message::Done => {
                                vm_thread.join();
                                break;
                            }
                        },
                        Err(e) => {
                            vm_thread.join();
                            // vm_thread.join().into()?;
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
            read_to_string(value.as_path()).map_err(|e| PipelineError::ReadError(e.to_string()))?,
        ))
    }
}

impl Into<Vec<Statement>> for Pipeline {
    fn into(self) -> Vec<Statement> {
        match self {
            Self::SyntaxTree(_, ast) => ast,
            _ => panic!("invalid into"),
        }
    }
}

impl Into<Vec<Token>> for Pipeline {
    fn into(self) -> Vec<Token> {
        match self {
            Self::Tokens(_, tokens) => tokens,
            _ => panic!("invalid into"),
        }
    }
}

impl Into<Vec<Instruction>> for Pipeline {
    fn into(self) -> Vec<Instruction> {
        match self {
            Self::IntermediateRepr(_, instructions) => instructions,
            _ => panic!("invalid into"),
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => run(file)?,
        Commands::Build { file } => build(file)?,
        Commands::Lex { file } => lex(file)?,
        Commands::Parse { file } => parse(file)?,
        _ => todo!("implement repl"),
    }
    Ok(())
}

fn lex(file: PathBuf) -> Result<()> {
    let tokens: Vec<Token> = Pipeline::try_from(file)?.lex()?.into();

    let mut indent: usize = 0;

    for token in tokens {
        print!("{} ", token);
        let ty = token.token_type();
        match ty {
            TokenType::Semicolon => print!("\n{}", "\t".repeat(indent)),
            TokenType::LeftBrace => {
                indent += 1;
                print!("\n{}", "\t".repeat(indent));
            }
            TokenType::RightBrace => {
                indent = usize::max(0, indent - 1);
                print!("\n{}", "\t".repeat(indent));
            }
            _ => (),
        }
    }
    Ok(())
}

fn parse(file: PathBuf) -> Result<()> {
    let ast: Vec<Statement> = Pipeline::try_from(file)?.lex()?.parse()?.into();
    for statement in ast {
        println!("{:?}", statement);
    }
    Ok(())
}

fn run(file: PathBuf) -> Result<()> {
    Pipeline::try_from(file)?.lex()?.parse()?.build()?.run()
}

fn build(file: PathBuf) -> Result<()> {
    let air: Vec<Instruction> = Pipeline::try_from(file)?.lex()?.parse()?.build()?.into();

    for instruction in air {
        print!("{instruction}");
    }
    Ok(())
}
