use std::fmt::Display;

use chumsky::{input::ValueInput, prelude::*};
use logos::Logos;

#[derive(Debug, Clone, Logos, PartialEq)]
pub enum Token<'src> {
    Error,

    #[regex(r"[+-]?[0-9]+")]
    Int(&'src str),

    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,

    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,

    #[regex(r"\s+", logos::skip)]
    Whitespace,
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Error => write!(f, "<error>"),
            Token::Int(_) => write!(f, ""),
            Token::Add => write!(f, "+"),
            Token::Sub => write!(f, "-"),
            Token::Mul => write!(f, "*"),
            Token::Div => write!(f, "/"),
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::Whitespace => write!(f, "<whitespace>"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(u64),
    BinOp(Op, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

pub fn parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>>>>
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    recursive(|expr| {
        let int = select! {
            Token::Int(i) => Expr::Int(i.parse().unwrap())
        };

        let paren = expr.delimited_by(just(Token::OpenParen), just(Token::CloseParen));

        let atom = int.or(paren);

        let product = atom.clone().foldl(
            choice((
                just(Token::Mul).to(Op::Mul), //
                just(Token::Div).to(Op::Div),
            ))
            .then(atom)
            .repeated(),
            |lhs, (op, rhs)| Expr::BinOp(op, Box::new(lhs), Box::new(rhs)),
        );

        let sum = product.clone().foldl(
            choice((
                just(Token::Add).to(Op::Add), //
                just(Token::Sub).to(Op::Sub),
            ))
            .then(product)
            .repeated(),
            |lhs, (op, rhs)| Expr::BinOp(op, Box::new(lhs), Box::new(rhs)),
        );

        sum
    })
}

impl Expr {
    pub fn eval(self) -> u64 {
        match self {
            Expr::Int(i) => i,
            Expr::BinOp(op, lhs, rhs) => match op {
                Op::Add => lhs.eval() + rhs.eval(),
                Op::Sub => lhs.eval() - rhs.eval(),
                Op::Mul => lhs.eval() * rhs.eval(),
                Op::Div => lhs.eval() / rhs.eval(),
            },
        }
    }
}
