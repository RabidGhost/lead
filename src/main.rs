use crate::{
    interpreter::{GlobalAlloc, Interpretable},
    parse::ast::{Literal, Statement},
};
use air::Lowerable;
use clap::{Parser, Subcommand};
use error::LangError;
use lex::Lexer;
use parse::LangParser;
use std::{fs::read_to_string, path::PathBuf};

mod air;
mod error;
mod interpreter;
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
    Repl,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => run(file),
        Commands::Build { file } => build(file),
        _ => todo!("implement repl"),
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

    let ast = match run_frontend(&input) {
        Ok(src) => src,
        Err(e) => {
            eprintln!("{e:#?}");
            return;
        }
    };

    let mut alloc: GlobalAlloc = GlobalAlloc::new();

    let mut out: Literal = Literal::Unit;
    for statement in ast {
        out = match statement.eval(&mut alloc) {
            Ok(lit) => lit,
            Err(e) => {
                eprintln!("{e:?}");
                return;
            }
        }
    }

    println!("{input}");
    println!(" = {:?}", out);
}

fn run_frontend(src: &str) -> Result<Vec<Statement>, LangError> {
    let mut lexer: Lexer = Lexer::new(src);
    let tokens = match lexer.run() {
        Ok(tokens) => tokens,
        Err(es) => return Err(es.first().unwrap().to_owned()),
    };
    let mut parser: LangParser = LangParser::new(&tokens);
    let statements_buf: Vec<Statement> = Vec::new();
    parser.parse_statement(statements_buf)
}

fn build(file: PathBuf) {
    let input: String = match read_to_string(file.as_path()) {
        Ok(src) => src,
        Err(e) => {
            eprintln!("error reading file: {e}");
            return;
        }
    };

    let ast = match run_frontend(&input) {
        Ok(src) => src,
        Err(e) => {
            eprintln!("{e:#?}");
            return;
        }
    };

    for statement in ast {
        match statement {
            Statement::Expr(expr) => match expr.lower() {
                Ok(air) => print!("{air}"),
                Err(e) => eprintln!("{e:#?}"),
            },
            _ => todo!(),
        }
    }
}
