use crate::parse::ast::Statement;
use air::{generate_program, GenerationState};
use clap::{Parser, Subcommand};
use error::LangError;
use lead_vm::{
    air::Instruction,
    vm::{Machine, Message},
};
use lex::{token::TokenType, Lexer};
use miette::Result;
use parse::LangParser;
use std::{fs::read_to_string, path::PathBuf, sync::mpsc::channel, thread};

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
    let input: String = match read_to_string(file.as_path()) {
        Ok(src) => src,
        Err(e) => {
            eprintln!("error reading file: {e}");
            return Ok(());
        }
    };

    let mut lexer: Lexer = Lexer::new(&input);
    let tokens = lexer.run().map_err(|err| err.with_src(input))?;

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
    let input: String = match read_to_string(file.as_path()) {
        Ok(src) => src,
        Err(e) => {
            eprintln!("error reading file: {e}");
            return Ok(());
        }
    };
    let mut lexer: Lexer = Lexer::new(&input);
    let tokens = lexer.run()?; // {
                               //     Ok(tokens) => tokens,
                               //     Err(es) => return Err(es.first().unwrap().to_owned()),
                               // };
    let mut parser: LangParser = LangParser::new(&tokens);
    let ast: Vec<Statement> = parser.parse_statement(Vec::new())?;
    for statement in ast {
        println!("{:?}", statement);
    }
    Ok(())
}

fn run(file: PathBuf) -> Result<()> {
    let input: String = match read_to_string(file.as_path()) {
        Ok(src) => src,
        Err(e) => {
            eprintln!("error reading file: {e}");
            return Ok(());
        }
    };

    let instructions: Vec<Instruction> = build_air(&input)?;

    run_instructions(instructions).map_err(|e| e.with_src(input))?;
    Ok(())
}

fn run_instructions(instructions: Vec<Instruction>) -> Result<(), LangError> {
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

fn build_air(src: &str) -> Result<Vec<Instruction>> {
    let mut lexer: Lexer = Lexer::new(src);
    let tokens = lexer.run()?; // {
                               //     Ok(tokens) => tokens,
                               //     Err(es) => return Err(es.first().unwrap().to_owned()),
                               // };
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

fn build(file: PathBuf) -> Result<()> {
    let input: String = match read_to_string(file.as_path()) {
        Ok(src) => src,
        Err(e) => {
            // really not sure why i cant use into_diagnostic with this
            // return Err(<std::io::Error as Into<std::error::Error>>::into(e).into_diagnostic());
            // return Err(e.int());
            eprintln!("error reading file: {e}");
            return Ok(());
        }
    };

    let air = build_air(&input)?;

    for instruction in air {
        print!("{instruction}");
    }
    Ok(())
}
