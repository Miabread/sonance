use crate::{
    interpret::{Value, error::InterpretError},
    parse_tree::ParseError,
    type_tree::error::TypeError,
};

pub mod interpret;
pub mod parse_tree;
#[cfg(test)]
pub mod test;
pub mod type_tree;

#[derive(Debug, Clone, PartialEq)]
pub struct DummyError;

#[derive(Debug, Clone, PartialEq)]
pub enum LibError<'src> {
    ParseError(ParseError<'src>),
    TypeError(Vec<TypeError>),
    InterpretError(InterpretError<'src>),
}

pub fn run<'src>(src: &'src str) -> Result<Vec<Value<'src>>, LibError<'src>> {
    let parse_tree = parse_tree::parse(src).map_err(LibError::ParseError)?;

    let mut ctx = type_tree::Context::new(src);
    let type_tree = type_tree::type_module(parse_tree, &mut ctx);

    let type_tree = if let Ok(value) = type_tree
        && ctx.errors.is_empty()
    {
        value
    } else {
        return Err(LibError::TypeError(ctx.errors));
    };

    let mut ctx = interpret::Context::new(src);
    interpret::eval_module(&type_tree, &mut ctx).map_err(LibError::InterpretError)?;

    Ok(ctx.test_output)
}
