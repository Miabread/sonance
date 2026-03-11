use chumsky::span::Spanned;

#[derive(Debug, Clone)]
pub enum Statement<'src> {
    Expr(Spanned<Expr<'src>>),
    Macro(Spanned<&'src str>, Vec<Spanned<Expr<'src>>>),
}

#[derive(Debug, Clone)]
pub enum Expr<'src> {
    Int(u64),
    String(&'src str),
    BinOp(Op, Box<Spanned<Expr<'src>>>, Box<Spanned<Expr<'src>>>),
}

#[derive(Debug, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}
