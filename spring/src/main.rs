use std::fs;

use spring::{Context, parse_tree};

fn main() {
    let src = fs::read_to_string("scratch/scratch.son").unwrap();
    let parse_tree = parse_tree::parse(&src).unwrap();

    let mut ctx = Context { source: &src };

    for stmt in parse_tree {
        stmt.inner.eval(&mut ctx).unwrap();
    }
}
