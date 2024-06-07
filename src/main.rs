use crate::parse::ast::Statement;
use air::{generate_program, GenerationState};
use clap::{Parser, Subcommand};
use error::LangError;
use lex::{token::TokenType, Lexer};
use parse::LangParser;
use std::{error::Error, fs::read_to_string, path::PathBuf, sync::mpsc::channel, thread};

use lead_vm::{
    air::Instruction,
    vm::{Machine, Message},
};

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
    Repl,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => run(file),
        Commands::Build { file } => build(file),
        Commands::Lex { file } => lex(file),
        _ => todo!("implement repl"),
    }
}

fn lex(file: PathBuf) {
    let input: String = match read_to_string(file.as_path()) {
        Ok(src) => src,
        Err(e) => {
            eprintln!("error reading file: {e}");
            return;
        }
    };

    let mut lexer: Lexer = Lexer::new(&input);
    let tokens = match lexer.run() {
        Ok(tokens) => tokens,
        Err(es) => {
            eprintln!("{:?}", es.first().unwrap().to_owned());
            return;
        }
    };

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
}

fn run(file: PathBuf) {
    let input: String = match read_to_string(file.as_path()) {
        Ok(src) => src,
        Err(e) => {
            eprintln!("error reading file: {e}");
            return;
        }
    };

    let instructions: Vec<Instruction> = match build_air(&input) {
        Ok(air) => air,
        Err(e) => {
            eprintln!("{e:#?}");
            return;
        }
    };

    match run_instructions(instructions) {
        Err(e) => eprintln!("{e}"),
        _ => (),
    }
}

fn run_instructions(instructions: Vec<Instruction>) -> Result<(), Box<dyn Error>> {
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
                return Err(e.into());
            }
        }
    }
    Ok(())
}

fn build_air(src: &str) -> Result<Vec<Instruction>, LangError> {
    let mut lexer: Lexer = Lexer::new(src);
    let tokens = match lexer.run() {
        Ok(tokens) => tokens,
        Err(es) => return Err(es.first().unwrap().to_owned()),
    };
    let mut parser: LangParser = LangParser::new(&tokens);
    let ast: Vec<Statement> = parser.parse_statement(Vec::new())?;

    let mut gen_state: GenerationState = GenerationState::new();
    // this is not efficient at the moment
    let air = generate_program(&mut gen_state, ast)?
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
    Ok(air)
}

fn build(file: PathBuf) {
    let input: String = match read_to_string(file.as_path()) {
        Ok(src) => src,
        Err(e) => {
            eprintln!("error reading file: {e}");
            return;
        }
    };

    let air = match build_air(&input) {
        Ok(air) => air,
        Err(e) => {
            eprintln!("{e:#?}");
            return;
        }
    };

    for instruction in air {
        print!("{instruction}");
    }
}
