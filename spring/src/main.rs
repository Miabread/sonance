use std::fs;

use spring::{interpret, parse_tree, type_tree};

fn main() {
    let src = fs::read_to_string("scratch/scratch.son").unwrap();
    let parse_tree = parse_tree::parse(&src).unwrap();

    let mut ctx = type_tree::Context {
        source: &src,
        error_count: 0,
    };

    let type_tree = type_tree::type_module(parse_tree, &mut ctx);

    if ctx.error_count > 0 {
        return;
    }

    let mut ctx = interpret::Context::new(&src);

    interpret::eval_module(&type_tree.unwrap(), &mut ctx).unwrap();

    if !ctx.test_output.is_empty() {
        dbg!(ctx.test_output);
    }
}
