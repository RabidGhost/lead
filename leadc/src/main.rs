use clap::Parser;
use lead::lex::{token::Token, token::TokenType};
use lead::parse::ast::Statement;
use lead_vm::air::Instruction;
use leadc::cli::{Cli, Commands, RunArgs};
use leadc::pipeline::Pipeline;
use miette::Result;
use std::{io::stdin, path::PathBuf};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => run(args)?,
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

fn run(args: RunArgs) -> Result<()> {
    match args.stdin {
        true => Pipeline::try_from(&mut stdin())?,
        false => Pipeline::try_from(
            args.file
                .clone()
                .expect("stdin and file should be mutally exclusive"),
        )?,
    }
    .with_run_args(args)?
    .lex()?
    .parse()?
    .build()?
    .run()
}

fn build(file: PathBuf) -> Result<()> {
    let air: Vec<Instruction> = Pipeline::try_from(file)?.lex()?.parse()?.build()?.into();

    for instruction in air {
        print!("{instruction}");
    }
    Ok(())
}
