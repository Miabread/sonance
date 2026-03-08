use std::fs;

use spring::parse_tree;

fn main() {
    let src = fs::read_to_string("scratch/scratch.son").unwrap();
    let parse_tree = parse_tree::parse(&src).unwrap();

    for stmt in parse_tree {
        stmt.inner.eval().unwrap();
    }
}
