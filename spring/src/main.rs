use std::fs;

use spring::{interpret, parse_tree, type_tree};

fn main() {
    let src = fs::read_to_string("scratch/scratch.son").unwrap();
    let parse_tree = parse_tree::parse(&src).unwrap();

    let mut ctx = type_tree::Context {
        source: &src,
        error_count: 0,
    };

    let type_tree: Vec<_> = parse_tree
        .into_iter()
        .map(|stmt| type_tree::type_statement(stmt, &mut ctx))
        .collect();

    if ctx.error_count > 0 {
        return;
    }

    let mut ctx = interpret::Context { source: &src };

    for stmt in type_tree {
        interpret::eval_stmt(&stmt.unwrap(), &mut ctx).unwrap();
    }
}
