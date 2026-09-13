use chumsky::span::Spanned;

#[derive(Debug, Clone, PartialEq)]
pub struct Ident<'src>(pub &'src str);

#[derive(Debug, Clone, PartialEq)]
pub struct Module<'src> {
    pub items: Vec<Spanned<Item<'src>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item<'src> {
    Func(FuncItem<'src>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncItem<'src> {
    pub name: Spanned<Ident<'src>>,
    pub args: Spanned<Vec<Argument<'src>>>,
    pub body: Spanned<Block<'src>>,
    pub return_type: Spanned<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument<'src> {
    pub name: Spanned<Ident<'src>>,
    pub ty: Spanned<Type>,
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
    BinOp(BinOpExpr<'src>),
    Match(MatchExpr<'src>),
    MacroCall(MacroCallExpr<'src>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinOpExpr<'src> {
    pub op: BinOp,
    pub lhs: Box<Spanned<Expr<'src>>>,
    pub rhs: Box<Spanned<Expr<'src>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpr<'src> {
    pub scrutinee: Box<Spanned<Expr<'src>>>,
    pub arms: Vec<(Spanned<Pattern>, Spanned<Expr<'src>>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Int(u64),
    Discard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacroCallExpr<'src> {
    pub name: Spanned<Ident<'src>>,
    pub args: Vec<Spanned<Expr<'src>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Unit,
    Int,
    Float,
    String,
}
