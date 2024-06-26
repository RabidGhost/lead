use lead::{
    air::air::Instruction,
    lex::{span::Spans, token::Token, token::TokenType},
    parse::ast::Statement,
};
use leadc::cli::{Cli, Commands, RunArgs};
use leadc::pipeline::Pipeline;

use clap::Parser;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use miette::Result;
use std::{io::stdin, path::PathBuf};

fn main() -> Result<()> {
    let mut cli = Cli::parse();

    // for now just set the level at debug.
    let mut file_path = String::from("logs/leadc.log");

    if let Commands::Run(ref mut args) = cli.command {
        file_path = match args.log_path.take() {
            Some(pth) => pth.into_os_string().into_string().unwrap(),
            None => file_path,
        };
    }

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(file_path)
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("logfile")
                .build(LevelFilter::Debug),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();

    match cli.command {
        Commands::Run(args) => run(args)?,
        Commands::Build { file } => build(file)?,
        Commands::Lex { file } => lex(file)?,
        Commands::Parse { file } => parse(file)?,

        #[cfg(debug_assertions)]
        Commands::Test { file } => test(file)?,

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
    let air: Vec<Instruction> = Pipeline::try_from(file)?
        .lex()?
        .parse()?
        .build()?
        .try_into()?;

    for instruction in air {
        print!("{instruction}");
    }
    Ok(())
}

#[cfg(debug_assertions)]
fn test(_file: PathBuf) -> Result<()> {
    println!("{:?}", std::env::current_dir().unwrap());
    Ok(())
    // let statements: Vec<Statement> = Pipeline::try_from(file)?.lex()?.parse()?.into();

    // for statement in statements {
    //     println!("{statement:?} : {:?}", statement.span().composing_ids())
    // }
    // Ok(())
}
