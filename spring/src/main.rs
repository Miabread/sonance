use std::fs;

use spring::run;

fn main() {
    let src = fs::read_to_string("scratch/scratch.son").unwrap();

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
