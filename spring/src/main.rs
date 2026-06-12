use std::{fs, path::PathBuf, process::exit};

use clap::Parser;
use spring::run;

#[derive(clap::Parser)]
struct Cli {
    input: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let src = fs::read_to_string(cli.input).unwrap();

    match run(&src) {
        Ok(exit_code) => exit(exit_code),
        Err(_) => exit(1),
    }
}
