use crate::{
    interpret::{Interpreter, Value, error::InterpretError},
    parse_tree::ParseError,
    type_tree::{TypeContext, error::TypeError},
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

    let mut type_ctx = TypeContext::new(src);
    let type_tree = type_ctx.type_module(parse_tree);

    let type_tree = if let Ok(value) = type_tree
        && type_ctx.errors.is_empty()
    {
        value
    } else {
        return Err(LibError::TypeError(type_ctx.errors));
    };

    let mut interpreter = Interpreter::new(src);
    interpreter
        .eval_module(&type_tree)
        .map_err(LibError::InterpretError)?;

    Ok(interpreter.test_output)
}
