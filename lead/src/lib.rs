use clap::{ArgGroup, Args, Parser, Subcommand};
use lead_vm::vm::DEFAULT_MEMORY_SIZE;
use std::path::PathBuf;

pub mod air;
pub mod error;
pub mod lex;
pub mod parse;
/// The compiler pipeline, from front to back.

#[derive(Parser)]
#[command(about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// run in the interpreter
    Run(RunArgs),
    //     file: PathBuf,
    //     args: RunArgs,
    // },
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

#[derive(Args, Clone)]
#[clap(group(
            ArgGroup::new("input")
                .required(true)
                .args(&["file", "stdin"]),
        ))]
struct RunArgs {
    file: Option<PathBuf>,
    #[clap(long)]
    stdin: bool,
    /// memory size of the virtual machine in bytes
    #[arg(default_value_t = DEFAULT_MEMORY_SIZE)]
    #[clap(short)]
    memory_size: usize,
}
