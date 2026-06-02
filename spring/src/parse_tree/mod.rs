pub mod data;
pub mod token;

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{
    input::{Stream, ValueInput},
    pratt::*,
    prelude::*,
};
use logos::Logos;

use crate::{DummyError, parse_tree::token::Token};

pub use data::*;

pub fn parse(src: &'_ str) -> Result<Spanned<Module<'_>>, DummyError> {
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
    module().parse(token_stream).into_result().map_err(|errs| {
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
        DummyError
    })
}

pub fn module<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Module<'src>>, extra::Err<Rich<'tokens, Token<'src>>>>
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    item()
        .repeated()
        .collect()
        .map(|items| Module { items })
        .spanned()
}

pub fn item<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Item<'src>>, extra::Err<Rich<'tokens, Token<'src>>>>
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    just(Token::Func)
        .ignore_then(select! { Token::Ident(i) => i}.spanned())
        .then_ignore(just(Token::OpenParen))
        .then_ignore(just(Token::CloseParen))
        .then(block())
        .map(|(name, body)| Item::Func { name, body })
        .spanned()
}

pub fn block<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Block<'src>>, extra::Err<Rich<'tokens, Token<'src>>>>
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    statement()
        .separated_by(just(Token::Semi))
        .allow_trailing()
        .collect()
        .delimited_by(just(Token::OpenBrace), just(Token::CloseBrace))
        .map(|statements| Block { body: statements })
        .spanned()
}

pub fn statement<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Statement<'src>>, extra::Err<Rich<'tokens, Token<'src>>>>
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    expr().map(Statement::Expr).spanned()
}

pub fn expr<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Expr<'src>>, extra::Err<Rich<'tokens, Token<'src>>>>
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    recursive(|expr| {
        let literal = select! {
            Token::Int(i) => Expr::Int(i),
            Token::Float(f) => Expr::Float(f),
            Token::String(s) => Expr::String(s),
        }
        .spanned();

        let paren = expr
            .clone()
            .delimited_by(just(Token::OpenParen), just(Token::CloseParen));

        let match_body = pattern()
            .then_ignore(just(Token::Arrow))
            .then(expr.clone())
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect()
            .delimited_by(just(Token::OpenBrace), just(Token::CloseBrace));

        let match_atom = just(Token::Match)
            .ignore_then(paren.clone())
            .then(match_body.clone())
            .map(|(scrutinee, arms)| Expr::Match {
                scrutinee: Box::new(scrutinee),
                arms,
            })
            .spanned();

        let macro_call = select! {
            Token::Ident(i) => i,
        }
        .spanned()
        .then_ignore(just(Token::Bang))
        .then(
            expr.separated_by(just(Token::Comma))
                .allow_trailing()
                .collect()
                .delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
        )
        .map(|(name, args)| Expr::Macro { name, args })
        .spanned();

        let atom = literal.or(paren).or(match_atom).or(macro_call);

        atom.pratt((
            postfix(
                3,
                just(Token::Dot)
                    .then(just(Token::Match))
                    .ignore_then(match_body.clone()),
                |scrutinee, arms, ctx| {
                    Expr::Match {
                        scrutinee: Box::new(scrutinee),
                        arms,
                    }
                    .with_span(ctx.span())
                },
            ),
            infix(left(2), just(Token::Mul), |lhs, _, rhs, ctx| {
                Expr::BinOp {
                    op: Op::Mul,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
                .with_span(ctx.span())
            }),
            infix(left(2), just(Token::Div), |lhs, _, rhs, ctx| {
                Expr::BinOp {
                    op: Op::Div,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
                .with_span(ctx.span())
            }),
            infix(left(1), just(Token::Add), |lhs, _, rhs, ctx| {
                Expr::BinOp {
                    op: Op::Add,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
                .with_span(ctx.span())
            }),
            infix(left(1), just(Token::Sub), |lhs, _, rhs, ctx| {
                Expr::BinOp {
                    op: Op::Sub,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
                .with_span(ctx.span())
            }),
        ))
    })
}

pub fn pattern<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Pattern>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    select! {
        Token::Int(i) => Pattern::Int(i),
        Token::Underscore => Pattern::Discard,
    }
    .spanned()
}
