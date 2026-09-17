pub mod data;
pub mod token;

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{
    input::{Stream, ValueInput},
    pratt::*,
    prelude::*,
};
use logos::Logos;

use crate::parse_tree::token::Token;

pub use data::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError<'src>(Vec<Rich<'src, Token<'src>>>);

// This code was copied from the chumsky example for ariadne integration I believe
pub fn parse<'src>(src: &'src str) -> Result<Spanned<Module<'src>>, ParseError<'src>> {
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
        for err in &errs {
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
        ParseError(errs)
    })
}

pub fn module<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Module<'src>>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    item()
        .repeated()
        .collect()
        .map(|items| Module { items })
        .spanned()
        .labelled("module")
}

pub fn item<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Item<'src>>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    func().map(Item::Func).spanned().labelled("item")
}

pub fn func<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, FuncItem<'src>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    just(Token::Func)
        .ignore_then(ident())
        .then(
            ident()
                .then_ignore(just(Token::Colon))
                .then(ty())
                .map(|(name, ty)| Argument { name, ty })
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect()
                .delimited_by(just(Token::OpenParen), just(Token::CloseParen))
                .spanned(),
        )
        .then_ignore(just(Token::Arrow))
        .then(ty())
        .then(block())
        .map(|(((name, args), return_type), body)| FuncItem {
            name,
            args,
            body,
            return_type,
        })
        .labelled("func item")
}

pub fn ident<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Ident<'src>>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    select! { Token::Ident(i) => Ident(i) }
        .spanned()
        .labelled("identifier")
}

pub fn ty<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Type>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    select! {
        Token::TUnit => Type::Unit,
        Token::TInt => Type::Int,
        Token::TFloat => Type::Float,
        Token::TString => Type::String,
    }
    .spanned()
    .labelled("type")
}

pub fn block<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Block<'src>>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    recursive(|block| {
        statement(block.clone())
            .then_ignore(just(Token::Semi))
            .repeated()
            .collect()
            .then(expr(block).or_not().spanned())
            .delimited_by(just(Token::OpenBrace), just(Token::CloseBrace))
            .map(|(body, trailing)| Block { body, trailing })
            .spanned()
            .labelled("block")
    })
}

pub fn statement<'tokens, 'src: 'tokens, I>(
    block: Recursive<
        dyn Parser<'tokens, I, Spanned<Block<'src>>, extra::Err<Rich<'tokens, Token<'src>>>>
            + 'tokens,
    >,
) -> impl Parser<'tokens, I, Spanned<Statement<'src>>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    expr(block)
        .map(Statement::Expr)
        .spanned()
        .labelled("statement")
}

pub fn expr<'tokens, 'src: 'tokens, I>(
    block: Recursive<
        dyn Parser<'tokens, I, Spanned<Block<'src>>, extra::Err<Rich<'tokens, Token<'src>>>>
            + 'tokens,
    >,
) -> impl Parser<'tokens, I, Spanned<Expr<'src>>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
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
            .delimited_by(just(Token::OpenParen), just(Token::CloseParen))
            .then(block)
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect()
            .delimited_by(just(Token::OpenBrace), just(Token::CloseBrace));

        let match_atom = just(Token::Match)
            .ignore_then(paren.clone())
            .then(match_body.clone())
            .map(|(scrutinee, arms)| {
                Expr::Match(MatchExpr {
                    scrutinee: Box::new(scrutinee),
                    arms,
                })
            })
            .spanned();

        let args = expr
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect()
            .delimited_by(just(Token::OpenParen), just(Token::CloseParen));

        let macro_call_atom = ident()
            .then_ignore(just(Token::Bang))
            .then(args.clone())
            .map(|(name, args)| Expr::MacroCall(MacroCallExpr { name, args }))
            .spanned();

        let func_call_atom = ident()
            .clone()
            .then(args.clone())
            .map(|(name, args)| Expr::FuncCall(FuncCallExpr { name, args }))
            .spanned();

        let var = ident().map(|v| Expr::Var(v.inner).with_span(v.span));

        let atom = literal
            .or(paren)
            .or(match_atom)
            .or(macro_call_atom)
            .or(func_call_atom)
            .or(var);

        atom.pratt((
            postfix(
                3,
                just(Token::Dot)
                    .then(just(Token::Match))
                    .ignore_then(match_body.clone()),
                |scrutinee, arms, ctx| {
                    Expr::Match(MatchExpr {
                        scrutinee: Box::new(scrutinee),
                        arms,
                    })
                    .with_span(ctx.span())
                },
            ),
            postfix(
                3,
                just(Token::Dot)
                    .ignore_then(ident().then_ignore(just(Token::Bang)))
                    .then(args.clone().repeated().at_most(1).collect()),
                |first_arg, (name, test): (_, Vec<_>), ctx| {
                    let mut args: Vec<_> = test.into_iter().next().unwrap_or_default();
                    args.insert(0, first_arg);
                    Expr::MacroCall(MacroCallExpr { name, args }).with_span(ctx.span())
                },
            ),
            postfix(
                3,
                just(Token::Dot)
                    .ignore_then(ident())
                    .then(args.repeated().at_most(1).collect()),
                |first_arg, (name, test): (_, Vec<_>), ctx| {
                    let mut args: Vec<_> = test.into_iter().next().unwrap_or_default();
                    args.insert(0, first_arg);
                    Expr::FuncCall(FuncCallExpr { name, args }).with_span(ctx.span())
                },
            ),
            infix(left(2), just(Token::Mul), |lhs, _, rhs, ctx| {
                Expr::BinOp(BinOpExpr {
                    op: BinOp::Mul,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                })
                .with_span(ctx.span())
            }),
            infix(left(2), just(Token::Div), |lhs, _, rhs, ctx| {
                Expr::BinOp(BinOpExpr {
                    op: BinOp::Div,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                })
                .with_span(ctx.span())
            }),
            infix(left(1), just(Token::Add), |lhs, _, rhs, ctx| {
                Expr::BinOp(BinOpExpr {
                    op: BinOp::Add,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                })
                .with_span(ctx.span())
            }),
            infix(left(1), just(Token::Sub), |lhs, _, rhs, ctx| {
                Expr::BinOp(BinOpExpr {
                    op: BinOp::Sub,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                })
                .with_span(ctx.span())
            }),
        ))
        .labelled("expression")
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
