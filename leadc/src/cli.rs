use clap::{ArgGroup, Args, Parser, Subcommand};
use lead_vm::DEFAULT_MEMORY_SIZE;
use std::path::PathBuf;

#[derive(Parser)]
#[command(about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
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
    #[cfg(debug_assertions)]
    Test {
        file: PathBuf,
    },
}

#[derive(Args, Clone)]
#[clap(groups(vec![
            ArgGroup::new("input")
                .required(true)
                .args(&["file", "stdin"]),
            ArgGroup::new("verbosity")
                .args(&["quiet", "verbose", "very_verbose"])
        ]))]
pub struct RunArgs {
    pub file: Option<PathBuf>,
    #[clap(long)]
    pub stdin: bool,
    /// memory size of the virtual machine in bytes
    #[arg(default_value_t = DEFAULT_MEMORY_SIZE)]
    #[clap(short)]
    pub memory_size: usize,
    /// quiet
    #[clap(short('q'))]
    pub quiet: bool,
    /// verbose
    #[clap(short('v'))]
    pub verbose: bool,
    /// very verbose
    #[clap(long("vv"), long("very-verbose"))]
    pub very_verbose: bool,

    /// a path to a folder to log to
    #[clap(short, long)]
    pub log_path: Option<PathBuf>,
}
