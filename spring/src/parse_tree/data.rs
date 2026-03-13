use chumsky::span::Spanned;

#[derive(Debug, Clone)]
pub enum Statement<'src> {
    Expr(Spanned<Expr<'src>>),
    Macro(Spanned<&'src str>, Vec<Spanned<Expr<'src>>>),
}

#[derive(Debug, Clone)]
pub enum Expr<'src> {
    Int(u64),
    Float(f64),
    String(&'src str),
    BinOp {
        op: Op,
        lhs: Box<Spanned<Expr<'src>>>,
        rhs: Box<Spanned<Expr<'src>>>,
    },
    Match {
        scrutinee: Box<Spanned<Expr<'src>>>,
        arms: Vec<(Spanned<Pattern>, Spanned<Expr<'src>>)>,
    },
}

#[derive(Debug, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Int(u64),
    Discard,
}
