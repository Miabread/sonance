use std::fmt::Display;

use chumsky::{input::ValueInput, prelude::*};
use logos::Logos;

#[derive(Debug, Clone, Logos, PartialEq)]
pub enum Token<'src> {
    Error,
    #[regex(r"\s+", logos::skip)]
    Whitespace,

    #[regex(r"[0-9]+")]
    Int(&'src str),
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident(&'src str),

    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,

    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("->")]
    Arrow,

    #[token("func")]
    Func,
    #[token("int")]
    TInt,
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Error => write!(f, "<error>"),
            Token::Whitespace => write!(f, "<whitespace>"),
            Token::Int(i) => write!(f, "{i}"),
            Token::Ident(i) => write!(f, "{i}"),
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::OpenBracket => write!(f, "["),
            Token::CloseBracket => write!(f, "]"),
            Token::OpenBrace => write!(f, "{{"),
            Token::CloseBrace => write!(f, "}}"),
            Token::Add => write!(f, "+"),
            Token::Sub => write!(f, "-"),
            Token::Mul => write!(f, "*"),
            Token::Div => write!(f, "/"),
            Token::Arrow => write!(f, "->"),
            Token::Func => write!(f, "func"),
            Token::TInt => write!(f, "int"),
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
    pub fn eval(self) -> Result<u64, String> {
        Ok(match self {
            Expr::Int(i) => i,
            Expr::BinOp(op, lhs, rhs) => match op {
                Op::Add => lhs.eval()? + rhs.eval()?,
                Op::Sub => lhs.eval()? - rhs.eval()?,
                Op::Mul => lhs.eval()? * rhs.eval()?,
                Op::Div => {
                    let rhs = rhs.eval()?;
                    if rhs == 0 {
                        return Err("div by 0".into());
                    }
                    lhs.eval()? / rhs
                }
            },
        })
    }
}
