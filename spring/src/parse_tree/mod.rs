use std::fmt::Display;

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{
    input::{Stream, ValueInput},
    prelude::*,
};
use logos::Logos;

#[derive(Debug, Clone)]
pub struct ParseError;

pub fn parse(src: &str) -> Result<Expr, ParseError> {
    // Create a logos lexer over the source code
    let token_iter = Token::lexer(src)
        .spanned()
        // Convert logos errors into tokens. We want parsing to be recoverable and not fail at the lexing stage, so
        // we have a dedicated `Token::Error` variant that represents a token error that was previously encountered
        .map(|(tok, span)| match tok {
            // Turn the `Range<usize>` spans logos gives us into chumsky's `SimpleSpan` via `Into`, because it's easier to work with
            Ok(tok) => (tok, span.into()),
            Err(()) => (Token::Error, span.into()),
        });

    // Turn the token iterator into a stream that chumsky can use for things like backtracking
    let token_stream = Stream::from_iter(token_iter)
        // Tell chumsky to split the (Token, SimpleSpan) stream into its parts so that it can handle the spans for us
        // This involves giving chumsky an 'end of input' span: we just use a zero-width span at the end of the string
        .map((0..src.len()).into(), |(t, s): (_, _)| (t, s));

    // Parse the token stream with our chumsky parser
    // If parsing was unsuccessful, generate a nice user-friendly diagnostic with ariadne
    parser().parse(token_stream).into_result().map_err(|errs| {
        for err in errs {
            Report::build(ReportKind::Error, ((), err.span().into_range()))
                .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
                .with_code(3)
                .with_message(err.to_string())
                .with_label(
                    Label::new(((), err.span().into_range()))
                        .with_message(err.reason().to_string())
                        .with_color(Color::Red),
                )
                .finish()
                .eprint(Source::from(src))
                .unwrap();
        }
        ParseError
    })
}

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

        product.clone().foldl(
            choice((
                just(Token::Add).to(Op::Add), //
                just(Token::Sub).to(Op::Sub),
            ))
            .then(product)
            .repeated(),
            |lhs, (op, rhs)| Expr::BinOp(op, Box::new(lhs), Box::new(rhs)),
        )
    })
}
