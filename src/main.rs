use crate::interpreter::Interpretable;
use lex::Lexer;
use parse::LangParser;
// use miette::NamedSource;
// use miette::{Diagnostic, Result, SourceSpan};
use clap::{Parser, Subcommand};
use std::{fs::read_to_string, path::PathBuf};

mod error;
mod interpreter;
mod lang;
mod lex;
mod parse;
mod repl;

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

    Repl,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => run(file),
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

    let mut lexer: Lexer = Lexer::new(&input);

    let tokens = match lexer.run() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("{e:?}");
            return;
        }
    };

    let mut parser: LangParser = LangParser::new(&tokens);
    let ast = match parser.parse() {
        Ok(expr) => expr,
        Err(e) => {
            eprintln!("{e:?}");
            return;
        }
    };
    let out = match ast.eval() {
        Ok(lit) => lit,
        Err(e) => {
            eprintln!("{e:?}");
            return;
        }
    };

    println!("{input}");
    println!(" = {:?}", out);
}
