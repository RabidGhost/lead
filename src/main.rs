use crate::{
    interpreter::{GlobalAlloc, Interpretable},
    parse::ast::{Literal, Statement},
};
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
    let statements_buf: Vec<Statement> = Vec::new();
    let ast = match parser.parse_statement(statements_buf) {
        Ok(expr) => expr,
        Err(e) => {
            eprintln!("{e:?}");
            return;
        }
    };

    let mut alloc: GlobalAlloc = GlobalAlloc::new();

    // alloc.insert(
    //     "my_var".to_owned(),
    //     parse::ast::Literal::Number {
    //         val: 6,
    //         span: (0, 0),
    //     },
    // );

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
