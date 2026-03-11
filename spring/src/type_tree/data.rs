use std::fmt::Display;

use chumsky::span::SimpleSpan;

pub use crate::parse_tree::Op;

#[derive(Debug, Clone)]
pub struct Ident<'src> {
    pub name: &'src str,
    pub span: SimpleSpan,
}

#[derive(Debug, Clone)]
pub struct Statement<'src> {
    pub kind: StatementKind<'src>,
    pub ty: Type,
    pub span: SimpleSpan,
}

#[derive(Debug, Clone)]
pub enum StatementKind<'src> {
    Expr(Expr<'src>),
    Macro(Ident<'src>, Vec<Expr<'src>>),
}

#[derive(Debug, Clone)]
pub struct Expr<'src> {
    pub kind: ExprKind<'src>,
    pub ty: Type,
    pub span: SimpleSpan,
}

#[derive(Debug, Clone)]
pub enum ExprKind<'src> {
    Int(u64),
    String(&'src str),
    BinOp(Op, Box<Expr<'src>>, Box<Expr<'src>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Error,
    Unit,
    Int,
    String,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Error => write!(f, "<error>"),
            Type::Unit => write!(f, "Unit"),
            Type::Int => write!(f, "Int"),
            Type::String => write!(f, "String"),
        }
    }
}
