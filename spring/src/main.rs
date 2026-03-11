use std::fs;

use spring::{
    interpret::{Context, eval_stmt},
    parse_tree, type_tree,
};

fn main() {
    let src = fs::read_to_string("scratch/scratch.son").unwrap();
    let parse_tree = parse_tree::parse(&src).unwrap();
    let type_tree: Vec<_> = parse_tree
        .into_iter()
        .map(|stmt| type_tree::type_statement(stmt))
        .collect();

    let mut ctx = Context { source: &src };

    for stmt in type_tree {
        eval_stmt(&stmt, &mut ctx).unwrap();
    }
}
