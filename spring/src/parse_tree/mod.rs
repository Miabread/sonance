pub mod token;

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{
    input::{Stream, ValueInput},
    prelude::*,
};
use logos::Logos;

use crate::parse_tree::token::Token;

#[derive(Debug, Clone)]
pub struct ParseError;

pub fn parse(src: &'_ str) -> Result<Vec<Spanned<Statement<'_>>>, ParseError> {
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
    statements()
        .parse(token_stream)
        .into_result()
        .map_err(|errs| {
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

pub enum Statement<'src> {
    Expr(Spanned<Expr<'src>>),
    Macro(Spanned<&'src str>, Vec<Spanned<Expr<'src>>>),
}

pub fn statements<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Vec<Spanned<Statement<'src>>>, extra::Err<Rich<'tokens, Token<'src>>>>
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    let macro_call = select! {
        Token::Ident(i) => i,
    }
    .map_with(|e, ctx| e.with_span(ctx.span()))
    .then_ignore(just(Token::Bang))
    .then(
        expr()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect()
            .delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
    )
    .map(|(name, args)| Statement::Macro(name, args));

    expr()
        .map(Statement::Expr)
        .or(macro_call)
        .map_with(|e, ctx| e.with_span(ctx.span()))
        .separated_by(just(Token::Semi))
        .allow_trailing()
        .collect()
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

pub fn expr<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Expr<'src>>, extra::Err<Rich<'tokens, Token<'src>>>>
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    recursive(|expr| {
        let literal = select! {
            Token::Int(i) => Expr::Int(i.parse().unwrap()),
            Token::String(s) => Expr::String(s),
        }
        .map_with(|e, ctx| e.with_span(ctx.span()));

        let paren = expr.delimited_by(just(Token::OpenParen), just(Token::CloseParen));

        let atom = literal.or(paren);

        let product = atom.clone().foldl_with(
            choice((
                just(Token::Mul).to(Op::Mul), //
                just(Token::Div).to(Op::Div),
            ))
            .then(atom)
            .repeated(),
            |lhs: Spanned<Expr<'_>>, (op, rhs): (_, Spanned<Expr<'_>>), ctx| {
                Expr::BinOp(op, Box::new(lhs), Box::new(rhs)).with_span(ctx.span())
            },
        );

        product.clone().foldl_with(
            choice((
                just(Token::Add).to(Op::Add), //
                just(Token::Sub).to(Op::Sub),
            ))
            .then(product)
            .repeated(),
            |lhs, (op, rhs), ctx| {
                Expr::BinOp(op, Box::new(lhs), Box::new(rhs)).with_span(ctx.span())
            },
        )
    })
}
