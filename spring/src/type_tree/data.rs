use std::fmt::Display;

use chumsky::span::{SimpleSpan, Spanned};

pub use crate::parse_tree::{BinOp, Pattern};

#[derive(Debug, Clone, PartialEq)]
pub struct Module<'src> {
    pub items: Vec<Item<'src>>,
    pub span: SimpleSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item<'src> {
    pub kind: ItemKind<'src>,
    pub span: SimpleSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind<'src> {
    Func(FuncItem<'src>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncItem<'src> {
    pub name: Ident<'src>,
    pub body: Block<'src>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block<'src> {
    pub body: Vec<Statement<'src>>,
    pub trailing: Expr<'src>,
    pub span: SimpleSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ident<'src> {
    pub name: &'src str,
    pub span: SimpleSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement<'src> {
    pub kind: StatementKind<'src>,
    pub ty: Type,
    pub span: SimpleSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind<'src> {
    Expr(Expr<'src>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr<'src> {
    pub kind: ExprKind<'src>,
    pub ty: Type,
    pub span: SimpleSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind<'src> {
    Unit,
    Int(u64),
    Float(f64),
    String(&'src str),
    Var(Ident<'src>),
    BinOp(BinOpExpr<'src>),
    Match(MatchExpr<'src>),
    MacroCall(MacroCallExpr<'src>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinOpExpr<'src> {
    pub op: BinOp,
    pub lhs: Box<Expr<'src>>,
    pub rhs: Box<Expr<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpr<'src> {
    pub scrutinee: Box<Expr<'src>>,
    pub arms: Vec<(Spanned<Pattern>, Block<'src>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacroCallExpr<'src> {
    pub name: Ident<'src>,
    pub args: Vec<Expr<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub kind: TypeKind,
    pub span: SimpleSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Error,
    Unit,
    Int,
    Float,
    String,
    Func {
        args: Vec<Type>,
        return_type: Box<Type>,
    },
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TypeKind::Error => write!(f, "<error>"),
            TypeKind::Unit => write!(f, "Unit"),
            TypeKind::Int => write!(f, "Int"),
            TypeKind::Float => write!(f, "Float"),
            TypeKind::String => write!(f, "String"),
            TypeKind::Func {
                args,
                return_type: ret,
            } => {
                write!(f, "func(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i == 0 {
                        write!(f, "{arg}")?;
                    } else {
                        write!(f, ", {arg}")?;
                    }
                }
                write!(f, ") -> {ret}")
            }
        }
    }
}
