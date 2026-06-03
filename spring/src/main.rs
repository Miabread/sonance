use std::fs;

use spring::run;

fn main() {
    let src = fs::read_to_string("scratch/scratch.son").unwrap();

    let Ok(output) = run(&src) else {
        return;
    };

    if !output.is_empty() {
        dbg!(output);
    }
}
