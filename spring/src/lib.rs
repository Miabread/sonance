use crate::interpret::Value;

pub mod interpret;
pub mod parse_tree;
#[cfg(test)]
pub mod test;
pub mod type_tree;

#[derive(Debug, Clone, PartialEq)]
pub struct DummyError;

pub fn run(src: &'_ str) -> Result<Vec<Value<'_>>, DummyError> {
    let parse_tree = parse_tree::parse(src).unwrap();

    let mut ctx = type_tree::Context {
        source: src,
        error_count: 0,
    };

    let type_tree = type_tree::type_module(parse_tree, &mut ctx);

    if ctx.error_count > 0 {
        return Err(DummyError);
    }

    let mut ctx = interpret::Context::new(src);

    interpret::eval_module(&type_tree.unwrap(), &mut ctx).unwrap();

    Ok(ctx.test_output)
}
