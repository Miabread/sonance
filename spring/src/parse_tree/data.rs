use chumsky::span::Spanned;

#[derive(Debug, Clone, PartialEq)]
pub struct Module<'src> {
    pub items: Vec<Spanned<Item<'src>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item<'src> {
    Func {
        name: Spanned<&'src str>,
        body: Spanned<Block<'src>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block<'src> {
    pub body: Vec<Spanned<Statement<'src>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement<'src> {
    Expr(Spanned<Expr<'src>>),
}

#[derive(Debug, Clone, PartialEq)]
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
    Macro {
        name: Spanned<&'src str>,
        args: Vec<Spanned<Expr<'src>>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Int(u64),
    Discard,
}
