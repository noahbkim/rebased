use clap::Parser;
use std::process::exit;

mod engine;

use engine::Engine;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)] 
struct Args {
    #[arg(short, long, default_value = ".")]
    repository: String
}

fn main() {
    let args = Args::parse();
    let engine = match Engine::open(&args.repository) {
        Ok(engine) => engine,
        Err(error) => {
            eprintln!("Failed to open repository {}: {}", args.repository, error);
            exit(1);
        }
    };
}
