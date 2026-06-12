use std::{fs, path::PathBuf};

use clap::Parser;
use spring::run;

#[derive(clap::Parser)]
struct Cli {
    input: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let src = fs::read_to_string(cli.input).unwrap();

    let Ok(test_output) = run(&src) else {
        return;
    };

    if !test_output.is_empty() {
        println!("\ntest_output:");
        for (i, output) in test_output.iter().enumerate() {
            println!("[{i}] {output:?}");
        }
    }
}
