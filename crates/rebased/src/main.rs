use std::process::exit;

use clap::Parser;
use clap::Subcommand;
use git2::Repository;

mod common;
mod engine;
mod stack;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = ".")]
    repository: String,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Stack {
        #[clap(short, long, default_value = "origin/master")]
        base: String,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let repository = match Repository::open(&args.repository) {
        Ok(engine) => engine,
        Err(error) => {
            eprintln!("Failed to open repository {}: {}", args.repository, error);
            exit(1);
        }
    };

    match args.command {
        Command::Stack { base } => stack::main(&repository, stack::Options { base }),
    }
}
